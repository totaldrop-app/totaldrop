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

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::future::Future;
use std::io::{self, IoSliceMut};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::task::{Context, Poll, ready};
use std::time::Duration;

use quinn::udp::{EcnCodepoint, RecvMeta, Transmit, UdpSocketState};
use quinn::{AsyncUdpSocket, UdpPoller};
use tokio::io::Interest;
use tokio::sync::mpsc;

use crate::ipv6::V6Source;

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
}

impl Fallback {
    pub(crate) fn new(relay: mpsc::Sender<Vec<u8>>) -> Self {
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
        }))
    }

    /// Note one segment of `transmit`, sent through the relay instead.
    fn rerouted(&self, transmit: &Transmit, src: ReplaySrc, segment: &[u8]) {
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
                src,
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
            let (initial_src, safe_default) = match r.src {
                ReplaySrc::Default => (None, true),
                ReplaySrc::Explicit { ip, safe_default } => (Some(ip), safe_default),
                ReplaySrc::Suppress => continue,
            };
            let mut transmit = Transmit {
                destination: r.destination,
                ecn: r.ecn,
                contents: &r.datagram,
                segment_size: None,
                src_ip: initial_src,
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
                        Err(e) if transmit.src_ip.is_some() && is_source_addr_error(&e) => {
                            if safe_default && !crate::ipv6::is_unsafe_default_v6(r.destination) {
                                transmit.src_ip = None;
                            } else {
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

/// How [`Fallback::replay`] should pick the source IP when sending a kept
/// datagram directly again (#405).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplaySrc {
    /// Send with `src_ip: None` (IPv4, loopback, or announceable IPv6 default).
    Default,
    /// Send with an explicit `src_ip`; fall back to `None` on source-address
    /// rejection only when `safe_default` is true and the kernel default is not
    /// modified EUI-64.
    Explicit { ip: IpAddr, safe_default: bool },
    /// Kernel default is unsafe (modified EUI-64) and no temporary IPv6 exists:
    /// skip direct replay so the hardware address is never leaked.
    Suppress,
}

/// A datagram sent through the relay while moved, kept to send again.
#[derive(Debug)]
struct Rerouted {
    destination: SocketAddr,
    ecn: Option<EcnCodepoint>,
    src: ReplaySrc,
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

/// Maximum distinct IPv6 destinations cached per [`PathSocket`].
const MAX_PREFERRED_V6_CACHE: usize = 32;

/// Preferred temporary IPv6 source selection for outbound datagrams (#405).
#[derive(Debug)]
enum PreferredV6 {
    /// Resolved once per distinct routable IPv6 destination and cached (bounded).
    Dynamic(HashMap<Ipv6Addr, V6Source>),
    /// Fixed source policy for unit tests (updated on send error).
    #[cfg(test)]
    Fixed(V6Source),
}

/// Whether `e` indicates that an explicit `src_ip` could not be used.
fn is_source_addr_error(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::AddrNotAvailable | io::ErrorKind::InvalidInput
    )
}

type WritableFuture = Pin<Box<dyn Future<Output = io::Result<()>> + Send>>;

/// Tokio-backed [`AsyncUdpSocket`] that propagates `AddrNotAvailable` and
/// `InvalidInput` when `transmit.src_ip` is set (#405).
///
/// `quinn::TokioRuntime::wrap_udp_socket` delegates `try_send` to
/// `quinn::udp::UdpSocketState::send`, which converts every non-`WouldBlock`
/// error (including `EADDRNOTAVAIL` and `EINVAL` from `IPV6_PKTINFO`) into
/// `Ok(())`. Calling `UdpSocketState::try_send` for explicit `src_ip` transmits
/// lets [`PathSocket`] and [`Fallback`] detect a rejected temporary address and
/// recover instead of blackholing the direct path.
#[derive(Debug)]
struct WrappedUdpSocket {
    io: tokio::net::UdpSocket,
    state: UdpSocketState,
}

struct WrappedUdpPoller {
    socket: Arc<WrappedUdpSocket>,
    fut: Mutex<Option<WritableFuture>>,
}

impl fmt::Debug for WrappedUdpPoller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WrappedUdpPoller").finish_non_exhaustive()
    }
}

impl UdpPoller for WrappedUdpPoller {
    fn poll_writable(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let mut guard = self
            .fut
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let fut = guard.get_or_insert_with(|| {
            let socket = self.socket.clone();
            Box::pin(async move { socket.io.writable().await })
        });
        let result = fut.as_mut().poll(cx);
        if result.is_ready() {
            *guard = None;
        }
        result
    }
}

impl AsyncUdpSocket for WrappedUdpSocket {
    fn create_io_poller(self: Arc<Self>) -> Pin<Box<dyn UdpPoller>> {
        Box::pin(WrappedUdpPoller {
            socket: self,
            fut: Mutex::new(None),
        })
    }

    fn try_send(&self, transmit: &Transmit) -> io::Result<()> {
        if transmit.src_ip.is_some() {
            match self.io.try_io(Interest::WRITABLE, || {
                self.state.try_send((&self.io).into(), transmit)
            }) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock || is_source_addr_error(&e) => {
                    Err(e)
                }
                // Match `UdpSocketState::send`: non-fatal UDP errors such as
                // `EMSGSIZE` or `ENETUNREACH` drop the datagram rather than
                // aborting quinn's connection driver.
                Err(_) => Ok(()),
            }
        } else {
            self.io.try_io(Interest::WRITABLE, || {
                self.state.send((&self.io).into(), transmit)
            })
        }
    }

    fn poll_recv(
        &self,
        cx: &mut Context,
        bufs: &mut [IoSliceMut<'_>],
        meta: &mut [RecvMeta],
    ) -> Poll<io::Result<usize>> {
        loop {
            ready!(self.io.poll_recv_ready(cx))?;
            if let Ok(res) = self.io.try_io(Interest::READABLE, || {
                self.state.recv((&self.io).into(), bufs, meta)
            }) {
                return Poll::Ready(Ok(res));
            }
        }
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.local_addr()
    }

    fn may_fragment(&self) -> bool {
        self.state.may_fragment()
    }

    fn max_transmit_segments(&self) -> usize {
        self.state.max_gso_segments()
    }

    fn max_receive_segments(&self) -> usize {
        self.state.gro_segments()
    }
}

/// Wrap `sock` for QUIC with real `IPV6_PKTINFO` error propagation (#405).
pub(crate) fn wrap_udp_socket(sock: std::net::UdpSocket) -> io::Result<Arc<dyn AsyncUdpSocket>> {
    let state = UdpSocketState::new((&sock).into())?;
    let io = tokio::net::UdpSocket::from_std(sock)?;
    Ok(Arc::new(WrappedUdpSocket { io, state }))
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
    /// Preferred temporary IPv6 source address cache (#405).
    preferred_v6: RwLock<PreferredV6>,
}

impl PathSocket {
    /// Wrap `inner`. The returned pipes connect it to a relay link. Until the
    /// link is up, datagrams for the relay wait in a short queue; once it is
    /// full, or when no link runs, they are dropped. The [`Fallback`] moves a
    /// direct connection onto the relay.
    pub fn new(inner: Arc<dyn AsyncUdpSocket>) -> (Arc<Self>, RelayPipes, Fallback) {
        Self::with_policy(inner, PreferredV6::Dynamic(HashMap::new()))
    }

    #[cfg(test)]
    pub(crate) fn with_preferred_v6(
        inner: Arc<dyn AsyncUdpSocket>,
        policy: V6Source,
    ) -> (Arc<Self>, RelayPipes, Fallback) {
        Self::with_policy(inner, PreferredV6::Fixed(policy))
    }

    fn with_policy(
        inner: Arc<dyn AsyncUdpSocket>,
        preferred_v6: PreferredV6,
    ) -> (Arc<Self>, RelayPipes, Fallback) {
        let (to_relay, outgoing) = mpsc::channel(QUEUE);
        let (incoming, from_relay) = mpsc::channel(QUEUE);
        let fallback = Fallback::new(to_relay.clone());
        let _ = fallback.0.socket.set(inner.clone());
        let ipv6 = inner.local_addr().is_ok_and(|a| a.is_ipv6());
        (
            Arc::new(Self {
                inner,
                to_relay,
                from_relay: Mutex::new(from_relay),
                fallback: fallback.clone(),
                ipv6,
                preferred_v6: RwLock::new(preferred_v6),
            }),
            RelayPipes { outgoing, incoming },
            fallback,
        )
    }

    fn resolve_v6_source(&self, destination: SocketAddr, dest_v6: Ipv6Addr) -> V6Source {
        {
            let guard = self
                .preferred_v6
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match &*guard {
                PreferredV6::Dynamic(cache) => {
                    if let Some(&cached) = cache.get(&dest_v6) {
                        return cached;
                    }
                }
                #[cfg(test)]
                PreferredV6::Fixed(policy) => return *policy,
            }
        }
        let resolved = crate::ipv6::source_policy_v6(destination);
        let mut guard = self
            .preferred_v6
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match &mut *guard {
            PreferredV6::Dynamic(cache) => {
                if let Some(&existing) = cache.get(&dest_v6) {
                    return existing;
                }
                if cache.len() < MAX_PREFERRED_V6_CACHE {
                    cache.insert(dest_v6, resolved);
                }
            }
            #[cfg(test)]
            PreferredV6::Fixed(_) => {}
        }
        resolved
    }

    fn set_v6_source(&self, dest_v6: Ipv6Addr, next: V6Source) {
        let mut guard = self
            .preferred_v6
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match &mut *guard {
            PreferredV6::Dynamic(cache) => {
                if cache.contains_key(&dest_v6) || cache.len() < MAX_PREFERRED_V6_CACHE {
                    cache.insert(dest_v6, next);
                }
            }
            #[cfg(test)]
            PreferredV6::Fixed(policy) => *policy = next,
        }
    }

    /// Retry a segmented (`UDP_SEGMENT` / GSO) transmit as individual
    /// single-segment datagrams while keeping `src_ip` intact (#405).
    ///
    /// Returns `Ok(true)` when GSO itself caused `InvalidInput` and the first
    /// single-segment datagram succeeded (preserving the cached temporary IPv6
    /// source), `Ok(false)` when the first single-segment send also failed with
    /// a source-address error (proving `src_ip` itself is unusable), or
    /// `Err(e)` for `WouldBlock` or other errors.
    fn try_send_split_segments(&self, patched: &Transmit) -> io::Result<bool> {
        let size = patched
            .segment_size
            .unwrap_or(patched.contents.len())
            .max(1);
        let mut segments = patched.contents.chunks(size);
        let Some(first) = segments.next() else {
            return Ok(true);
        };
        let single = |contents| Transmit {
            segment_size: None,
            contents,
            ..patched.clone()
        };
        match self.inner.try_send(&single(first)) {
            Ok(()) => {}
            Err(e) if is_source_addr_error(&e) => return Ok(false),
            Err(e) => return Err(e),
        }
        for segment in segments {
            match self.inner.try_send(&single(segment)) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Err(e),
                // Remaining segments are best-effort once the first succeeded;
                // QUIC retransmits any dropped datagram.
                Err(_) => break,
            }
        }
        Ok(true)
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
        if is_relay(transmit.destination) {
            self.relay_send(transmit);
            return Ok(());
        }
        let v6_policy = if transmit.src_ip.is_none() {
            crate::ipv6::routable_v6_ip(transmit.destination).map(|dest_v6| {
                (
                    dest_v6,
                    self.resolve_v6_source(transmit.destination, dest_v6),
                )
            })
        } else {
            None
        };
        if self.fallback.reroutes(transmit.destination) {
            self.relay_send(transmit);
            let replay_src = match (transmit.src_ip, v6_policy) {
                (Some(ip), _) => ReplaySrc::Explicit {
                    ip,
                    safe_default: true,
                },
                (None, Some((_, V6Source::Temporary { addr, safe_default }))) => {
                    ReplaySrc::Explicit {
                        ip: IpAddr::V6(addr),
                        safe_default,
                    }
                }
                (None, Some((_, V6Source::UnsafeDefault))) => ReplaySrc::Suppress,
                (None, Some((_, V6Source::SystemDefault))) | (None, None) => ReplaySrc::Default,
            };
            let size = transmit
                .segment_size
                .unwrap_or(transmit.contents.len())
                .max(1);
            for segment in transmit.contents.chunks(size) {
                self.fallback.rerouted(transmit, replay_src, segment);
            }
            return Ok(());
        }
        match v6_policy {
            Some((_, V6Source::UnsafeDefault)) => Ok(()),
            Some((dest_v6, V6Source::Temporary { addr, safe_default })) => {
                let patched = Transmit {
                    src_ip: Some(IpAddr::V6(addr)),
                    ..transmit.clone()
                };
                match self.inner.try_send(&patched) {
                    Ok(()) => Ok(()),
                    Err(e) if is_source_addr_error(&e) => {
                        if e.kind() == io::ErrorKind::InvalidInput
                            && patched
                                .segment_size
                                .is_some_and(|s| s > 0 && s < patched.contents.len())
                        {
                            match self.try_send_split_segments(&patched) {
                                Ok(true) => return Ok(()),
                                Ok(false) => {}
                                Err(err) => return Err(err),
                            }
                        }
                        if safe_default && !crate::ipv6::is_unsafe_default_v6(transmit.destination)
                        {
                            self.set_v6_source(dest_v6, V6Source::SystemDefault);
                            self.inner.try_send(transmit)
                        } else {
                            self.set_v6_source(dest_v6, V6Source::UnsafeDefault);
                            Ok(())
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            // Never return a source-address error to quinn when quinn itself
            // echoed `RecvMeta.dst_ip`: `ConnectionDriver` terminates on any
            // non-`WouldBlock` error from `try_send`.
            Some((_, V6Source::SystemDefault)) | None => match self.inner.try_send(transmit) {
                Err(e) if transmit.src_ip.is_some() && is_source_addr_error(&e) => Ok(()),
                other => other,
            },
        }
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
#[allow(clippy::panic)]
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

    #[derive(Debug)]
    struct ReadyPoller;

    impl UdpPoller for ReadyPoller {
        fn poll_writable(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    type RecordedSend = (SocketAddr, Option<IpAddr>, Vec<u8>);

    #[derive(Debug)]
    struct FakeSocket {
        sent: Mutex<Vec<RecordedSend>>,
        outcomes: Mutex<VecDeque<io::Result<()>>>,
        fail_gso: AtomicBool,
    }

    impl FakeSocket {
        fn new(outcomes: Vec<io::Result<()>>) -> Arc<Self> {
            Arc::new(Self {
                sent: Mutex::new(Vec::new()),
                outcomes: Mutex::new(outcomes.into()),
                fail_gso: AtomicBool::new(false),
            })
        }

        fn gso_rejecting() -> Arc<Self> {
            Arc::new(Self {
                sent: Mutex::new(Vec::new()),
                outcomes: Mutex::new(VecDeque::new()),
                fail_gso: AtomicBool::new(true),
            })
        }

        fn recorded(&self) -> Vec<RecordedSend> {
            self.sent
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        }
    }

    impl AsyncUdpSocket for FakeSocket {
        fn create_io_poller(self: Arc<Self>) -> Pin<Box<dyn UdpPoller>> {
            Box::pin(ReadyPoller)
        }

        fn try_send(&self, transmit: &Transmit) -> io::Result<()> {
            self.sent
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((
                    transmit.destination,
                    transmit.src_ip,
                    transmit.contents.to_vec(),
                ));
            if self.fail_gso.load(Ordering::Acquire) && transmit.segment_size.is_some() {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }
            self.outcomes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .pop_front()
                .unwrap_or(Ok(()))
        }

        fn poll_recv(
            &self,
            _cx: &mut Context,
            _bufs: &mut [IoSliceMut<'_>],
            _meta: &mut [RecvMeta],
        ) -> Poll<io::Result<usize>> {
            Poll::Pending
        }

        fn local_addr(&self) -> io::Result<SocketAddr> {
            Ok(SocketAddr::from((Ipv6Addr::UNSPECIFIED, 5000)))
        }
    }

    /// #405: `wrap_udp_socket` propagates real OS `AddrNotAvailable` /
    /// `InvalidInput` when `src_ip` is unassigned (rather than swallowing it as
    /// `Ok(())`), while delivering valid sends and receiving via `poll_recv`.
    #[tokio::test]
    #[allow(clippy::unwrap_used)]
    async fn wrap_udp_socket_propagates_bad_src_ip_and_delivers_valid_loopback() {
        let Ok(peer_std) = std::net::UdpSocket::bind("[::1]:0") else {
            assert!(
                std::env::var_os("TOTALDROP_REQUIRE_IPV6").is_none(),
                "TOTALDROP_REQUIRE_IPV6 is set but IPv6 loopback is unavailable"
            );
            return;
        };
        let peer_addr = peer_std.local_addr().unwrap();
        let peer = wrap_udp_socket(peer_std).unwrap();
        let sender_std = crate::transport::bind_socket().unwrap();
        let sender = wrap_udp_socket(sender_std).unwrap();

        assert!(sender.local_addr().unwrap().is_ipv6());
        assert!(sender.max_transmit_segments() >= 1);
        assert!(sender.max_receive_segments() >= 1);
        let _ = sender.may_fragment();

        let mut poller = sender.clone().create_io_poller();
        assert!(format!("{poller:?}").contains("WrappedUdpPoller"));
        std::future::poll_fn(|cx| poller.as_mut().poll_writable(cx))
            .await
            .unwrap();

        // Unassigned RFC 3849 address MUST fail with AddrNotAvailable or
        // InvalidInput on the real OS kernel, NOT return Ok(()).
        let bogus: Ipv6Addr = "2001:db8::dead:beef".parse().unwrap();
        let err = sender
            .try_send(&Transmit {
                destination: peer_addr,
                ecn: None,
                contents: b"bad-src",
                segment_size: None,
                src_ip: Some(IpAddr::V6(bogus)),
            })
            .unwrap_err();
        assert!(
            is_source_addr_error(&err),
            "expected AddrNotAvailable/InvalidInput from OS kernel, got {err:?}"
        );

        // Subsequent valid sends (explicit ::1 and default src_ip: None) must
        // succeed on the same WrappedUdpSocket and arrive via poll_recv.
        for (src_ip, payload) in [
            (Some(IpAddr::V6(Ipv6Addr::LOCALHOST)), &b"ok-pktinfo"[..]),
            (None, &b"ok-default"[..]),
        ] {
            sender
                .try_send(&Transmit {
                    destination: peer_addr,
                    ecn: None,
                    contents: payload,
                    segment_size: None,
                    src_ip,
                })
                .unwrap();
            let mut buf = [0u8; 64];
            let mut meta = [RecvMeta::default()];
            let n = tokio::time::timeout(
                Duration::from_millis(500),
                std::future::poll_fn(|cx| {
                    let mut slices = [IoSliceMut::new(&mut buf)];
                    peer.poll_recv(cx, &mut slices, &mut meta)
                }),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(n, 1);
            assert_eq!(buf.get(..meta[0].len), Some(payload));
        }
    }

    /// #405: `PathSocket` stamps preferred temporary IPv6 `src_ip` only on
    /// routable unicast IPv6 destinations when `src_ip` is unset, leaving
    /// explicit `src_ip`, IPv4, IPv4-mapped IPv6, loopback, unspecified,
    /// multicast, and relay transmits untouched.
    #[test]
    #[allow(clippy::unwrap_used)]
    fn preferred_v6_stamps_only_real_ipv6_without_existing_src_ip() {
        let temp: Ipv6Addr = "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap();
        let explicit: Ipv6Addr = "2a02:4e0:2d8a:1b00:1111:2222:3333:4444".parse().unwrap();
        let fake = FakeSocket::new(Vec::new());
        let (socket, mut pipes, _fallback) = PathSocket::with_preferred_v6(
            fake.clone(),
            V6Source::Temporary {
                addr: temp,
                safe_default: true,
            },
        );

        let global_v6: SocketAddr = "[2001:db8::1]:4000".parse().unwrap();
        let mapped_v6: SocketAddr = "[::ffff:192.0.2.7]:4000".parse().unwrap();
        let loopback_v6: SocketAddr = "[::1]:4000".parse().unwrap();
        let unspec_v6: SocketAddr = "[::]:4000".parse().unwrap();
        let mcast_v6: SocketAddr = "[ff02::1]:4000".parse().unwrap();

        let send = |destination: SocketAddr, src_ip: Option<IpAddr>, payload: &[u8]| {
            socket
                .try_send(&Transmit {
                    destination,
                    ecn: None,
                    contents: payload,
                    segment_size: None,
                    src_ip,
                })
                .unwrap();
        };

        send(global_v6, None, b"g1");
        send(global_v6, Some(IpAddr::V6(explicit)), b"g2");
        send(A, None, b"v4");
        send(mapped_v6, None, b"mp");
        send(loopback_v6, None, b"lo");
        send(unspec_v6, None, b"un");
        send(mcast_v6, None, b"mc");
        send(RELAY_PEER, None, b"rl");

        assert_eq!(
            fake.recorded(),
            vec![
                (global_v6, Some(IpAddr::V6(temp)), b"g1".to_vec()),
                (global_v6, Some(IpAddr::V6(explicit)), b"g2".to_vec()),
                (A, None, b"v4".to_vec()),
                (mapped_v6, None, b"mp".to_vec()),
                (loopback_v6, None, b"lo".to_vec()),
                (unspec_v6, None, b"un".to_vec()),
                (mcast_v6, None, b"mc".to_vec()),
            ]
        );
        assert_eq!(pipes.outgoing.try_recv().ok(), Some(b"rl".to_vec()));
    }

    /// #405: `AddrNotAvailable` and `InvalidInput` transition `safe_default:
    /// true` to `SystemDefault` and retry with `src_ip: None`, preserve cache
    /// on `WouldBlock` and unrelated errors, and absorb source-address errors
    /// on quinn-stamped transmits so `ConnectionDriver` is never killed.
    #[test]
    #[allow(clippy::unwrap_used)]
    fn preferred_v6_clears_and_retries_on_source_addr_errors() {
        let temp: Ipv6Addr = "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap();
        let dest: SocketAddr = "[2001:db8::2]:4000".parse().unwrap();

        for kind in [io::ErrorKind::AddrNotAvailable, io::ErrorKind::InvalidInput] {
            let fake = FakeSocket::new(vec![Err(io::Error::from(kind)), Ok(()), Ok(())]);
            let (socket, _pipes, _fallback) = PathSocket::with_preferred_v6(
                fake.clone(),
                V6Source::Temporary {
                    addr: temp,
                    safe_default: true,
                },
            );

            let tx = Transmit {
                destination: dest,
                ecn: None,
                contents: b"first",
                segment_size: None,
                src_ip: None,
            };
            socket.try_send(&tx).unwrap();
            let tx2 = Transmit {
                destination: dest,
                ecn: None,
                contents: b"second",
                segment_size: None,
                src_ip: None,
            };
            socket.try_send(&tx2).unwrap();

            assert_eq!(
                fake.recorded(),
                vec![
                    (dest, Some(IpAddr::V6(temp)), b"first".to_vec()),
                    (dest, None, b"first".to_vec()),
                    (dest, None, b"second".to_vec()),
                ]
            );
        }

        // WouldBlock propagates to quinn and preserves Temporary in cache.
        let fake = FakeSocket::new(vec![
            Err(io::Error::from(io::ErrorKind::WouldBlock)),
            Ok(()),
        ]);
        let (socket, _pipes, _fallback) = PathSocket::with_preferred_v6(
            fake.clone(),
            V6Source::Temporary {
                addr: temp,
                safe_default: true,
            },
        );
        let wb = socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"wb",
                segment_size: None,
                src_ip: None,
            })
            .unwrap_err();
        assert_eq!(wb.kind(), io::ErrorKind::WouldBlock);
        socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"after-wb",
                segment_size: None,
                src_ip: None,
            })
            .unwrap();
        assert_eq!(
            fake.recorded(),
            vec![
                (dest, Some(IpAddr::V6(temp)), b"wb".to_vec()),
                (dest, Some(IpAddr::V6(temp)), b"after-wb".to_vec()),
            ]
        );

        // Unrelated errors propagate without clearing or retrying.
        let fake = FakeSocket::new(vec![Err(io::Error::from(io::ErrorKind::PermissionDenied))]);
        let (socket, _pipes, _fallback) = PathSocket::with_preferred_v6(
            fake.clone(),
            V6Source::Temporary {
                addr: temp,
                safe_default: true,
            },
        );
        let err = socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"perm",
                segment_size: None,
                src_ip: None,
            })
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(
            fake.recorded(),
            vec![(dest, Some(IpAddr::V6(temp)), b"perm".to_vec())]
        );

        // Pre-stamped src_ip from quinn: source-address errors convert to Ok(())
        // rather than killing ConnectionDriver; WouldBlock still propagates.
        let fake = FakeSocket::new(vec![
            Err(io::Error::from(io::ErrorKind::AddrNotAvailable)),
            Err(io::Error::from(io::ErrorKind::InvalidInput)),
            Err(io::Error::from(io::ErrorKind::WouldBlock)),
        ]);
        let (socket, _pipes, _fallback) = PathSocket::new(fake);
        let tx = Transmit {
            destination: dest,
            ecn: None,
            contents: b"echo",
            segment_size: None,
            src_ip: Some(IpAddr::V6(temp)),
        };
        socket.try_send(&tx).unwrap();
        socket.try_send(&tx).unwrap();
        assert_eq!(
            socket.try_send(&tx).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
    }

    /// #405: when the kernel's default IPv6 source is unsafe (modified EUI-64),
    /// neither fallback nor TOCTOU ever sends a direct packet with `src_ip:
    /// None`; direct packets drop cleanly with `Ok(())` so IPv4 or relay wins.
    #[test]
    #[allow(clippy::unwrap_used)]
    fn unsafe_default_suppresses_direct_sends_without_leaking_eui64() {
        let temp: Ipv6Addr = "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap();
        let dest: SocketAddr = "[2001:db8::40]:4000".parse().unwrap();

        for kind in [io::ErrorKind::AddrNotAvailable, io::ErrorKind::InvalidInput] {
            let fake = FakeSocket::new(vec![Err(io::Error::from(kind))]);
            let (socket, _pipes, _fallback) = PathSocket::with_preferred_v6(
                fake.clone(),
                V6Source::Temporary {
                    addr: temp,
                    safe_default: false,
                },
            );
            socket
                .try_send(&Transmit {
                    destination: dest,
                    ecn: None,
                    contents: b"first",
                    segment_size: None,
                    src_ip: None,
                })
                .unwrap();
            // Subsequent send hits cached UnsafeDefault and never touches inner.
            socket
                .try_send(&Transmit {
                    destination: dest,
                    ecn: None,
                    contents: b"second",
                    segment_size: None,
                    src_ip: None,
                })
                .unwrap();
            assert_eq!(
                fake.recorded(),
                vec![(dest, Some(IpAddr::V6(temp)), b"first".to_vec())]
            );
        }

        // Direct UnsafeDefault (TOCTOU miss) drops without calling inner.
        let fake = FakeSocket::new(Vec::new());
        let (socket, _pipes, _fallback) =
            PathSocket::with_preferred_v6(fake.clone(), V6Source::UnsafeDefault);
        socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"drop-me",
                segment_size: None,
                src_ip: None,
            })
            .unwrap();
        assert!(fake.recorded().is_empty());
    }

    /// #405: GSO-only `InvalidInput` splits into individual segments keeping
    /// `src_ip` and preserves `Temporary` in cache; if single-segment sends
    /// also fail with a source-address error, normal fallback/suppression runs.
    #[test]
    #[allow(clippy::unwrap_used)]
    fn gso_einval_preserves_temporary_source_while_real_src_error_falls_back() {
        let temp: Ipv6Addr = "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap();
        let dest: SocketAddr = "[2001:db8::50]:4000".parse().unwrap();

        // GSO rejected, single segments succeed with the same temp src_ip.
        let fake = FakeSocket::gso_rejecting();
        let (socket, _pipes, _fallback) = PathSocket::with_preferred_v6(
            fake.clone(),
            V6Source::Temporary {
                addr: temp,
                safe_default: true,
            },
        );
        socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"aabbcc",
                segment_size: Some(2),
                src_ip: None,
            })
            .unwrap();
        // Next single transmit must STILL carry `src_ip: Some(temp)`.
        socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"next",
                segment_size: None,
                src_ip: None,
            })
            .unwrap();
        assert_eq!(
            fake.recorded(),
            vec![
                (dest, Some(IpAddr::V6(temp)), b"aabbcc".to_vec()),
                (dest, Some(IpAddr::V6(temp)), b"aa".to_vec()),
                (dest, Some(IpAddr::V6(temp)), b"bb".to_vec()),
                (dest, Some(IpAddr::V6(temp)), b"cc".to_vec()),
                (dest, Some(IpAddr::V6(temp)), b"next".to_vec()),
            ]
        );

        // When BOTH GSO and the first split segment fail with InvalidInput,
        // `src_ip` itself is rejected -> fall back when safe_default is true.
        let fake = FakeSocket::new(vec![
            Err(io::Error::from(io::ErrorKind::InvalidInput)),
            Err(io::Error::from(io::ErrorKind::InvalidInput)),
            Ok(()),
            Ok(()),
        ]);
        let (socket, _pipes, _fallback) = PathSocket::with_preferred_v6(
            fake.clone(),
            V6Source::Temporary {
                addr: temp,
                safe_default: true,
            },
        );
        socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"aabb",
                segment_size: Some(2),
                src_ip: None,
            })
            .unwrap();
        socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"after",
                segment_size: None,
                src_ip: None,
            })
            .unwrap();
        assert_eq!(
            fake.recorded(),
            vec![
                (dest, Some(IpAddr::V6(temp)), b"aabb".to_vec()),
                (dest, Some(IpAddr::V6(temp)), b"aa".to_vec()),
                (dest, None, b"aabb".to_vec()),
                (dest, None, b"after".to_vec()),
            ]
        );

        // WouldBlock on a split segment propagates WouldBlock and preserves temp.
        let fake = FakeSocket::new(vec![
            Err(io::Error::from(io::ErrorKind::InvalidInput)),
            Ok(()),
            Err(io::Error::from(io::ErrorKind::WouldBlock)),
            Ok(()),
        ]);
        let (socket, _pipes, _fallback) = PathSocket::with_preferred_v6(
            fake.clone(),
            V6Source::Temporary {
                addr: temp,
                safe_default: true,
            },
        );
        let wb = socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"xxyy",
                segment_size: Some(2),
                src_ip: None,
            })
            .unwrap_err();
        assert_eq!(wb.kind(), io::ErrorKind::WouldBlock);
        socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"keep",
                segment_size: None,
                src_ip: None,
            })
            .unwrap();
        assert_eq!(
            fake.recorded().last(),
            Some(&(dest, Some(IpAddr::V6(temp)), b"keep".to_vec()))
        );
    }

    /// #405: `PreferredV6::Dynamic` caches per-destination lookups up to
    /// `MAX_PREFERRED_V6_CACHE` entries, handles overflow destinations safely,
    /// and updates existing entries in a full cache on source-address errors.
    #[test]
    #[allow(clippy::unwrap_used)]
    fn dynamic_preferred_v6_caches_lookup_bounds_size_and_updates_on_error() {
        let dest1: SocketAddr = "[2001:db8::10]:4000".parse().unwrap();
        let dest2_ip: Ipv6Addr = "2001:db8::20".parse().unwrap();
        let dest2 = SocketAddr::new(IpAddr::V6(dest2_ip), 4000);
        let temp: Ipv6Addr = "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap();

        let fake = FakeSocket::new(vec![
            Ok(()),
            Ok(()),
            Ok(()),
            Err(io::Error::from(io::ErrorKind::AddrNotAvailable)),
            Ok(()),
            Ok(()),
        ]);
        let (socket, _pipes, _fallback) = PathSocket::new(fake.clone());

        // Cache miss then cache hit for dest1.
        for payload in [&b"m1"[..], b"m2"] {
            socket
                .try_send(&Transmit {
                    destination: dest1,
                    ecn: None,
                    contents: payload,
                    segment_size: None,
                    src_ip: None,
                })
                .unwrap();
        }
        // Fill cache to MAX_PREFERRED_V6_CACHE including dest2.
        if let PreferredV6::Dynamic(cache) = &mut *socket
            .preferred_v6
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            cache.insert(
                dest2_ip,
                V6Source::Temporary {
                    addr: temp,
                    safe_default: true,
                },
            );
            for i in 0..u16::try_from(MAX_PREFERRED_V6_CACHE).unwrap() {
                if cache.len() >= MAX_PREFERRED_V6_CACHE {
                    break;
                }
                let ip = Ipv6Addr::new(0x2001, 0xdb8, 1, 0, 0, 0, 0, i.saturating_add(1));
                cache.insert(ip, V6Source::SystemDefault);
            }
            assert_eq!(cache.len(), MAX_PREFERRED_V6_CACHE);
        }
        // 33rd distinct destination must not grow the cache past the cap.
        let dest33: SocketAddr = "[2001:db8:2::33]:4000".parse().unwrap();
        socket
            .try_send(&Transmit {
                destination: dest33,
                ecn: None,
                contents: b"d33",
                segment_size: None,
                src_ip: None,
            })
            .unwrap();
        if let PreferredV6::Dynamic(cache) = &*socket
            .preferred_v6
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            assert_eq!(cache.len(), MAX_PREFERRED_V6_CACHE);
        }

        // Existing entry dest2 in the full cache MUST still update on error.
        socket
            .try_send(&Transmit {
                destination: dest2,
                ecn: None,
                contents: b"d1",
                segment_size: None,
                src_ip: None,
            })
            .unwrap();
        socket
            .try_send(&Transmit {
                destination: dest2,
                ecn: None,
                contents: b"d2",
                segment_size: None,
                src_ip: None,
            })
            .unwrap();

        let recorded = fake.recorded();
        let tail = recorded.get(3..).unwrap();
        assert_eq!(
            tail,
            [
                (dest2, Some(IpAddr::V6(temp)), b"d1".to_vec()),
                (dest2, None, b"d1".to_vec()),
                (dest2, None, b"d2".to_vec()),
            ]
        );
    }

    /// #405: `Fallback::rerouted` preserves the resolved temporary IPv6
    /// `src_ip` and fallback safety, `Fallback::replay` recovers safely when
    /// `safe_default` is true, and never leaks EUI-64 when `safe_default` is
    /// false or when `V6Source::UnsafeDefault` suppressed direct sends.
    #[tokio::test]
    #[allow(clippy::unwrap_used)]
    async fn fallback_reroute_and_replay_preserve_and_recover_preferred_v6() {
        let temp: Ipv6Addr = "2a02:4e0:2d8a:1b00:abcd:ef01:2345:6789".parse().unwrap();
        let dest: SocketAddr = "[2001:db8::30]:4000".parse().unwrap();

        // safe_default: true recovers across WouldBlock + AddrNotAvailable/InvalidInput.
        let fake = FakeSocket::new(vec![
            Err(io::Error::from(io::ErrorKind::WouldBlock)),
            Err(io::Error::from(io::ErrorKind::AddrNotAvailable)),
            Ok(()),
            Err(io::Error::from(io::ErrorKind::InvalidInput)),
            Ok(()),
        ]);
        let (socket, mut pipes, f) = PathSocket::with_preferred_v6(
            fake.clone(),
            V6Source::Temporary {
                addr: temp,
                safe_default: true,
            },
        );
        if let Ok(mut d) = f.0.direct.write() {
            *d = Some(dest);
        }
        f.set_moved(true);
        for payload in [&b"p1"[..], b"p2"] {
            socket
                .try_send(&Transmit {
                    destination: dest,
                    ecn: None,
                    contents: payload,
                    segment_size: None,
                    src_ip: None,
                })
                .unwrap();
        }
        assert_eq!(pipes.outgoing.try_recv().ok(), Some(b"p1".to_vec()));
        assert_eq!(pipes.outgoing.try_recv().ok(), Some(b"p2".to_vec()));
        assert!(fake.recorded().is_empty());

        f.set_moved(false);
        f.replay().await;
        assert_eq!(f.replayed(), 2);
        assert_eq!(
            fake.recorded(),
            vec![
                (dest, Some(IpAddr::V6(temp)), b"p1".to_vec()),
                (dest, Some(IpAddr::V6(temp)), b"p1".to_vec()),
                (dest, None, b"p1".to_vec()),
                (dest, Some(IpAddr::V6(temp)), b"p2".to_vec()),
                (dest, None, b"p2".to_vec()),
            ]
        );

        // safe_default: false MUST NOT retry with src_ip: None on replay.
        let fake = FakeSocket::new(vec![
            Err(io::Error::from(io::ErrorKind::AddrNotAvailable)),
            Err(io::Error::from(io::ErrorKind::InvalidInput)),
        ]);
        let (socket, mut pipes, f) = PathSocket::with_preferred_v6(
            fake.clone(),
            V6Source::Temporary {
                addr: temp,
                safe_default: false,
            },
        );
        if let Ok(mut d) = f.0.direct.write() {
            *d = Some(dest);
        }
        f.set_moved(true);
        for payload in [&b"u1"[..], b"u2"] {
            socket
                .try_send(&Transmit {
                    destination: dest,
                    ecn: None,
                    contents: payload,
                    segment_size: None,
                    src_ip: None,
                })
                .unwrap();
        }
        assert_eq!(pipes.outgoing.try_recv().ok(), Some(b"u1".to_vec()));
        assert_eq!(pipes.outgoing.try_recv().ok(), Some(b"u2".to_vec()));
        f.set_moved(false);
        f.replay().await;
        assert_eq!(f.replayed(), 0);
        assert_eq!(
            fake.recorded(),
            vec![
                (dest, Some(IpAddr::V6(temp)), b"u1".to_vec()),
                (dest, Some(IpAddr::V6(temp)), b"u2".to_vec()),
            ]
        );

        // UnsafeDefault still forwards over the relay while moved, and never
        // replays directly.
        let fake = FakeSocket::new(Vec::new());
        let (socket, mut pipes, f) =
            PathSocket::with_preferred_v6(fake.clone(), V6Source::UnsafeDefault);
        if let Ok(mut d) = f.0.direct.write() {
            *d = Some(dest);
        }
        f.set_moved(true);
        socket
            .try_send(&Transmit {
                destination: dest,
                ecn: None,
                contents: b"sup",
                segment_size: None,
                src_ip: None,
            })
            .unwrap();
        assert_eq!(pipes.outgoing.try_recv().ok(), Some(b"sup".to_vec()));
        assert_eq!(f.rerouted_total(), 1);
        f.set_moved(false);
        f.replay().await;
        assert_eq!(f.replayed(), 0);
        assert!(fake.recorded().is_empty());
    }
}
