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

use std::io::{self, IoSliceMut};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::task::{Context, Poll};
use std::time::Duration;

use quinn::udp::{RecvMeta, Transmit};
use quinn::{AsyncUdpSocket, UdpPoller};
use tokio::sync::mpsc;

/// The stand-in address of the other device through the relay. 240.0.0.0/4
/// is reserved (RFC 1112) and never routed, so it cannot collide with a real
/// address.
pub const RELAY_PEER: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(240, 84, 68, 1), 1));

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
        }))
    }

    fn now(&self) -> u64 {
        u64::try_from(self.0.start.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    fn direct(&self) -> Option<SocketAddr> {
        self.0.direct.read().ok().and_then(|d| *d)
    }

    /// Whether datagrams for `addr` go through the relay instead.
    fn reroutes(&self, addr: SocketAddr) -> bool {
        self.0.moved.load(Ordering::Acquire) && self.direct() == Some(addr)
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
            && meta.iter().any(|m| m.addr == direct)
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
                    self.0.moved_at.store(now, Ordering::Release);
                    self.0.moved.store(true, Ordering::Release);
                    gate.open();
                }
                // The relay link closes itself once it has been idle.
                Step::Back => self.0.moved.store(false, Ordering::Release),
            }
        }
    }
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
    if s.direct != Some(s.remote) && s.remote != RELAY_PEER {
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

/// A UDP socket that also reaches the other device through the relay.
#[derive(Debug)]
pub struct PathSocket {
    inner: Arc<dyn AsyncUdpSocket>,
    to_relay: mpsc::Sender<Vec<u8>>,
    from_relay: Mutex<mpsc::Receiver<Vec<u8>>>,
    fallback: Fallback,
}

impl PathSocket {
    /// Wrap `inner`. The returned pipes connect it to a relay link. Until the
    /// link is up, datagrams for the relay wait in a short queue; once it is
    /// full, or when no link runs, they are dropped. The [`Fallback`] moves a
    /// direct connection onto the relay.
    pub fn new(inner: Arc<dyn AsyncUdpSocket>) -> (Arc<Self>, RelayPipes, Fallback) {
        let (to_relay, outgoing) = mpsc::channel(QUEUE);
        let (incoming, from_relay) = mpsc::channel(QUEUE);
        let fallback = Fallback::new(to_relay.clone());
        (
            Arc::new(Self {
                inner,
                to_relay,
                from_relay: Mutex::new(from_relay),
                fallback: fallback.clone(),
            }),
            RelayPipes { outgoing, incoming },
            fallback,
        )
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
                        addr: self.fallback.relay_source(),
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
        if transmit.destination == RELAY_PEER || self.fallback.reroutes(transmit.destination) {
            self.relay_send(transmit);
            Ok(())
        } else {
            self.inner.try_send(transmit)
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
}
