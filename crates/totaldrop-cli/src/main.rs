//! `totaldrop`: the command-line client.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use clap::{Parser, Subcommand};
use totaldrop_core::{Event, Events, IncomingOffer, PreparedSend, ReceiveOptions, SendOptions};
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

fn human(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    #[allow(clippy::cast_precision_loss)]
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1000.0 && unit < UNITS.len().saturating_sub(1) {
        value /= 1000.0;
        unit = unit.saturating_add(1);
    }
    let name = UNITS.get(unit).copied().unwrap_or("B");
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {name}")
    }
}

/// Prints events to stderr; progress at most once per percent.
fn printer() -> Events {
    let last = Arc::new(AtomicU64::new(u64::MAX));
    Arc::new(move |event| match event {
        Event::Waiting => eprintln!("Waiting for the receiver…"),
        Event::Connected { words } => {
            eprintln!("Connected, end-to-end encrypted.");
            eprintln!("Verification words: {}", words.join(" "));
        }
        Event::Progress { done, total } => {
            let pct = done.saturating_mul(100).checked_div(total).unwrap_or(100);
            if last.swap(pct, Ordering::Relaxed) != pct {
                eprint!("\r{pct:3}%  {} of {}", human(done), human(total));
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

async fn send(
    server: String,
    paths: Vec<PathBuf>,
    strong: bool,
) -> Result<(), totaldrop_core::Error> {
    eprintln!("Preparing…");
    let opts = SendOptions {
        server,
        words: if strong { 3 } else { 2 },
    };
    let prepared = PreparedSend::new(&opts, paths).await?;
    let (entries, total) = prepared.totals();
    if prepared.skipped() > 0 {
        eprintln!(
            "Skipping {} symbolic link(s) or special file(s).",
            prepared.skipped()
        );
    }
    eprintln!("Sending {entries} item(s), {}.", human(total));
    eprintln!();
    eprintln!("On the other device, run:");
    println!("totaldrop receive {}", prepared.code());
    eprintln!();
    let report = prepared.run(printer()).await?;
    if report.files_failed > 0 {
        eprintln!(
            "Done, but {} file(s) did not arrive intact and were discarded.",
            report.files_failed
        );
    } else {
        eprintln!("Done: {} file(s) sent and verified.", report.files_ok);
    }
    Ok(())
}

async fn receive(
    server: String,
    code: String,
    out: Option<PathBuf>,
    yes: bool,
) -> Result<(), totaldrop_core::Error> {
    let dest = match out {
        Some(d) => d,
        None => std::env::current_dir()?,
    };
    eprintln!("Connecting…");
    let offer = IncomingOffer::connect(&ReceiveOptions { server }, &code).await?;
    let s = offer.summary().clone();
    eprintln!("Connected, end-to-end encrypted.");
    eprintln!("Verification words: {}", offer.words().join(" "));
    eprintln!(
        "Offered: {} file(s), {} folder(s), {} in total:",
        s.files,
        s.dirs,
        human(s.total_size)
    );
    for name in s.top_level.iter().take(10) {
        eprintln!("  {name}");
    }
    if s.top_level.len() > 10 {
        eprintln!("  … and {} more", s.top_level.len().saturating_sub(10));
    }
    let shown = dest.display().to_string();
    let accepted = yes
        || tokio::task::spawn_blocking(move || {
            eprint!("Save into {shown}? [y/N] ");
            let _ = std::io::stderr().flush();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).is_ok()
                && matches!(
                    line.trim().to_lowercase().as_str(),
                    "y" | "yes" | "e" | "evet"
                )
        })
        .await
        .unwrap_or(false);
    if !accepted {
        offer.decline().await?;
        eprintln!("Declined.");
        return Ok(());
    }
    let report = offer.accept(&dest, printer()).await?;
    for p in &report.placed {
        eprintln!("Saved {}", p.display());
    }
    if report.files_failed > 0 {
        eprintln!(
            "{} file(s) did not arrive intact and were discarded.",
            report.files_failed
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
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
            runtime.block_on(send(server(cli.server), paths, strong))
        }
        Command::Receive { code, out, yes } => {
            runtime.block_on(receive(server(cli.server), code, out, yes))
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
