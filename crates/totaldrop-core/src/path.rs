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
//! each side can move on its own clock (#185).

use std::io::{self, IoSliceMut};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
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

/// Moves a direct connection onto the relay when its path goes quiet: a
/// Wi-Fi that drops, a NAT mapping that expires, a network that lets the
/// handshake through and then stops carrying anything (#185). Cloning
/// shares the state.
#[derive(Debug, Clone)]
pub struct Fallback(Arc<FallbackState>);

#[derive(Debug)]
struct FallbackState {
    /// The direct address being watched, set once by [`Fallback::watch`].
    direct: OnceLock<SocketAddr>,
    /// Whether the connection to `direct` now goes through the relay. It
    /// never moves back.
    moved: AtomicBool,
    /// When a datagram from `direct` last arrived, in milliseconds after
    /// `start`.
    heard: AtomicU64,
    start: tokio::time::Instant,
}

impl Fallback {
    fn new() -> Self {
        Self(Arc::new(FallbackState {
            direct: OnceLock::new(),
            moved: AtomicBool::new(false),
            heard: AtomicU64::new(0),
            start: tokio::time::Instant::now(),
        }))
    }

    fn now(&self) -> u64 {
        u64::try_from(self.0.start.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// Whether datagrams for `addr` go through the relay instead.
    fn reroutes(&self, addr: SocketAddr) -> bool {
        self.0.moved.load(Ordering::Acquire) && self.0.direct.get() == Some(&addr)
    }

    /// The address datagrams from the relay appear to come from.
    fn relay_source(&self) -> SocketAddr {
        if self.0.moved.load(Ordering::Acquire) {
            self.0.direct.get().copied().unwrap_or(RELAY_PEER)
        } else {
            RELAY_PEER
        }
    }

    /// Note datagrams that came in on the real socket.
    fn heard_from(&self, meta: &[RecvMeta]) {
        if let Some(direct) = self.0.direct.get()
            && meta.iter().any(|m| m.addr == *direct)
        {
            self.0.heard.store(self.now(), Ordering::Release);
        }
    }

    /// Whether the connection moved onto the relay.
    pub fn moved(&self) -> bool {
        self.0.moved.load(Ordering::Acquire)
    }

    /// Watch the direct connection to `direct` and, once nothing came from
    /// it for `silence`, send it through the relay instead and open `gate`
    /// so the relay link connects. Returns once it moved; a watch on
    /// another address than the first one returns at once.
    pub async fn watch(&self, direct: SocketAddr, gate: crate::relay::Gate, silence: Duration) {
        self.0.heard.store(self.now(), Ordering::Release);
        if *self.0.direct.get_or_init(|| direct) != direct {
            return;
        }
        let silence = u64::try_from(silence.as_millis()).unwrap_or(u64::MAX);
        let mut tick = tokio::time::interval(WATCH_TICK);
        loop {
            tick.tick().await;
            let quiet = self
                .now()
                .saturating_sub(self.0.heard.load(Ordering::Acquire));
            if quiet >= silence {
                // Open the gate first: datagrams wait in the queue until the
                // link is up.
                gate.open();
                self.0.moved.store(true, Ordering::Release);
                return;
            }
        }
    }
}

/// A [`Fallback::watch`] that runs until dropped.
#[derive(Debug)]
pub struct Watch(tokio::task::JoinHandle<()>);

impl Watch {
    /// Run `fallback.watch(direct, gate, silence)` in the background.
    pub fn spawn(
        fallback: &Fallback,
        direct: SocketAddr,
        gate: crate::relay::Gate,
        silence: Duration,
    ) -> Self {
        let fallback = fallback.clone();
        Self(tokio::spawn(async move {
            fallback.watch(direct, gate, silence).await;
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
        let fallback = Fallback::new();
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
