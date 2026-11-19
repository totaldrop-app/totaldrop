// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
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
/// English, the given server, and no relay, STUN or router gateway, so that
/// nothing leaves this machine.
fn totaldrop(server: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_totaldrop"));
    for var in [
        "TOTALDROP_SERVER",
        "TOTALDROP_RELAY",
        "TOTALDROP_STUN",
        "TOTALDROP_TEST_GATEWAY",
        "LANGUAGE",
        "LC_MESSAGES",
        "LC_NUMERIC",
    ] {
        cmd.env_remove(var);
    }
    cmd.env("LC_ALL", "en_US.UTF-8")
        .env("LANG", "en_US.UTF-8")
        .env("TOTALDROP_TEST_GATEWAY", "")
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
    start_send_with_gateway(server, what, "")
}

fn start_send_with_gateway(
    server: &str,
    what: &Path,
    gateway: &str,
) -> (Child, BufReader<ChildStdout>, String) {
    let mut child = totaldrop(server)
        .env("TOTALDROP_TEST_GATEWAY", gateway)
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
fn destination_errors_name_hidden_characters() {
    let dir = Scratch::new("hostile-out");
    let missing = dir.join("missing-\x1b[31m-\u{202e}fdp.exe");
    let regular = dir.join("file-\x1b[31m-\u{202e}fdp.exe");
    std::fs::write(&regular, b"x").unwrap();

    for (path, expected_tail) in [
        (&missing, "missing-<U+001B>[31m-<U+202E>fdp.exe"),
        (&regular, "file-<U+001B>[31m-<U+202E>fdp.exe"),
    ] {
        let out = totaldrop("ws://127.0.0.1:1")
            .args(["receive", "7-harbor-rocket", "--yes", "--out"])
            .arg(path)
            .output()
            .unwrap();
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(1), "{err}");
        assert!(
            !err.contains('\x1b') && !err.contains('\u{202e}'),
            "{err:?}"
        );
        assert!(err.contains(expected_tail), "{err}");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let ro = dir.join("readonly-\x1b[31m-\u{202e}dir");
        std::fs::create_dir_all(&ro).unwrap();
        std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555)).unwrap();
        let unprivileged = std::fs::File::create(ro.join("probe")).is_err();
        let out = totaldrop("ws://127.0.0.1:1")
            .args(["receive", "7-harbor-rocket", "--yes", "--out"])
            .arg(&ro)
            .output()
            .unwrap();
        std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o755)).unwrap();
        if unprivileged {
            let err = stderr(&out);
            assert_eq!(out.status.code(), Some(1), "{err}");
            assert!(
                !err.contains('\x1b') && !err.contains('\u{202e}'),
                "{err:?}"
            );
            assert!(err.contains("Cannot write to"), "{err}");
            assert!(err.contains("readonly-<U+001B>[31m-<U+202E>dir"), "{err}");
        }
    }
}

#[test]
fn argument_and_code_errors_name_hidden_characters() {
    let out = totaldrop("ws://127.0.0.1:1")
        .arg("--\x1b[31m\x1b]2;pwnd\x07\r\u{009b}31m\u{202e}relay-only")
        .output()
        .unwrap();
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(
        !err.contains('\x1b')
            && !err.contains('\x07')
            && !err.contains('\r')
            && !err.contains('\u{009b}')
            && !err.contains('\u{202e}'),
        "{err:?}"
    );
    assert!(
        err.contains("<U+001B>[31m<U+001B>]2;pwnd<U+0007><U+000D><U+009B>31m<U+202E>relay-only"),
        "{err}"
    );

    for code in ["7-\u{202e}notaword-rocket", "7-\x1b\u{009b}notaword-rocket"] {
        let out = totaldrop("ws://127.0.0.1:1")
            .args(["code", code])
            .output()
            .unwrap();
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(2), "{err}");
        assert!(
            !err.contains('\x1b') && !err.contains('\u{009b}') && !err.contains('\u{202e}'),
            "{err:?}"
        );
        assert!(err.contains("invalid code"), "{err}");
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        let out = Command::new(env!("CARGO_BIN_EXE_totaldrop"))
            .arg0("evil-\x1b[31m\x1b]2;pwnd\x07\r\u{009b}31m\u{202e}bin")
            .env("LC_ALL", "en_US.UTF-8")
            .env("LANG", "en_US.UTF-8")
            .arg("--help")
            .output()
            .unwrap();
        let err = stderr(&out);
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(out.status.success(), "{err}");
        assert!(
            !stdout.contains('\x1b')
                && !stdout.contains('\x07')
                && !stdout.contains('\r')
                && !stdout.contains('\u{009b}')
                && !stdout.contains('\u{202e}'),
            "{stdout:?}"
        );
        assert!(
            stdout
                .contains("evil-<U+001B>[31m<U+001B>]2;pwnd<U+0007><U+000D><U+009B>31m<U+202E>bin"),
            "{stdout}"
        );
    }
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_save_prompt_names_hidden_characters_in_the_destination_folder() {
    let server = server().await;
    let src = Scratch::new("prompt-src");
    let file = src.join("note.txt");
    std::fs::write(&file, b"hello").unwrap();
    let base = Scratch::new("prompt-dest");
    let dest = base.join("save-\x1b[31m-\u{202e}here");
    std::fs::create_dir_all(&dest).unwrap();

    let (sender, _stdout, code) = start_send(&server, &file);
    let receiver = totaldrop(&server)
        .args(["receive", &code, "--out"])
        .arg(&dest)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (got, _sent) = tokio::join!(finish(receiver), finish(sender));
    let err = stderr(&got);
    assert!(got.status.success(), "receiver: {err}");
    assert!(
        !err.contains('\x1b') && !err.contains('\u{202e}'),
        "{err:?}"
    );
    assert!(err.contains("save-<U+001B>[31m-<U+202E>here"), "{err}");
}

#[cfg(unix)]
mod unix_signals {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddr};
    use std::sync::Arc;
    use tokio::net::UdpSocket;
    use tokio::sync::mpsc;
    use totaldrop_core::{
        Error, IncomingOffer, PathOptions, PreparedSend, ReceiveOptions, SendOptions,
        portmap::Gateway,
    };
    use totaldrop_proto::portmap::answer;
    use totaldrop_proto::wire::RejectReason;

    /// A loopback PCP router that grants external port 51000 and records the
    /// requested lifetime of every PCP MAP packet (`0` is a release).
    async fn pcp_router() -> (SocketAddr, mpsc::UnboundedReceiver<u32>) {
        let socket = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let addr = socket.local_addr().unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut buf = [0u8; 1500];
            while let Ok((n, from)) = socket.recv_from(&mut buf).await {
                if n >= 60 && buf[0] == 2 {
                    let req: [u8; 60] = buf[..60].try_into().unwrap();
                    let lifetime = u32::from_be_bytes(req[4..8].try_into().unwrap());
                    let _ = tx.send(lifetime);
                    let reply = answer::pcp(
                        &req,
                        0,
                        (Ipv4Addr::new(203, 0, 113, 7).into(), 51_000),
                        lifetime,
                    );
                    let _ = socket.send_to(&reply, from).await;
                }
            }
        });
        (addr, rx)
    }

    fn send_signal(pid: u32, sig: &str) {
        let status = Command::new("kill")
            .args(["-s", sig, &pid.to_string()])
            .status()
            .unwrap();
        assert!(status.success(), "kill -s {sig} {pid} failed: {status}");
    }

    fn local_paths() -> PathOptions {
        PathOptions {
            relay: String::new(),
            stun: String::new(),
            relay_only: false,
            gateway: Gateway::Off,
        }
    }

    /// `SIGTERM` and `SIGHUP` stop an active `totaldrop send`, tell the
    /// connected receiver, send both copies of the PCP port release before
    /// exiting, and return the signal's exit code (#527).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sigterm_and_sighup_stop_send_and_release_router_port() {
        let server = server().await;
        let src = Scratch::new("sig-send-src");
        let file = src.join("payload.bin");
        std::fs::write(&file, vec![9u8; 64_000]).unwrap();

        for (sig, expected_code) in [("TERM", 143), ("HUP", 129)] {
            let (router_addr, mut lifetimes) = pcp_router().await;
            let (sender, _stdout, code) =
                start_send_with_gateway(&server, &file, &router_addr.to_string());
            let pid = sender.id();

            let offer = IncomingOffer::connect(
                &ReceiveOptions {
                    server: server.clone(),
                    paths: local_paths(),
                },
                &code,
            )
            .await
            .unwrap();

            send_signal(pid, sig);

            let dest = Scratch::new("sig-send-dest");
            // Hold `offer` at the prompt until `sender` has handled the signal
            // and exited, so loopback `accept` cannot race ahead of the signal
            // and complete the 64 kB transfer first.
            let sent = finish(sender).await;
            let recv_res = offer.accept(&dest, Arc::new(|_| {})).await;

            assert_eq!(
                sent.status.code(),
                Some(expected_code),
                "{sig} stderr: {}",
                stderr(&sent)
            );
            assert!(
                !stderr(&sent).contains("press Ctrl-C again"),
                "{sig} must not print Ctrl-C hint: {}",
                stderr(&sent)
            );
            assert!(
                matches!(recv_res, Err(Error::Cancelled(_))),
                "{sig} receiver should hear Cancelled, got: {recv_res:?}"
            );

            let mut seen = Vec::new();
            while let Ok(lt) = lifetimes.try_recv() {
                seen.push(lt);
            }
            assert!(
                seen.iter().any(|&lt| lt > 0),
                "{sig}: expected initial PCP map request, saw {seen:?}"
            );
            let releases = seen.iter().filter(|&&lt| lt == 0).count();
            assert!(
                releases >= 2,
                "{sig}: expected both PCP release copies before exit (#502, #527), saw {seen:?}"
            );
        }
    }

    /// `SIGHUP` while `totaldrop receive` sits at the offer declines cleanly
    /// and releases its mapped router port before exiting (#527).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sighup_at_the_offer_declines_and_releases_router_port() {
        let server = server().await;
        let src = Scratch::new("sig-recv-src");
        let file = src.join("note.txt");
        std::fs::write(&file, b"hello").unwrap();
        let dest = Scratch::new("sig-recv-dest");

        let prepared = PreparedSend::new(
            &SendOptions {
                server: server.clone(),
                words: 2,
                paths: local_paths(),
            },
            vec![file],
        )
        .await
        .unwrap();
        let code = prepared.code().to_string();
        let sender_task = tokio::spawn(prepared.run(Arc::new(|_| {})));

        let (router_addr, mut lifetimes) = pcp_router().await;
        let mut receiver = totaldrop(&server)
            .env("TOTALDROP_TEST_GATEWAY", router_addr.to_string())
            .args(["receive", &code, "--out"])
            .arg(&dest)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let pid = receiver.id();
        // Keep stdin open so `receive` stays at the y/N prompt until SIGHUP.
        let _stdin = receiver.stdin.take().unwrap();
        let mut err_reader = BufReader::new(receiver.stderr.take().unwrap());
        let err_reader = tokio::task::spawn_blocking(move || {
            let mut line = String::new();
            loop {
                line.clear();
                let n = err_reader.read_line(&mut line).unwrap();
                assert!(n > 0, "receiver exited before showing offered file");
                if line.contains("note.txt") {
                    break;
                }
            }
            err_reader
        })
        .await
        .unwrap();

        send_signal(pid, "HUP");

        let (got_status, sent_res) = tokio::join!(
            tokio::task::spawn_blocking(move || {
                let status = receiver.wait().unwrap();
                drop(err_reader);
                status
            }),
            sender_task,
        );
        let got_status = got_status.unwrap();
        let sent_res = sent_res.unwrap();

        assert_eq!(got_status.code(), Some(129), "status: {got_status}");
        assert!(
            matches!(sent_res, Err(Error::Rejected(RejectReason::Declined))),
            "sender should hear Declined, got: {sent_res:?}"
        );
        assert!(!dest.join("note.txt").exists());

        let mut seen = Vec::new();
        while let Ok(lt) = lifetimes.try_recv() {
            seen.push(lt);
        }
        assert!(seen.iter().any(|&lt| lt > 0), "saw {seen:?}");
        assert!(
            seen.iter().filter(|&&lt| lt == 0).count() >= 2,
            "expected both PCP release copies before exit, saw {seen:?}"
        );
    }

    /// Under `nohup` (`SIG_IGN` inherited for `SIGHUP`), Linux keeps ignoring
    /// `SIGHUP` so `totaldrop send` stays alive and finishes the transfer
    /// instead of exiting with 129 (#527).
    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn ignored_sighup_keeps_send_running_and_completes_transfer() {
        let server = server().await;
        let src = Scratch::new("sig-nohup-src");
        let file = src.join("payload.bin");
        let payload = vec![7u8; 32_768];
        std::fs::write(&file, &payload).unwrap();

        let mut sender = Command::new("sh")
            .args([
                "-c",
                "trap '' HUP; exec \"$@\"",
                "_",
                env!("CARGO_BIN_EXE_totaldrop"),
                "--server",
                &server,
                "send",
            ])
            .arg(&file)
            .env("LC_ALL", "C")
            .env("LANGUAGE", "en")
            .env("TOTALDROP_TEST_GATEWAY", "")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let pid = sender.id();
        let mut stdout = BufReader::new(sender.stdout.take().unwrap());
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        let Some(code) = line.trim().strip_prefix("totaldrop receive ") else {
            let out = sender.wait_with_output().unwrap();
            panic!("no receive line on stdout: {line:?}\n{}", stderr(&out));
        };
        let code = code.to_owned();

        let offer = IncomingOffer::connect(
            &ReceiveOptions {
                server: server.clone(),
                paths: local_paths(),
            },
            &code,
        )
        .await
        .unwrap();

        // Deliver SIGHUP after signal handlers are installed and peer connected.
        send_signal(pid, "HUP");

        let dest = Scratch::new("sig-nohup-dest");
        let recv_res = offer.accept(&dest, Arc::new(|_| {})).await;
        let sent = finish(sender).await;

        assert!(
            recv_res.is_ok(),
            "ignored SIGHUP must not cancel transfer, got: {recv_res:?}\nstderr: {}",
            stderr(&sent)
        );
        assert!(
            sent.status.success(),
            "expected exit 0 with ignored SIGHUP, got {}:\n{}",
            sent.status,
            stderr(&sent)
        );
        assert_eq!(std::fs::read(dest.join("payload.bin")).unwrap(), payload);
    }
}
