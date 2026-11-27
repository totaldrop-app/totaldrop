// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Tests that `totaldrop receive` exits promptly on `SIGINT` while waiting at
//! the confirmation prompt (#92), without touched files in `program.rs`.

#![cfg(unix)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use totaldrop_core::{Error, PathOptions, PreparedSend, SendOptions, no_events};
use totaldrop_proto::wire::RejectReason;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let mut rnd = [0u8; 6];
        getrandom::fill(&mut rnd).unwrap();
        let hex: String = rnd.iter().map(|b| format!("{b:02x}")).collect();
        let dir = std::env::temp_dir().join(format!("totaldrop-cancel-prompt-{name}-{hex}"));
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

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sigint_at_the_prompt_declines_and_exits_within_two_seconds() {
    let (addr, _server, _task) =
        totaldrop_rendezvous::spawn("127.0.0.1:0".parse().unwrap(), Default::default())
            .await
            .unwrap();
    let server = format!("ws://{addr}");

    let src = Scratch::new("src");
    let file = src.join("report.txt");
    std::fs::write(&file, b"confidential").unwrap();
    let dest = Scratch::new("dest");

    let prepared = PreparedSend::new(
        &SendOptions {
            server: server.clone(),
            words: 2,
            paths: PathOptions {
                relay: String::new(),
                stun: String::new(),
                relay_only: false,
                gateway: totaldrop_core::portmap::Gateway::Off,
            },
        },
        vec![file],
    )
    .await
    .unwrap();
    let code = prepared.code().to_string();
    let sender = tokio::spawn(prepared.run(no_events()));

    let mut receiver = totaldrop(&server)
        .args(["receive", &code, "--out"])
        .arg(&*dest)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Keep stdin open across `wait()` so `ask()` remains blocked on
    // `read_line`: this proves `runtime.shutdown_background()` lets the
    // process exit promptly on `SIGINT` without waiting for stdin EOF.
    let _stdin = receiver.stdin.take().unwrap();
    let mut child_stderr = receiver.stderr.take().unwrap();
    let (prompt_tx, prompt_rx) = tokio::sync::oneshot::channel::<()>();

    let stderr_collector = std::thread::spawn(move || {
        let mut prompt_tx = Some(prompt_tx);
        let mut all = Vec::new();
        let mut buf = [0u8; 256];
        loop {
            match child_stderr.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    all.extend_from_slice(&buf[..n]);
                    if prompt_tx.is_some()
                        && String::from_utf8_lossy(&all).contains("[y/N] ")
                        && let Some(tx) = prompt_tx.take()
                    {
                        let _ = tx.send(());
                    }
                }
            }
        }
        String::from_utf8_lossy(&all).into_owned()
    });

    tokio::time::timeout(Duration::from_secs(10), prompt_rx)
        .await
        .expect("receiver did not reach the confirmation prompt in time")
        .expect("receiver exited before printing the confirmation prompt");

    let pid = receiver.id().to_string();
    let started = Instant::now();
    let kill_status = Command::new("kill")
        .args(["-s", "INT", &pid])
        .status()
        .expect("failed to invoke kill -s INT");
    assert!(kill_status.success(), "kill -s INT failed: {kill_status}");

    let status = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::task::spawn_blocking(move || receiver.wait().unwrap()),
    )
    .await
    .expect("receiver hung after SIGINT")
    .unwrap();
    let elapsed = started.elapsed();

    let err_output = stderr_collector.join().unwrap();
    assert!(
        elapsed <= Duration::from_secs(2),
        "expected CLI to exit within 2s on SIGINT at prompt, took {elapsed:?}\nstderr:\n{err_output}"
    );
    assert_eq!(
        status.code(),
        Some(130),
        "expected exit code 130 on SIGINT, got {status:?}\nstderr:\n{err_output}"
    );
    assert!(
        err_output.contains("Stopped."),
        "expected Stopped message on stderr, got:\n{err_output}"
    );

    let sent = tokio::time::timeout(Duration::from_secs(5), sender)
        .await
        .expect("sender hung after receiver SIGINT")
        .unwrap();
    assert!(
        matches!(sent, Err(Error::Rejected(RejectReason::Declined))),
        "expected sender Rejected(Declined), got {sent:?}"
    );
    assert_eq!(
        std::fs::read_dir(&*dest).unwrap().count(),
        0,
        "destination must remain empty after SIGINT at prompt"
    );
}
