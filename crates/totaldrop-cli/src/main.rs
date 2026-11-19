// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! `totaldrop`: the command-line client.

#![deny(clippy::disallowed_macros, clippy::disallowed_methods)]

use std::io::IsTerminal as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Duration;

use clap::{
    Arg as ClapArg, ArgAction, CommandFactory as _, FromArgMatches as _, Parser, Subcommand,
};
use totaldrop_core::{
    Event, Events, IncomingOffer, PathOptions, PreparedSend, ReceiveOptions, SendOptions, Stop,
};
use totaldrop_l10n::{Domain, Strings, is_yes};
use totaldrop_proto::TransferCode;

use crate::term::{Clean, CleanArg};

mod sandbox;
mod term;

// The doc comments are the English help; `help` puts each in the person's
// language from l10n/cli under the key named next to it.
#[derive(Parser)]
#[command(
    name = "totaldrop",
    version,
    about = "Send any file to anyone, anywhere, end-to-end encrypted."
)]
struct Cli {
    /// Rendezvous server. Only wss:// is allowed, except ws:// on this machine.
    #[arg(long, global = true, env = "TOTALDROP_SERVER")]
    server: Option<String>,
    /// Relay for when no direct path works. When receiving, the sender's
    /// relay is used only if it is this same address. An empty value turns
    /// the relay off.
    #[arg(long, global = true, env = "TOTALDROP_RELAY")]
    relay: Option<String>,
    /// STUN server (host:port) that tells this device its public address.
    /// An empty value turns it off.
    #[arg(long, global = true, env = "TOTALDROP_STUN")]
    stun: Option<String>,
    /// Never connect directly and announce no IP address: everything goes
    /// through the relay. Slower, but the other side never learns your
    /// address. When receiving, the sender must use the same relay.
    #[arg(long, global = true)]
    relay_only: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Send files and folders, and print a transfer code for the receiver.
    Send {
        /// Files or folders to send; - sends the text read from standard
        /// input.
        #[arg(required_unless_present = "text", conflicts_with = "text")]
        paths: Vec<PathBuf>,
        /// Send this text instead of files: the receiver sees it rather
        /// than a file.
        #[arg(long)]
        text: Option<String>,
        /// Use a three-word code (33 bits) instead of two.
        #[arg(long)]
        strong: bool,
    },
    /// Receive files with a transfer code, for example 7-harbor-rocket.
    Receive {
        /// The transfer code you were given.
        code: String,
        /// Folder to save into (default: the current folder; in Flatpak,
        /// ~/Downloads/TotalDrop).
        #[arg(long, short)]
        out: Option<PathBuf>,
        /// Accept without asking.
        #[arg(long, short)]
        yes: bool,
    },
    /// Check a transfer code and show its canonical form.
    Code {
        /// A transfer code, for example 7-harbor-rocket.
        code: String,
    },
}

/// Prints events to stderr; progress at most once per percent.
fn printer(s: Arc<Strings>) -> Events {
    let last = Arc::new(AtomicU64::new(u64::MAX));
    Arc::new(move |event| match event {
        Event::Waiting => term::errln(&Clean::msg(&s, "cliWaiting", &[])),
        Event::Connected { words } => {
            term::errln(&Clean::msg(&s, "cliConnected", &[]));
            let words = Clean::visible(&words.join(" "));
            term::errln(&Clean::msg(
                &s,
                "cliVerification",
                &[("words", CleanArg::Text(&words))],
            ));
        }
        Event::Progress { done, total } => {
            let pct = done.saturating_mul(100).checked_div(total).unwrap_or(100);
            if last.swap(pct, Ordering::Relaxed) != pct {
                let (done_s, total_s) = (Clean::bytes(&s, done), Clean::bytes(&s, total));
                let line = Clean::msg(
                    &s,
                    "cliProgress",
                    &[
                        ("percent", CleanArg::Count(pct)),
                        ("done", CleanArg::Text(&done_s)),
                        ("total", CleanArg::Text(&total_s)),
                    ],
                );
                term::progress(&line, done >= total);
            }
        }
    })
}

/// Help text keys of the arguments, by command (`""` for the top level) and
/// argument id.
const ARG_HELP: [(&str, &str, &str); 11] = [
    ("", "server", "cliHelpServer"),
    ("", "relay", "cliHelpRelay"),
    ("", "stun", "cliHelpStun"),
    ("", "relay_only", "cliHelpRelayOnly"),
    ("send", "paths", "cliHelpSendPaths"),
    ("send", "strong", "cliHelpSendStrong"),
    ("send", "text", "cliHelpSendText"),
    ("receive", "code", "cliHelpReceiveCode"),
    ("receive", "out", "cliHelpReceiveOut"),
    ("receive", "yes", "cliHelpReceiveYes"),
    ("code", "code", "cliHelpCodeCode"),
];

/// Help text keys of the commands.
const COMMAND_HELP: [(&str, &str); 4] = [
    ("", "cliHelpAbout"),
    ("send", "cliHelpSend"),
    ("receive", "cliHelpReceive"),
    ("code", "cliHelpCode"),
];

/// The command line with its help in the language of `s`: descriptions,
/// headings and the help and version flags. Clap's own error messages
/// stay English.
fn help(s: &Strings) -> clap::Command {
    // A command's own options, then the shared ones, then help and version.
    const SHARED: usize = 100;
    const LAST: usize = 1000;
    let t = |key: &str| s.get(key, &[]);
    let (options, arguments) = (t("cliHelpOptions"), t("cliHelpArguments"));
    let template = format!(
        "{{about-with-newline}}\n{}: {{usage}}\n\n{{all-args}}{{after-help}}",
        t("cliHelpUsage")
    );
    let localize = |name: &str, cmd: clap::Command| {
        let mut cmd = cmd
            .help_template(template.clone())
            .subcommand_help_heading(t("cliHelpCommands"))
            .disable_help_flag(true)
            .arg(
                ClapArg::new("help")
                    .short('h')
                    .long("help")
                    .action(ArgAction::Help)
                    .help(t("cliHelpHelp"))
                    .display_order(LAST),
            );
        if let Some((_, key)) = COMMAND_HELP.iter().find(|(c, _)| *c == name) {
            cmd = cmd.about(t(key));
        }
        let ids: Vec<String> = cmd
            .get_arguments()
            .map(|a| a.get_id().to_string())
            .collect();
        for (i, id) in ids.into_iter().enumerate() {
            cmd = cmd.mut_arg(&id, |a| {
                // The shared options follow a command's own.
                let a = if name.is_empty() && a.get_display_order() < LAST {
                    a.display_order(SHARED.saturating_add(i))
                } else {
                    a
                };
                let a = match ARG_HELP.iter().find(|(c, i, _)| *c == name && *i == id) {
                    Some((_, _, key)) => a.help(t(key)),
                    None => a,
                };
                let heading = if a.is_positional() {
                    &arguments
                } else {
                    &options
                };
                a.help_heading(heading.clone())
            });
        }
        cmd
    };
    let cmd = Cli::command()
        .styles(clap::builder::styling::Styles::plain())
        .disable_version_flag(true)
        .arg(
            ClapArg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::Version)
                .help(t("cliHelpVersion"))
                .display_order(LAST),
        );
    let mut cmd = localize("", cmd).mut_subcommands(|sub| {
        let name = sub.get_name().to_owned();
        localize(&name, sub)
    });
    // Clap adds `help` as a command only when it builds the parser. Built
    // here, `help help` lists the commands instead of taking them as
    // arguments, and its own lines stay English; nothing else changes.
    cmd.build();
    cmd.mut_subcommand("help", |h| h.about(t("cliHelpHelpCommand")))
}

fn server(cli: Option<String>) -> String {
    cli.unwrap_or_else(|| totaldrop_core::rendezvous::DEFAULT_SERVER.to_owned())
}

fn gateway() -> totaldrop_core::portmap::Gateway {
    if let Ok(val) = std::env::var("TOTALDROP_TEST_GATEWAY") {
        if val.is_empty() {
            return totaldrop_core::portmap::Gateway::Off;
        }
        if let Ok(addr) = val.parse::<std::net::SocketAddr>()
            && addr.ip().is_loopback()
        {
            return totaldrop_core::portmap::Gateway::At(addr);
        }
    }
    totaldrop_core::portmap::Gateway::Auto
}

fn paths(relay: Option<String>, stun: Option<String>, relay_only: bool) -> PathOptions {
    PathOptions {
        relay: relay.unwrap_or_else(|| totaldrop_core::relay::DEFAULT_RELAY.to_owned()),
        stun: stun.unwrap_or_else(|| totaldrop_core::relay::DEFAULT_STUN.to_owned()),
        relay_only,
        gateway: gateway(),
    }
}

/// How long a stopped transfer may take to tell the other side and end
/// before it is dropped so its router port can close; matches the desktop
/// window's quit bound (#433, #527).
const STOPPING: Duration = Duration::from_millis(1500);

/// How long an aborted task gets to unwind and drop what it held.
const ABORTED: Duration = Duration::from_millis(200);

/// Stops the transfer on `SIGINT`, `SIGTERM` or `SIGHUP` and remembers which
/// signal arrived (`130`, `143` or `129`) for the process's exit code (#527).
struct SignalStop {
    stop: Stop,
    code: Arc<AtomicU8>,
}

impl SignalStop {
    fn new() -> Self {
        Self {
            stop: Stop::new(),
            code: Arc::new(AtomicU8::new(0)),
        }
    }

    /// Register the signal handlers on `runtime` before any transfer starts,
    /// so no signal slips past. Ctrl-C tells the person how to exit at once
    /// with a second Ctrl-C; `SIGTERM` and `SIGHUP` stop quietly because a
    /// closing terminal or session may send both together.
    fn install(&self, runtime: &tokio::runtime::Runtime, s: &Arc<Strings>) -> Stop {
        let (handle, code, s) = (self.stop.clone(), Arc::clone(&self.code), Arc::clone(s));
        let _enter = runtime.enter();
        #[cfg(unix)]
        let (mut interrupt, mut terminate, mut hangup) = {
            use tokio::signal::unix::{SignalKind, signal};
            (
                signal(SignalKind::interrupt()).ok(),
                signal(SignalKind::terminate()).ok(),
                signal(SignalKind::hangup()).ok(),
            )
        };
        runtime.spawn(async move {
            #[cfg(unix)]
            let first = {
                async fn recv(sig: &mut Option<tokio::signal::unix::Signal>) -> Option<()> {
                    match sig {
                        Some(s) => s.recv().await,
                        None => None,
                    }
                }
                tokio::select! {
                    Some(()) = recv(&mut interrupt) => 130_u8,
                    Some(()) = recv(&mut terminate) => 143_u8,
                    Some(()) = recv(&mut hangup) => 129_u8,
                    else => return,
                }
            };
            #[cfg(not(unix))]
            let first = if tokio::signal::ctrl_c().await.is_ok() {
                130_u8
            } else {
                return;
            };
            let _ = code.compare_exchange(0, first, Ordering::AcqRel, Ordering::Acquire);
            if first == 130 {
                term::err_blank();
                term::errln(&Clean::msg(&s, "cliStopping", &[]));
            }
            handle.stop();
            #[cfg(unix)]
            let again = first == 130
                && match &mut interrupt {
                    Some(i) => i.recv().await.is_some(),
                    None => false,
                };
            #[cfg(not(unix))]
            let again = tokio::signal::ctrl_c().await.is_ok();
            if again {
                std::process::exit(130);
            }
        });
        self.stop.clone()
    }

    fn exit_code(&self) -> ExitCode {
        match self.code.load(Ordering::Acquire) {
            0 => ExitCode::from(130),
            c => ExitCode::from(c),
        }
    }
}

/// The outcome of [`run_then_release`]: `Some` of the work's result, `None`
/// when a stopped task did not finish within `stopping` and was dropped so
/// its router port could release (#527), or the panic it ended with (#433).
type Ran<T> = Result<Option<T>, Box<dyn std::any::Any + Send>>;

/// Run `work`, then `release`, on `runtime`. `release` runs also when `work`
/// panics or when `stop` fires and a silent peer keeps `work` waiting past
/// `stopping`: what the work held (a router port, #433, #527) is dropped and
/// its release sent while the runtime still runs. The caller resumes a panic
/// afterwards.
fn run_then_release<T: Send + 'static>(
    runtime: &tokio::runtime::Runtime,
    stop: &Stop,
    stopping: Duration,
    work: impl std::future::Future<Output = T> + Send + 'static,
    release: impl std::future::Future<Output = ()>,
) -> Ran<T> {
    // As a task, a panic or post-stop abort unwinds only the task: its locals
    // drop while the runtime runs, and a panic comes back as a value.
    let outcome = runtime.block_on(async {
        let mut task = runtime.spawn(work);
        tokio::select! {
            res = &mut task => Some(res),
            () = stop.stopped() => match tokio::time::timeout(stopping, &mut task).await {
                Ok(res) => Some(res),
                Err(_) => {
                    task.abort();
                    match tokio::time::timeout(ABORTED, &mut task).await {
                        Ok(Ok(val)) => Some(Ok(val)),
                        Ok(Err(e)) if e.is_panic() => Some(Err(e)),
                        Ok(Err(_)) | Err(_) => None,
                    }
                }
            },
        }
    });
    let ran = match outcome {
        Some(Ok(val)) => Ok(Some(val)),
        None => Ok(None),
        Some(Err(e)) => Err(match e.try_into_panic() {
            Ok(panic) => panic,
            // Not cancelled by `stop`, and no panic hook saw it: told here,
            // and kept as a panic so it is not lost.
            Err(e) => {
                term::err_totaldrop(&Clean::error(&e));
                Box::new(e.to_string())
            }
        }),
    };
    runtime.block_on(release);
    ran
}

/// What a send sends: files, or a text (#313).
enum What {
    Files(Vec<PathBuf>),
    Text(String),
}

/// What `send` was asked to send: `--text`, `-` alone for the text on
/// standard input, or files (`./-` for a file named `-`). A text must be
/// UTF-8, not empty, and at most [`totaldrop_core::MAX_TEXT`] bytes; the
/// person is told otherwise. Reading from a `terminal`, they are told how
/// to end the text.
fn what_to_send(
    s: &Strings,
    paths: Vec<PathBuf>,
    text: Option<String>,
    stdin: &mut impl std::io::Read,
    terminal: bool,
) -> Result<What, Failure> {
    let text = match (text, paths.as_slice()) {
        (Some(t), _) => t,
        (None, [one]) if one.as_os_str() == "-" => {
            if terminal {
                // A Windows console ends input with Ctrl-Z, then Enter.
                let key = if cfg!(windows) {
                    "cliTypeTextWindows"
                } else {
                    "cliTypeText"
                };
                term::errln(&Clean::msg(s, key, &[]));
            }
            let mut bytes = Vec::new();
            // One byte more than allowed is enough to know it is too long.
            let limit = totaldrop_core::MAX_TEXT.saturating_add(1);
            std::io::Read::read_to_end(&mut std::io::Read::take(stdin, limit), &mut bytes)?;
            if bytes.len() as u64 > totaldrop_core::MAX_TEXT {
                return Err(too_long(s));
            }
            match String::from_utf8(bytes) {
                Ok(t) => t,
                Err(_) => {
                    term::err_totaldrop(&Clean::msg(s, "cliTextNotUtf8", &[]));
                    return Err(Failure::Shown);
                }
            }
        }
        (None, _) if paths.iter().any(|p| p.as_os_str() == "-") => {
            term::err_totaldrop(&Clean::msg(s, "cliDashAlone", &[]));
            return Err(Failure::Shown);
        }
        _ => return Ok(What::Files(paths)),
    };
    if text.is_empty() {
        term::err_totaldrop(&Clean::msg(s, "cliTextEmpty", &[]));
        return Err(Failure::Shown);
    }
    if text.len() as u64 > totaldrop_core::MAX_TEXT {
        return Err(too_long(s));
    }
    Ok(What::Text(text))
}

fn too_long(s: &Strings) -> Failure {
    let size = Clean::bytes(s, totaldrop_core::MAX_TEXT);
    term::err_totaldrop(&Clean::msg(
        s,
        "cliTextTooLong",
        &[("size", CleanArg::Text(&size))],
    ));
    Failure::Shown
}

async fn send(
    s: Arc<Strings>,
    server: String,
    paths: PathOptions,
    what: What,
    strong: bool,
    stop: Stop,
) -> Result<(), Failure> {
    term::errln(&Clean::msg(&s, "cliPreparing", &[]));
    let opts = SendOptions {
        server,
        words: if strong { 3 } else { 2 },
        paths,
    };
    let is_text = matches!(what, What::Text(_));
    let prepared = tokio::select! {
        r = async {
            match &what {
                What::Files(files) => PreparedSend::new(&opts, files.clone()).await,
                What::Text(text) => PreparedSend::text(&opts, text, totaldrop_core::TEXT_FILE).await,
            }
        } => r?,
        () = stop.stopped() => return Err(totaldrop_core::Error::Stopped.into()),
    };
    let (entries, total) = prepared.totals();
    if prepared.skipped() > 0 {
        let n = u64::try_from(prepared.skipped()).unwrap_or(u64::MAX);
        term::errln(&Clean::msg(
            &s,
            "cliSkipping",
            &[("count", CleanArg::Count(n))],
        ));
    }
    let size = Clean::bytes(&s, total);
    let entries = u64::try_from(entries).unwrap_or(u64::MAX);
    if is_text {
        term::errln(&Clean::msg(
            &s,
            "cliSendingText",
            &[("size", CleanArg::Text(&size))],
        ));
    } else {
        term::errln(&Clean::msg(
            &s,
            "cliSending",
            &[
                ("count", CleanArg::Count(entries)),
                ("size", CleanArg::Text(&size)),
            ],
        ));
    }
    term::err_blank();
    term::errln(&Clean::msg(&s, "cliRunThis", &[]));
    // The command itself is never translated: it is what the other side types.
    term::out_receive_command(prepared.code());
    term::err_blank();
    let report = prepared.run_with(printer(Arc::clone(&s)), stop).await?;
    if report.files_failed > 0 {
        let n = u64::try_from(report.files_failed).unwrap_or(u64::MAX);
        term::errln(&Clean::msg(
            &s,
            "cliSendSomeFailed",
            &[("count", CleanArg::Count(n))],
        ));
    } else {
        let n = u64::try_from(report.files_ok).unwrap_or(u64::MAX);
        term::errln(&Clean::msg(
            &s,
            "cliSendDone",
            &[("count", CleanArg::Count(n))],
        ));
    }
    Ok(())
}

/// The code as parsed, in its English form, and the line that says it back
/// before connecting. A pasted code can carry a direction override, so the
/// terminal may show it in another order than the parser reads; the person
/// sees the code that is used, in plain ASCII (#149).
fn connecting_line(
    s: &Strings,
    typed: &str,
) -> Result<(String, Clean), totaldrop_proto::CodeError> {
    let code = TransferCode::parse(typed)?.to_english();
    let shown = Clean::visible(&code);
    let line = Clean::msg(s, "cliConnectingCode", &[("code", CleanArg::Text(&shown))]);
    Ok((code, line))
}

fn destination_error_line(s: &Strings, dest: &Path, kind: std::io::ErrorKind) -> Clean {
    let shown = Clean::path(dest);
    let key = match kind {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory => "cliNoFolder",
        _ => "cliFolderNotWritable",
    };
    Clean::msg(s, key, &[("folder", CleanArg::Text(&shown))])
}

fn folder_not_kept_line(s: &Strings, folder: &Path, downloads: &Path) -> Clean {
    let shown = Clean::path(folder);
    let kept = Clean::path(downloads);
    Clean::msg(
        s,
        "cliFolderNotKept",
        &[
            ("folder", CleanArg::Text(&shown)),
            ("downloads", CleanArg::Text(&kept)),
        ],
    )
}

fn save_prompt(s: &Strings, dest: &Path) -> Clean {
    let shown = Clean::path(dest);
    Clean::msg(s, "cliPromptSave", &[("folder", CleanArg::Text(&shown))])
}

async fn receive(
    s: Arc<Strings>,
    server: String,
    paths: PathOptions,
    code: String,
    out: Option<PathBuf>,
    yes: bool,
    stop: Stop,
) -> Result<(), Failure> {
    let sandboxed = sandbox::sandboxed();
    let dest = match out {
        Some(d) => d,
        None => {
            let d =
                sandbox::default_destination(sandboxed, std::env::current_dir, sandbox::downloads)?;
            if sandboxed {
                // Flatpak makes it at start; if that failed, the check below
                // names the folder.
                let _ = std::fs::create_dir_all(&d);
            }
            d
        }
    };
    // Check the destination, as the core will, before the code is used up.
    let dest = match totaldrop_core::check_destination(&dest) {
        Ok(d) => d,
        Err(totaldrop_core::Error::Io(e)) => {
            // Said here, with the folder's name: better than any generic
            // sentence main could choose.
            term::err_totaldrop(&destination_error_line(&s, &dest, e.kind()));
            return Err(Failure::Shown);
        }
        Err(e) => return Err(e.into()),
    };
    // Inside Flatpak a writable folder may still be memory only: refuse it
    // before the code is used up, rather than say "Saved" for a file that
    // goes with the sandbox (#350).
    if sandboxed
        && let Ok(table) = std::fs::read_to_string("/proc/self/mountinfo")
        && let Ok(real) = std::fs::canonicalize(&dest)
        && !sandbox::kept(&table, &real)
    {
        term::err_totaldrop(&folder_not_kept_line(&s, &dest, &sandbox::downloads()));
        return Err(Failure::Shown);
    }
    let (code, line) = connecting_line(&s, &code).map_err(totaldrop_core::Error::Code)?;
    term::errln(&line);
    let offer =
        IncomingOffer::connect_with(&ReceiveOptions { server, paths }, &code, &stop).await?;
    let summary = offer.summary().clone();
    term::errln(&Clean::msg(&s, "cliConnected", &[]));
    let words = Clean::visible(&offer.words().join(" "));
    term::errln(&Clean::msg(
        &s,
        "cliVerification",
        &[("words", CleanArg::Text(&words))],
    ));
    let size = Clean::bytes(&s, summary.total_size);
    let count = |n: usize| CleanArg::Count(u64::try_from(n).unwrap_or(u64::MAX));
    let prompt = if summary.text {
        // A text is shown here, not saved (#313).
        term::errln(&Clean::msg(
            &s,
            "cliOfferedText",
            &[("size", CleanArg::Text(&size))],
        ));
        Clean::msg(&s, "cliPromptShow", &[])
    } else {
        term::errln(&Clean::msg(
            &s,
            "cliOffered",
            &[
                ("files", count(summary.files)),
                ("dirs", count(summary.dirs)),
                ("size", CleanArg::Text(&size)),
            ],
        ));
        for name in summary.top_level.iter().take(10) {
            term::err_indented(&Clean::visible(name));
        }
        if summary.top_level.len() > 10 {
            let more = summary.top_level.len().saturating_sub(10);
            term::err_indented(&Clean::msg(&s, "cliAndMore", &[("count", count(more))]));
        }
        save_prompt(&s, &dest)
    };
    let yes_words = s.get("cliYes", &[]);
    let accepted = if yes {
        Some(true)
    } else {
        let answer = tokio::task::spawn_blocking(move || {
            term::ask_stderr(&prompt).map(|l| is_yes(&l, &yes_words))
        });
        tokio::select! {
            // Join error or a read error on stdin (such as `EIO` when a
            // controlling terminal closes alongside `SIGHUP`) is not a
            // normal "no": stop cleanly rather than exiting with 0 or 1.
            a = answer => a.ok().flatten(),
            // A signal at the question declines, so the sender is not left waiting.
            () = stop.stopped() => None,
        }
    };
    let Some(accepted) = accepted else {
        // Best effort: the sender may be gone already; stopping is the answer.
        let _ = offer.decline().await;
        // When PTY hangup (`EIO` on stdin) woke `ask` just ahead of `SIGHUP`
        // reaching the signal task, give the signal task a moment to record
        // `129` before reading the exit code.
        if !stop.is_stopped() {
            let _ = tokio::time::timeout(Duration::from_millis(50), stop.stopped()).await;
        }
        return Err(totaldrop_core::Error::Stopped.into());
    };
    if !accepted {
        let declined = offer.decline().await;
        // On BSD/macOS a closing terminal can wake `ask` with EOF (`Some("")`)
        // alongside `SIGHUP`; if a signal arrived by now, report the stop.
        if stop.is_stopped() {
            return Err(totaldrop_core::Error::Stopped.into());
        }
        declined?;
        term::errln(&Clean::msg(&s, "cliDeclined", &[]));
        return Ok(());
    }
    let report = if summary.text {
        // In memory: nothing of a text is written to disk (#313).
        offer.accept_text_with(&stop).await?
    } else {
        offer
            .accept_with(&dest, printer(Arc::clone(&s)), &stop)
            .await?
    };
    if let Some(text) = &report.text {
        term::write_received_stdout(text)?;
    }
    for p in &report.placed {
        let name = Clean::path(p);
        term::errln(&Clean::msg(
            &s,
            "cliSaved",
            &[("name", CleanArg::Text(&name))],
        ));
    }
    if report.files_failed > 0 {
        let n = u64::try_from(report.files_failed).unwrap_or(u64::MAX);
        term::errln(&Clean::msg(
            &s,
            "cliReceiveSomeFailed",
            &[("count", CleanArg::Count(n))],
        ));
    }
    Ok(())
}

fn main() -> ExitCode {
    let strings = Arc::new(Strings::system(Domain::Cli));
    let matches = match help(&strings).try_get_matches() {
        Ok(m) => m,
        Err(e) => return term::exit_clap(&e),
    };
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(e) => {
            let mut cmd = help(&strings);
            return term::exit_clap(&e.format(&mut cmd));
        }
    };
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            term::err_totaldrop(&Clean::error(&e));
            return ExitCode::FAILURE;
        }
    };
    let route = paths(cli.relay, cli.stun, cli.relay_only);
    if route.relay_only && route.relay.is_empty() {
        term::err_totaldrop(&Clean::msg(&strings, "cliRelayOnlyNeedsRelay", &[]));
        return ExitCode::from(2);
    }
    let strings_for_exit = Arc::clone(&strings);
    let signal = SignalStop::new();
    let sending = matches!(cli.command, Command::Send { .. });
    let result = match cli.command {
        Command::Code { code } => {
            return match TransferCode::parse(&code) {
                Ok(c) => {
                    term::out_code(&c);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    term::err_invalid_code(&e);
                    ExitCode::from(2)
                }
            };
        }
        Command::Send {
            paths,
            text,
            strong,
        } => match what_to_send(
            &strings,
            paths,
            text,
            &mut std::io::stdin().lock(),
            std::io::stdin().is_terminal(),
        ) {
            // A port the router opened over UPnP is closed with an HTTP
            // request, which dies with the runtime; wait for it, briefly
            // (#406), also after a panic (#433) or a signal (#527).
            Ok(what) => {
                let stop = signal.install(&runtime, &strings);
                run_then_release(
                    &runtime,
                    &signal.stop,
                    STOPPING,
                    send(strings, server(cli.server), route, what, strong, stop),
                    totaldrop_core::portmap::released(),
                )
            }
            Err(e) => Ok(Some(Err(e))),
        },
        Command::Receive { code, out, yes } => run_then_release(
            &runtime,
            &signal.stop,
            STOPPING,
            receive(
                strings,
                server(cli.server),
                route,
                code,
                out,
                yes,
                signal.install(&runtime, &strings_for_exit),
            ),
            totaldrop_core::portmap::released(),
        ),
    };
    let result = match result {
        Ok(Some(result)) => result,
        Ok(None) => Err(Failure::Core(totaldrop_core::Error::Stopped)),
        Err(panic) => {
            // The panic hook, or `run_then_release` for a task that was
            // cancelled, has printed the message already.
            runtime.shutdown_background();
            std::panic::resume_unwind(panic);
        }
    };
    // A question still waiting for a line on stdin, or hashing a stopped
    // send left running, must not keep the process alive.
    runtime.shutdown_background();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Shown) => ExitCode::FAILURE,
        Err(Failure::Core(totaldrop_core::Error::Stopped)) => {
            term::errln(&Clean::msg(&strings_for_exit, "cliStopped", &[]));
            signal.exit_code()
        }
        Err(Failure::Core(e)) => {
            // One sentence in the person's language, then the detail for a
            // bug report, which stays English.
            term::err_totaldrop(&Clean::msg(&strings_for_exit, error_key(&e, sending), &[]));
            // Why, also in the person's language (#212).
            let cause = e.cause().as_str();
            if !cause.is_empty() {
                term::err_indented(&Clean::msg(
                    &strings_for_exit,
                    &format!("cause{cause}"),
                    &[],
                ));
            }
            term::errln(&detail_line(&e));
            ExitCode::FAILURE
        }
    }
}

/// The English detail of an error, for a bug report. It can carry text the
/// other side chose (the reason of its `cancel`, its QUIC close reason), so
/// every control and hidden character is named, as in a received name (T17).
fn detail_line(e: &totaldrop_core::Error) -> Clean {
    Clean::detail(e)
}

/// The message for each kind of failure, as the apps show it. Every variant
/// is listed, so a new one has to be given a message. What the server says
/// when no sender uses a code reaches a sender only when its own code
/// expired with nobody coming.
fn error_key(e: &totaldrop_core::Error, sending: bool) -> &'static str {
    use totaldrop_core::Error as E;
    use totaldrop_proto::wire::RejectReason;
    match e {
        E::WrongCode => "cliErrWrongCode",
        E::PeerLeft => "cliErrPeerLeft",
        E::Rejected(RejectReason::Declined) => "cliErrDeclined",
        E::Rejected(RejectReason::TooLarge | RejectReason::NoSpace) => "cliErrNoRoom",
        E::Rejected(RejectReason::Invalid) => "cliErrInvalidOffer",
        E::Cancelled(_) => "cliErrCancelled",
        E::Code(_) => "cliErrInvalidCode",
        // Only waiting gives "mailbox expired"; allocating does not.
        E::Rendezvous(r) if sending && r == "mailbox expired" => "cliErrCodeExpired",
        E::Rendezvous(_) if e.is_code_not_in_use() => "cliErrCodeNotInUse",
        E::Rendezvous(_) if e.is_bad_server_address() => "cliErrBadServer",
        E::Rendezvous(_) => "cliErrServer",
        E::Connect(_) => "cliErrConnect",
        E::InvalidOffer(_) => "cliErrInvalidOffer",
        E::Io(_) => "cliErrStorage",
        E::Stopped => "cliStopped",
        E::Protocol(_) | E::Random => "cliErrOther",
    }
}

/// How a command ends badly: an error to explain, or one already explained.
#[derive(Debug)]
enum Failure {
    Core(totaldrop_core::Error),
    /// Already printed, in the person's words.
    Shown,
}

impl From<totaldrop_core::Error> for Failure {
    fn from(e: totaldrop_core::Error) -> Self {
        Failure::Core(e)
    }
}

impl From<std::io::Error> for Failure {
    fn from(e: std::io::Error) -> Self {
        Failure::Core(e.into())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::path::Path;

    use super::{
        ARG_HELP, COMMAND_HELP, What, connecting_line, destination_error_line, detail_line,
        error_key, folder_not_kept_line, help, run_then_release, save_prompt, what_to_send,
    };
    use crate::term::{Clean, ask, shown_text};
    use totaldrop_core::{Error, Stop};
    use totaldrop_l10n::{Domain, Strings, is_hidden, reveal_text};
    use totaldrop_proto::wire::RejectReason;

    /// A panic in a send or receive must not skip the release of a router
    /// port (#433). The work holds something whose drop starts a release
    /// on the runtime, as a UPnP mapping does; the release step must see
    /// it finish, and the panic comes back to the caller.
    #[test]
    #[allow(clippy::panic)]
    fn a_panic_in_the_work_still_releases_what_it_held() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::Duration;

        struct Held(Arc<AtomicBool>);
        impl Drop for Held {
            fn drop(&mut self) {
                let done = Arc::clone(&self.0);
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    done.store(true, Ordering::SeqCst);
                });
            }
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let stop = Stop::new();
        let done = Arc::new(AtomicBool::new(false));
        let held = Held(Arc::clone(&done));
        let seen = Arc::clone(&done);
        let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_then_release(
                &runtime,
                &stop,
                Duration::from_secs(2),
                async move {
                    let _held = held;
                    tokio::task::yield_now().await;
                    panic!("a bug in the transfer");
                },
                async move {
                    let _ = tokio::time::timeout(Duration::from_secs(2), async {
                        while !seen.load(Ordering::SeqCst) {
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    })
                    .await;
                },
            )
        }));
        assert!(
            matches!(ran, Ok(Err(_))),
            "the panic comes back as a value, after the release"
        );
        assert!(done.load(Ordering::SeqCst), "the release finished");
    }

    /// A stopped transfer waiting on a silent peer past `stopping` is dropped
    /// before `release` runs, so its held router port still closes within the
    /// bounded quit timeout (#527).
    #[test]
    fn a_stopped_work_that_does_not_end_in_time_is_dropped_and_releases() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::{Duration, Instant};

        struct Held {
            dropped: Arc<AtomicBool>,
            released: Arc<AtomicBool>,
        }
        impl Drop for Held {
            fn drop(&mut self) {
                self.dropped.store(true, Ordering::SeqCst);
                let released = Arc::clone(&self.released);
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    released.store(true, Ordering::SeqCst);
                });
            }
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let stop = Stop::new();
        let (dropped, released, dropped_before_release) = (
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
        );
        let held = Held {
            dropped: Arc::clone(&dropped),
            released: Arc::clone(&released),
        };
        let (d, r, seen) = (
            Arc::clone(&dropped),
            Arc::clone(&released),
            Arc::clone(&dropped_before_release),
        );
        stop.stop();
        let started = Instant::now();
        let ran = run_then_release(
            &runtime,
            &stop,
            Duration::from_millis(50),
            async move {
                let _held = held;
                std::future::pending::<()>().await
            },
            async move {
                seen.store(d.load(Ordering::SeqCst), Ordering::SeqCst);
                let _ = tokio::time::timeout(Duration::from_secs(2), async {
                    while !r.load(Ordering::SeqCst) {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                })
                .await;
            },
        );
        assert!(matches!(ran, Ok(None)));
        assert!(
            dropped_before_release.load(Ordering::SeqCst),
            "held resource dropped before release began"
        );
        assert!(released.load(Ordering::SeqCst), "release finished");
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    /// `--help` speaks the person's language, as every other line does
    /// (#171): each description, the headings and the help flag.
    #[test]
    fn help_is_in_the_persons_language() {
        let strings = |tag: &str| {
            Strings::new(
                Domain::Cli,
                &[totaldrop_l10n::locale::parse(tag).unwrap()],
                None,
            )
        };
        help(&strings("en")).debug_assert();
        let tr = strings("tr");
        let mut cmd = help(&tr);
        let text = |cmd: &mut clap::Command, name: &str| match name {
            "" => cmd.render_help().to_string(),
            _ => cmd
                .find_subcommand_mut(name)
                .unwrap()
                .render_help()
                .to_string(),
        };
        for name in ["", "send", "receive", "code"] {
            let shown = text(&mut cmd, name);
            let mut keys: Vec<&str> = ARG_HELP
                .iter()
                .filter(|(c, _, _)| c.is_empty() || *c == name)
                .map(|(_, _, k)| *k)
                .collect();
            keys.extend(
                COMMAND_HELP
                    .iter()
                    .filter(|(c, _)| *c == name)
                    .map(|(_, k)| *k),
            );
            keys.extend(["cliHelpUsage", "cliHelpOptions", "cliHelpHelp"]);
            if !name.is_empty() {
                keys.push("cliHelpArguments");
            }
            for key in keys {
                let line = tr.get(key, &[]);
                assert!(shown.contains(&line), "{name}: {key} missing:\n{shown}");
            }
            for english in ["Usage", "Options", "Print help", "Arguments"] {
                assert!(!shown.contains(english), "{name}: {english} in:\n{shown}");
            }
        }
    }

    /// The code is said back before connecting, in its logical order and
    /// plain ASCII: an override such as U+202E can show a pasted code in
    /// another order on screen than the one the parser reads (#149).
    #[test]
    fn the_parsed_code_is_said_before_connecting() {
        let strings = |tag: &str| {
            Strings::new(
                Domain::Cli,
                &[totaldrop_l10n::locale::parse(tag).unwrap()],
                None,
            )
        };
        let en = strings("en");
        for typed in [
            "7-harbor-rocket",
            "\u{202E}7-harbor-rocket\u{202C}",
            "7 HARBOR \u{202E}rocket",
            "\u{0667}-840-1499",
        ] {
            let (code, line) = connecting_line(&en, typed).unwrap();
            assert_eq!(code, "7-harbor-rocket");
            assert_eq!(
                line.as_str(),
                "Connecting with 7-harbor-rocket…",
                "{typed:?}"
            );
            assert!(code.is_ascii());
        }
        let (_, line) = connecting_line(&strings("tr"), "\u{202E}7-harbor-rocket").unwrap();
        assert_eq!(line.as_str(), "7-harbor-rocket koduyla bağlanılıyor…");
        assert!(connecting_line(&en, "7-harbor").is_err());
    }

    /// `-` alone reads the text from standard input; `--text` takes it as
    /// given; anything else is files, and `-` among files is refused (#313).
    #[test]
    fn what_send_sends() {
        let s = Strings::new(
            Domain::Cli,
            &[totaldrop_l10n::locale::parse("en").unwrap()],
            None,
        );
        let none = &mut std::io::empty();
        let what =
            what_to_send(&s, vec!["-".into()], None, &mut &b"hi there\n"[..], false).unwrap();
        assert!(matches!(what, What::Text(t) if t == "hi there\n"));
        let what = what_to_send(&s, vec![], Some("note".into()), none, false).unwrap();
        assert!(matches!(what, What::Text(t) if t == "note"));
        let what = what_to_send(&s, vec!["a".into(), "./-".into()], None, none, false).unwrap();
        assert!(matches!(what, What::Files(f) if f.len() == 2));
        assert!(what_to_send(&s, vec!["a".into(), "-".into()], None, none, false).is_err());
        // Empty, not UTF-8, too long: said, and nothing is sent.
        assert!(what_to_send(&s, vec!["-".into()], None, none, false).is_err());
        assert!(
            what_to_send(&s, vec!["-".into()], None, &mut &[0xff_u8, 0xfe][..], false).is_err()
        );
        let long = vec![b'a'; usize::try_from(totaldrop_core::MAX_TEXT).unwrap() + 1];
        assert!(what_to_send(&s, vec!["-".into()], None, &mut &long[..], false).is_err());
        let exact = vec![b'a'; usize::try_from(totaldrop_core::MAX_TEXT).unwrap()];
        assert!(what_to_send(&s, vec!["-".into()], None, &mut &exact[..], false).is_ok());
    }

    /// A received text cannot move the cursor or hide its words in a
    /// terminal; into a pipe it goes unchanged.
    #[test]
    fn a_received_text_shows_its_hidden_characters() {
        let text = "one\ttwo\r\nred\u{1b}[31m \u{202e}evil\nfake\rreal\r\t\n";
        let shown = shown_text(text, true);
        assert!(!shown.contains('\u{1b}') && !shown.contains('\u{202e}'));
        assert!(shown.contains("<U+001B>") && shown.contains("<U+202E>"));
        assert!(shown.starts_with("one\ttwo\nred"));
        // Only a \r that ends a line goes; one inside a line is named.
        assert!(
            shown.ends_with("\nfake<U+000D>real<U+000D>\t\n"),
            "{shown:?}"
        );
        assert_eq!(shown_text("end\r", true), "end<U+000D>");
        assert_eq!(shown_text(text, false), text);
    }

    /// An answer that is not typed (a pipe or a file) is not echoed, so the
    /// question ends its own line: what comes next starts on a new one (#163).
    #[test]
    fn an_answer_from_a_pipe_ends_the_line() {
        let prompt = Clean::visible("Save? ");
        for (typed, shown) in [(false, "Save? \n"), (true, "Save? ")] {
            let mut out = Vec::new();
            let line = ask(&prompt, &mut &b"y\n"[..], &mut out, typed);
            assert_eq!(line.as_deref(), Some("y\n"));
            assert_eq!(String::from_utf8(out).unwrap(), shown, "typed: {typed}");
        }
        // Nothing to read: no answer, and the line still ends.
        let mut out = Vec::new();
        assert_eq!(
            ask(&prompt, &mut &b""[..], &mut out, false).as_deref(),
            Some("")
        );
        assert_eq!(out, b"Save? \n");
        // Ctrl-D at a terminal: no Enter is echoed, so the line ends here.
        let mut out = Vec::new();
        assert_eq!(
            ask(&prompt, &mut &b""[..], &mut out, true).as_deref(),
            Some("")
        );
        assert_eq!(out, b"Save? \n");
    }

    /// Every kind of failure has its own sentence in English and in
    /// Turkish (not an English fallback).
    #[test]
    fn every_error_has_a_translated_message() {
        let errors = [
            Error::WrongCode,
            Error::PeerLeft,
            Error::Rendezvous("connection closed".into()),
            Error::Rendezvous("unknown nameplate".into()),
            Error::Rendezvous("the server address must start with wss://".into()),
            Error::Connect("x".into()),
            Error::Protocol("x"),
            Error::InvalidOffer("x".into()),
            Error::Rejected(RejectReason::Declined),
            Error::Rejected(RejectReason::TooLarge),
            Error::Rejected(RejectReason::NoSpace),
            Error::Rejected(RejectReason::Invalid),
            Error::Cancelled("stopped".into()),
            Error::Stopped,
            Error::Io(std::io::Error::other("x")),
            Error::Code(totaldrop_proto::TransferCode::parse("x").unwrap_err()),
            Error::Random,
        ];
        let strings = |tag: &str| {
            Strings::new(
                Domain::Cli,
                &[totaldrop_l10n::locale::parse(tag).unwrap()],
                None,
            )
        };
        let (en, tr) = (strings("en"), strings("tr"));
        for e in &errors {
            for sending in [false, true] {
                let key = error_key(e, sending);
                assert!(en.keys().contains(key), "{key}");
                assert_ne!(
                    en.get(key, &[]),
                    tr.get(key, &[]),
                    "{key} is not in Turkish"
                );
            }
        }
        // The cause line the command line adds (#212) comes from the app
        // strings, in the person's language too.
        // Every cause the core can name has one, so the apps have it too.
        for cause in totaldrop_core::Cause::ALL {
            let key = format!("cause{}", cause.as_str());
            assert!(en.keys().contains(key.as_str()), "{key}");
            assert_ne!(
                en.get(&key, &[]),
                tr.get(&key, &[]),
                "{key} is not in Turkish"
            );
        }
        let timed_out = Error::Connect("timed out".into());
        assert_eq!(timed_out.cause(), totaldrop_core::Cause::TimedOut);
        assert_eq!(error_key(&errors[3], false), "cliErrCodeNotInUse");
        assert_eq!(error_key(&errors[4], false), "cliErrBadServer");
        assert_eq!(error_key(&errors[2], false), "cliErrServer");
        // The sender's own code expired with nobody coming (#164).
        let expired = Error::Rendezvous("mailbox expired".into());
        assert_eq!(error_key(&expired, true), "cliErrCodeExpired");
        assert_eq!(error_key(&expired, false), "cliErrCodeNotInUse");
        // A failed claim of the sender's own number is not that.
        let claim = Error::Rendezvous("unknown nameplate".into());
        assert_eq!(error_key(&claim, true), "cliErrCodeNotInUse");
    }

    /// The detail of an error can carry text the other side chose: the
    /// reason of its `cancel`, or its QUIC close reason. Its control
    /// characters are named, not acted on by the terminal (T17).
    #[test]
    fn the_error_detail_names_the_other_sides_controls() {
        let hostile = "\u{1b}[2J\u{9b}31mok\u{7}";
        for e in [
            Error::Cancelled(hostile.into()),
            Error::Connect(format!("closed by peer: 0: {hostile}")),
        ] {
            let line = detail_line(&e);
            assert!(
                line.as_str().contains("<U+001B>[2J<U+009B>31mok<U+0007>"),
                "{}",
                line.as_str()
            );
            assert!(!line.as_str().chars().any(char::is_control), "{line:?}");
        }
        assert_eq!(
            detail_line(&Error::Cancelled("stopped".into())).as_str(),
            "  (cancelled by the other side: stopped)"
        );
    }

    /// Every folder path formatted into `cliNoFolder`, `cliFolderNotWritable`,
    /// `cliFolderNotKept` (including `XDG_DOWNLOAD_DIR`) or `cliPromptSave`
    /// names hidden control and Bidi characters instead of sending them raw
    /// to the terminal (#399).
    #[test]
    fn destination_and_sandbox_lines_name_hidden_characters() {
        let s = Strings::new(
            Domain::Cli,
            &[totaldrop_l10n::locale::parse("en").unwrap()],
            None,
        );
        let folder = Path::new("/tmp/out-\x1b[31m-\u{202e}fdp.exe");
        let downloads = Path::new("/home/ada/dl-\x1b]2;pwnd\x07-\u{2066}TotalDrop");

        let missing = destination_error_line(&s, folder, std::io::ErrorKind::NotFound);
        assert!(!missing.as_str().chars().any(is_hidden));
        assert!(
            missing
                .as_str()
                .contains("/tmp/out-<U+001B>[31m-<U+202E>fdp.exe")
        );

        let unwritable = destination_error_line(&s, folder, std::io::ErrorKind::PermissionDenied);
        assert!(!unwritable.as_str().chars().any(is_hidden));
        assert!(unwritable.as_str().contains("Cannot write to"));
        assert!(
            unwritable
                .as_str()
                .contains("/tmp/out-<U+001B>[31m-<U+202E>fdp.exe")
        );

        let not_kept = folder_not_kept_line(&s, folder, downloads);
        assert!(!not_kept.as_str().chars().any(is_hidden));
        assert!(
            not_kept
                .as_str()
                .contains("/tmp/out-<U+001B>[31m-<U+202E>fdp.exe")
        );
        assert!(
            not_kept
                .as_str()
                .contains("/home/ada/dl-<U+001B>]2;pwnd<U+0007>-<U+2066>TotalDrop")
        );

        let prompt = save_prompt(&s, folder);
        assert!(!prompt.as_str().chars().any(is_hidden));
        assert!(
            prompt
                .as_str()
                .contains("/tmp/out-<U+001B>[31m-<U+202E>fdp.exe")
        );
    }

    /// Sanitising Clap's rendered `--help` with `reveal_text` preserves every
    /// localized CLI help screen unchanged (#399).
    #[test]
    fn reveal_text_preserves_localized_help_in_every_language() {
        let mut tags = totaldrop_l10n::available(Domain::Cli);
        tags.extend(totaldrop_l10n::PSEUDO.iter().map(|s| (*s).to_owned()));
        for tag in tags {
            let s = Strings::new(
                Domain::Cli,
                &[totaldrop_l10n::locale::parse(&tag).unwrap()],
                None,
            );
            let mut cmd = help(&s);
            for sub in ["", "send", "receive", "code"] {
                let rendered = match sub {
                    "" => cmd.render_help().ansi().to_string(),
                    _ => cmd
                        .find_subcommand_mut(sub)
                        .unwrap()
                        .render_help()
                        .ansi()
                        .to_string(),
                };
                assert!(!rendered.contains("<U+"), "{tag} {sub}: {rendered}");
                assert_eq!(reveal_text(&rendered), rendered, "{tag} {sub}");
            }
        }
    }
}
