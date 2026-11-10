// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Asking the home router to forward the transfer's port (decision 0018,
//! step 2; #406). With PCP (RFC 6887) or NAT-PMP (RFC 6886) the router
//! opens an external UDP port to our QUIC socket, and the other device can
//! dial it even where hole punching fails.
//!
//! A mapping is short-lived: it is requested for [`LIFETIME`], renewed
//! while the transfer runs and released when the [`Mapping`] is dropped. If
//! the app dies, the router closes it on its own once the lifetime it
//! granted ends. Only the gateway of the default route is asked; a
//! connected socket drops answers from anyone else.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::time::Instant;
use totaldrop_proto::portmap::{self as wire, Answer};

/// Lifetime asked for, in seconds. A router may grant more or less; the
/// mapping is renewed at half of what it grants.
pub const LIFETIME: u32 = 120;

/// How long one request waits for an answer before it is sent again.
const RESEND: Duration = Duration::from_millis(250);

/// Sends of one request before giving up on that protocol.
const TRIES: u32 = 3;

/// Upper bound for the whole attempt.
pub const LIMIT: Duration = Duration::from_millis(1600);

/// Shortest and longest wait between renewals, whatever the router grants.
const RENEW_MIN: Duration = Duration::from_secs(1);
const RENEW_MAX: Duration = Duration::from_secs(30 * 60);

/// Wait before trying a renewal again after one went unanswered.
const RENEW_RETRY: Duration = Duration::from_secs(5);

/// Which router to ask.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Gateway {
    /// The gateway of the default IPv4 route, where the system tells us
    /// (Linux today; other systems ask no one yet).
    #[default]
    Auto,
    /// Ask no one.
    Off,
    /// This address, for a platform that knows its gateway (Android) and
    /// for tests.
    At(SocketAddr),
}

impl Gateway {
    fn address(&self) -> Option<SocketAddr> {
        match self {
            Self::Auto => default_gateway().map(|ip| SocketAddr::from((ip, wire::SERVER_PORT))),
            Self::Off => None,
            Self::At(a) => Some(*a),
        }
    }
}

/// Which protocol holds the mapping, with what releasing it needs.
#[derive(Clone, Copy, Debug)]
enum Held {
    Pcp { nonce: [u8; 12], client: IpAddr },
    Pmp,
}

/// A port the router forwards to us. Dropping it stops the renewals and
/// asks the router to close the port.
#[derive(Debug)]
pub struct Mapping {
    external: SocketAddr,
    renew: tokio::task::JoinHandle<()>,
    release: Option<Release>,
}

impl Mapping {
    /// The public address and port the other device can dial.
    pub fn external(&self) -> SocketAddr {
        self.external
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        self.renew.abort();
        if let Some(r) = self.release.take() {
            r.send();
        }
    }
}

/// What it takes to ask the router to close a port.
#[derive(Debug)]
struct Release {
    socket: Arc<UdpSocket>,
    held: Held,
    port: u16,
}

impl Release {
    /// Send the release at once, without waiting: the process may be about
    /// to exit and take its runtime with it. A second copy follows if the
    /// runtime is still there. Releasing is a courtesy; the lifetime closes
    /// the port anyway.
    fn send(self) {
        let msg = request(self.held, self.port, 0);
        let _ = self.socket.try_send(&msg);
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                tokio::time::sleep(RESEND).await;
                let _ = self.socket.send(&msg).await;
            });
        }
    }
}

/// Ask the router for a mapping to our UDP `port`. `None` when there is no
/// gateway to ask, it does not answer within [`LIMIT`], refuses, or gives
/// an address that is not public (a second NAT behind the router, or the
/// operator's carrier-grade NAT, where the port would not help).
pub async fn map(gateway: Gateway, port: u16) -> Option<Mapping> {
    let target = gateway.address()?;
    tokio::time::timeout(LIMIT, map_at(target, port))
        .await
        .ok()
        .flatten()
}

async fn map_at(target: SocketAddr, port: u16) -> Option<Mapping> {
    if port == 0 || !target.is_ipv4() {
        return None;
    }
    let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
        .await
        .ok()?;
    socket.connect(target).await.ok()?;
    let client = socket.local_addr().ok()?.ip();
    let socket = Arc::new(socket);

    let mut nonce = [0u8; 12];
    getrandom::fill(&mut nonce).ok()?;
    let pcp = Held::Pcp { nonce, client };
    let (held, external, lifetime) = match ask(&socket, pcp, port).await {
        Got::Mapped(external, lifetime) => (pcp, external, lifetime),
        Got::OtherVersion | Got::Silent => {
            let (external, lifetime) = ask_pmp(&socket, port).await?;
            (Held::Pmp, external, lifetime)
        }
        Got::Refused => return None,
    };
    let release = Release {
        socket: Arc::clone(&socket),
        held,
        port,
    };
    if lifetime == 0 || !public(external.ip()) {
        // The router may have opened a port we will not use.
        release.send();
        return None;
    }
    let renew = tokio::spawn(renew(socket, held, port, lifetime));
    Some(Mapping {
        external,
        renew,
        release: Some(release),
    })
}

/// What one protocol gave.
enum Got {
    Mapped(SocketAddr, u32),
    Refused,
    OtherVersion,
    Silent,
}

fn request(held: Held, port: u16, lifetime: u32) -> Vec<u8> {
    match held {
        Held::Pcp { nonce, client } => wire::pcp_map(&nonce, client, port, lifetime).to_vec(),
        Held::Pmp => wire::pmp_map(port, lifetime).to_vec(),
    }
}

/// Send the mapping request for `held` and read the answer.
async fn ask(socket: &UdpSocket, held: Held, port: u16) -> Got {
    let msg = request(held, port, LIFETIME);
    let parse = |m: &[u8]| -> Option<Got> {
        match held {
            Held::Pcp { nonce, .. } => Some(match wire::parse_pcp_map(m, &nonce, port)? {
                Answer::Mapped { external, lifetime } => {
                    Got::Mapped(SocketAddr::from(external), lifetime)
                }
                Answer::Refused(_) => Got::Refused,
                Answer::OtherVersion => Got::OtherVersion,
            }),
            // The address comes from a separate request; 0.0.0.0 stands
            // in until then.
            Held::Pmp => Some(match wire::parse_pmp_map(m, port)? {
                Ok((p, lifetime)) => {
                    Got::Mapped(SocketAddr::from((Ipv4Addr::UNSPECIFIED, p)), lifetime)
                }
                Err(_) => Got::Refused,
            }),
        }
    };
    exchange(socket, &msg, parse).await.unwrap_or(Got::Silent)
}

/// NAT-PMP: the public address, then the mapping.
async fn ask_pmp(socket: &UdpSocket, port: u16) -> Option<(SocketAddr, u32)> {
    let ip = exchange(socket, &wire::pmp_address(), wire::parse_pmp_address)
        .await?
        .ok()?;
    match ask(socket, Held::Pmp, port).await {
        Got::Mapped(external, lifetime) => {
            Some((SocketAddr::from((ip, external.port())), lifetime))
        }
        _ => None,
    }
}

/// Send `msg` up to [`TRIES`] times and return the first datagram `parse`
/// accepts. Datagrams it refuses (stray or forged answers) are skipped.
async fn exchange<T>(
    socket: &UdpSocket,
    msg: &[u8],
    parse: impl Fn(&[u8]) -> Option<T>,
) -> Option<T> {
    let mut buf = [0u8; wire::PCP_MAX_LEN];
    for _ in 0..TRIES {
        socket.send(msg).await.ok()?;
        let deadline = Instant::now().checked_add(RESEND)?;
        loop {
            let n = match tokio::time::timeout_at(deadline, socket.recv(&mut buf)).await {
                Ok(Ok(n)) => n,
                // An ICMP error (nothing listens on the gateway) reads as an
                // error on a connected socket: no server there.
                Ok(Err(_)) => return None,
                Err(_) => break,
            };
            if let Some(got) = buf.get(..n).and_then(&parse) {
                return Some(got);
            }
        }
    }
    None
}

/// Keep the mapping alive: renew at half of each granted lifetime, and
/// after an unanswered renewal try again every [`RENEW_RETRY`] until the
/// mapping would have run out (RFC 6887, section 11.2.1).
async fn renew(socket: Arc<UdpSocket>, held: Held, port: u16, lifetime: u32) {
    let granted_at = |lifetime: u32| {
        let full = Duration::from_secs(u64::from(lifetime));
        let half = Duration::from_secs(u64::from(lifetime / 2)).clamp(RENEW_MIN, RENEW_MAX);
        let now = Instant::now();
        (now.checked_add(half), now.checked_add(full))
    };
    let (mut next, mut expiry) = granted_at(lifetime);
    loop {
        let (Some(at), Some(end)) = (next, expiry) else {
            return;
        };
        tokio::time::sleep_until(at).await;
        if let Got::Mapped(_, granted) = ask(&socket, held, port).await
            && granted != 0
        {
            (next, expiry) = granted_at(granted);
        } else if Instant::now() < end {
            next = Instant::now().checked_add(RENEW_RETRY);
        } else {
            return;
        }
    }
}

/// Whether the router's external address is one the internet can reach:
/// not private, shared (100.64.0.0/10), loopback, link-local, "this
/// network" (0.0.0.0/8), IETF protocol assignments (192.0.0.0/24),
/// benchmarking (198.18.0.0/15), reserved (240.0.0.0/4, broadcast among
/// them) or multicast.
fn public(ip: IpAddr) -> bool {
    let IpAddr::V4(v4) = ip else {
        return false;
    };
    let [a, b, c, _] = v4.octets();
    let shared = a == 100 && (b & 0xc0) == 64;
    let this_network = a == 0;
    let protocol = a == 192 && b == 0 && c == 0;
    let benchmark = a == 198 && (b & 0xfe) == 18;
    let reserved = a >= 240;
    !(v4.is_private()
        || shared
        || this_network
        || protocol
        || benchmark
        || reserved
        || v4.is_loopback()
        || v4.is_link_local()
        || v4.is_multicast())
}

/// The gateway of the default IPv4 route.
#[cfg(target_os = "linux")]
fn default_gateway() -> Option<Ipv4Addr> {
    // Android does not let apps read this file; it passes the gateway in.
    std::fs::read_to_string("/proc/net/route")
        .ok()
        .and_then(|t| gateway_in_route_table(&t))
}

#[cfg(not(target_os = "linux"))]
fn default_gateway() -> Option<Ipv4Addr> {
    None
}

/// The gateway of the default route with the lowest metric that is up, in
/// Linux's `/proc/net/route`: addresses in hexadecimal, in the machine's
/// byte order (little-endian on every platform we build for).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn gateway_in_route_table(table: &str) -> Option<Ipv4Addr> {
    const UP: u32 = 0x1;
    const GATEWAY: u32 = 0x2;
    let mut best: Option<(u32, Ipv4Addr)> = None;
    for line in table.lines().skip(1) {
        // Iface Destination Gateway Flags RefCnt Use Metric ...
        let f: Vec<&str> = line.split_whitespace().collect();
        let hex = |i: usize| f.get(i).and_then(|s| u32::from_str_radix(s, 16).ok());
        let metric = f
            .get(6)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(u32::MAX);
        let (Some(dest), Some(gw), Some(flags)) = (hex(1), hex(2), hex(3)) else {
            continue;
        };
        if dest == 0
            && flags & (UP | GATEWAY) == UP | GATEWAY
            && gw != 0
            && best.is_none_or(|(m, _)| metric < m)
        {
            best = Some((metric, Ipv4Addr::from(gw.to_le_bytes())));
        }
    }
    best.map(|(_, gw)| gw)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing)]

    use super::*;
    use tokio::sync::mpsc;
    use totaldrop_proto::portmap::answer;

    const PUBLIC: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 7);
    const OPENED: u16 = 51_000;

    #[derive(Clone, Copy)]
    enum Router {
        Pcp,
        Pmp,
        Silent,
        Refuses,
        Private,
        /// Grants 4 seconds whatever is asked.
        Short,
    }

    /// A router on loopback. Every request it receives is passed on, so
    /// tests can see renewals and releases.
    async fn router(kind: Router) -> (Gateway, mpsc::UnboundedReceiver<Vec<u8>>) {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let at = socket.local_addr().unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut buf = [0u8; 1500];
            loop {
                // On Windows an answer to a socket that is already closed
                // comes back as an error on the next receive (ICMP port
                // unreachable); a real router keeps serving, so does this one.
                let (n, from) = match socket.recv_from(&mut buf).await {
                    Ok(got) => got,
                    Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => continue,
                    Err(_) => break,
                };
                let m = buf[..n].to_vec();
                let _ = tx.send(m.clone());
                let reply: Option<Vec<u8>> = match (kind, m[0]) {
                    (Router::Silent, _) => None,
                    (Router::Pcp | Router::Refuses | Router::Private | Router::Short, 2) => {
                        let req: [u8; 60] = m[..60].try_into().unwrap();
                        let mut lifetime = u32::from_be_bytes(m[4..8].try_into().unwrap());
                        if matches!(kind, Router::Short) && lifetime != 0 {
                            lifetime = 4;
                        }
                        Some(
                            match kind {
                                Router::Refuses => answer::pcp(&req, 2, (PUBLIC.into(), 0), 0),
                                Router::Private => answer::pcp(
                                    &req,
                                    0,
                                    (Ipv4Addr::new(100, 72, 1, 2).into(), OPENED),
                                    lifetime,
                                ),
                                _ => answer::pcp(&req, 0, (PUBLIC.into(), OPENED), lifetime),
                            }
                            .to_vec(),
                        )
                    }
                    (Router::Pmp, 2) => Some(answer::pmp_unsupported_version().to_vec()),
                    (Router::Pmp, 0) if m.len() == 2 => {
                        Some(answer::pmp_address(0, PUBLIC).to_vec())
                    }
                    (Router::Pmp, 0) => {
                        let internal = u16::from_be_bytes(m[4..6].try_into().unwrap());
                        let lifetime = u32::from_be_bytes(m[8..12].try_into().unwrap());
                        let opened = if lifetime == 0 { 0 } else { OPENED };
                        Some(answer::pmp_map(0, internal, opened, lifetime).to_vec())
                    }
                    _ => None,
                };
                if let Some(r) = reply {
                    // A stray datagram first: an answer to someone else.
                    let _ = socket.send_to(&[2, 0x81, 0, 0], from).await;
                    let _ = socket.send_to(&r, from).await;
                }
            }
        });
        (Gateway::At(at), rx)
    }

    fn lifetime_of(m: &[u8]) -> u32 {
        let at = if m[0] == 2 { 4 } else { 8 };
        u32::from_be_bytes(m[at..][..4].try_into().unwrap())
    }

    #[tokio::test]
    async fn pcp_mapping_is_announced_and_released() {
        let (gw, mut seen) = router(Router::Pcp).await;
        let m = map(gw.clone(), 40_000).await.unwrap();
        assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED)));
        let req = seen.recv().await.unwrap();
        assert_eq!(req[0], 2);
        assert_eq!(lifetime_of(&req), LIFETIME);
        drop(m);
        let release = seen.recv().await.unwrap();
        assert_eq!((release[0], lifetime_of(&release)), (2, 0));
        assert_eq!(release[24..36], req[24..36], "same nonce");
    }

    #[tokio::test]
    async fn nat_pmp_is_used_when_pcp_is_not_spoken() {
        let (gw, mut seen) = router(Router::Pmp).await;
        let m = map(gw.clone(), 40_000).await.unwrap();
        assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED)));
        drop(m);
        // PCP, then NAT-PMP address and mapping, then the release.
        let mut last = Vec::new();
        while let Ok(Some(r)) = tokio::time::timeout(Duration::from_secs(1), seen.recv()).await {
            last = r;
            if last.len() == 12 && lifetime_of(&last) == 0 {
                break;
            }
        }
        assert_eq!((last[0], last[1], lifetime_of(&last)), (0, 1, 0));
    }

    // The clock is paused only once the mapping is made: a paused clock
    // jumps ahead whenever nothing is runnable, also while a real datagram
    // is still on its way, so the attempt would time out at random.
    #[tokio::test]
    async fn mapping_is_renewed_before_it_runs_out() {
        let (gw, mut seen) = router(Router::Pcp).await;
        let m = map(gw.clone(), 40_000).await.unwrap();
        let first = seen.recv().await.unwrap();
        tokio::time::pause();
        tokio::time::sleep(Duration::from_secs(u64::from(LIFETIME / 2) + 1)).await;
        let again = seen.recv().await.unwrap();
        assert_eq!(lifetime_of(&again), LIFETIME);
        assert_eq!(again[24..36], first[24..36]);
        drop(m);
    }

    #[tokio::test]
    async fn a_silent_or_refusing_router_gives_nothing_in_time() {
        for kind in [Router::Silent, Router::Refuses, Router::Private] {
            let (gw, _seen) = router(kind).await;
            let started = std::time::Instant::now();
            assert!(map(gw.clone(), 40_000).await.is_none());
            assert!(started.elapsed() <= LIMIT + Duration::from_millis(300));
        }
    }

    /// A port opened on an address we do not use is closed again.
    #[tokio::test]
    async fn an_unusable_mapping_is_released() {
        let (gw, mut seen) = router(Router::Private).await;
        assert!(map(gw, 40_000).await.is_none());
        let first = seen.recv().await.unwrap();
        let release = seen.recv().await.unwrap();
        assert_eq!(lifetime_of(&first), LIFETIME);
        assert_eq!(lifetime_of(&release), 0);
    }

    #[tokio::test]
    async fn a_short_grant_is_renewed_in_time() {
        let (gw, mut seen) = router(Router::Short).await;
        let m = map(gw, 40_000).await.unwrap();
        let _ = seen.recv().await.unwrap();
        tokio::time::pause();
        let started = Instant::now();
        let again = seen.recv().await.unwrap();
        assert_eq!(lifetime_of(&again), LIFETIME);
        assert!(
            started.elapsed() <= Duration::from_secs(3),
            "{:?}",
            started.elapsed()
        );
        drop(m);
    }

    /// The release goes out even when the runtime ends right after the
    /// mapping is dropped, as when the command line exits.
    #[test]
    fn release_is_sent_before_the_runtime_ends() {
        let (tx, rx) = std::sync::mpsc::channel();
        let gw = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let at = gw.local_addr().unwrap();
        std::thread::spawn(move || {
            let mut buf = [0u8; 1500];
            while let Ok((n, from)) = gw.recv_from(&mut buf) {
                let m = buf[..n].to_vec();
                if m.len() == 60 && lifetime_of(&m) != 0 {
                    let req: [u8; 60] = m[..].try_into().unwrap();
                    let a = answer::pcp(&req, 0, (PUBLIC.into(), OPENED), LIFETIME);
                    let _ = gw.send_to(&a, from);
                }
                let _ = tx.send(m);
            }
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let m = map(Gateway::At(at), 40_000).await.unwrap();
            drop(m);
        });
        drop(rt);
        let _request = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let release = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(lifetime_of(&release), 0);
    }

    #[tokio::test]
    async fn no_gateway_asks_no_one() {
        assert!(map(Gateway::Off, 40_000).await.is_none());
        // Nothing listens there: the ICMP error ends the attempt early.
        let closed = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let at = closed.local_addr().unwrap();
        drop(closed);
        let started = std::time::Instant::now();
        assert!(map(Gateway::At(at), 40_000).await.is_none());
        assert!(started.elapsed() < LIMIT);
        let v6 = Gateway::At("[::1]:5351".parse().unwrap());
        assert!(map(v6, 40_000).await.is_none());
        assert!(
            map(Gateway::At("127.0.0.1:5351".parse().unwrap()), 0)
                .await
                .is_none()
        );
    }

    /// The mapped address goes into our `HINTS` and the mapping lives as
    /// long as the caller keeps it; privacy mode asks the router nothing.
    #[tokio::test]
    async fn hints_carry_the_mapping_except_in_privacy_mode() {
        let (gw, mut seen) = router(Router::Pcp).await;
        let socket = crate::transport::bind_socket().unwrap();
        let key = crate::transport::EndpointKey::generate().unwrap();
        let mut paths = crate::PathOptions {
            relay: String::new(),
            stun: String::new(),
            relay_only: false,
            gateway: gw,
        };
        let (h, mapping) = crate::own_hints(&socket, &key, &paths).await.unwrap();
        let external = SocketAddr::from((PUBLIC, OPENED));
        assert_eq!(
            h.public.iter().map(|a| a.0).collect::<Vec<_>>(),
            vec![external]
        );
        assert_eq!(mapping.as_ref().map(Mapping::external), Some(external));
        let req = seen.recv().await.unwrap();
        let port = socket.local_addr().unwrap().port();
        assert_eq!(req[40..42], port.to_be_bytes(), "the QUIC socket's port");
        drop(mapping);
        assert_eq!(lifetime_of(&seen.recv().await.unwrap()), 0);
        // The release is sent twice; let it finish.
        tokio::time::sleep(RESEND * 3).await;
        while seen.try_recv().is_ok() {}

        paths.relay = "wss://relay.example".into();
        paths.relay_only = true;
        let (h, mapping) = crate::own_hints(&socket, &key, &paths).await.unwrap();
        assert!(h.public.is_empty() && h.local.is_empty() && mapping.is_none());
        let asked = tokio::time::timeout(Duration::from_millis(300), seen.recv()).await;
        assert!(asked.is_err(), "privacy mode asked the router: {asked:?}");
    }

    /// A router that never answers holds the transfer back by the grace
    /// period at most, not by the whole attempt.
    #[tokio::test]
    async fn a_silent_router_barely_delays_the_hints() {
        let (gw, _seen) = router(Router::Silent).await;
        let socket = crate::transport::bind_socket().unwrap();
        let key = crate::transport::EndpointKey::generate().unwrap();
        let paths = crate::PathOptions {
            relay: String::new(),
            stun: String::new(),
            relay_only: false,
            gateway: gw,
        };
        let started = std::time::Instant::now();
        let (_, mapping) = crate::own_hints(&socket, &key, &paths).await.unwrap();
        assert!(mapping.is_none());
        assert!(
            started.elapsed() < Duration::from_millis(700),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn only_public_addresses_count() {
        for ip in ["203.0.113.7", "8.8.8.8"] {
            assert!(public(ip.parse().unwrap()), "{ip}");
        }
        for ip in [
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "100.64.0.1",
            "100.127.255.254",
            "127.0.0.1",
            "169.254.1.1",
            "0.0.0.0",
            "255.255.255.255",
            "224.0.0.1",
            "2001:db8::1",
        ] {
            assert!(!public(ip.parse().unwrap()), "{ip}");
        }
        assert!(public("100.128.0.1".parse().unwrap()));
        for ip in [
            "0.1.2.3",
            "192.0.0.9",
            "198.18.0.1",
            "198.19.255.1",
            "240.0.0.1",
        ] {
            assert!(!public(ip.parse().unwrap()), "{ip}");
        }
        assert!(public("198.20.0.1".parse().unwrap()));
    }

    #[test]
    fn default_route_gateway_is_read() {
        let table = "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT\n\
                     enp8s0\t0001A8C0\t00000000\t0001\t0\t0\t100\t00FFFFFF\t0\t0\t0\n\
                     wg0\t00000000\t00000000\t0001\t0\t0\t50\t00000000\t0\t0\t0\n\
                     enp8s0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0\n";
        assert_eq!(
            gateway_in_route_table(table),
            Some(Ipv4Addr::new(192, 168, 1, 1))
        );
        assert_eq!(gateway_in_route_table("Iface\tDestination\n"), None);
        assert_eq!(gateway_in_route_table(""), None);
        let down = "h\nenp8s0\t00000000\t0101A8C0\t0002\t0\n";
        assert_eq!(gateway_in_route_table(down), None);
        assert_eq!(gateway_in_route_table("h\nbad line\n"), None);
        let two = "h\n\
                   wlan0\t00000000\t0101A8C0\t0003\t0\t0\t600\t00000000\n\
                   enp8s0\t00000000\t01020A0A\t0003\t0\t0\t100\t00000000\n";
        assert_eq!(
            gateway_in_route_table(two),
            Some(Ipv4Addr::new(10, 10, 2, 1)),
            "lowest metric"
        );
        let after_bad = "h\nbad line\nenp8s0\t00000000\t0101A8C0\t0003\t0\n";
        assert_eq!(
            gateway_in_route_table(after_bad),
            Some(Ipv4Addr::new(192, 168, 1, 1))
        );
    }
}
