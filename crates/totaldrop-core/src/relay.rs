//! Client side of the relay (spec section 6): address discovery and the
//! link that carries QUIC datagrams when no direct path works.

use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use totaldrop_proto::relay as wire;

use crate::error::{Error, Result};
use crate::path::RelayPipes;
use crate::transport::EndpointKey;

/// The project's relay. Address discovery answers on UDP at the same host and
/// port as this URL.
pub const DEFAULT_RELAY: &str = "wss://relay.totaldrop.app/v0/relay";

/// How long one discovery probe waits for its answer.
const PROBE_WAIT: Duration = Duration::from_millis(700);

/// How often an idle relay link pings, so proxies keep it open.
const KEEPALIVE: Duration = Duration::from_secs(20);

/// How long connecting and signing in to the relay may take.
const CONNECT_WAIT: Duration = Duration::from_secs(10);

/// Check a relay URL the same way as a rendezvous server address:
/// `wss://`, or `ws://` only on this machine.
pub fn check_url(url: &str) -> Result<()> {
    crate::rendezvous::check_url(url).map_err(|_| Error::Connect("invalid relay address".into()))
}

/// The UDP address of `url`'s discovery service: same host and port.
async fn discovery_address(url: &str, ipv4: bool) -> Option<SocketAddr> {
    let uri: tokio_tungstenite::tungstenite::http::Uri = url.parse().ok()?;
    let host = uri
        .host()?
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_owned();
    let port = uri.port_u16().unwrap_or(match uri.scheme_str() {
        Some("ws") => 80,
        _ => 443,
    });
    tokio::net::lookup_host((host.as_str(), port))
        .await
        .ok()?
        .find(|a| a.is_ipv4() == ipv4)
}

/// Ask the relay at `url` which public address `socket` has: two probes at
/// most, sent from the socket QUIC will use, so the answer is the mapping
/// the other device must aim at. `None` when the relay does not answer (for
/// example when UDP is blocked).
pub async fn discover(socket: &UdpSocket, url: &str) -> Option<SocketAddr> {
    let local = socket.local_addr().ok()?;
    let target = discovery_address(url, local.is_ipv4()).await?;
    // A second handle on the same socket; QUIC gets the original.
    let clone = socket.try_clone().ok()?;
    clone.set_nonblocking(true).ok()?;
    let udp = tokio::net::UdpSocket::from_std(clone).ok()?;
    for _ in 0..2 {
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).ok()?;
        udp.send_to(&wire::probe(&nonce), target).await.ok()?;
        let deadline = tokio::time::Instant::now().checked_add(PROBE_WAIT)?;
        let mut buf = [0u8; 64];
        while let Ok(Ok((n, from))) =
            tokio::time::timeout_at(deadline, udp.recv_from(&mut buf)).await
        {
            if from == target
                && let Some(addr) = buf.get(..n).and_then(|a| wire::parse_answer(a, &nonce))
            {
                return Some(addr);
            }
        }
    }
    None
}

/// Ends the relay link when dropped.
#[derive(Debug)]
pub struct Link(tokio::task::JoinHandle<()>);

impl Drop for Link {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Connect to the relay at `url` as `own`, and carry datagrams between the
/// pipes and the device with endpoint key `peer` until the link is dropped.
/// Failures are silent: the relay is only one of the paths.
pub fn spawn(url: String, own: Arc<EndpointKey>, peer: [u8; 32], pipes: RelayPipes) -> Link {
    Link(tokio::spawn(async move {
        let _ = run(&url, &own, peer, pipes).await;
    }))
}

async fn run(url: &str, own: &EndpointKey, peer: [u8; 32], mut pipes: RelayPipes) -> Result<()> {
    check_url(url)?;
    let mut ws = tokio::time::timeout(CONNECT_WAIT, async {
        let mut ws = crate::rendezvous::open_ws(url, wire::MAX_MESSAGE).await?;
        let challenge = match ws.next().await {
            Some(Ok(Message::Binary(b))) => wire::parse_challenge(&b),
            _ => None,
        }
        .ok_or(Error::Connect("relay: no challenge".into()))?;
        let signature = own.sign(&wire::signed_bytes(&challenge))?;
        ws.send(Message::Binary(
            wire::hello(&own.public(), &signature).into(),
        ))
        .await
        .map_err(|e| Error::Connect(format!("relay: {e}")))?;
        match ws.next().await {
            Some(Ok(Message::Binary(b))) if wire::is_ok(&b) => Ok(ws),
            _ => Err(Error::Connect("relay refused the endpoint key".into())),
        }
    })
    .await
    .map_err(|_| Error::Connect("relay: timed out".into()))??;

    let mut ping = tokio::time::interval(KEEPALIVE);
    ping.tick().await;
    loop {
        tokio::select! {
            out = pipes.outgoing.recv() => {
                let Some(datagram) = out else { break };
                ws.send(Message::Binary(wire::frame(&peer, &datagram).into()))
                    .await
                    .map_err(|e| Error::Connect(format!("relay: {e}")))?;
            }
            msg = ws.next() => match msg {
                Some(Ok(Message::Binary(b))) => {
                    if let Some((from, datagram)) = wire::parse_frame(&b)
                        && from == peer
                    {
                        let _ = pipes.incoming.try_send(datagram.to_vec());
                    }
                }
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                _ => break,
            },
            _ = ping.tick() => {
                ws.send(Message::Ping(Vec::new().into()))
                    .await
                    .map_err(|e| Error::Connect(format!("relay: {e}")))?;
            }
        }
    }
    let _ = ws.close(None).await;
    Ok(())
}
