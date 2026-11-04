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

/// How long a relay link may send with nothing at all coming back: no
/// datagram, no pong to the pings it sends meanwhile ([`LIVENESS`]). An active QUIC connection answers within a round
/// trip, so a link that stays silent this long is dead, and QUIC gives up
/// after 30 seconds.
const SILENCE: Duration = Duration::from_secs(10);

/// How often the link pings the relay while datagrams went out with nothing
/// back. The relay answers even when the other device is not there yet, so
/// a link that waits for it is not taken for a dead one (#185); a dead
/// socket answers nothing.
const LIVENESS: Duration = Duration::from_secs(2);

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
    /// How long datagrams may go out with nothing at all coming back
    /// before the connection counts as dead.
    silence: Duration,
}

/// The relays' limits for one connection (protocol notes, item 23): the
/// link keeps them over all its connections, so that its own reconnects do
/// not get round them. The relays count per connection only.
const LIMITS: Limits = Limits {
    time: Duration::from_secs(6 * 60 * 60),
    bytes: 32 * 1024 * 1024 * 1024,
    refused: 10,
    silence: SILENCE,
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
/// pause (#185): both relays end the other side's connection when one
/// side's closes, and QUIC rides out a short gap. The relay lets a new
/// sign-in with the same key replace a socket that died unnoticed (#190).
/// After [`Limits::refused`] tries in a row that never signed
/// in, the link gives up. Failures are otherwise silent: the relay is only
/// one of the paths. A connection that dies without either end noticing
/// is given up once datagrams go out and nothing comes back for 10 seconds
/// ([`SILENCE`]).
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
                run(
                    &url,
                    &own,
                    peer,
                    &mut pipes,
                    &mut sent,
                    &mut signed_in,
                    limits.silence,
                ),
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
/// setting `signed_in` once the relay accepted the endpoint key. It ends
/// when datagrams went out and nothing came back for `silence`.
async fn run(
    url: &str,
    own: &EndpointKey,
    peer: [u8; 32],
    pipes: &mut RelayPipes,
    sent: &mut u64,
    signed_in: &mut bool,
    silence: Duration,
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
    let mut liveness = tokio::time::interval(LIVENESS);
    liveness.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    liveness.tick().await;
    // Since when datagrams went out with nothing coming back. A dead TCP
    // connection is not reported for minutes; this notices it in time.
    let mut unanswered: Option<tokio::time::Instant> = None;
    let far = tokio::time::Instant::now()
        .checked_add(LIMITS.time)
        .ok_or(Error::Connect("relay: time".into()))?;
    loop {
        let dead = unanswered
            .and_then(|t| t.checked_add(silence))
            .unwrap_or(far);
        tokio::select! {
            () = tokio::time::sleep_until(dead) => return Ok(Ended::Relay),
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
                unanswered.get_or_insert_with(tokio::time::Instant::now);
                // A dead socket whose send buffer is full blocks the write.
                let write = async {
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
                        .map_err(|e| Error::Connect(format!("relay: {e}")))
                };
                match tokio::time::timeout(silence, write).await {
                    Ok(written) => written?,
                    Err(_) => return Ok(Ended::Relay),
                }
            }
            msg = ws.next() => match msg {
                // The relay reads the answer to learn how much it still
                // holds for this device (#190); it goes back at once.
                Some(Ok(Message::Binary(b))) if wire::is_probe(&b) => {
                    unanswered = None;
                    match tokio::time::timeout(silence, ws.send(Message::Binary(b))).await {
                        Ok(written) => written.map_err(|e| Error::Connect(format!("relay: {e}")))?,
                        Err(_) => return Ok(Ended::Relay),
                    }
                }
                Some(Ok(Message::Binary(b))) => {
                    unanswered = None;
                    if let Some((from, datagrams)) = wire::parse_frame(&b)
                        && from == peer
                    {
                        for d in datagrams {
                            let _ = pipes.incoming.try_send(d.to_vec());
                        }
                    }
                }
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => unanswered = None,
                _ => return Ok(Ended::Relay),
            },
            _ = liveness.tick(), if unanswered.is_some() => {
                match tokio::time::timeout(silence, ws.send(Message::Ping(Vec::new().into()))).await {
                    Ok(written) => written.map_err(|e| Error::Connect(format!("relay: {e}")))?,
                    Err(_) => return Ok(Ended::Relay),
                }
            }
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
        let (_endpoint, pipes, _) =
            crate::transport::endpoint(socket, &own, &peer.public()).unwrap();
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
        let (_endpoint, pipes, _) =
            crate::transport::endpoint(socket, &own, &peer.public()).unwrap();
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
            silence: SILENCE,
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
            silence: SILENCE,
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

    /// A connection that died without either end noticing (NAT rebinding,
    /// a Wi-Fi switch) is dropped once datagrams go out and nothing at all
    /// comes back for `silence`, and the link connects again; the relay
    /// lets the new sign-in replace the dead socket (#190). Before, the
    /// link kept writing into the dead socket past QUIC's idle timeout.
    #[tokio::test]
    async fn link_connects_again_when_the_relay_goes_silent() {
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
            silence: Duration::from_secs(1),
            ..LIMITS
        };
        let _link = spawn_with(url, own, peer.public(), pipes, gate, limits);
        let mut ws = sign_in(&relay).await;
        // Nothing sent, nothing owed: a quiet link is not dropped.
        let quiet = tokio::time::timeout(Duration::from_secs(2), relay.accept()).await;
        assert!(quiet.is_err(), "an idle link connected again");
        // Datagrams go out; the relay takes them and never answers.
        out.send(vec![1; 100]).await.unwrap();
        let _frame = ws.next().await.unwrap().unwrap();
        let again = tokio::time::timeout(Duration::from_secs(4), relay.accept()).await;
        assert!(
            again.is_ok_and(|r| r.is_ok()),
            "the link stayed on a silent relay"
        );
        drop(ws);
    }

    /// A link that waits on the relay for the other device stays: the relay
    /// drops its datagrams while the other device is not there, but answers
    /// its pings (#185 review). Before, the link took the missing answers
    /// for a dead socket and connected again after `silence`, and the
    /// relay then also ended the other device's fresh connection.
    #[tokio::test]
    async fn link_waiting_for_the_other_device_stays() {
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
            silence: Duration::from_secs(3),
            ..LIMITS
        };
        let _link = spawn_with(url, own, peer.public(), pipes, gate, limits);
        let mut ws = sign_in(&relay).await;
        // The relay reads everything, so it answers pings, and forwards
        // nothing.
        let reader = tokio::spawn(async move { while let Some(Ok(_)) = ws.next().await {} });
        let sending = tokio::spawn(async move {
            loop {
                if out.send(vec![1; 100]).await.is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });
        let again = tokio::time::timeout(Duration::from_secs(8), relay.accept()).await;
        sending.abort();
        reader.abort();
        assert!(
            again.is_err(),
            "a link that the relay answers connected again"
        );
    }

    /// The link sends each relay probe straight back, unchanged and in
    /// order, between datagrams, so the relay knows the device has read
    /// everything before it (#190). A near miss is not a probe: it is
    /// dropped and the link stays.
    #[tokio::test]
    async fn link_sends_probes_back() {
        let relay = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/v0/relay", relay.local_addr().unwrap());
        let (own, peer) = (
            Arc::new(EndpointKey::generate().unwrap()),
            EndpointKey::generate().unwrap(),
        );
        let (_out, pipes) = pipes();
        let gate = Gate::default();
        gate.open();
        let _link = spawn_with(url, own, peer.public(), pipes, gate, LIMITS);
        let mut ws = sign_in(&relay).await;
        let (first, second) = (wire::probe(&[5; 16]), wire::probe(&[6; 16]));
        let near = [b"TDR0probX".as_slice(), &[5; 16]].concat();
        for m in [
            wire::frame(&peer.public(), &[1; 50]),
            near,
            first.clone(),
            wire::frame(&peer.public(), &[2; 50]),
            second.clone(),
        ] {
            ws.send(Message::Binary(m.into())).await.unwrap();
        }
        for want in [first, second] {
            let back = tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    match ws.next().await {
                        Some(Ok(Message::Binary(b))) => break Some(b),
                        Some(Ok(_)) => {}
                        _ => break None,
                    }
                }
            })
            .await
            .expect("the link did not send the probe back")
            .expect("the link went away");
            assert_eq!(back.as_ref(), want.as_slice());
        }
        let again = tokio::time::timeout(Duration::from_secs(1), relay.accept()).await;
        assert!(again.is_err(), "the link connected again");
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
            silence: SILENCE,
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
