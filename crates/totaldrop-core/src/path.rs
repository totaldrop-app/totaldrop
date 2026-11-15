// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! One QUIC socket, two kinds of path.
//!
//! [`PathSocket`] is what the QUIC endpoint sends and receives through. It
//! passes ordinary datagrams to the real UDP socket, so direct paths work as
//! before, and it sends datagrams addressed to [`RELAY_PEER`] through the
//! relay instead. Datagrams the relay delivers appear to come from
//! [`RELAY_PEER`]. QUIC therefore treats the relay as one more address of the
//! other device, and the existing race between paths picks whichever answers
//! first.
//!
//! Every endpoint talks to exactly one peer (endpoint keys are single-use),
//! so one stand-in address is enough.
//!
//! A direct connection can also move onto the relay once it is up
//! ([`Fallback`]): from then on, datagrams for its address go through the
//! relay, and datagrams from the relay appear to come from that address. QUIC
//! sees the same address throughout, so neither side has to migrate, and
//! each side can move on its own clock, and back (#185).

use std::collections::VecDeque;
use std::io::{self, IoSliceMut};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::task::{Context, Poll};
use std::time::Duration;

use quinn::udp::{EcnCodepoint, RecvMeta, Transmit};
use quinn::{AsyncUdpSocket, UdpPoller};
use tokio::sync::mpsc;

/// The stand-in address of the other device through the relay. 240.0.0.0/4
/// is reserved (RFC 1112) and never routed, so it cannot collide with a real
/// address.
pub const RELAY_PEER: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(240, 84, 68, 1), 1));

/// Whether `addr` is [`RELAY_PEER`]. On a dual-stack socket (#403) QUIC
/// reports IPv4 addresses as IPv4-mapped IPv6 ones (`::ffff:a.b.c.d`), the
/// stand-in among them, so every comparison with it goes through here.
pub fn is_relay(addr: SocketAddr) -> bool {
    canonical(addr) == RELAY_PEER
}

/// `addr` with an IPv4-mapped IPv6 address turned back into IPv4.
pub fn canonical(addr: SocketAddr) -> SocketAddr {
    SocketAddr::new(addr.ip().to_canonical(), addr.port())
}

/// `addr` as a socket of the given family reports it: IPv4 addresses
/// become IPv4-mapped IPv6 ones on an IPv6 (dual-stack) socket, as quinn
/// does for the addresses it dials.
pub fn in_family(addr: SocketAddr, ipv6: bool) -> SocketAddr {
    match addr {
        SocketAddr::V4(a) if ipv6 => {
            SocketAddr::V6(SocketAddrV6::new(a.ip().to_ipv6_mapped(), a.port(), 0, 0))
        }
        _ => addr,
    }
}

/// Datagrams waiting for the relay in each direction. QUIC handles loss, so a
/// full queue simply drops.
const QUEUE: usize = 1024;

/// The relay's side of a [`PathSocket`]: datagrams to forward, and a way to
/// hand over the ones that arrive.
#[derive(Debug)]
pub struct RelayPipes {
    /// Datagrams QUIC sent to [`RELAY_PEER`].
    pub outgoing: mpsc::Receiver<Vec<u8>>,
    /// Datagrams from the other device, to deliver to QUIC.
    pub incoming: mpsc::Sender<Vec<u8>>,
}

/// How long a direct connection may go without a single datagram from the
/// other device before it moves onto the relay (#185). Both sides send a
/// keep-alive every 2 seconds when they have nothing else to send, so this
/// is four of them lost in a row. Should only one direction die, the side
/// that stops hearing moves first; the other then stops hearing too and
/// follows one wait later, well within QUIC's 30-second idle timeout.
pub const FALLBACK_SILENCE: Duration = Duration::from_secs(8);

/// How many of the last datagrams sent through the relay while moved a
/// side sends again on the direct path when it comes back (#435). One that
/// reaches the other device and asks for an acknowledgement is enough: the
/// acknowledgement resets QUIC's doubled wait and shows everything older as
/// lost, which QUIC then resends at once. What a sender sends while its
/// path is gone is mostly such probes; a few allow for one being lost, or
/// being a bare acknowledgement.
const REPLAY: usize = 4;

/// How long one datagram sent again may wait for room in the socket.
const REPLAY_WAIT: Duration = Duration::from_millis(100);

/// How often [`Fallback::watch`] looks at the clock.
const WATCH_TICK: Duration = Duration::from_millis(500);

/// A tick this much later than due means this device itself was paused
/// (an app frozen in the background): the quiet was its own, not the
/// path's.
const PAUSED: Duration = Duration::from_secs(2);

/// Moves a direct connection onto the relay when its path goes quiet: a
/// Wi-Fi that drops, a NAT mapping that expires, a network that lets the
/// handshake through and then stops carrying anything (#185). It moves
/// back as soon as a datagram comes from the direct address again, so a
/// peer that was only paused, or did not move itself, brings the
/// connection back. Cloning shares the state.
#[derive(Debug, Clone)]
pub struct Fallback(Arc<FallbackState>);

/// Maximum number of local connection ID reset tokens remembered per
/// single-use endpoint (#512). Quinn issues up to eight active CIDs at a
/// time; keeping more covers CID rotation while bounding memory.
const MAX_RESET_TOKENS: usize = 32;

/// [`quinn::crypto::HmacKey`] for an endpoint's stateless-reset key that
/// remembers the 16-byte reset tokens derived from it (#512).
pub(crate) struct ResetTokens {
    key: ring::hmac::Key,
    issued: Mutex<VecDeque<[u8; 16]>>,
}

impl core::fmt::Debug for ResetTokens {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ResetTokens").finish_non_exhaustive()
    }
}

impl ResetTokens {
    pub(crate) fn new() -> Result<Arc<Self>, crate::error::Error> {
        let mut raw = [0u8; 64];
        getrandom::fill(&mut raw).map_err(|_| crate::error::Error::Random)?;
        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &raw);
        zeroize::Zeroize::zeroize(&mut raw);
        Ok(Arc::new(Self {
            key,
            issued: Mutex::new(VecDeque::with_capacity(MAX_RESET_TOKENS)),
        }))
    }

    /// Whether `segment` is an outgoing QUIC stateless reset carrying one of
    /// this endpoint's reset tokens in its last 16 bytes (RFC 9000, section
    /// 10.3).
    fn is_reset(&self, segment: &[u8]) -> bool {
        if segment.len() < 21 || segment.first().is_some_and(|b| b & 0x80 != 0) {
            return false;
        }
        let Some(tail) = segment
            .len()
            .checked_sub(16)
            .and_then(|i| segment.get(i..))
            .and_then(|s| <[u8; 16]>::try_from(s).ok())
        else {
            return false;
        };
        self.issued
            .lock()
            .is_ok_and(|issued| issued.iter().any(|t| t == &tail))
    }
}

impl quinn::crypto::HmacKey for ResetTokens {
    fn sign(&self, data: &[u8], out: &mut [u8]) {
        let tag = ring::hmac::sign(&self.key, data);
        out.copy_from_slice(tag.as_ref());
        if let Some(token) = tag
            .as_ref()
            .get(..16)
            .and_then(|s| <[u8; 16]>::try_from(s).ok())
            && let Ok(mut issued) = self.issued.lock()
            && !issued.contains(&token)
        {
            if issued.len() >= MAX_RESET_TOKENS {
                issued.pop_front();
            }
            issued.push_back(token);
        }
    }

    fn signature_len(&self) -> usize {
        32
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), quinn::crypto::CryptoError> {
        ring::hmac::verify(&self.key, data, signature).map_err(|_| quinn::crypto::CryptoError)
    }
}

/// Maximum number of post-drain `CONNECTION_CLOSE` replays sent for a single
/// stopped session, spaced at power-of-two incoming packet counts (`1, 2, 4,
/// 8, 16, 32`) per RFC 9000, section 10.2.1 (#512).
const MAX_CLOSE_REPLAYS: u8 = 6;

/// Saved `CONNECTION_CLOSE` state for a stopped session (#512).
#[derive(Debug, Default)]
struct Closing {
    /// Canonical address of the authenticated peer whose connection is
    /// closing with `stopped`.
    peer: Option<SocketAddr>,
    /// Latest encrypted short-header `CONNECTION_CLOSE` packet sent to
    /// `peer`, re-sent in place of post-drain stateless resets (RFC 9000,
    /// section 10.2.1).
    saved: Option<Vec<u8>>,
    /// Number of post-drain datagrams from `peer` that triggered a stateless
    /// reset check.
    post_drain_hits: u32,
    /// Number of `saved` replays sent so far (at most [`MAX_CLOSE_REPLAYS`]).
    replays_sent: u8,
}

#[derive(Debug)]
struct FallbackState {
    /// The direct address being watched: the connection's remote address,
    /// followed while the connection is direct.
    direct: RwLock<Option<SocketAddr>>,
    /// Whether datagrams for `direct` now go through the relay.
    moved: AtomicBool,
    /// When a datagram from `direct` last arrived, in milliseconds after
    /// `start`.
    heard: AtomicU64,
    /// When the connection last moved onto the relay, likewise.
    moved_at: AtomicU64,
    start: tokio::time::Instant,
    /// The queue to the relay link; closed once the link has given up
    /// (refused, unreachable, or out of its limits) or was never started.
    relay: mpsc::Sender<Vec<u8>>,
    /// The last [`REPLAY`] datagrams sent through the relay while moved,
    /// as QUIC gave them (#435).
    rerouted: Mutex<VecDeque<Rerouted>>,
    /// How many datagrams were sent again on coming back, for tests.
    replayed: AtomicU64,
    /// How many datagrams went through the relay while moved, for tests.
    #[cfg(test)]
    rerouted_total: AtomicU64,
    /// The real UDP socket, to send `rerouted` again on coming back.
    socket: OnceLock<Arc<dyn AsyncUdpSocket>>,
    /// Stateless-reset tokens issued by the QUIC endpoint on this socket.
    reset_tokens: Option<Arc<ResetTokens>>,
    /// Fast-path flag set once [`Fallback::arm_closing`] is called.
    closing_armed: AtomicBool,
    /// Saved `CONNECTION_CLOSE` packet for post-drain peers (#512).
    closing: Mutex<Closing>,
    /// Woken when a post-drain packet from the peer has been answered with
    /// the saved `CONNECTION_CLOSE` (#512).
    answered: tokio::sync::Notify,
}

impl Fallback {
    #[cfg(test)]
    pub(crate) fn new(relay: mpsc::Sender<Vec<u8>>) -> Self {
        Self::with_reset_tokens(relay, None)
    }

    pub(crate) fn with_reset_tokens(
        relay: mpsc::Sender<Vec<u8>>,
        reset_tokens: Option<Arc<ResetTokens>>,
    ) -> Self {
        Self(Arc::new(FallbackState {
            direct: RwLock::new(None),
            moved: AtomicBool::new(false),
            heard: AtomicU64::new(0),
            moved_at: AtomicU64::new(0),
            start: tokio::time::Instant::now(),
            relay,
            rerouted: Mutex::new(VecDeque::with_capacity(REPLAY)),
            replayed: AtomicU64::new(0),
            #[cfg(test)]
            rerouted_total: AtomicU64::new(0),
            socket: OnceLock::new(),
            reset_tokens,
            closing_armed: AtomicBool::new(false),
            closing: Mutex::new(Closing::default()),
            answered: tokio::sync::Notify::new(),
        }))
    }

    pub(crate) fn downgrade(&self) -> FallbackWeak {
        FallbackWeak(Arc::downgrade(&self.0))
    }

    /// Record that `peer` is about to be closed with `stopped` after its
    /// cancel notice went unacknowledged (#512).
    pub(crate) fn arm_closing(&self, peer: SocketAddr) {
        if let Ok(mut closing) = self.0.closing.lock() {
            closing.peer = Some(canonical(peer));
        }
        self.0.closing_armed.store(true, Ordering::Release);
    }

    /// Whether [`Fallback::arm_closing`] armed post-drain linger for a stop
    /// whose cancel notice had not been acknowledged by the peer.
    pub(crate) fn lingers(&self) -> bool {
        self.0.closing_armed.load(Ordering::Acquire)
    }

    /// Wait until at least one post-drain packet from the closing peer has
    /// been answered with the saved `CONNECTION_CLOSE` (#512).
    pub(crate) async fn answered(&self) {
        let notified = self.0.answered.notified();
        if self.0.closing.lock().is_ok_and(|c| c.replays_sent > 0) {
            return;
        }
        notified.await;
    }

    /// Inspect an outgoing `transmit` once closing is armed (#512). Returns
    /// `None` for normal packets (saving the closing connection's non-GSO
    /// short-header packet to `peer`) and `Some(replacement)` when `transmit`
    /// is a stateless reset that should be replaced with `CONNECTION_CLOSE`
    /// (or dropped when `None`).
    fn on_send(&self, transmit: &Transmit) -> Option<Option<Vec<u8>>> {
        if !self.0.closing_armed.load(Ordering::Acquire) {
            return None;
        }
        if transmit.segment_size.is_none()
            && self
                .0
                .reset_tokens
                .as_ref()
                .is_some_and(|rt| rt.is_reset(transmit.contents))
        {
            let replacement = self.0.closing.lock().ok().and_then(|mut c| {
                // Never answer third-party addresses with the saved packet or
                // a stateless reset: a single-use endpoint talks only to its
                // authenticated peer.
                if c.peer != Some(canonical(transmit.destination)) {
                    return None;
                }
                let saved = c.saved.as_ref()?.clone();
                c.post_drain_hits = c.post_drain_hits.saturating_add(1);
                // Exponential packet-count backoff (RFC 9000, section 10.2.1)
                // also guarantees that two drained endpoints stop after two
                // exchanges instead of ping-ponging.
                if !c.post_drain_hits.is_power_of_two() || c.replays_sent >= MAX_CLOSE_REPLAYS {
                    return None;
                }
                c.replays_sent = c.replays_sent.saturating_add(1);
                Some(saved)
            });
            if replacement.is_some() {
                self.0.answered.notify_waiters();
            }
            return Some(replacement);
        }
        if transmit.segment_size.is_none()
            && transmit.contents.len() >= 21
            && transmit.contents.first().is_some_and(|b| b & 0x80 == 0)
            && let Ok(mut closing) = self.0.closing.lock()
            && closing.peer == Some(canonical(transmit.destination))
        {
            // Keep the first connection's 8-byte QUIC DCID (`1..9`) so a
            // losing dual-dial connection draining on the same socket cannot
            // overwrite the winning connection's `CONNECTION_CLOSE`.
            let same_dcid = closing
                .saved
                .as_ref()
                .is_none_or(|prev| prev.get(1..9) == transmit.contents.get(1..9));
            if same_dcid {
                closing.saved = Some(transmit.contents.to_vec());
            }
        }
        None
    }

    /// Note one segment of `transmit`, sent through the relay instead.
    fn rerouted(&self, transmit: &Transmit, segment: &[u8]) {
        #[cfg(test)]
        self.0.rerouted_total.fetch_add(1, Ordering::AcqRel);
        if let Ok(mut kept) = self.0.rerouted.lock() {
            if kept.len() >= REPLAY {
                kept.pop_front();
            }
            kept.push_back(Rerouted {
                destination: transmit.destination,
                // Sent again with QUIC's own ECN mark: without it QUIC
                // would take the acknowledged copy for a network that
                // clears marks and turn ECN off.
                ecn: transmit.ecn,
                src_ip: transmit.src_ip,
                datagram: segment.to_vec(),
            });
        }
    }

    /// How many datagrams went through the relay while moved so far.
    #[cfg(test)]
    pub(crate) fn rerouted_total(&self) -> u64 {
        self.0.rerouted_total.load(Ordering::Acquire)
    }

    /// How many datagrams were sent again on coming back so far.
    #[cfg(test)]
    pub(crate) fn replayed(&self) -> u64 {
        self.0.replayed.load(Ordering::Acquire)
    }

    /// Back on the direct path: send the last datagrams that went to the
    /// relay again, directly (#435). While moved, the other side may not
    /// have been on the relay at all (it never moved), so they were lost;
    /// and while the path was gone QUIC doubled its wait before each
    /// resend, so without this the transfer stands still until its next
    /// resend, 10 seconds or more. These are QUIC's own packets, already
    /// encrypted: the other side takes a copy it has seen as a duplicate
    /// and drops it (RFC 9000, section 12.3).
    async fn replay(&self) {
        let kept = match self.0.rerouted.lock() {
            Ok(mut kept) => std::mem::take(&mut *kept),
            Err(_) => return,
        };
        let Some(socket) = self.0.socket.get() else {
            return;
        };
        let direct = self.direct().map(canonical);
        let mut poller = None;
        for r in &kept {
            // Only to the address watched now: one QUIC left meanwhile may
            // belong to someone else by now.
            if Some(canonical(r.destination)) != direct {
                continue;
            }
            let transmit = Transmit {
                destination: r.destination,
                ecn: r.ecn,
                contents: &r.datagram,
                segment_size: None,
                src_ip: r.src_ip,
            };
            // Best effort, briefly: a socket that stays full drops it, as
            // the network could, and QUIC's own resend still follows.
            let _ = tokio::time::timeout(REPLAY_WAIT, async {
                loop {
                    match socket.try_send(&transmit) {
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            let p = poller.get_or_insert_with(|| socket.clone().create_io_poller());
                            if std::future::poll_fn(|cx| p.as_mut().poll_writable(cx))
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                        Ok(()) => {
                            self.0.replayed.fetch_add(1, Ordering::AcqRel);
                            return;
                        }
                        Err(_) => return,
                    }
                }
            })
            .await;
        }
    }

    /// Move onto the relay at `now`. What was kept from an earlier move
    /// is not sent again after this one.
    fn move_now(&self, now: u64) {
        if let Ok(mut kept) = self.0.rerouted.lock() {
            kept.clear();
        }
        self.0.moved_at.store(now, Ordering::Release);
        self.0.moved.store(true, Ordering::Release);
    }

    fn now(&self) -> u64 {
        u64::try_from(self.0.start.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    fn direct(&self) -> Option<SocketAddr> {
        self.0.direct.read().ok().and_then(|d| *d)
    }

    /// Whether datagrams for `addr` go through the relay instead.
    fn reroutes(&self, addr: SocketAddr) -> bool {
        self.0.moved.load(Ordering::Acquire)
            && self.direct().map(canonical) == Some(canonical(addr))
    }

    /// The address datagrams from the relay appear to come from: the
    /// watched direct address once a watch runs, whether or not this side
    /// moved, so that QUIC never sees a second address for the other
    /// device (quinn drops a client's packets from one and migrates a
    /// server to it); otherwise the stand-in.
    fn relay_source(&self) -> SocketAddr {
        self.direct().unwrap_or(RELAY_PEER)
    }

    /// Note datagrams that came in on the real socket.
    fn heard_from(&self, meta: &[RecvMeta]) {
        if let Some(direct) = self.direct()
            && meta.iter().any(|m| canonical(m.addr) == canonical(direct))
        {
            self.0.heard.store(self.now(), Ordering::Release);
        }
    }

    /// Whether the connection currently goes through the relay.
    pub fn moved(&self) -> bool {
        self.0.moved.load(Ordering::Acquire)
    }

    /// Say whether the connection is on the relay, as a watch would.
    #[cfg(test)]
    pub(crate) fn set_moved(&self, moved: bool) {
        self.0.moved.store(moved, Ordering::Release);
    }

    /// Watch the direct connection `conn` until it closes. Once nothing
    /// came from its address for `silence`, send what is addressed to it
    /// through the relay instead and open `gate`; as soon as something
    /// comes from the address again (the other side never moved, or was
    /// only paused), send directly again. Once both sides moved, nothing
    /// goes to the direct address any more, so the connection stays on the
    /// relay. The address follows QUIC's, so a peer whose NAT mapping
    /// changed is not taken for a silent one.
    pub async fn watch(
        &self,
        conn: quinn::Connection,
        gate: crate::relay::Gate,
        silence: Duration,
    ) {
        let silence = u64::try_from(silence.as_millis()).unwrap_or(u64::MAX);
        let mut tick = tokio::time::interval(WATCH_TICK);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut last = self.now();
        loop {
            tick.tick().await;
            if conn.close_reason().is_some() {
                return;
            }
            let now = self.now();
            let seen = Seen {
                now,
                last_tick: last,
                remote: conn.remote_address(),
                direct: self.direct(),
                heard: self.0.heard.load(Ordering::Acquire),
                moved: self.moved(),
                moved_at: self.0.moved_at.load(Ordering::Acquire),
                link: !self.0.relay.is_closed(),
            };
            last = now;
            match step(&seen, silence) {
                Step::Wait => {}
                Step::Follow(remote) => {
                    if let Ok(mut d) = self.0.direct.write() {
                        *d = Some(remote);
                    }
                    self.0.heard.store(now, Ordering::Release);
                }
                Step::Recount(heard) => self.0.heard.store(heard, Ordering::Release),
                Step::Move => {
                    // Mark the move before the link can deliver anything;
                    // outgoing datagrams wait in the queue until it is up.
                    self.move_now(now);
                    gate.open();
                }
                // The relay link closes itself once it has been idle.
                Step::Back => {
                    self.0.moved.store(false, Ordering::Release);
                    // Holds this loop for at most REPLAY × REPLAY_WAIT
                    // (0.4 s), well inside a tick's worth of slack; the
                    // next tick checks the close and the address as usual.
                    self.replay().await;
                }
            }
        }
    }
}

/// A datagram sent through the relay while moved, kept to send again.
#[derive(Debug)]
struct Rerouted {
    destination: SocketAddr,
    ecn: Option<EcnCodepoint>,
    src_ip: Option<IpAddr>,
    datagram: Vec<u8>,
}

/// After this device's own pause, how long it waits for the other side
/// before moving: short, since the other side's QUIC idle timer kept
/// running meanwhile.
const AFTER_PAUSE: u64 = 2_000;

/// How long after a move a datagram from the direct address can still be
/// one that was on its way before: it does not bring the side back.
const LATE: u64 = 500;

/// What [`Fallback::watch`] sees at one tick, times in milliseconds.
#[derive(Debug, Clone, Copy)]
struct Seen {
    now: u64,
    last_tick: u64,
    /// QUIC's remote address for the connection.
    remote: SocketAddr,
    /// The address watched so far.
    direct: Option<SocketAddr>,
    heard: u64,
    moved: bool,
    moved_at: u64,
    /// Whether the relay link still runs.
    link: bool,
}

/// What one tick decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Wait,
    /// Watch this address from now on (the first tick, or QUIC moved).
    Follow(SocketAddr),
    /// This device was paused: count the silence from this time instead.
    Recount(u64),
    Move,
    Back,
}

fn step(s: &Seen, silence: u64) -> Step {
    if s.direct != Some(s.remote) && !is_relay(s.remote) {
        return Step::Follow(s.remote);
    }
    let paused = u64::try_from(WATCH_TICK.saturating_add(PAUSED).as_millis()).unwrap_or(u64::MAX);
    if !s.link {
        // Nothing would carry the connection on the relay: the direct
        // path is the only chance left.
        return if s.moved { Step::Back } else { Step::Wait };
    }
    if s.moved {
        // A datagram that was already on its way when the side moved does
        // not show the direct path working again.
        return if s.heard > s.moved_at.saturating_add(LATE) {
            Step::Back
        } else {
            Step::Wait
        };
    }
    if s.now.saturating_sub(s.last_tick) >= paused {
        // The quiet was this device's own, not the path's.
        let from = s.now.saturating_sub(silence.saturating_sub(AFTER_PAUSE));
        return Step::Recount(from.max(s.heard));
    }
    if s.now.saturating_sub(s.heard) >= silence {
        Step::Move
    } else {
        Step::Wait
    }
}

/// A [`Fallback::watch`] that runs until dropped.
#[derive(Debug)]
pub struct Watch(tokio::task::JoinHandle<()>);

impl Watch {
    /// Run `fallback.watch(conn, gate, silence)` in the background.
    pub fn spawn(
        fallback: &Fallback,
        conn: quinn::Connection,
        gate: crate::relay::Gate,
        silence: Duration,
    ) -> Self {
        let fallback = fallback.clone();
        Self(tokio::spawn(async move {
            fallback.watch(conn, gate, silence).await;
        }))
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Non-owning handle to a [`Fallback`] while its [`PathSocket`] lives.
#[derive(Debug, Clone)]
pub(crate) struct FallbackWeak(std::sync::Weak<FallbackState>);

impl FallbackWeak {
    pub(crate) fn upgrade(&self) -> Option<Fallback> {
        self.0.upgrade().map(Fallback)
    }

    pub(crate) fn is_alive(&self) -> bool {
        self.0.strong_count() > 0
    }
}

/// A UDP socket that also reaches the other device through the relay.
#[derive(Debug)]
pub struct PathSocket {
    inner: Arc<dyn AsyncUdpSocket>,
    to_relay: mpsc::Sender<Vec<u8>>,
    from_relay: Mutex<mpsc::Receiver<Vec<u8>>>,
    fallback: Fallback,
    /// Whether `inner` is an IPv6 (dual-stack) socket, so that datagrams
    /// from the relay carry a source address in its family.
    ipv6: bool,
}

impl PathSocket {
    /// Wrap `inner`. The returned pipes connect it to a relay link. Until the
    /// link is up, datagrams for the relay wait in a short queue; once it is
    /// full, or when no link runs, they are dropped. The [`Fallback`] moves a
    /// direct connection onto the relay.
    pub fn new(inner: Arc<dyn AsyncUdpSocket>) -> (Arc<Self>, RelayPipes, Fallback) {
        Self::build(inner, None)
    }

    pub(crate) fn with_reset_tokens(
        inner: Arc<dyn AsyncUdpSocket>,
        reset_tokens: Arc<ResetTokens>,
    ) -> (Arc<Self>, RelayPipes, Fallback) {
        Self::build(inner, Some(reset_tokens))
    }

    fn build(
        inner: Arc<dyn AsyncUdpSocket>,
        reset_tokens: Option<Arc<ResetTokens>>,
    ) -> (Arc<Self>, RelayPipes, Fallback) {
        let (to_relay, outgoing) = mpsc::channel(QUEUE);
        let (incoming, from_relay) = mpsc::channel(QUEUE);
        let fallback = Fallback::with_reset_tokens(to_relay.clone(), reset_tokens);
        let _ = fallback.0.socket.set(inner.clone());
        let ipv6 = inner.local_addr().is_ok_and(|a| a.is_ipv6());
        (
            Arc::new(Self {
                inner,
                to_relay,
                from_relay: Mutex::new(from_relay),
                fallback: fallback.clone(),
                ipv6,
            }),
            RelayPipes { outgoing, incoming },
            fallback,
        )
    }

    fn send_one(&self, transmit: &Transmit, record_rerouted: bool) -> io::Result<()> {
        if is_relay(transmit.destination) {
            self.relay_send(transmit);
            Ok(())
        } else if self.fallback.reroutes(transmit.destination) {
            self.relay_send(transmit);
            if record_rerouted {
                let size = transmit
                    .segment_size
                    .unwrap_or(transmit.contents.len())
                    .max(1);
                for segment in transmit.contents.chunks(size) {
                    self.fallback.rerouted(transmit, segment);
                }
            }
            Ok(())
        } else {
            self.inner.try_send(transmit)
        }
    }

    fn relay_send(&self, transmit: &Transmit) {
        let size = transmit
            .segment_size
            .unwrap_or(transmit.contents.len())
            .max(1);
        for segment in transmit.contents.chunks(size) {
            // A full queue or a missing relay drops the datagram, as a busy
            // network would; QUIC retransmits.
            let _ = self.to_relay.try_send(segment.to_vec());
        }
    }

    fn poll_relay(
        &self,
        cx: &mut Context,
        bufs: &mut [IoSliceMut<'_>],
        meta: &mut [RecvMeta],
    ) -> Poll<usize> {
        let Ok(mut rx) = self.from_relay.lock() else {
            return Poll::Pending;
        };
        loop {
            match rx.poll_recv(cx) {
                Poll::Ready(Some(datagram)) => {
                    let (Some(buf), Some(m)) = (bufs.first_mut(), meta.first_mut()) else {
                        return Poll::Pending;
                    };
                    if datagram.len() > buf.len() {
                        // Larger than QUIC can take: drop it and look again.
                        continue;
                    }
                    if let Some(dst) = buf.get_mut(..datagram.len()) {
                        dst.copy_from_slice(&datagram);
                    }
                    *m = RecvMeta {
                        addr: in_family(self.fallback.relay_source(), self.ipv6),
                        len: datagram.len(),
                        stride: datagram.len(),
                        ecn: None,
                        dst_ip: None,
                    };
                    return Poll::Ready(1);
                }
                // No relay link (any more): only the direct path is left.
                Poll::Ready(None) | Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl AsyncUdpSocket for PathSocket {
    fn create_io_poller(self: Arc<Self>) -> Pin<Box<dyn UdpPoller>> {
        self.inner.clone().create_io_poller()
    }

    fn try_send(&self, transmit: &Transmit) -> io::Result<()> {
        if let Some(saved) = self.fallback.on_send(transmit) {
            // Post-drain packet after `close_stopped` (#512): answer with
            // the saved `CONNECTION_CLOSE` instead of a stateless reset
            // (RFC 9000, section 10.2.1), or drop the reset if none fits.
            if let Some(contents) = saved {
                let replacement = Transmit {
                    destination: transmit.destination,
                    ecn: None,
                    contents: &contents,
                    segment_size: None,
                    src_ip: transmit.src_ip,
                };
                let _ = self.send_one(&replacement, false);
            }
            return Ok(());
        }
        self.send_one(transmit, true)
    }

    fn poll_recv(
        &self,
        cx: &mut Context,
        bufs: &mut [IoSliceMut<'_>],
        meta: &mut [RecvMeta],
    ) -> Poll<io::Result<usize>> {
        if let Poll::Ready(n) = self.poll_relay(cx, bufs, meta) {
            return Poll::Ready(Ok(n));
        }
        let polled = self.inner.poll_recv(cx, bufs, meta);
        if let Poll::Ready(Ok(n)) = &polled {
            self.fallback.heard_from(meta.get(..*n).unwrap_or_default());
        }
        polled
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }

    fn max_transmit_segments(&self) -> usize {
        self.inner.max_transmit_segments()
    }

    fn max_receive_segments(&self) -> usize {
        self.inner.max_receive_segments()
    }

    fn may_fragment(&self) -> bool {
        self.inner.may_fragment()
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    const A: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 7), 4000));
    const B: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 8), 4001));
    const SILENCE: u64 = 8_000;

    fn seen() -> Seen {
        Seen {
            now: 20_000,
            last_tick: 19_500,
            remote: A,
            direct: Some(A),
            heard: 19_000,
            moved: false,
            moved_at: 0,
            link: true,
        }
    }

    #[test]
    fn the_first_tick_follows_quic() {
        let s = Seen {
            direct: None,
            ..seen()
        };
        assert_eq!(step(&s, SILENCE), Step::Follow(A));
    }

    /// A peer whose NAT mapping changed is followed, not taken for a
    /// silent one (#185 review); also while moved.
    #[test]
    fn a_new_address_is_followed() {
        for moved in [false, true] {
            let s = Seen {
                remote: B,
                heard: 0,
                moved,
                ..seen()
            };
            assert_eq!(step(&s, SILENCE), Step::Follow(B));
        }
        // QUIC never sees the stand-in once a watch runs; if it did, the
        // watched address stays.
        let s = Seen {
            remote: RELAY_PEER,
            ..seen()
        };
        assert_eq!(step(&s, SILENCE), Step::Wait);
    }

    /// #403: a moved connection's datagrams go to the relay, and the
    /// direct address counts as heard, whichever family a platform reports
    /// the address in.
    #[tokio::test]
    async fn a_moved_address_matches_in_either_family() {
        let (tx, _rx) = mpsc::channel(1);
        let f = Fallback::new(tx);
        if let Ok(mut d) = f.0.direct.write() {
            *d = Some(in_family(A, true));
        }
        f.set_moved(true);
        assert!(f.reroutes(A) && f.reroutes(in_family(A, true)));
        assert!(!f.reroutes(B));
        let meta = RecvMeta {
            addr: A,
            len: 1,
            stride: 1,
            ecn: None,
            dst_ip: None,
        };
        f.0.heard.store(0, Ordering::Release);
        tokio::time::sleep(Duration::from_millis(5)).await;
        f.heard_from(&[meta]);
        assert!(f.0.heard.load(Ordering::Acquire) > 0);
    }

    /// #435: coming back sends the last [`REPLAY`] datagrams that went to
    /// the relay again, directly, one segment each, and only once; what
    /// went directly or to the stand-in is not kept.
    #[tokio::test]
    #[allow(clippy::unwrap_used)]
    async fn coming_back_sends_the_last_rerouted_datagrams_again() {
        let peer = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        peer.set_read_timeout(Some(Duration::from_millis(300)))
            .unwrap();
        let to = peer.local_addr().unwrap();
        let own = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let inner = quinn::default_runtime()
            .unwrap()
            .wrap_udp_socket(own)
            .unwrap();
        let (socket, _pipes, f) = PathSocket::new(inner);
        // A new socket is not known to be writable until polled.
        let mut poller = socket.clone().create_io_poller();
        std::future::poll_fn(|cx| poller.as_mut().poll_writable(cx))
            .await
            .unwrap();
        if let Ok(mut d) = f.0.direct.write() {
            *d = Some(to);
        }
        let send = |contents: &[u8], segment_size: Option<usize>, destination| {
            socket
                .try_send(&Transmit {
                    destination,
                    ecn: None,
                    contents,
                    segment_size,
                    src_ip: None,
                })
                .unwrap();
        };
        let mut buf = [0u8; 16];
        let mut recv = || {
            let n = peer.recv(&mut buf).ok()?;
            buf.get(..n).map(<[u8]>::to_vec)
        };
        // Direct, before the move: sent, not kept.
        send(b"d", None, to);
        assert_eq!(recv().as_deref(), Some(&b"d"[..]));
        f.set_moved(true);
        send(b"1", None, to);
        send(b"2", None, to);
        send(b"r", None, RELAY_PEER);
        // Two segments in one transmit, as with GSO.
        send(b"3a3b", Some(2), to);
        send(b"4", None, to);
        send(b"5", None, to);
        assert_eq!(recv(), None, "a moved side sent directly");
        f.set_moved(false);
        f.replay().await;
        let mut again = Vec::new();
        while let Some(d) = recv() {
            again.push(d);
        }
        assert_eq!(again, [&b"3a"[..], b"3b", b"4", b"5"]);
        assert_eq!(f.replayed(), 4);
        f.replay().await;
        assert_eq!(recv(), None, "sent again twice");

        // A new move forgets what an earlier one kept.
        f.move_now(1);
        send(b"6", None, to);
        f.move_now(2);
        send(b"7", None, to);
        f.set_moved(false);
        f.replay().await;
        assert_eq!(recv().as_deref(), Some(&b"7"[..]));
        assert_eq!(recv(), None, "kept from an earlier move");

        // Kept for an address QUIC has left since: not sent there.
        let other = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        other
            .set_read_timeout(Some(Duration::from_millis(300)))
            .unwrap();
        f.move_now(3);
        send(b"8", None, to);
        if let Ok(mut d) = f.0.direct.write() {
            *d = Some(other.local_addr().unwrap());
        }
        f.set_moved(false);
        f.replay().await;
        assert_eq!(recv(), None, "sent again to an address left");
        let mut b = [0u8; 4];
        assert!(other.recv(&mut b).is_err(), "sent to the new address");
        assert_eq!(f.replayed(), 5);
    }

    /// #403: on a dual-stack socket QUIC reports the stand-in as an
    /// IPv4-mapped address; it is still the stand-in, not a new address.
    #[test]
    fn the_mapped_stand_in_is_the_relay() {
        let mapped = in_family(RELAY_PEER, true);
        assert!(mapped.is_ipv6());
        assert!(is_relay(RELAY_PEER) && is_relay(mapped));
        assert!(!is_relay(A) && !is_relay(in_family(A, true)));
        assert_eq!(canonical(mapped), RELAY_PEER);
        assert_eq!(in_family(RELAY_PEER, false), RELAY_PEER);
        let v6 = SocketAddr::new(
            std::net::Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1).into(),
            9,
        );
        assert_eq!(in_family(v6, true), v6);
        assert_eq!(canonical(v6), v6);
        let s = Seen {
            remote: mapped,
            direct: Some(in_family(A, true)),
            ..seen()
        };
        assert_eq!(step(&s, SILENCE), Step::Wait);
    }

    #[test]
    fn it_moves_only_after_the_whole_silence() {
        let just_short = Seen {
            heard: 20_000 - SILENCE + 1,
            ..seen()
        };
        assert_eq!(step(&just_short, SILENCE), Step::Wait);
        let quiet = Seen {
            heard: 20_000 - SILENCE,
            ..seen()
        };
        assert_eq!(step(&quiet, SILENCE), Step::Move);
    }

    /// A late tick means this device was paused: its silence is not the
    /// path's, so it waits a little more instead of moving at once
    /// (#185 review).
    #[test]
    fn a_pause_of_its_own_is_not_silence() {
        let late = Seen {
            last_tick: 20_000 - 2_500,
            heard: 0,
            ..seen()
        };
        let Step::Recount(heard) = step(&late, SILENCE) else {
            panic!("a late tick moved");
        };
        assert_eq!(heard, 20_000 - SILENCE + AFTER_PAUSE);
        let later = Seen {
            now: 20_000 + AFTER_PAUSE,
            last_tick: 20_000 + AFTER_PAUSE - 500,
            heard,
            ..seen()
        };
        assert_eq!(step(&later, SILENCE), Step::Move);
        // A tick just under the bound is not late.
        let on_time = Seen {
            last_tick: 20_000 - 2_499,
            heard: 0,
            ..seen()
        };
        assert_eq!(step(&on_time, SILENCE), Step::Move);
    }

    #[test]
    fn it_comes_back_only_when_heard_after_moving() {
        let moved = Seen {
            moved: true,
            moved_at: 18_000,
            heard: 18_000,
            ..seen()
        };
        assert_eq!(step(&moved, SILENCE), Step::Wait);
        // A datagram already on its way when the side moved.
        let late = Seen {
            heard: 18_000 + LATE,
            ..moved
        };
        assert_eq!(step(&late, SILENCE), Step::Wait);
        let heard = Seen {
            heard: 18_001 + LATE,
            ..moved
        };
        assert_eq!(step(&heard, SILENCE), Step::Back);
    }

    /// With the relay link gone (refused, unreachable, out of its limits)
    /// the side does not move, and a side that moved goes back: the
    /// direct path is the only chance left (#185 review).
    #[test]
    fn without_a_relay_link_it_stays_direct() {
        let quiet = Seen {
            heard: 0,
            link: false,
            ..seen()
        };
        assert_eq!(step(&quiet, SILENCE), Step::Wait);
        let moved = Seen {
            moved: true,
            moved_at: 18_000,
            ..quiet
        };
        assert_eq!(step(&moved, SILENCE), Step::Back);
    }

    /// Counting afresh after a pause never forgets a datagram heard since
    /// (#185 review).
    #[test]
    fn a_pause_does_not_forget_what_was_heard() {
        let late = Seen {
            last_tick: 20_000 - 3_000,
            heard: 19_900,
            ..seen()
        };
        assert_eq!(step(&late, SILENCE), Step::Recount(19_900));
    }

    #[tokio::test]
    async fn post_drain_close_replay_guards_peer_dcid_and_backoff() {
        use quinn::crypto::HmacKey;

        let peer: SocketAddr = "192.0.2.10:5000".parse().unwrap();
        let stranger: SocketAddr = "198.51.100.99:5000".parse().unwrap();
        let tokens = ResetTokens::new().unwrap();
        let (tx, _rx) = mpsc::channel(4);
        let fb = Fallback::with_reset_tokens(tx, Some(tokens.clone()));
        fb.arm_closing(peer);

        // A 240-byte short-header CONNECTION_CLOSE carrying multi-range ACKs
        // with winning DCID `[1; 8]`.
        let mut close_pkt = vec![0x40u8; 240];
        close_pkt[1..9].copy_from_slice(&[1u8; 8]);
        assert!(
            fb.on_send(&Transmit {
                destination: peer,
                ecn: None,
                contents: &close_pkt,
                segment_size: None,
                src_ip: None,
            })
            .is_none()
        );

        // A losing dual-dial connection with DCID `[2; 8]` on the same peer
        // address cannot overwrite the saved packet.
        let mut losing_pkt = vec![0x40u8; 48];
        losing_pkt[1..9].copy_from_slice(&[2u8; 8]);
        assert!(
            fb.on_send(&Transmit {
                destination: peer,
                ecn: None,
                contents: &losing_pkt,
                segment_size: None,
                src_ip: None,
            })
            .is_none()
        );

        // Build a 21-byte minimum stateless reset carrying a valid token.
        let mut sig = [0u8; 32];
        tokens.sign(b"cid-0", &mut sig);
        let mut reset = [0x40u8; 21];
        reset[5..21].copy_from_slice(&sig[..16]);

        // Spoofed third-party source address: reset is dropped (`Some(None)`),
        // nothing is reflected, and `answered` is not woken.
        assert_eq!(
            fb.on_send(&Transmit {
                destination: stranger,
                ecn: None,
                contents: &reset,
                segment_size: None,
                src_ip: None,
            }),
            Some(None)
        );
        assert!(
            tokio::time::timeout(Duration::ZERO, fb.answered())
                .await
                .is_err()
        );

        // Legitimate peer: even a 21-byte reset gets the 240-byte saved close
        // on power-of-two hits `1, 2, 4, 8, 16, 32` (6 total) and drops all
        // others (so two drained endpoints stop after hit 2 instead of looping).
        let mut sent_at = Vec::new();
        for hit in 1..=70u32 {
            let out = fb.on_send(&Transmit {
                destination: peer,
                ecn: None,
                contents: &reset,
                segment_size: None,
                src_ip: None,
            });
            match out {
                Some(Some(pkt)) => {
                    assert_eq!(pkt, close_pkt);
                    sent_at.push(hit);
                }
                Some(None) => {}
                None => panic!("stateless reset was not intercepted at hit {hit}"),
            }
        }
        assert_eq!(sent_at, vec![1, 2, 4, 8, 16, 32]);
        fb.answered().await;
    }
}
