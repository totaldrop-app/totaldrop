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

/// The project's relay.
pub const DEFAULT_RELAY: &str = "wss://relay.totaldrop.app/v0/relay";

/// The STUN server used to find this device's public address.
pub const DEFAULT_STUN: &str = wire::DEFAULT_STUN;

/// How long one discovery probe waits for its answer.
const PROBE_WAIT: Duration = Duration::from_millis(700);

/// How often an idle relay link pings, so proxies keep it open.
const KEEPALIVE: Duration = Duration::from_secs(20);

/// How long connecting and signing in to the relay may take.
const CONNECT_WAIT: Duration = Duration::from_secs(10);

/// How long a link waits before connecting again after losing the relay,
/// at first; it doubles up to [`RETRY_MAX`].
const RETRY_FIRST: Duration = Duration::from_millis(500);

/// The longest wait between two tries.
const RETRY_MAX: Duration = Duration::from_secs(4);

/// A connection that lasted this long counts as having worked: the next
/// wait starts again from [`RETRY_FIRST`].
const RETRY_RESET: Duration = Duration::from_secs(30);

/// What one side of a transfer may use the relay for, all its connections
/// together.
#[derive(Debug, Clone, Copy)]
struct Limits {
    /// From the first try on.
    time: Duration,
    /// Bytes sent.
    bytes: u64,
    /// Tries in a row that did not get signed in.
    refused: u32,
}

/// The relays' limits for one connection (protocol notes, item 23): the
/// link keeps them over all its connections, so that its own reconnects do
/// not get round them. The relays count per connection only.
const LIMITS: Limits = Limits {
    time: Duration::from_secs(6 * 60 * 60),
    bytes: 32 * 1024 * 1024 * 1024,
    refused: 10,
};

/// Check a relay URL the same way as a rendezvous server address:
/// `wss://`, or `ws://` only on this machine.
pub fn check_url(url: &str) -> Result<()> {
    crate::rendezvous::check_url(url)
        .map_err(|_| Error::Connect(crate::error::msg::INVALID_RELAY.into()))
}

/// The address of the STUN server `host:port`, in the socket's family.
async fn stun_address(server: &str, ipv4: bool) -> Option<SocketAddr> {
    tokio::net::lookup_host(server)
        .await
        .ok()?
        .find(|a| a.is_ipv4() == ipv4)
}

/// Ask the STUN server `server` (`host:port`) which public address `socket`
/// has: two requests at most, sent from the socket QUIC will use, so the
/// answer is the mapping the other device must aim at. `None` when nothing
/// answers (for example when UDP is blocked).
pub async fn discover(socket: &UdpSocket, server: &str) -> Option<SocketAddr> {
    let local = socket.local_addr().ok()?;
    let target = stun_address(server, local.is_ipv4()).await?;
    // A second handle on the same socket; QUIC gets the original.
    let clone = socket.try_clone().ok()?;
    clone.set_nonblocking(true).ok()?;
    let udp = tokio::net::UdpSocket::from_std(clone).ok()?;
    for _ in 0..2 {
        let mut id = [0u8; 12];
        getrandom::fill(&mut id).ok()?;
        udp.send_to(&wire::stun_request(&id), target).await.ok()?;
        let deadline = tokio::time::Instant::now().checked_add(PROBE_WAIT)?;
        let mut buf = [0u8; 512];
        while let Ok(Ok((n, from))) =
            tokio::time::timeout_at(deadline, udp.recv_from(&mut buf)).await
        {
            if from == target
                && let Some(addr) = buf.get(..n).and_then(|a| wire::parse_stun_response(a, &id))
            {
                return Some(addr);
            }
        }
    }
    None
}

/// Holds a relay link back until a connection attempt needs the relay, so
/// that a transfer a direct path carries never contacts the relay at all.
#[derive(Debug, Clone, Default)]
pub struct Gate(Arc<GateInner>);

#[derive(Debug, Default)]
struct GateInner {
    notify: tokio::sync::Notify,
    opened: std::sync::atomic::AtomicBool,
}

impl Gate {
    /// Let the link connect. Calling it again does nothing.
    pub fn open(&self) {
        if !self
            .0
            .opened
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            // Stores a permit when the link is not waiting yet.
            self.0.notify.notify_one();
        }
    }

    /// Whether [`Gate::open`] was called.
    pub fn is_open(&self) -> bool {
        self.0.opened.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn opened(&self) {
        if !self.is_open() {
            self.0.notify.notified().await;
        }
    }
}

/// A relay link. It connects only once its [`Gate`] opens, and stops when
/// dropped.
#[derive(Debug)]
pub struct Link(tokio::task::JoinHandle<()>);

impl Drop for Link {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Connect to the relay at `url` as `own`, and carry datagrams between the
/// pipes and the device with endpoint key `peer` until the link is dropped.
/// A lost or refused connection to the relay is tried again, with a growing
/// pause (#185): the Cloudflare relay ends the other side's connection when
/// one side's closes, and QUIC rides out a short gap. A connection that
/// dies without the relay noticing is only replaced once the relay lets the
/// old one go. After [`Limits::refused`] tries in a row that never signed
/// in, the link gives up. Failures are otherwise silent: the relay is only
/// one of the paths.
pub fn spawn(
    url: String,
    own: Arc<EndpointKey>,
    peer: [u8; 32],
    pipes: RelayPipes,
    gate: Gate,
) -> Link {
    spawn_with(url, own, peer, pipes, gate, LIMITS)
}

fn spawn_with(
    url: String,
    own: Arc<EndpointKey>,
    peer: [u8; 32],
    mut pipes: RelayPipes,
    gate: Gate,
    limits: Limits,
) -> Link {
    Link(tokio::spawn(async move {
        gate.opened().await;
        if check_url(&url).is_err() {
            return;
        }
        let Some(end) = tokio::time::Instant::now().checked_add(limits.time) else {
            return;
        };
        let mut sent = 0u64;
        let mut refused = 0u32;
        let mut pause = RETRY_FIRST;
        loop {
            let started = tokio::time::Instant::now();
            if started >= end || sent >= limits.bytes || refused >= limits.refused {
                return;
            }
            let mut signed_in = false;
            let ended = tokio::time::timeout_at(
                end,
                run(&url, &own, peer, &mut pipes, &mut sent, &mut signed_in),
            )
            .await;
            match ended {
                // The endpoint is gone, or the time is up.
                Ok(Ok(Ended::Outgoing)) | Err(_) => return,
                Ok(Ok(Ended::Relay) | Err(_)) => {}
            }
            refused = if signed_in {
                0
            } else {
                refused.saturating_add(1)
            };
            if started.elapsed() >= RETRY_RESET {
                pause = RETRY_FIRST;
            }
            tokio::time::sleep(pause).await;
            pause = pause.saturating_mul(2).min(RETRY_MAX);
        }
    }))
}

/// Why a connection to the relay ended without an error.
enum Ended {
    /// This side has nothing more to send: the endpoint was dropped.
    Outgoing,
    /// The relay closed the connection.
    Relay,
}

/// One connection to the relay, counting the bytes it sends in `sent` and
/// setting `signed_in` once the relay accepted the endpoint key.
async fn run(
    url: &str,
    own: &EndpointKey,
    peer: [u8; 32],
    pipes: &mut RelayPipes,
    sent: &mut u64,
    signed_in: &mut bool,
) -> Result<Ended> {
    // Both sides meet under the name of their two keys.
    let url = format!(
        "{}/{}",
        url.trim_end_matches('/'),
        wire::pair_name(&own.public(), &peer)
    );
    let mut ws = tokio::time::timeout(CONNECT_WAIT, async {
        let mut ws = crate::rendezvous::open_ws(&url, wire::MAX_MESSAGE).await?;
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
    *signed_in = true;

    let mut ping = tokio::time::interval(KEEPALIVE);
    ping.tick().await;
    loop {
        tokio::select! {
            out = pipes.outgoing.recv() => {
                let Some(first) = out else {
                    let _ = ws.close(None).await;
                    return Ok(Ended::Outgoing);
                };
                // Send whatever else is already queued in the same message.
                let mut batch = wire::Batch::new(&peer);
                batch.push(&first);
                let mut carry = None;
                while let Ok(next) = pipes.outgoing.try_recv() {
                    if !batch.push(&next) {
                        carry = Some(next);
                        break;
                    }
                }
                for message in [Some(batch.finish()), carry.map(|d| wire::frame(&peer, &d))]
                    .into_iter()
                    .flatten()
                {
                    if message.len() > 32 {
                        *sent = sent.saturating_add(u64::try_from(message.len()).unwrap_or(u64::MAX));
                        ws.feed(Message::Binary(message.into()))
                            .await
                            .map_err(|e| Error::Connect(format!("relay: {e}")))?;
                    }
                }
                ws.flush()
                    .await
                    .map_err(|e| Error::Connect(format!("relay: {e}")))?;
            }
            msg = ws.next() => match msg {
                Some(Ok(Message::Binary(b))) => {
                    if let Some((from, datagrams)) = wire::parse_frame(&b)
                        && from == peer
                    {
                        for d in datagrams {
                            let _ = pipes.incoming.try_send(d.to_vec());
                        }
                    }
                }
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                _ => return Ok(Ended::Relay),
            },
            _ = ping.tick() => {
                ws.send(Message::Ping(Vec::new().into()))
                    .await
                    .map_err(|e| Error::Connect(format!("relay: {e}")))?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    /// A link whose gate stays shut never connects to the relay; once the
    /// gate opens, it does (#63).
    #[tokio::test]
    async fn link_connects_only_after_its_gate_opens() {
        let relay = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/v0/relay", relay.local_addr().unwrap());
        let (own, peer) = (
            Arc::new(EndpointKey::generate().unwrap()),
            EndpointKey::generate().unwrap(),
        );
        let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let (_endpoint, pipes) = crate::transport::endpoint(socket, &own, &peer.public()).unwrap();
        let gate = Gate::default();
        let _link = spawn(url, own, peer.public(), pipes, gate.clone());

        let early = tokio::time::timeout(Duration::from_millis(500), relay.accept()).await;
        assert!(
            early.is_err(),
            "the relay was contacted before it was needed"
        );

        gate.open();
        let late = tokio::time::timeout(Duration::from_secs(5), relay.accept()).await;
        assert!(late.is_ok_and(|r| r.is_ok()));
    }

    /// A link that loses its connection to the relay, or does not get one,
    /// connects again (#185): the relay ends the other side's connection
    /// too when one side's drops, so neither side may give up after one.
    #[tokio::test]
    async fn link_connects_again_after_losing_the_relay() {
        let relay = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/v0/relay", relay.local_addr().unwrap());
        let (own, peer) = (
            Arc::new(EndpointKey::generate().unwrap()),
            EndpointKey::generate().unwrap(),
        );
        let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let (_endpoint, pipes) = crate::transport::endpoint(socket, &own, &peer.public()).unwrap();
        let gate = Gate::default();
        gate.open();
        let link = spawn(url, own, peer.public(), pipes, gate);

        for _ in 0..3 {
            let (stream, _) = tokio::time::timeout(Duration::from_secs(5), relay.accept())
                .await
                .expect("the link did not connect again")
                .unwrap();
            drop(stream);
        }

        // Once dropped, the link stays away.
        drop(link);
        let after = tokio::time::timeout(Duration::from_secs(6), relay.accept()).await;
        assert!(after.is_err(), "a dropped link connected again");
    }

    /// A stand-in relay at `listener` that signs in whoever connects,
    /// without checking anything.
    async fn sign_in(listener: &tokio::net::TcpListener) -> crate::rendezvous::Ws {
        let (stream, _) = tokio::time::timeout(Duration::from_secs(5), listener.accept())
            .await
            .expect("the link did not connect")
            .unwrap();
        let mut ws =
            tokio_tungstenite::accept_async(tokio_tungstenite::MaybeTlsStream::Plain(stream))
                .await
                .unwrap();
        ws.send(Message::Binary(wire::challenge(&[7; 32]).into()))
            .await
            .unwrap();
        let _hello = ws.next().await.unwrap().unwrap();
        ws.send(Message::Binary(wire::ok().into())).await.unwrap();
        ws
    }

    fn pipes() -> (tokio::sync::mpsc::Sender<Vec<u8>>, RelayPipes) {
        let (out_tx, outgoing) = tokio::sync::mpsc::channel(16);
        let (incoming, _) = tokio::sync::mpsc::channel(16);
        (out_tx, RelayPipes { outgoing, incoming })
    }

    /// Connecting again does not get round the relay's time limit: the
    /// link keeps it over all its connections.
    #[tokio::test]
    async fn link_keeps_the_time_limit_over_its_connections() {
        let relay = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/v0/relay", relay.local_addr().unwrap());
        let (own, peer) = (
            Arc::new(EndpointKey::generate().unwrap()),
            EndpointKey::generate().unwrap(),
        );
        let (_out, pipes) = pipes();
        let gate = Gate::default();
        gate.open();
        let limits = Limits {
            time: Duration::from_millis(1200),
            bytes: u64::MAX,
            refused: u32::MAX,
        };
        let _link = spawn_with(url, own, peer.public(), pipes, gate, limits);
        // Tries at 0 and 0.5 seconds; the next would be at 1.5.
        for _ in 0..2 {
            let (stream, _) = tokio::time::timeout(Duration::from_secs(5), relay.accept())
                .await
                .expect("the link did not connect again")
                .unwrap();
            drop(stream);
        }
        let after = tokio::time::timeout(Duration::from_secs(4), relay.accept()).await;
        assert!(after.is_err(), "the link connected after its time was up");
    }

    /// Connecting again does not get round the relay's byte limit either.
    #[tokio::test]
    async fn link_keeps_the_byte_limit_over_its_connections() {
        let relay = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/v0/relay", relay.local_addr().unwrap());
        let (own, peer) = (
            Arc::new(EndpointKey::generate().unwrap()),
            EndpointKey::generate().unwrap(),
        );
        let (out, pipes) = pipes();
        let gate = Gate::default();
        gate.open();
        let limits = Limits {
            time: Duration::from_secs(3600),
            bytes: 1000,
            refused: u32::MAX,
        };
        let _link = spawn_with(url, own, peer.public(), pipes, gate, limits);
        let mut ws = sign_in(&relay).await;
        out.send(vec![1; 1200]).await.unwrap();
        let frame = ws.next().await.unwrap().unwrap();
        assert!(frame.len() > 1200);
        drop(ws);
        let after = tokio::time::timeout(Duration::from_secs(3), relay.accept()).await;
        assert!(
            after.is_err(),
            "the link connected after its bytes were spent"
        );
    }

    /// A relay that keeps refusing the link is not tried for ever: after
    /// `refused` tries in a row that never signed in, the link gives up.
    #[tokio::test]
    async fn link_gives_up_on_a_relay_that_keeps_refusing() {
        let relay = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/v0/relay", relay.local_addr().unwrap());
        let (own, peer) = (
            Arc::new(EndpointKey::generate().unwrap()),
            EndpointKey::generate().unwrap(),
        );
        let (_out, pipes) = pipes();
        let gate = Gate::default();
        gate.open();
        let limits = Limits {
            time: Duration::from_secs(3600),
            bytes: u64::MAX,
            refused: 3,
        };
        let _link = spawn_with(url, own, peer.public(), pipes, gate, limits);
        // Tries at 0, 0.5 and 1.5 seconds; the next would be at 3.5.
        for _ in 0..3 {
            let (stream, _) = tokio::time::timeout(Duration::from_secs(5), relay.accept())
                .await
                .expect("the link did not try again")
                .unwrap();
            drop(stream);
        }
        let after = tokio::time::timeout(Duration::from_secs(4), relay.accept()).await;
        assert!(after.is_err(), "the link kept trying a relay that refuses");
    }
}
