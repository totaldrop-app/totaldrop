//! `totaldrop`: the command-line client.

use std::io::{IsTerminal as _, Write as _};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use clap::{Parser, Subcommand};
use totaldrop_core::{
    Event, Events, IncomingOffer, PathOptions, PreparedSend, ReceiveOptions, SendOptions,
};
use totaldrop_l10n::{Arg, Domain, Strings, reveal_controls as visible};
use totaldrop_proto::TransferCode;
use unicode_normalization::UnicodeNormalization as _;

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
    /// Relay for when no direct path works, and for finding this device's
    /// public address. An empty value turns the relay off.
    #[arg(long, global = true, env = "TOTALDROP_RELAY")]
    relay: Option<String>,
    /// STUN server (host:port) that tells this device its public address.
    /// An empty value turns it off.
    #[arg(long, global = true, env = "TOTALDROP_STUN")]
    stun: Option<String>,
    /// Never connect directly and announce no IP address: everything goes
    /// through the relay. Slower, but the other side never learns your
    /// address.
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

async fn send(
    s: Arc<Strings>,
    server: String,
    paths: PathOptions,
    files: Vec<PathBuf>,
    strong: bool,
) -> Result<(), totaldrop_core::Error> {
    eprintln!("{}", s.get("cliPreparing", &[]));
    let opts = SendOptions {
        server,
        words: if strong { 3 } else { 2 },
        paths,
    };
    let prepared = PreparedSend::new(&opts, files).await?;
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
    let report = prepared.run(printer(Arc::clone(&s))).await?;
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

async fn receive(
    s: Arc<Strings>,
    server: String,
    paths: PathOptions,
    code: String,
    out: Option<PathBuf>,
    yes: bool,
) -> Result<(), totaldrop_core::Error> {
    let dest = match out {
        Some(d) => d,
        None => std::env::current_dir()?,
    };
    eprintln!("{}", s.get("cliConnecting", &[]));
    let offer = IncomingOffer::connect(&ReceiveOptions { server, paths }, &code).await?;
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
    let accepted = yes
        || tokio::task::spawn_blocking(move || {
            eprint!("{prompt}");
            let _ = std::io::stderr().flush();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).is_ok() && is_yes(&line, &yes_words)
        })
        .await
        .unwrap_or(false);
    if !accepted {
        offer.decline().await?;
        eprintln!("{}", s.get("cliDeclined", &[]));
        return Ok(());
    }
    let report = offer.accept(&dest, printer(Arc::clone(&s))).await?;
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

/// Whether an answer means yes: English `y`/`yes`, or one of the
/// translation's words (`cliYes`, separated by `|`). Case is folded without
/// regard to language, so a Turkish `E` or `EVET` works on any system.
fn is_yes(answer: &str, words: &str) -> bool {
    // Compatibility forms fold too: a full-width "ｙ" or a decomposed "sí".
    let fold = |w: &str| -> String {
        w.trim()
            .nfkc()
            .collect::<String>()
            .replace(['İ', 'I'], "i")
            .replace('ı', "i")
            .to_lowercase()
    };
    let answer = fold(answer);
    ["y", "yes"]
        .into_iter()
        .chain(words.split('|'))
        .any(|w| !fold(w).is_empty() && fold(w) == answer)
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
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("totaldrop: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_yes;

    #[test]
    fn yes_in_any_case_and_language() {
        assert!(is_yes("y\n", "e|evet"));
        assert!(is_yes("YES", "e|evet"));
        assert!(is_yes("EVET", "e|evet"));
        assert!(is_yes("E", "e|evet"));
        assert!(!is_yes("", "e|evet"));
        assert!(!is_yes("n", "e|evet"));
        assert!(!is_yes("hayır", "e|evet"));
        assert!(is_yes("ｙ", ""));
        assert!(is_yes("si\u{301}", "sí|s"));
    }
}
