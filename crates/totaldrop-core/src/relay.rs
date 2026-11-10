// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Client side of the relay (spec section 6): address discovery and the
//! link that carries QUIC datagrams when no direct path works.

use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use totaldrop_proto::relay as wire;

use crate::error::{Error, Result};
use crate::path::{Fallback, RelayPipes};
use crate::transport::EndpointKey;

/// The project's relay.
pub const DEFAULT_RELAY: &str = "wss://relay.totaldrop.app/v0/relay";

/// The STUN server used to find this device's public address.
pub const DEFAULT_STUN: &str = wire::DEFAULT_STUN;

/// How long one discovery probe waits for its answer.
const PROBE_WAIT: Duration = Duration::from_millis(700);
/// How much longer a STUN round waits once one server has answered.
const AFTER_ANSWER: Duration = Duration::from_millis(150);

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
    /// How often an idle connection is pinged.
    keepalive: Duration,
}

/// The relays' limits for one connection (protocol notes, item 23): the
/// link keeps them over all its connections, so that its own reconnects do
/// not get round them. The relays count per connection only.
const LIMITS: Limits = Limits {
    time: Duration::from_secs(6 * 60 * 60),
    bytes: 32 * 1024 * 1024 * 1024,
    refused: 10,
    silence: SILENCE,
    keepalive: KEEPALIVE,
};

/// Check a relay URL the same way as a rendezvous server address:
/// `wss://`, or `ws://` only on this machine.
pub fn check_url(url: &str) -> Result<()> {
    crate::rendezvous::check_url(url)
        .map_err(|_| Error::Connect(crate::error::msg::INVALID_RELAY.into()))
}

/// Ask the STUN server `server` (`host:port`) which public addresses
/// `socket` has: over IPv4, and over IPv6 too when the socket is
/// dual-stack (#403). Both requests go out from one handle on the socket
/// QUIC will use, so each answer is the mapping the other device must aim
/// at, and one loop reads every answer and matches it to its request by
/// server address and transaction id: two readers on one socket would
/// take each other's answers. Once one server has answered, the other
/// gets [`AFTER_ANSWER`] more; a second round asks again only when
/// nothing answered. Empty when nothing answers (for example when UDP is
/// blocked). IPv6 is asked only where a dial would go out from an
/// announced address, so the hardware (EUI-64) address never reaches a
/// STUN server.
pub async fn discover(socket: &UdpSocket, server: &str) -> Vec<SocketAddr> {
    let Ok(found) = tokio::net::lookup_host(server).await else {
        return Vec::new();
    };
    let found: Vec<SocketAddr> = found.collect();
    let ipv6 = socket.local_addr().is_ok_and(|a| a.is_ipv6());
    discover_at(socket, stun_targets(&found, ipv6)).await
}

/// The servers to ask: the first IPv4 address, and the first IPv6 one
/// when the socket is dual-stack and a dial there would go out from an
/// address this device announces (never its hardware, EUI-64, address).
fn stun_targets(found: &[SocketAddr], ipv6: bool) -> Vec<SocketAddr> {
    [
        found.iter().find(|a| a.is_ipv4()).copied(),
        found
            .iter()
            .find(|a| a.is_ipv6())
            .copied()
            .filter(|a| ipv6 && crate::transport::dialable(*a)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// [`discover`], asking each of `pending` once per round.
async fn discover_at(socket: &UdpSocket, mut pending: Vec<SocketAddr>) -> Vec<SocketAddr> {
    let Ok(local) = socket.local_addr() else {
        return Vec::new();
    };
    let ipv6 = local.is_ipv6();
    // A dual-stack socket sends to IPv4 through IPv4-mapped addresses and
    // reports them as such; compare in canonical form.
    let Some(udp) = socket
        .try_clone()
        .ok()
        .filter(|c| c.set_nonblocking(true).is_ok())
        .and_then(|c| tokio::net::UdpSocket::from_std(c).ok())
    else {
        return Vec::new();
    };
    let mut seen = Vec::new();
    for _ in 0..2 {
        let mut asked: Vec<(SocketAddr, [u8; 12])> = Vec::new();
        for server in &pending {
            let mut id = [0u8; 12];
            if getrandom::fill(&mut id).is_err() {
                return seen;
            }
            // A family without a route fails here at once and is dropped.
            if udp
                .send_to(
                    &wire::stun_request(&id),
                    crate::path::in_family(*server, ipv6),
                )
                .await
                .is_ok()
            {
                asked.push((*server, id));
            }
        }
        pending.retain(|s| asked.iter().any(|(a, _)| a == s));
        let Some(mut deadline) = tokio::time::Instant::now().checked_add(PROBE_WAIT) else {
            return seen;
        };
        let mut buf = [0u8; 512];
        while !asked.is_empty()
            && let Ok(Ok((n, from))) =
                tokio::time::timeout_at(deadline, udp.recv_from(&mut buf)).await
        {
            let from = crate::path::canonical(from);
            let answer = asked.iter().position(|(server, id)| {
                *server == from
                    && buf
                        .get(..n)
                        .and_then(|m| wire::parse_stun_response(m, id))
                        .is_some()
            });
            if let Some(i) = answer {
                let (server, id) = asked.swap_remove(i);
                if let Some(addr) = buf.get(..n).and_then(|m| wire::parse_stun_response(m, &id)) {
                    seen.push(crate::path::canonical(addr));
                }
                pending.retain(|s| *s != server);
                // One answer is enough to go on with; the other family
                // gets only a short while more.
                if let Some(soon) = tokio::time::Instant::now().checked_add(AFTER_ANSWER) {
                    deadline = deadline.min(soon);
                }
            }
        }
        if pending.is_empty() || !seen.is_empty() {
            break;
        }
    }
    seen
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

/// How long a parked link keeps a connection that carries nothing,
/// because its connection moved back to the direct path: then it closes
/// it, and connects again only if the connection moves once more.
const PARKED_IDLE: Duration = Duration::from_secs(30);

/// How often a parked link checks for [`PARKED_IDLE`].
const IDLE_CHECK: Duration = Duration::from_secs(1);

/// A relay link. It connects only once its [`Gate`] opens, and stops when
/// dropped.
#[derive(Debug)]
pub struct Link {
    task: tokio::task::JoinHandle<()>,
    park: Arc<Park>,
}

/// What [`Link::park`] tells the link's task.
#[derive(Debug, Default)]
struct Park {
    fallback: std::sync::OnceLock<Fallback>,
    notify: tokio::sync::Notify,
}

impl Park {
    /// Whether the link may carry datagrams now: always, unless it was
    /// parked for a direct connection that is on its direct path.
    fn carries(&self) -> bool {
        self.fallback.get().is_none_or(Fallback::moved)
    }
}

impl Link {
    /// Keep the link for `fallback`'s direct connection (#185): close any
    /// connection to the relay now, and from then on connect only while
    /// that connection has moved onto the relay. Datagrams queued
    /// meanwhile (such as the closing of a connection that lost the race)
    /// are dropped. Calling it again does nothing.
    pub fn park(&self, fallback: &Fallback) {
        if self.park.fallback.set(fallback.clone()).is_ok() {
            self.park.notify.notify_one();
        }
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        self.task.abort();
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
    let park = Arc::new(Park::default());
    let parked = park.clone();
    let task = tokio::spawn(async move {
        let park = parked;
        gate.opened().await;
        if park.fallback.get().is_some() {
            // The gate opened because a parked link's connection moved:
            // what waited behind it belongs to the race that connection
            // won (such as the closing of the attempt that lost), not to
            // the connection, so it must not reach the other device under
            // the connection's address. A datagram the connection queued
            // in the meantime may go too; QUIC sends it again.
            while pipes.outgoing.try_recv().is_ok() {}
        }
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
            // A parked link waits for a datagram that has to go through the
            // relay before it connects.
            let first = if park.fallback.get().is_some() {
                let wanted = tokio::time::timeout_at(end, async {
                    while let Some(d) = pipes.outgoing.recv().await {
                        if park.carries() {
                            return Some(d);
                        }
                    }
                    None
                })
                .await;
                match wanted {
                    Ok(Some(d)) => Some(d),
                    _ => return,
                }
            } else {
                None
            };
            let mut signed_in = false;
            let ended = tokio::time::timeout_at(
                end,
                run(
                    &url,
                    &own,
                    peer,
                    Conn {
                        pipes: &mut pipes,
                        sent: &mut sent,
                        signed_in: &mut signed_in,
                        silence: limits.silence,
                        keepalive: limits.keepalive,
                        first,
                        park: &park,
                    },
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
    });
    Link { task, park }
}

/// Why a connection to the relay ended without an error.
enum Ended {
    /// This side has nothing more to send: the endpoint was dropped.
    Outgoing,
    /// The relay closed the connection.
    Relay,
}

/// The state one connection to the relay works with.
struct Conn<'a> {
    pipes: &'a mut RelayPipes,
    /// Bytes sent over all connections.
    sent: &'a mut u64,
    /// Set once the relay accepted the endpoint key.
    signed_in: &'a mut bool,
    silence: Duration,
    keepalive: Duration,
    /// A datagram to send as soon as the connection is up.
    first: Option<Vec<u8>>,
    park: &'a Park,
}

/// Close `ws`, waiting at most `wait`: a relay that stopped reading never
/// takes the close, and the link must not wait on it (#424). Whether the
/// close got through does not matter; the relay ends the connection when
/// the socket goes.
async fn close(ws: &mut crate::rendezvous::Ws, wait: Duration) {
    let _ = tokio::time::timeout(wait, ws.close(None)).await;
}

/// One connection to the relay. It ends when datagrams went out and
/// nothing came back for `silence`, and, once parked, when its connection
/// is on its direct path again: at once when parked, and after
/// [`PARKED_IDLE`] without sending later.
async fn run(url: &str, own: &EndpointKey, peer: [u8; 32], c: Conn<'_>) -> Result<Ended> {
    let Conn {
        pipes,
        sent,
        signed_in,
        silence,
        keepalive,
        first,
        park,
    } = c;
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
    if let Some(d) = first {
        let message = wire::frame(&peer, &d);
        *sent = sent.saturating_add(u64::try_from(message.len()).unwrap_or(u64::MAX));
        match tokio::time::timeout(silence, ws.send(Message::Binary(message.into()))).await {
            Ok(written) => written.map_err(|e| Error::Connect(format!("relay: {e}")))?,
            Err(_) => return Ok(Ended::Relay),
        }
    }
    let mut last_out = tokio::time::Instant::now();

    let mut ping = tokio::time::interval(keepalive);
    ping.tick().await;
    let mut idle = tokio::time::interval(IDLE_CHECK);
    idle.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    idle.tick().await;
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
                    close(&mut ws, silence).await;
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
                last_out = tokio::time::Instant::now();
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
            () = park.notify.notified() => {
                if !park.carries() {
                    close(&mut ws, silence).await;
                    return Ok(Ended::Relay);
                }
            }
            _ = idle.tick(), if park.fallback.get().is_some() => {
                if !park.carries() && last_out.elapsed() >= PARKED_IDLE {
                    close(&mut ws, silence).await;
                    return Ok(Ended::Relay);
                }
            }
            _ = ping.tick() => {
                // Like every other write here: a relay that stopped
                // reading must not hold the loop past `silence` (#424).
                match tokio::time::timeout(silence, ws.send(Message::Ping(Vec::new().into()))).await {
                    Ok(written) => written.map_err(|e| Error::Connect(format!("relay: {e}")))?,
                    Err(_) => return Ok(Ended::Relay),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    /// A STUN server on IPv4 loopback that first sends a stray answer with
    /// a wrong transaction id, then the real one, as `seen` (#403).
    async fn stun_server(seen: SocketAddr) -> SocketAddr {
        let server = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr = server.local_addr().unwrap();
        tokio::spawn(async move {
            let mut buf = [0u8; 512];
            while let Ok((n, from)) = server.recv_from(&mut buf).await {
                if let Some(id) = buf.get(..n).and_then(wire::parse_stun_request) {
                    let stray = wire::stun_response(&[0xee; 12], "198.51.100.9:9".parse().unwrap());
                    let _ = server.send_to(&stray, from).await;
                    let _ = server.send_to(&wire::stun_response(&id, seen), from).await;
                }
            }
        });
        addr
    }

    /// #403: the answer over IPv4 is found on the socket QUIC uses, also
    /// when it is dual-stack (it sends to a mapped address and hears back
    /// from one), and an answer to another request is not taken for it.
    #[tokio::test]
    async fn stun_finds_the_public_address_on_either_socket() {
        let seen: SocketAddr = "203.0.113.5:40000".parse().unwrap();
        let server = stun_server(seen).await;
        let plain = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        assert_eq!(discover(&plain, &server.to_string()).await, vec![seen]);
        let ours = crate::transport::bind_socket().unwrap();
        assert_eq!(discover(&ours, &server.to_string()).await, vec![seen]);
    }

    /// #403: nothing answering gives no address, within the two rounds.
    #[tokio::test]
    async fn stun_without_an_answer_gives_nothing() {
        let quiet = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server = quiet.local_addr().unwrap().to_string();
        let ours = crate::transport::bind_socket().unwrap();
        let start = tokio::time::Instant::now();
        assert!(discover(&ours, &server).await.is_empty());
        assert!(start.elapsed() < PROBE_WAIT * 3);
    }

    /// #403: STUN over IPv6 goes out only where a dial would, so a
    /// hardware (EUI-64) source address never reaches the STUN server.
    #[test]
    fn stun_asks_only_where_a_dial_would() {
        let v4: SocketAddr = "192.0.2.1:3478".parse().unwrap();
        let link_local: SocketAddr = "[fe80::1]:3478".parse().unwrap();
        assert_eq!(stun_targets(&[v4, link_local], true), vec![v4]);
        let loopback: SocketAddr = "[::1]:3478".parse().unwrap();
        assert_eq!(stun_targets(&[v4, loopback], true), vec![v4, loopback]);
        assert_eq!(stun_targets(&[v4, loopback], false), vec![v4]);
    }

    /// #403: once one server has answered (IPv4, say), a silent one (an
    /// IPv6 server behind a dead route) holds `HINTS` back only briefly,
    /// not for two whole rounds.
    #[tokio::test]
    async fn a_silent_server_barely_delays_the_answer() {
        let seen: SocketAddr = "203.0.113.5:40000".parse().unwrap();
        let answers = stun_server(seen).await;
        let quiet = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let silent = quiet.local_addr().unwrap();
        let ours = crate::transport::bind_socket().unwrap();
        let start = tokio::time::Instant::now();
        assert_eq!(discover_at(&ours, vec![answers, silent]).await, vec![seen]);
        assert!(start.elapsed() < PROBE_WAIT / 2, "{:?}", start.elapsed());
    }

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
            keepalive: KEEPALIVE,
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
            keepalive: KEEPALIVE,
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

    /// #424: a link whose relay stops reading is given up within
    /// `silence` and connects again, also when the write that finds the
    /// send buffer full is the keepalive ping. The relay here keeps
    /// pinging (so the link counts it as alive and writes nothing else)
    /// but reads nothing, so the link's answers fill its send buffer.
    /// Before, the keepalive ping was written without a time limit and
    /// the connection loop stayed in it until the kernel gave up on the
    /// TCP connection (minutes): the link neither connected again nor
    /// read anything.
    #[tokio::test]
    async fn link_connects_again_when_a_ping_finds_the_relay_stuck() {
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
            silence: Duration::from_secs(1),
            keepalive: Duration::from_millis(200),
            ..LIMITS
        };
        let _link = spawn_with(url, own, peer.public(), pipes, gate, limits);
        let mut ws = sign_in(&relay).await;
        let pinging = tokio::spawn(async move {
            while ws.send(Message::Ping(vec![0; 125].into())).await.is_ok() {}
        });
        let again = tokio::time::timeout(Duration::from_secs(20), relay.accept()).await;
        pinging.abort();
        assert!(
            again.is_ok_and(|r| r.is_ok()),
            "the link stayed in a ping to a relay that does not read"
        );
    }

    /// #424: closing a connection to a relay that stopped reading does
    /// not hold the link either. A parked link closes its connection at
    /// once; when its connection then moves onto the relay, it connects
    /// again. Before, the close waited without a time limit for the full
    /// send buffer to drain, so the link never connected again.
    #[tokio::test]
    async fn a_parked_link_does_not_wait_on_a_stuck_close() {
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
        let link = spawn_with(url, own, peer.public(), pipes, gate, limits);
        let mut ws = sign_in(&relay).await;
        // As above: the link's answers to these pings fill its send buffer.
        let pinging = tokio::spawn(async move {
            while ws.send(Message::Ping(vec![0; 125].into())).await.is_ok() {}
        });
        // No event says the buffer is full. Too short a wait could only let
        // the test pass without the fix (the close would go through), never
        // fail with it; without the fix it failed every time with this one.
        tokio::time::sleep(Duration::from_secs(1)).await;
        let fallback = Fallback::new(tokio::sync::mpsc::channel(1).0);
        link.park(&fallback);
        // Parked on the direct path, the link closes; then the connection
        // moves onto the relay and has a datagram for it. The pause lets
        // the link see the park before the move (nothing to wait on here:
        // the close never reaches the stand-in relay).
        tokio::time::sleep(Duration::from_millis(300)).await;
        fallback.set_moved(true);
        out.send(vec![2; 100]).await.unwrap();
        let again = tokio::time::timeout(Duration::from_secs(20), relay.accept()).await;
        pinging.abort();
        assert!(
            again.is_ok_and(|r| r.is_ok()),
            "the link stayed in closing a connection to a relay that does not read"
        );
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

    /// A parked link (#185) closes the connection it has at once, sends
    /// nothing to the relay while its connection is direct, and connects
    /// again, with the waiting datagram, as soon as the connection moves
    /// onto the relay.
    #[tokio::test]
    async fn a_parked_link_connects_only_while_moved() {
        let relay = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/v0/relay", relay.local_addr().unwrap());
        let (own, peer) = (
            Arc::new(EndpointKey::generate().unwrap()),
            EndpointKey::generate().unwrap(),
        );
        let (out, pipes) = pipes();
        let gate = Gate::default();
        gate.open();
        let link = spawn_with(url, own, peer.public(), pipes, gate, LIMITS);
        let mut ws = sign_in(&relay).await;

        let fallback = Fallback::new(tokio::sync::mpsc::channel(1).0);
        link.park(&fallback);
        let closed = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match ws.next().await {
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(_)) => {}
                }
            }
        })
        .await;
        assert!(closed.is_ok(), "parking left the connection open");

        // On the direct path: what is queued goes nowhere.
        out.send(vec![1; 100]).await.unwrap();
        let early = tokio::time::timeout(Duration::from_secs(3), relay.accept()).await;
        assert!(
            early.is_err(),
            "a parked link connected for a direct connection"
        );

        fallback.set_moved(true);
        out.send(vec![2; 100]).await.unwrap();
        let mut ws = sign_in(&relay).await;
        let frame = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match ws.next().await {
                    Some(Ok(Message::Binary(b))) => break b,
                    Some(Ok(_)) => {}
                    other => panic!("no datagram: {other:?}"),
                }
            }
        })
        .await
        .expect("the waiting datagram was not sent");
        let (to, datagrams) = wire::parse_frame(&frame).unwrap();
        assert_eq!(to, peer.public());
        assert_eq!(datagrams, vec![&[2u8; 100][..]]);

        // Back on the direct path, the idle connection closes after
        // PARKED_IDLE, not a keep-alive later (#185 review).
        fallback.set_moved(false);
        let started = tokio::time::Instant::now();
        let closed = tokio::time::timeout(PARKED_IDLE + Duration::from_secs(5), async {
            loop {
                match ws.next().await {
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(_)) => {}
                }
            }
        })
        .await;
        assert!(closed.is_ok(), "an idle parked link stayed connected");
        assert!(started.elapsed() < PARKED_IDLE + IDLE_CHECK + Duration::from_secs(2));
    }

    /// What waited behind a parked link's shut gate belongs to the race
    /// its connection won; when the connection moves, it is dropped, not
    /// sent to the other device under the connection's address (#185
    /// review).
    #[tokio::test]
    async fn a_parked_link_drops_what_waited_behind_its_gate() {
        let relay = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/v0/relay", relay.local_addr().unwrap());
        let (own, peer) = (
            Arc::new(EndpointKey::generate().unwrap()),
            EndpointKey::generate().unwrap(),
        );
        let (out, pipes) = pipes();
        let gate = Gate::default();
        let link = spawn_with(url, own, peer.public(), pipes, gate.clone(), LIMITS);
        let fallback = Fallback::new(tokio::sync::mpsc::channel(1).0);
        link.park(&fallback);
        out.send(vec![1; 100]).await.unwrap();

        fallback.set_moved(true);
        gate.open();
        tokio::time::sleep(Duration::from_millis(200)).await;
        out.send(vec![2; 100]).await.unwrap();
        let mut ws = sign_in(&relay).await;
        let frame = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match ws.next().await {
                    Some(Ok(Message::Binary(b))) => break b,
                    Some(Ok(_)) => {}
                    other => panic!("no datagram: {other:?}"),
                }
            }
        })
        .await
        .expect("the connection's datagram was not sent");
        let (_, datagrams) = wire::parse_frame(&frame).unwrap();
        assert_eq!(datagrams, vec![&[2u8; 100][..]]);
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
            keepalive: KEEPALIVE,
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
