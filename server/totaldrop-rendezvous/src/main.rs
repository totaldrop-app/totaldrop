//! `totaldrop-rendezvous`: run the rendezvous server.
//!
//! Usage: `totaldrop-rendezvous [ADDRESS]`, default `127.0.0.1:4000`.
//! Put a TLS 1.3 terminator in front of it for anything but local testing.

use std::net::SocketAddr;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:4000".into());
    let Ok(addr) = addr.parse::<SocketAddr>() else {
        eprintln!("usage: totaldrop-rendezvous [ADDRESS:PORT]");
        return std::process::ExitCode::from(2);
    };
    match totaldrop_rendezvous::spawn(addr, totaldrop_rendezvous::Config::default()).await {
        Ok((local, _server, task)) => {
            // The only thing ever printed: where we listen. No client data is logged.
            eprintln!("totaldrop-rendezvous listening on {local}");
            let _ = task.await;
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("cannot listen on {addr}: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
