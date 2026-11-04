//! Tests of the `totaldrop` program itself: its commands, answers and exit
//! codes, against a rendezvous server on this machine. The end-to-end tests
//! next to this file drive the core library; these start the real binary,
//! as a person would.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Output, Stdio};

/// The program under test, with no setting from the person's environment:
/// English, the given server, and no relay or STUN, so that nothing leaves
/// this machine.
fn totaldrop(server: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_totaldrop"));
    for var in [
        "TOTALDROP_SERVER",
        "TOTALDROP_RELAY",
        "TOTALDROP_STUN",
        "LANGUAGE",
        "LC_MESSAGES",
        "LC_NUMERIC",
    ] {
        cmd.env_remove(var);
    }
    cmd.env("LC_ALL", "en_US.UTF-8")
        .env("LANG", "en_US.UTF-8")
        .args(["--server", server, "--relay", "", "--stun", ""]);
    cmd
}

async fn server() -> String {
    let (addr, _server, _task) =
        totaldrop_rendezvous::spawn("127.0.0.1:0".parse().unwrap(), Default::default())
            .await
            .unwrap();
    format!("ws://{addr}")
}

/// A folder of its own under the temporary folder, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let mut rnd = [0u8; 6];
        getrandom::fill(&mut rnd).unwrap();
        let hex: String = rnd.iter().map(|b| format!("{b:02x}")).collect();
        let dir = std::env::temp_dir().join(format!("totaldrop-program-{name}-{hex}"));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl std::ops::Deref for Scratch {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for Scratch {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_os_str()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Starts `totaldrop send` on `what` and returns it with the code it
/// printed. The code is the only thing on stdout; the reader is kept open
/// with the child, so that the program never writes to a closed pipe.
fn start_send(server: &str, what: &Path) -> (Child, BufReader<ChildStdout>, String) {
    let mut child = totaldrop(server)
        .arg("send")
        .arg(what)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    let Some(code) = line.trim().strip_prefix("totaldrop receive ") else {
        let out = child.wait_with_output().unwrap();
        panic!("no receive line on stdout: {line:?}\n{}", stderr(&out));
    };
    let code = code.to_owned();
    (child, stdout, code)
}

/// Waits for `child` to end on a thread of its own, so that the server on
/// this runtime keeps answering.
async fn finish(child: Child) -> Output {
    tokio::task::spawn_blocking(move || child.wait_with_output().unwrap())
        .await
        .unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn code_shows_the_canonical_form() {
    let out = totaldrop("ws://127.0.0.1:1")
        .args(["code", "7-Harbor-Rocket"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines,
        ["code:      7-harbor-rocket", "canonical: 7-840-1499"],
        "{stdout}"
    );
}

#[test]
fn an_invalid_code_exits_with_2() {
    let out = totaldrop("ws://127.0.0.1:1")
        .args(["code", "not-a-code"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(stderr(&out).contains("invalid code"));
}

#[test]
fn relay_only_without_a_relay_exits_with_2() {
    let out = totaldrop("ws://127.0.0.1:1")
        .args(["--relay-only", "code", "7-harbor-rocket"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(stderr(&out).contains("--relay-only"), "{}", stderr(&out));
}

#[test]
fn receiving_into_a_missing_folder_fails_before_connecting() {
    let dir = Scratch::new("missing");
    let missing = dir.join("not-here");
    // No server listens on port 1: the folder is checked first, so the
    // program never gets as far as connecting.
    let out = totaldrop("ws://127.0.0.1:1")
        .args(["receive", "7-harbor-rocket", "--yes", "--out"])
        .arg(&missing)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("not-here"), "{}", stderr(&out));
}

#[test]
fn a_failed_connection_says_why_and_exits_with_1() {
    let dir = Scratch::new("noserver");
    let file = dir.join("a.txt");
    std::fs::write(&file, b"x").unwrap();
    let out = totaldrop("ws://127.0.0.1:1")
        .arg("send")
        .arg(&file)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    // One sentence on what failed, then the English detail for a bug report.
    let err = stderr(&out);
    assert!(
        err.contains("totaldrop: Could not reach the TotalDrop server."),
        "{err}"
    );
    assert!(err.trim_end().ends_with(')'), "{err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_folder_sent_with_the_program_arrives() {
    let server = server().await;
    let src_dir = Scratch::new("send-src");
    let src = src_dir.join("photos");
    std::fs::create_dir_all(src.join("beach")).unwrap();
    std::fs::write(src.join("beach/sea.txt"), b"blue").unwrap();
    std::fs::write(src.join("list.txt"), vec![7u8; 300_000]).unwrap();
    let dest = Scratch::new("send-dest");

    let (sender, _stdout, code) = start_send(&server, &src);
    let receiver = totaldrop(&server)
        .args(["receive", &code, "--yes", "--out"])
        .arg(&dest)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (got, sent) = tokio::join!(finish(receiver), finish(sender));

    assert!(got.status.success(), "receiver: {}", stderr(&got));
    assert!(sent.status.success(), "sender: {}", stderr(&sent));
    assert_eq!(
        std::fs::read(dest.join("photos/beach/sea.txt")).unwrap(),
        b"blue"
    );
    assert_eq!(
        std::fs::read(dest.join("photos/list.txt")).unwrap(),
        vec![7u8; 300_000]
    );
    // Both sides show the same verification words.
    let words = |s: &str| {
        s.lines()
            .find(|l| l.starts_with("Verification words: "))
            .map(str::to_owned)
    };
    assert!(words(&stderr(&got)).is_some(), "{}", stderr(&got));
    assert_eq!(words(&stderr(&got)), words(&stderr(&sent)));
}

/// Answers the question on stdin with `answer` and returns both sides and
/// the folder received into.
async fn answer_with(answer: &'static [u8]) -> (Output, Output, Scratch) {
    let server = server().await;
    let src = Scratch::new("ask-src");
    let file = src.join("note.txt");
    std::fs::write(&file, b"hello").unwrap();
    let dest = Scratch::new("ask-dest");

    let (sender, _stdout, code) = start_send(&server, &file);
    let mut receiver = totaldrop(&server)
        .args(["receive", &code, "--out"])
        .arg(&dest)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = receiver.stdin.take().unwrap();
    stdin.write_all(answer).unwrap();
    drop(stdin);
    let (got, sent) = tokio::join!(finish(receiver), finish(sender));
    (got, sent, dest)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn yes_at_the_question_saves_the_file() {
    let (got, sent, dest) = answer_with(b"yes\n").await;
    assert!(got.status.success(), "receiver: {}", stderr(&got));
    assert!(sent.status.success(), "sender: {}", stderr(&sent));
    assert_eq!(std::fs::read(dest.join("note.txt")).unwrap(), b"hello");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn no_answer_declines_and_the_sender_is_told() {
    // The end of input without a line is no: the default answer.
    let (got, sent, dest) = answer_with(b"").await;
    assert!(got.status.success(), "receiver: {}", stderr(&got));
    assert!(stderr(&got).contains("Declined"), "{}", stderr(&got));
    assert_eq!(sent.status.code(), Some(1), "sender: {}", stderr(&sent));
    assert!(
        stderr(&sent).contains("The receiver declined the files."),
        "sender: {}",
        stderr(&sent)
    );
    assert!(!dest.join("note.txt").exists());
}
