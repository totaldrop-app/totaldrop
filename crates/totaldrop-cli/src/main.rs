//! `totaldrop`: the command-line client.

use std::io::{IsTerminal as _, Write as _};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use clap::{Parser, Subcommand};
use totaldrop_core::{
    Event, Events, IncomingOffer, PathOptions, PreparedSend, ReceiveOptions, SendOptions, Stop,
};
use totaldrop_l10n::{Arg, Domain, Strings, is_yes, reveal_controls as visible};
use totaldrop_proto::TransferCode;

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
        /// Files or folders to send.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Use a three-word code (33 bits) instead of two.
        #[arg(long)]
        strong: bool,
    },
    /// Receive files with a transfer code, for example 7-harbor-rocket.
    Receive {
        /// The transfer code you were given.
        code: String,
        /// Folder to save into (default: the current folder).
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
        Event::Waiting => eprintln!("{}", s.get("cliWaiting", &[])),
        Event::Connected { words } => {
            eprintln!("{}", s.get("cliConnected", &[]));
            let words = words.join(" ");
            eprintln!(
                "{}",
                s.get("cliVerification", &[("words", Arg::Text(&words))])
            );
        }
        Event::Progress { done, total } => {
            let pct = done.saturating_mul(100).checked_div(total).unwrap_or(100);
            if last.swap(pct, Ordering::Relaxed) != pct {
                let (done_s, total_s) = (s.bytes(done), s.bytes(total));
                let line = s.get(
                    "cliProgress",
                    &[
                        ("percent", Arg::Count(pct)),
                        ("done", Arg::Text(&done_s)),
                        ("total", Arg::Text(&total_s)),
                    ],
                );
                // On a terminal, clear the rest of the line: a shorter
                // translation must not leave old characters behind.
                let clear = if std::io::stderr().is_terminal() {
                    "\x1b[K"
                } else {
                    ""
                };
                eprint!("\r{line}{clear}");
                let _ = std::io::stderr().flush();
                if done >= total {
                    eprintln!();
                }
            }
        }
    })
}

fn server(cli: Option<String>) -> String {
    cli.unwrap_or_else(|| totaldrop_core::rendezvous::DEFAULT_SERVER.to_owned())
}

fn paths(relay: Option<String>, stun: Option<String>, relay_only: bool) -> PathOptions {
    PathOptions {
        relay: relay.unwrap_or_else(|| totaldrop_core::relay::DEFAULT_RELAY.to_owned()),
        stun: stun.unwrap_or_else(|| totaldrop_core::relay::DEFAULT_STUN.to_owned()),
        relay_only,
    }
}

/// Ctrl-C stops the transfer cleanly: the other side is told and partial
/// files are removed. A second Ctrl-C quits at once, cleanup or not.
fn stop_on_ctrl_c(s: &Arc<Strings>) -> Stop {
    let stop = Stop::new();
    let (handle, s) = (stop.clone(), Arc::clone(s));
    // Registered here, before anything runs, so no Ctrl-C slips past.
    #[cfg(unix)]
    let mut interrupt =
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()) {
            Ok(i) => i,
            Err(_) => return stop,
        };
    tokio::spawn(async move {
        #[cfg(unix)]
        let first = interrupt.recv().await.is_some();
        #[cfg(not(unix))]
        let first = tokio::signal::ctrl_c().await.is_ok();
        if !first {
            return;
        }
        eprintln!();
        eprintln!("{}", s.get("cliStopping", &[]));
        handle.stop();
        #[cfg(unix)]
        let again = interrupt.recv().await.is_some();
        #[cfg(not(unix))]
        let again = tokio::signal::ctrl_c().await.is_ok();
        if again {
            std::process::exit(130);
        }
    });
    stop
}

async fn send(
    s: Arc<Strings>,
    server: String,
    paths: PathOptions,
    files: Vec<PathBuf>,
    strong: bool,
) -> Result<(), Failure> {
    eprintln!("{}", s.get("cliPreparing", &[]));
    let opts = SendOptions {
        server,
        words: if strong { 3 } else { 2 },
        paths,
    };
    let stop = stop_on_ctrl_c(&s);
    let prepared = tokio::select! {
        r = PreparedSend::new(&opts, files) => r?,
        () = stop.stopped() => return Err(totaldrop_core::Error::Stopped.into()),
    };
    let (entries, total) = prepared.totals();
    if prepared.skipped() > 0 {
        let n = u64::try_from(prepared.skipped()).unwrap_or(u64::MAX);
        eprintln!("{}", s.get("cliSkipping", &[("count", Arg::Count(n))]));
    }
    let size = s.bytes(total);
    let entries = u64::try_from(entries).unwrap_or(u64::MAX);
    eprintln!(
        "{}",
        s.get(
            "cliSending",
            &[("count", Arg::Count(entries)), ("size", Arg::Text(&size))]
        )
    );
    eprintln!();
    eprintln!("{}", s.get("cliRunThis", &[]));
    // The command itself is never translated: it is what the other side types.
    println!("totaldrop receive {}", prepared.code());
    eprintln!();
    let report = prepared.run_with(printer(Arc::clone(&s)), stop).await?;
    if report.files_failed > 0 {
        let n = u64::try_from(report.files_failed).unwrap_or(u64::MAX);
        eprintln!(
            "{}",
            s.get("cliSendSomeFailed", &[("count", Arg::Count(n))])
        );
    } else {
        let n = u64::try_from(report.files_ok).unwrap_or(u64::MAX);
        eprintln!("{}", s.get("cliSendDone", &[("count", Arg::Count(n))]));
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
) -> Result<(String, String), totaldrop_proto::CodeError> {
    let code = TransferCode::parse(typed)?.to_english();
    let line = s.get("cliConnectingCode", &[("code", Arg::Text(&code))]);
    Ok((code, line))
}

async fn receive(
    s: Arc<Strings>,
    server: String,
    paths: PathOptions,
    code: String,
    out: Option<PathBuf>,
    yes: bool,
) -> Result<(), Failure> {
    let dest = match out {
        Some(d) => d,
        None => std::env::current_dir()?,
    };
    // Check the destination, as the core will, before the code is used up.
    let dest = match totaldrop_core::check_destination(&dest) {
        Ok(d) => d,
        Err(totaldrop_core::Error::Io(e)) => {
            let shown = dest.display().to_string();
            let key = match e.kind() {
                std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory => "cliNoFolder",
                _ => "cliFolderNotWritable",
            };
            // Said here, with the folder's name: better than any generic
            // sentence main could choose.
            eprintln!(
                "totaldrop: {}",
                s.get(key, &[("folder", Arg::Text(&shown))])
            );
            return Err(Failure::Shown);
        }
        Err(e) => return Err(e.into()),
    };
    let (code, line) = connecting_line(&s, &code).map_err(totaldrop_core::Error::Code)?;
    let stop = stop_on_ctrl_c(&s);
    eprintln!("{line}");
    let offer =
        IncomingOffer::connect_with(&ReceiveOptions { server, paths }, &code, &stop).await?;
    let summary = offer.summary().clone();
    eprintln!("{}", s.get("cliConnected", &[]));
    let words = offer.words().join(" ");
    eprintln!(
        "{}",
        s.get("cliVerification", &[("words", Arg::Text(&words))])
    );
    let size = s.bytes(summary.total_size);
    let count = |n: usize| Arg::Count(u64::try_from(n).unwrap_or(u64::MAX));
    eprintln!(
        "{}",
        s.get(
            "cliOffered",
            &[
                ("files", count(summary.files)),
                ("dirs", count(summary.dirs)),
                ("size", Arg::Text(&size)),
            ],
        )
    );
    for name in summary.top_level.iter().take(10) {
        eprintln!("  {}", visible(name));
    }
    if summary.top_level.len() > 10 {
        let more = summary.top_level.len().saturating_sub(10);
        eprintln!("  {}", s.get("cliAndMore", &[("count", count(more))]));
    }
    let shown = dest.display().to_string();
    let prompt = s.get("cliPromptSave", &[("folder", Arg::Text(&shown))]);
    let yes_words = s.get("cliYes", &[]);
    let accepted = yes || {
        let answer = tokio::task::spawn_blocking(move || {
            // The answer shows in the log only when both are terminals.
            let typed = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
            let line = ask(
                &prompt,
                &mut std::io::stdin().lock(),
                &mut std::io::stderr(),
                typed,
            );
            line.is_some_and(|l| is_yes(&l, &yes_words))
        });
        tokio::select! {
            a = answer => a.unwrap_or(false),
            // Ctrl-C at the question declines, so the sender is not left waiting.
            () = stop.stopped() => false,
        }
    };
    if stop.is_stopped() {
        // Best effort: the sender may be gone already; stopping is the answer.
        let _ = offer.decline().await;
        return Err(totaldrop_core::Error::Stopped.into());
    }
    if !accepted {
        offer.decline().await?;
        eprintln!("{}", s.get("cliDeclined", &[]));
        return Ok(());
    }
    let report = offer
        .accept_with(&dest, printer(Arc::clone(&s)), &stop)
        .await?;
    for p in &report.placed {
        let name = visible(&p.display().to_string());
        eprintln!("{}", s.get("cliSaved", &[("name", Arg::Text(&name))]));
    }
    if report.files_failed > 0 {
        let n = u64::try_from(report.files_failed).unwrap_or(u64::MAX);
        eprintln!(
            "{}",
            s.get("cliReceiveSomeFailed", &[("count", Arg::Count(n))])
        );
    }
    Ok(())
}

/// Asks `prompt` on `out` and reads one line of answer from `input`:
/// `None` on a read error, empty at the end of input. A terminal echoes what
/// is `typed`, Enter included; an answer from a pipe or a file is not
/// echoed, and Ctrl-D echoes no Enter, so then the line is ended here and
/// what comes next does not follow the question (#163).
fn ask(
    prompt: &str,
    input: &mut impl std::io::BufRead,
    out: &mut impl std::io::Write,
    typed: bool,
) -> Option<String> {
    let _ = write!(out, "{prompt}");
    let _ = out.flush();
    let mut line = String::new();
    let read = input.read_line(&mut line);
    if !typed || !line.ends_with('\n') {
        let _ = writeln!(out);
    }
    read.ok().map(|_| line)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let strings = Arc::new(Strings::system(Domain::Cli));
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("totaldrop: {e}");
            return ExitCode::FAILURE;
        }
    };
    let route = paths(cli.relay, cli.stun, cli.relay_only);
    if route.relay_only && route.relay.is_empty() {
        eprintln!("totaldrop: {}", strings.get("cliRelayOnlyNeedsRelay", &[]));
        return ExitCode::from(2);
    }
    let strings_for_exit = Arc::clone(&strings);
    let sending = matches!(cli.command, Command::Send { .. });
    let result = match cli.command {
        Command::Code { code } => {
            return match TransferCode::parse(&code) {
                Ok(c) => {
                    println!("code:      {c}");
                    println!("canonical: {}", c.canonical());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("totaldrop: invalid code: {e}");
                    ExitCode::from(2)
                }
            };
        }
        Command::Send { paths, strong } => {
            runtime.block_on(send(strings, server(cli.server), route, paths, strong))
        }
        Command::Receive { code, out, yes } => {
            runtime.block_on(receive(strings, server(cli.server), route, code, out, yes))
        }
    };
    // A question still waiting for a line on stdin, or hashing a stopped
    // send left running, must not keep the process alive.
    runtime.shutdown_background();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Shown) => ExitCode::FAILURE,
        Err(Failure::Core(totaldrop_core::Error::Stopped)) => {
            eprintln!("{}", strings_for_exit.get("cliStopped", &[]));
            ExitCode::from(130)
        }
        Err(Failure::Core(e)) => {
            // One sentence in the person's language, then the detail for a
            // bug report, which stays English.
            eprintln!(
                "totaldrop: {}",
                strings_for_exit.get(error_key(&e, sending), &[])
            );
            eprintln!("  ({e})");
            ExitCode::FAILURE
        }
    }
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

    use super::{ask, connecting_line, error_key};
    use totaldrop_core::Error;
    use totaldrop_l10n::{Domain, Strings};
    use totaldrop_proto::wire::RejectReason;

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
            assert_eq!(line, "Connecting with 7-harbor-rocket…", "{typed:?}");
            assert!(code.is_ascii());
        }
        let (_, line) = connecting_line(&strings("tr"), "\u{202E}7-harbor-rocket").unwrap();
        assert_eq!(line, "7-harbor-rocket koduyla bağlanılıyor…");
        assert!(connecting_line(&en, "7-harbor").is_err());
    }

    /// An answer that is not typed (a pipe or a file) is not echoed, so the
    /// question ends its own line: what comes next starts on a new one (#163).
    #[test]
    fn an_answer_from_a_pipe_ends_the_line() {
        for (typed, shown) in [(false, "Save? \n"), (true, "Save? ")] {
            let mut out = Vec::new();
            let line = ask("Save? ", &mut &b"y\n"[..], &mut out, typed);
            assert_eq!(line.as_deref(), Some("y\n"));
            assert_eq!(String::from_utf8(out).unwrap(), shown, "typed: {typed}");
        }
        // Nothing to read: no answer, and the line still ends.
        let mut out = Vec::new();
        assert_eq!(
            ask("Save? ", &mut &b""[..], &mut out, false).as_deref(),
            Some("")
        );
        assert_eq!(out, b"Save? \n");
        // Ctrl-D at a terminal: no Enter is echoed, so the line ends here.
        let mut out = Vec::new();
        assert_eq!(
            ask("Save? ", &mut &b""[..], &mut out, true).as_deref(),
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
}
