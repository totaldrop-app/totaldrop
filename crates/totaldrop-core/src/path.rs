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

use std::io::{self, IoSliceMut};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

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

/// A UDP socket that also reaches the other device through the relay.
#[derive(Debug)]
pub struct PathSocket {
    inner: Arc<dyn AsyncUdpSocket>,
    to_relay: mpsc::Sender<Vec<u8>>,
    from_relay: Mutex<mpsc::Receiver<Vec<u8>>>,
}

impl PathSocket {
    /// Wrap `inner`. The returned pipes connect it to a relay link. Until the
    /// link is up, datagrams for the relay wait in a short queue; once it is
    /// full, or when no link runs, they are dropped.
    pub fn new(inner: Arc<dyn AsyncUdpSocket>) -> (Arc<Self>, RelayPipes) {
        let (to_relay, outgoing) = mpsc::channel(QUEUE);
        let (incoming, from_relay) = mpsc::channel(QUEUE);
        (
            Arc::new(Self {
                inner,
                to_relay,
                from_relay: Mutex::new(from_relay),
            }),
            RelayPipes { outgoing, incoming },
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
                        addr: RELAY_PEER,
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
        if transmit.destination == RELAY_PEER {
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
        self.inner.poll_recv(cx, bufs, meta)
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
