//! `totaldrop-relay`: run the relay.
//!
//! Usage: `totaldrop-relay [WEBSOCKET_ADDRESS] [DISCOVERY_ADDRESS]`, default
//! `127.0.0.1:4100` for both. In production a TLS terminator serves
//! `wss://<host>/v0/relay/<key A>-<key B>` on TCP 443 and passes it to the
//! WebSocket address, and discovery listens on UDP 443 of the public address,
//! because clients probe the same host and port as the relay URL.

use std::net::SocketAddr;

fn parse(arg: Option<String>, default: &str) -> Option<SocketAddr> {
    arg.unwrap_or_else(|| default.into()).parse().ok()
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let mut args = std::env::args().skip(1);
    let (Some(ws), Some(udp)) = (
        parse(args.next(), "127.0.0.1:4100"),
        parse(args.next(), "127.0.0.1:4100"),
    ) else {
        eprintln!("usage: totaldrop-relay [WEBSOCKET_ADDRESS:PORT] [DISCOVERY_ADDRESS:PORT]");
        return std::process::ExitCode::from(2);
    };
    match totaldrop_relay::spawn(ws, udp, totaldrop_relay::Config::default()).await {
        Ok((ws, udp, _relay, task)) => {
            // The only things ever printed: where we listen. Nothing about
            // clients is logged.
            eprintln!("totaldrop-relay: WebSocket on {ws}, discovery on udp {udp}");
            tokio::select! {
                _ = task => {}
                _ = tokio::signal::ctrl_c() => {}
            }
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("cannot listen: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
