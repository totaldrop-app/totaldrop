//! `totaldrop`: the command-line client. In this early skeleton only `code`
//! works; `send` and `receive` arrive with the internet core (phase 1).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use totaldrop_proto::TransferCode;

#[derive(Parser)]
#[command(
    name = "totaldrop",
    version,
    about = "Send any file to anyone, anywhere, end-to-end encrypted."
)]
struct Cli {
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
    },
    /// Check a transfer code and show its canonical form.
    Code {
        /// A transfer code, for example 7-harbor-rocket.
        code: String,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Code { code } => match TransferCode::parse(&code) {
            Ok(c) => {
                println!("code:      {c}");
                println!("canonical: {}", c.canonical());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("totaldrop: invalid code: {e:?}");
                ExitCode::from(2)
            }
        },
        Command::Send { .. } | Command::Receive { .. } => {
            eprintln!("totaldrop: sending and receiving are not implemented yet (phase 1)");
            ExitCode::from(2)
        }
    }
}
