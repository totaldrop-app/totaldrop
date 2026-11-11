// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Asking the home router to forward the transfer's port (decision 0018,
//! step 2; #406). With PCP (RFC 6887) or NAT-PMP (RFC 6886) the router
//! opens an external UDP port to our QUIC socket, and the other device can
//! dial it even where hole punching fails. Routers that speak neither
//! answer UPnP IGD (`igd-next`), asked only at the same gateway's address.
//!
//! A mapping is short-lived: it is requested for [`LIFETIME`], renewed
//! while the transfer runs and released when the [`Mapping`] is dropped. If
//! the app dies, the router closes it on its own once the lifetime it
//! granted ends. Only the gateway of the default route is asked; a
//! connected socket drops answers from anyone else.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use igd_next::aio::tokio::Tokio;
use igd_next::{GatewayIpVersion, PortMappingProtocol, SearchOptions};
use tokio::net::UdpSocket;
use tokio::sync::{oneshot, watch};
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

/// The port a UPnP gateway listens on for SSDP searches.
const SSDP_PORT: u16 = 1900;

/// How the mapping is labelled in the router's list.
const UPNP_DESCRIPTION: &str = "TotalDrop";

type Igd = igd_next::aio::Gateway<Tokio>;

/// Which router to ask.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Gateway {
    /// The gateway of the default IPv4 route, where the system tells us
    /// (Linux today; other systems ask no one yet).
    #[default]
    Auto,
    /// Ask no one.
    Off,
    /// This address, for tests and for a platform that knows its gateway
    /// and may pass it in (Android could; it does not yet).
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
    /// Counted for [`released`] while the mapping lives; dropped after
    /// `release` has started the delete, which counts itself.
    pending: Option<Pending>,
    /// The race's loser, when it holds the same port (#425).
    later: LaterSlot,
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
        release_later(&self.later);
    }
}

/// UPnP attempts and deletes, and race losers, still under way, for
/// [`released`].
static RELEASING: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static RELEASED: tokio::sync::Notify = tokio::sync::Notify::const_new();

/// Counts, for [`released`], one UPnP attempt, mapping or delete, or one
/// PCP or NAT-PMP race loser waiting for its answer (#461), that may still
/// leave a port open. Dropping it, also with a future that never
/// ran, takes it off the count.
#[derive(Debug)]
struct Pending;

impl Pending {
    fn new() -> Self {
        RELEASING.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        Self
    }
}

impl Clone for Pending {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        RELEASING.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        RELEASED.notify_waiters();
    }
}

/// Wait, at most [`LIMIT`], until no UPnP attempt is still under way,
/// every dropped UPnP mapping has been deleted, and every PCP or NAT-PMP
/// race loser has had its answer and released a second mapping (#461). A
/// process about to exit calls this before it shuts its runtime down;
/// without it the delete or release dies with the runtime and the port
/// stays open until its lease runs out.
pub async fn released() {
    use std::sync::atomic::Ordering;
    let _ = tokio::time::timeout(LIMIT, async {
        loop {
            let notified = RELEASED.notified();
            if RELEASING.load(Ordering::Acquire) == 0 {
                return;
            }
            notified.await;
        }
    })
    .await;
}

/// What it takes to ask the router to close a port.
#[derive(Debug)]
enum Release {
    Udp {
        socket: Arc<UdpSocket>,
        held: Held,
        port: u16,
    },
    Upnp {
        gateway: Arc<Igd>,
        external_port: u16,
    },
}

impl Release {
    /// Send the release at once, without waiting: the process may be about
    /// to exit and take its runtime with it. A second copy follows if the
    /// runtime is still there. Releasing is a courtesy; the lifetime closes
    /// the port anyway.
    fn send(self) {
        match self {
            Self::Udp { socket, held, port } => {
                let msg = request(held, port, 0);
                let _ = socket.try_send(&msg);
                if let Ok(rt) = tokio::runtime::Handle::try_current() {
                    rt.spawn(async move {
                        tokio::time::sleep(RESEND).await;
                        let _ = socket.send(&msg).await;
                    });
                }
            }
            // UPnP takes an HTTP request, which cannot go out without the
            // runtime; without it the lease closes the port.
            Self::Upnp {
                gateway,
                external_port,
            } => {
                if let Ok(rt) = tokio::runtime::Handle::try_current() {
                    let pending = Pending::new();
                    rt.spawn(async move {
                        let _pending = pending;
                        let _ = tokio::time::timeout(
                            LIMIT,
                            gateway.remove_port(PortMappingProtocol::UDP, external_port),
                        )
                        .await;
                    });
                }
            }
        }
    }
}

/// The protocol that lost the race for the mapping, still waiting for its
/// answer (#425).
struct Loser {
    answer: std::pin::Pin<Box<dyn Future<Output = Option<SocketAddr>> + Send>>,
    release: Release,
}

/// A loser's release that waits for the winner's [`Mapping`] to go.
#[derive(Debug, Default)]
struct Later {
    /// The mapping was dropped (or never made): send at once.
    now: bool,
    release: Option<Release>,
}

type LaterSlot = Arc<std::sync::Mutex<Later>>;

impl Loser {
    /// What to do with the loser's mapping once its answer comes, if it
    /// comes within [`LIMIT`] (a later one is not waited for, and the
    /// router closes that mapping once its lifetime ends):
    /// - on another external port than the winner's, `kept`, it is a
    ///   second mapping: released at once;
    /// - on the same port it is, in a router that keeps one table for
    ///   both protocols, the winner's own mapping, perhaps now held under
    ///   the loser's protocol (miniupnpd relabels a rule that NAT-PMP
    ///   asks for again; unverified). Releasing it now would close the
    ///   port in use, so the release waits in `later` and goes with the
    ///   winner's when the [`Mapping`] is dropped;
    /// - with no winner kept (`None`: its mapping was not usable), it is
    ///   released at once.
    async fn close(self, kept: Option<u16>, later: LaterSlot) {
        let Ok(Some(external)) = tokio::time::timeout(LIMIT, self.answer).await else {
            return;
        };
        if kept != Some(external.port()) {
            self.release.send();
            return;
        }
        let mut slot = later
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot.now {
            drop(slot);
            self.release.send();
        } else {
            slot.release = Some(self.release);
        }
    }
}

/// Send a loser's release that waited for the mapping, or let a loser
/// still waiting for its answer send it at once.
fn release_later(later: &LaterSlot) {
    let mut slot = later
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    slot.now = true;
    if let Some(r) = slot.release.take() {
        drop(slot);
        r.send();
    }
}

/// Start asking the router for a mapping to our UDP `port`; [`Asking::wait`]
/// gives the result. There is none when there is no gateway to ask, it does
/// not answer within [`LIMIT`], refuses, or gives an address that is not
/// public (a second NAT behind the router, or the operator's carrier-grade
/// NAT, where the port would not help).
pub fn start(gateway: Gateway, port: u16) -> Asking {
    match gateway.address() {
        Some(target) => start_at(target, Some(SocketAddr::new(target.ip(), SSDP_PORT)), port),
        None => Asking {
            task: tokio::spawn(async { None }),
            upnp: watch::channel(Upnp::Searching).1,
        },
    }
}

fn start_at(target: SocketAddr, ssdp: Option<SocketAddr>, port: u16) -> Asking {
    let (upnp_tx, upnp) = watch::channel(Upnp::Searching);
    // Counted from now: the task may not have run yet when the process
    // decides to exit.
    let pending = ssdp.map(|_| Pending::new());
    let task = tokio::spawn(async move {
        let got = tokio::time::timeout(
            LIMIT,
            map_with(target, ssdp, port, upnp_tx, pending.clone()),
        )
        .await
        .ok()
        .flatten();
        got.map(|mut m| {
            if m.pending.is_none() {
                m.pending = pending;
            }
            m
        })
    });
    Asking { task, upnp }
}

/// A mapping being asked for, from [`start`].
pub struct Asking {
    task: tokio::task::JoinHandle<Option<Mapping>>,
    /// How far UPnP has got.
    upnp: watch::Receiver<Upnp>,
}

impl Asking {
    /// The mapping, waiting at most `grace` more for it.
    ///
    /// PCP and NAT-PMP answer within milliseconds, but UPnP takes several
    /// requests (the search, two descriptions, the external address, the
    /// mapping). So once the gateway has answered over UPnP, the wait goes
    /// on until the attempt ends, at most [`LIMIT`] after [`start`]. A mapping
    /// that comes after the wait is dropped, which releases it.
    pub async fn wait(mut self, grace: Duration) -> Option<Mapping> {
        match tokio::time::timeout(grace, &mut self.task).await {
            Ok(got) => got.ok().flatten(),
            Err(_) if *self.upnp.borrow() == Upnp::Found => {
                let mut upnp = self.upnp;
                // Waiting is only for UPnP: once it has given up, so does
                // this, without waiting out PCP and NAT-PMP retries.
                let failed = async move {
                    if upnp.wait_for(|u| *u == Upnp::Failed).await.is_err() {
                        std::future::pending::<()>().await;
                    }
                };
                tokio::select! {
                    got = self.task => got.ok().flatten(),
                    () = failed => None,
                }
            }
            Err(_) => None,
        }
    }
}

/// How far UPnP has got, for [`Asking::wait`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Upnp {
    Searching,
    /// The gateway answered the search.
    Found,
    /// It gave no mapping.
    Failed,
}

/// PCP and NAT-PMP at `target`, and UPnP at `ssdp` when given. UPnP runs
/// alongside: whichever gives a mapping first is used. PCP and NAT-PMP
/// usually answer within milliseconds, while UPnP is still searching, and
/// then UPnP sends no SOAP action. `upnp` tells how far UPnP has got. A UPnP mapping that comes too late is dropped
/// with its task, and so released.
async fn map_with(
    target: SocketAddr,
    ssdp: Option<SocketAddr>,
    port: u16,
    upnp_state: watch::Sender<Upnp>,
    pending: Option<Pending>,
) -> Option<Mapping> {
    if port == 0 || !target.is_ipv4() {
        return None;
    }
    let pcp_socket = connected(target).await?;
    let client = pcp_socket.local_addr().ok()?.ip();
    // NAT-PMP gets its own socket: one reader per socket, so neither
    // exchange takes the other's answers.
    let pmp_socket = connected(target).await?;

    // Dropping `stop` tells the UPnP task its mapping is not wanted.
    let (stop, stopped) = oneshot::channel::<()>();
    let mut upnp = ssdp.map(|at| {
        tokio::spawn(upnp(
            at,
            SocketAddr::new(client, port),
            stopped,
            upnp_state,
            pending,
        ))
    });
    let udp = map_udp(pcp_socket, pmp_socket, client, port);
    tokio::pin!(udp);
    let result = match upnp.as_mut() {
        Some(task) => tokio::select! {
            got = &mut udp => match got {
                Udp::Mapped(m) => Some(m),
                Udp::Refused => None,
                Udp::Nothing => task.await.ok().flatten(),
            },
            got = &mut *task => match got.ok().flatten() {
                Some(m) => Some(m),
                None => match udp.await {
                    Udp::Mapped(m) => Some(m),
                    Udp::Refused | Udp::Nothing => None,
                },
            },
        },
        None => match udp.await {
            Udp::Mapped(m) => Some(m),
            Udp::Refused | Udp::Nothing => None,
        },
    };
    drop(stop);
    result
}

/// What PCP and NAT-PMP gave.
enum Udp {
    Mapped(Mapping),
    /// The router speaks PCP and said no; UPnP is not asked either.
    Refused,
    Nothing,
}

async fn map_udp(
    pcp_socket: Arc<UdpSocket>,
    pmp_socket: Arc<UdpSocket>,
    client: IpAddr,
    port: u16,
) -> Udp {
    let mut nonce = [0u8; 12];
    if getrandom::fill(&mut nonce).is_err() {
        return Udp::Nothing;
    }
    let pcp = Held::Pcp { nonce, client };
    let mut pcp_try = Box::pin({
        let socket = Arc::clone(&pcp_socket);
        async move { ask(&socket, pcp, port).await }
    });
    // Set once the NAT-PMP mapping request has gone out: from then on the
    // router may hold a NAT-PMP mapping even if PCP wins (#425).
    let pmp_asked = Arc::new(AtomicBool::new(false));
    // A router that speaks only NAT-PMP may not answer PCP at all; it is
    // asked over NAT-PMP after one unanswered PCP try, not after all
    // [`TRIES`], so its mapping still comes in time for `HINTS`.
    let pmp_try = |wait: Duration| {
        let socket = Arc::clone(&pmp_socket);
        let asked = Arc::clone(&pmp_asked);
        async move {
            tokio::time::sleep(wait).await;
            ask_pmp(&socket, port, &asked).await
        }
    };
    let mut pmp_soon = Box::pin(pmp_try(RESEND));
    let (held, socket, external, lifetime, loser) = tokio::select! {
        got = &mut pcp_try => match got {
            Got::Mapped(external, lifetime) => {
                // The NAT-PMP try lost the race. If its request is out,
                // it keeps listening for the answer, so that a second
                // mapping can be closed (#425); if not, it stops here and
                // never asks.
                let loser = pmp_asked.load(Ordering::Acquire).then(|| Loser {
                    answer: Box::pin(async move {
                        pmp_soon
                            .await
                            .and_then(|(e, l)| (l != 0 && e.port() != 0).then_some(e))
                    }),
                    release: Release::Udp {
                        socket: Arc::clone(&pmp_socket),
                        held: Held::Pmp,
                        port,
                    },
                });
                (pcp, pcp_socket, external, lifetime, loser)
            }
            Got::Refused => return Udp::Refused,
            // "Unsupported version" may come before the NAT-PMP try has
            // started; a fresh one starts now (the other is no longer
            // polled, so it reads nothing from the socket).
            Got::OtherVersion => match pmp_try(Duration::ZERO).await {
                Some((external, lifetime)) => (Held::Pmp, pmp_socket, external, lifetime, None),
                None => return Udp::Nothing,
            },
            Got::Silent => match pmp_soon.await {
                Some((external, lifetime)) => (Held::Pmp, pmp_socket, external, lifetime, None),
                None => return Udp::Nothing,
            },
        },
        got = &mut pmp_soon => match got {
            Some((external, lifetime)) => {
                // The PCP try lost; its request went out first of all.
                let loser = Loser {
                    answer: Box::pin(async move {
                        match pcp_try.await {
                            Got::Mapped(e, l) if l != 0 => Some(e),
                            _ => None,
                        }
                    }),
                    release: Release::Udp {
                        socket: Arc::clone(&pcp_socket),
                        held: pcp,
                        port,
                    },
                };
                (Held::Pmp, pmp_socket, external, lifetime, Some(loser))
            }
            None => match pcp_try.await {
                Got::Mapped(external, lifetime) => (pcp, pcp_socket, external, lifetime, None),
                Got::Refused => return Udp::Refused,
                Got::OtherVersion | Got::Silent => return Udp::Nothing,
            },
        },
    };
    let usable = lifetime != 0 && public(external.ip());
    let later = LaterSlot::default();
    if let Some(loser) = loser {
        let kept = usable.then_some(external.port());
        // Counted from now, before the task runs: the process may exit
        // as soon as the mapping goes (#461).
        let pending = Pending::new();
        let later = Arc::clone(&later);
        tokio::spawn(async move {
            let _pending = pending;
            loser.close(kept, later).await;
        });
    }
    let release = Release::Udp {
        socket: Arc::clone(&socket),
        held,
        port,
    };
    if !usable {
        // The router may have opened a port we will not use.
        release.send();
        return Udp::Nothing;
    }
    let renew = tokio::spawn(renew(socket, held, port, lifetime));
    Udp::Mapped(Mapping {
        external,
        renew,
        release: Some(release),
        pending: None,
        later,
    })
}

/// A UDP socket connected to the router, so only its datagrams arrive.
async fn connected(target: SocketAddr) -> Option<Arc<UdpSocket>> {
    let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
        .await
        .ok()?;
    socket.connect(target).await.ok()?;
    Some(Arc::new(socket))
}

/// UPnP IGD: search at the gateway's own address (unicast M-SEARCH), take
/// the answer only if the device it describes is on that address, check
/// that its external address is public, and only then map, unless `stop`
/// says the mapping is no longer wanted. The lease is [`LIFETIME`]; a
/// router that answers it offers only permanent leases (error 725) gets
/// no mapping. One that silently ignores the lease keeps the port until
/// the mapping is deleted or the router restarts.
async fn upnp(
    ssdp: SocketAddr,
    local: SocketAddr,
    stop: oneshot::Receiver<()>,
    state: watch::Sender<Upnp>,
    pending: Option<Pending>,
) -> Option<Mapping> {
    let got = upnp_with(ssdp, search_options(ssdp), local, stop, &state).await;
    match got {
        Some(mut m) => {
            m.pending = pending;
            Some(m)
        }
        None => {
            state.send_replace(Upnp::Failed);
            None
        }
    }
}

/// How the gateway at `ssdp` is searched for: unicast to it, IPv4 only,
/// within the attempt's time, and trusting only its own address for the
/// description and control URLs. Naming that address explicitly also lets
/// the tests' loopback gateway through, which `igd-next` otherwise refuses.
fn search_options(ssdp: SocketAddr) -> SearchOptions {
    let mut options = SearchOptions::default();
    options.broadcast_address = ssdp;
    // The search ends by the time `map` would give up anyway.
    options.timeout = Some(LIMIT.saturating_sub(RESEND));
    options.single_search_timeout = options.timeout;
    options.gateway_ip_version = GatewayIpVersion::V4;
    options.allowed_gateway_ips = vec![ssdp.ip()];
    options
}

async fn upnp_with(
    ssdp: SocketAddr,
    options: SearchOptions,
    local: SocketAddr,
    mut stop: oneshot::Receiver<()>,
    state: &watch::Sender<Upnp>,
) -> Option<Mapping> {
    // Every request ends with the attempt, also when nobody waits for
    // this task any more.
    let started = Instant::now();
    let deadline = started.checked_add(LIMIT)?;
    let stopped = |stop: &mut oneshot::Receiver<()>| {
        !matches!(stop.try_recv(), Err(oneshot::error::TryRecvError::Empty))
    };
    // The search only reads the gateway's description, so it starts at
    // once: UPnP takes several requests and would otherwise come too late
    // for `HINTS`.
    let gateway = tokio::select! {
        _ = &mut stop => return None,
        got = tokio::time::timeout_at(deadline, igd_next::aio::tokio::search_gateway(options)) => got.ok()?.ok()?,
    };
    // `igd-next` checks only that the description is on a trusted address
    // (the gateway's, see `search_options`) or on the address that
    // answered; a LAN host describing itself is refused here.
    if gateway.addr.ip() != ssdp.ip() || stopped(&mut stop) {
        return None;
    }
    state.send_replace(Upnp::Found);
    // The router is asked for something only if PCP and NAT-PMP have not
    // mapped a port by now; they usually answer within milliseconds.
    tokio::select! {
        _ = &mut stop => return None,
        () = tokio::time::sleep_until(started.checked_add(RESEND)?) => {}
    }
    let external_ip = tokio::time::timeout_at(deadline, gateway.get_external_ip())
        .await
        .ok()?
        .ok()?;
    if !public(external_ip) || stopped(&mut stop) {
        return None;
    }
    // Cut off at the deadline, a request the router has already carried
    // out leaves a mapping we never learn of, open until its lease ends.
    let external_port = tokio::time::timeout_at(
        deadline,
        gateway.add_any_port(PortMappingProtocol::UDP, local, LIFETIME, UPNP_DESCRIPTION),
    )
    .await
    .ok()?
    .ok()
    .filter(|p| *p != 0)?;
    let gateway = Arc::new(gateway);
    let renew = tokio::spawn(renew_upnp(
        Arc::clone(&gateway),
        external_port,
        local,
        LIFETIME,
    ));
    Some(Mapping {
        external: SocketAddr::new(external_ip, external_port),
        renew,
        release: Some(Release::Upnp {
            gateway,
            external_port,
        }),
        pending: None,
        later: LaterSlot::default(),
    })
}

/// Keep a UPnP mapping alive: asking again for the same external port
/// renews its lease. After a failed renewal, try again every
/// [`RENEW_RETRY`] until the lease would have run out.
async fn renew_upnp(gateway: Arc<Igd>, external_port: u16, local: SocketAddr, lifetime: u32) {
    let lease = Duration::from_secs(u64::from(lifetime));
    let half = Duration::from_secs(u64::from(lifetime / 2));
    let mut expiry = Instant::now().checked_add(lease);
    let mut next = Instant::now().checked_add(half);
    loop {
        let (Some(at), Some(end)) = (next, expiry) else {
            return;
        };
        tokio::time::sleep_until(at).await;
        let renewed = tokio::time::timeout(
            LIMIT,
            gateway.add_port(
                PortMappingProtocol::UDP,
                external_port,
                local,
                lifetime,
                UPNP_DESCRIPTION,
            ),
        )
        .await;
        if matches!(renewed, Ok(Ok(()))) {
            expiry = Instant::now().checked_add(lease);
            next = Instant::now().checked_add(half);
        } else if Instant::now() < end {
            next = Instant::now().checked_add(RENEW_RETRY);
        } else {
            return;
        }
    }
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
/// `asked` is set just before the mapping request goes out.
async fn ask_pmp(socket: &UdpSocket, port: u16, asked: &AtomicBool) -> Option<(SocketAddr, u32)> {
    let ip = exchange(socket, &wire::pmp_address(), wire::parse_pmp_address)
        .await?
        .ok()?;
    asked.store(true, Ordering::Release);
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
    // Android does not let apps read this file; it may pass the gateway
    // in as `Gateway::At` (not wired up yet, so Android asks no one).
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
/// byte order: the raw `u32` as it sits in memory.
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
            best = Some((metric, Ipv4Addr::from(gw.to_ne_bytes())));
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
        /// NAT-PMP that does not answer PCP at all.
        PmpOnly,
        Silent,
        Refuses,
        Private,
        /// Grants 4 seconds whatever is asked.
        Short,
        /// Speaks both, each answering a mapping request after a wait in
        /// milliseconds; NAT-PMP opens `pmp_port` (#425).
        Both {
            pcp_wait: u64,
            pmp_wait: u64,
            pmp_port: u16,
            /// PCP grants a private address (a second NAT behind it).
            pcp_private: bool,
        },
    }

    /// A router on loopback. Every request it receives is passed on, so
    /// tests can see renewals and releases.
    async fn router(kind: Router) -> (Gateway, mpsc::UnboundedReceiver<Vec<u8>>) {
        let socket = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
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
                    (Router::PmpOnly, 2) => None,
                    (Router::Pmp | Router::PmpOnly, 0) if m.len() == 2 => {
                        Some(answer::pmp_address(0, PUBLIC).to_vec())
                    }
                    (Router::Pmp | Router::PmpOnly, 0) => {
                        let internal = u16::from_be_bytes(m[4..6].try_into().unwrap());
                        let lifetime = u32::from_be_bytes(m[8..12].try_into().unwrap());
                        let opened = if lifetime == 0 { 0 } else { OPENED };
                        Some(answer::pmp_map(0, internal, opened, lifetime).to_vec())
                    }
                    (Router::Both { .. }, 0) if m.len() == 2 => {
                        Some(answer::pmp_address(0, PUBLIC).to_vec())
                    }
                    (
                        Router::Both {
                            pcp_wait,
                            pmp_wait,
                            pmp_port,
                            pcp_private,
                        },
                        version,
                    ) => {
                        let (wait, r) = if version == 2 {
                            let req: [u8; 60] = m[..60].try_into().unwrap();
                            let lifetime = lifetime_of(&m);
                            let at = if pcp_private {
                                Ipv4Addr::new(10, 0, 0, 2)
                            } else {
                                PUBLIC
                            };
                            let r = answer::pcp(&req, 0, (at.into(), OPENED), lifetime);
                            (if lifetime == 0 { 0 } else { pcp_wait }, r.to_vec())
                        } else {
                            let internal = u16::from_be_bytes(m[4..6].try_into().unwrap());
                            let lifetime = lifetime_of(&m);
                            let opened = if lifetime == 0 { 0 } else { pmp_port };
                            let r = answer::pmp_map(0, internal, opened, lifetime);
                            (if lifetime == 0 { 0 } else { pmp_wait }, r.to_vec())
                        };
                        let socket = Arc::clone(&socket);
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(wait)).await;
                            let _ = socket.send_to(&r, from).await;
                        });
                        None
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

    /// A sender nobody listens to, for calls that do not need to know
    /// whether UPnP answered.
    fn unseen() -> watch::Sender<Upnp> {
        watch::channel(Upnp::Searching).0
    }

    /// PCP and NAT-PMP only: these tests must not reach whatever listens
    /// on the machine's SSDP port.
    async fn pcp_only(gateway: Gateway, port: u16) -> Option<Mapping> {
        let target = gateway.address()?;
        tokio::time::timeout(LIMIT, map_with(target, None, port, unseen(), None))
            .await
            .ok()
            .flatten()
    }

    fn lifetime_of(m: &[u8]) -> u32 {
        let at = if m[0] == 2 { 4 } else { 8 };
        u32::from_be_bytes(m[at..][..4].try_into().unwrap())
    }

    /// A router that ignores PCP and speaks only NAT-PMP is asked over
    /// NAT-PMP after one unanswered PCP try, so its mapping still comes
    /// in time for `HINTS` (`MAPPING_GRACE` in `own_hints`).
    #[tokio::test]
    async fn a_router_silent_to_pcp_is_asked_over_nat_pmp_soon() {
        let (gw, _seen) = router(Router::PmpOnly).await;
        let started = std::time::Instant::now();
        let m = pcp_only(gw, 40_000).await.unwrap();
        assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED)));
        assert!(started.elapsed() < RESEND * 2, "{:?}", started.elapsed());
    }

    /// Mapping requests and releases the router saw, until `deadline`.
    async fn seen_until(
        seen: &mut mpsc::UnboundedReceiver<Vec<u8>>,
        deadline: Duration,
    ) -> Vec<Vec<u8>> {
        let mut all = Vec::new();
        let _ = tokio::time::timeout(deadline, async {
            while let Some(m) = seen.recv().await {
                all.push(m);
            }
        })
        .await;
        all
    }

    /// Read what the router sees into `all` until `done` holds for it, at
    /// most until `deadline`; whether it held.
    async fn seen_when(
        seen: &mut mpsc::UnboundedReceiver<Vec<u8>>,
        all: &mut Vec<Vec<u8>>,
        deadline: Duration,
        done: impl Fn(&[Vec<u8>]) -> bool,
    ) -> bool {
        tokio::time::timeout(deadline, async {
            while !done(all) {
                match seen.recv().await {
                    Some(m) => all.push(m),
                    None => return false,
                }
            }
            true
        })
        .await
        .unwrap_or(false)
    }

    /// Release requests (lifetime 0) of `version` among `all`; each
    /// release is sent twice ([`Release::send`]).
    fn releases(all: &[Vec<u8>], version: u8) -> usize {
        all.iter()
            .filter(|m| m[0] == version && m.len() > 2 && lifetime_of(m) == 0)
            .count()
    }

    /// #425: PCP answers late, after NAT-PMP was asked too, and wins; the
    /// router then also opens a NAT-PMP mapping on another port. That one
    /// is closed at once. Before, it stayed open until its lifetime ran
    /// out, though nothing used it.
    #[tokio::test]
    async fn a_nat_pmp_mapping_that_lost_to_pcp_is_closed() {
        let (gw, mut seen) = router(Router::Both {
            pcp_wait: 400,
            pmp_wait: 600,
            pmp_port: OPENED + 1,
            pcp_private: false,
        })
        .await;
        let m = pcp_only(gw, 40_000).await.unwrap();
        assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED)));
        let mut all = Vec::new();
        let closed = seen_when(&mut seen, &mut all, LIMIT * 2, |a| releases(a, 0) > 0).await;
        assert!(closed, "the NAT-PMP mapping stayed open");
        assert_eq!(releases(&all, 2), 0, "the mapping in use was closed");
        drop(m);
    }

    /// #425: the other way round, NAT-PMP wins and the PCP mapping that
    /// answers after it, on another port, is closed.
    #[tokio::test]
    async fn a_pcp_mapping_that_lost_to_nat_pmp_is_closed() {
        let (gw, mut seen) = router(Router::Both {
            pcp_wait: 500,
            pmp_wait: 0,
            pmp_port: OPENED + 1,
            pcp_private: false,
        })
        .await;
        let m = pcp_only(gw, 40_000).await.unwrap();
        assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED + 1)));
        let mut all = Vec::new();
        let closed = seen_when(&mut seen, &mut all, LIMIT * 2, |a| releases(a, 2) > 0).await;
        assert!(closed, "the PCP mapping stayed open");
        assert_eq!(releases(&all, 0), 0, "the mapping in use was closed");
        drop(m);
    }

    /// #425: a router with one table for both protocols gives the same
    /// mapping to both requests; the loser's answer then names the
    /// winner's port, and nothing is released while the mapping is used.
    #[tokio::test]
    async fn a_shared_mapping_is_not_closed_for_the_loser() {
        let (gw, mut seen) = router(Router::Both {
            pcp_wait: 400,
            pmp_wait: 600,
            pmp_port: OPENED,
            pcp_private: false,
        })
        .await;
        let m = pcp_only(gw, 40_000).await.unwrap();
        // A window, not a condition: nothing may come while it is used.
        let mut all = seen_until(&mut seen, LIMIT + Duration::from_secs(1)).await;
        assert!(
            all.iter().any(|m| m[0] == 0 && m.len() == 12),
            "NAT-PMP was not asked, so the test shows nothing"
        );
        assert_eq!(releases(&all, 0) + releases(&all, 2), 0);
        // Once it goes, both protocols release it: a router that moved the
        // mapping to NAT-PMP's name may refuse PCP's release (#425 review).
        drop(m);
        let both = seen_when(&mut seen, &mut all, LIMIT, |a| {
            releases(a, 0) > 0 && releases(a, 2) > 0
        })
        .await;
        assert!(both, "the shared mapping was not released by both");
    }

    /// #425 review: the mapping goes before the loser's answer comes, on
    /// the same port; the loser then releases it as soon as it answers.
    #[tokio::test]
    async fn a_shared_mapping_dropped_early_is_released_by_the_loser_too() {
        let (gw, mut seen) = router(Router::Both {
            pcp_wait: 400,
            // Answered within NAT-PMP's tries (750 ms), after PCP's.
            pmp_wait: 600,
            pmp_port: OPENED,
            pcp_private: false,
        })
        .await;
        drop(pcp_only(gw, 40_000).await.unwrap());
        let mut all = Vec::new();
        let both = seen_when(&mut seen, &mut all, LIMIT * 2, |a| {
            releases(a, 0) > 0 && releases(a, 2) > 0
        })
        .await;
        assert!(both, "the loser's mapping was not released");
    }

    /// #425 review: a winner that cannot be used is released, and so is
    /// the loser's mapping, even on the same port.
    #[tokio::test]
    async fn an_unusable_winner_takes_the_loser_with_it() {
        let (gw, mut seen) = router(Router::Both {
            pcp_wait: 400,
            pmp_wait: 600,
            pmp_port: OPENED,
            pcp_private: true,
        })
        .await;
        assert!(pcp_only(gw, 40_000).await.is_none());
        let mut all = Vec::new();
        let both = seen_when(&mut seen, &mut all, LIMIT * 2, |a| {
            releases(a, 0) > 0 && releases(a, 2) > 0
        })
        .await;
        assert!(both, "the loser's mapping stayed open");
    }

    /// #425 review: NAT-PMP wins and PCP, answering later, names the same
    /// port; nothing is released while the mapping is used, and both
    /// protocols release it once it goes.
    #[tokio::test]
    async fn a_shared_mapping_won_by_nat_pmp_is_not_closed_for_the_loser() {
        let (gw, mut seen) = router(Router::Both {
            pcp_wait: 500,
            pmp_wait: 0,
            pmp_port: OPENED,
            pcp_private: false,
        })
        .await;
        let m = pcp_only(gw, 40_000).await.unwrap();
        assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED)));
        // The port is the same either way; the case is NAT-PMP winning.
        assert!(
            matches!(
                m.release,
                Some(Release::Udp {
                    held: Held::Pmp,
                    ..
                })
            ),
            "PCP won, so the test shows the other case"
        );
        let mut all = seen_until(&mut seen, LIMIT + Duration::from_secs(1)).await;
        assert_eq!(releases(&all, 0) + releases(&all, 2), 0);
        drop(m);
        let both = seen_when(&mut seen, &mut all, LIMIT, |a| {
            releases(a, 0) > 0 && releases(a, 2) > 0
        })
        .await;
        assert!(both, "the shared mapping was not released by both");
    }

    /// #425 review: the loser never answers within its tries; the winner
    /// is kept and released alone, and the loser releases nothing.
    #[tokio::test]
    async fn a_loser_that_never_answers_releases_nothing() {
        let (gw, mut seen) = router(Router::Both {
            // Far past LIMIT: PCP's answers never come in time.
            pcp_wait: 60_000,
            pmp_wait: 0,
            pmp_port: OPENED + 1,
            pcp_private: false,
        })
        .await;
        let m = pcp_only(gw, 40_000).await.unwrap();
        assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED + 1)));
        let mut all = seen_until(&mut seen, LIMIT + Duration::from_secs(1)).await;
        assert_eq!(releases(&all, 0) + releases(&all, 2), 0);
        drop(m);
        let closed = seen_when(&mut seen, &mut all, LIMIT, |a| releases(a, 0) > 0).await;
        assert!(closed, "the mapping in use was not released");
        all.extend(seen_until(&mut seen, Duration::from_millis(300)).await);
        assert_eq!(
            releases(&all, 2),
            0,
            "a PCP mapping that never came was released"
        );
    }

    /// #425: when PCP answers before NAT-PMP is asked for a mapping, the
    /// NAT-PMP try stops and never asks; nothing is released.
    #[tokio::test]
    async fn a_fast_pcp_answer_leaves_nat_pmp_unasked() {
        let (gw, mut seen) = router(Router::Both {
            pcp_wait: 0,
            pmp_wait: 0,
            pmp_port: OPENED + 1,
            pcp_private: false,
        })
        .await;
        let m = pcp_only(gw, 40_000).await.unwrap();
        assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED)));
        let all = seen_until(&mut seen, LIMIT + Duration::from_secs(1)).await;
        assert!(all.iter().all(|m| m[0] == 2), "NAT-PMP was asked");
        drop(m);
    }

    #[tokio::test]
    async fn pcp_mapping_is_announced_and_released() {
        let (gw, mut seen) = router(Router::Pcp).await;
        let m = pcp_only(gw.clone(), 40_000).await.unwrap();
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
        let m = pcp_only(gw.clone(), 40_000).await.unwrap();
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
        let m = pcp_only(gw.clone(), 40_000).await.unwrap();
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
            assert!(pcp_only(gw.clone(), 40_000).await.is_none());
            assert!(started.elapsed() <= LIMIT + Duration::from_millis(300));
        }
    }

    /// A port opened on an address we do not use is closed again.
    #[tokio::test]
    async fn an_unusable_mapping_is_released() {
        let (gw, mut seen) = router(Router::Private).await;
        assert!(pcp_only(gw, 40_000).await.is_none());
        let first = seen.recv().await.unwrap();
        let release = seen.recv().await.unwrap();
        assert_eq!(lifetime_of(&first), LIFETIME);
        assert_eq!(lifetime_of(&release), 0);
    }

    #[tokio::test]
    async fn a_short_grant_is_renewed_in_time() {
        let (gw, mut seen) = router(Router::Short).await;
        let m = pcp_only(gw, 40_000).await.unwrap();
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
            let m = pcp_only(Gateway::At(at), 40_000).await.unwrap();
            drop(m);
        });
        drop(rt);
        let _request = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let release = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(lifetime_of(&release), 0);
    }

    /// #461: PCP wins after NAT-PMP was asked, and the process exits right
    /// after the mapping goes, as the CLI does: [`released`], then the
    /// runtime shuts down. The loser's task must still close the NAT-PMP
    /// mapping on the other port; before, `released` did not wait for it
    /// and the mapping stayed open for its lifetime.
    #[test]
    fn released_waits_for_the_losers_release() {
        let (tx, rx) = std::sync::mpsc::channel();
        let gw = Arc::new(std::net::UdpSocket::bind("127.0.0.1:0").unwrap());
        let at = gw.local_addr().unwrap();
        std::thread::spawn(move || {
            let mut buf = [0u8; 1500];
            while let Ok((n, from)) = gw.recv_from(&mut buf) {
                let m = buf[..n].to_vec();
                let _ = tx.send(m.clone());
                // PCP answers after NAT-PMP was asked, NAT-PMP's mapping
                // (on another port) after that.
                let (wait, r) = match (m[0], m.len()) {
                    (0, 2) => (0, answer::pmp_address(0, PUBLIC).to_vec()),
                    (0, 12) if lifetime_of(&m) != 0 => {
                        let internal = u16::from_be_bytes(m[4..6].try_into().unwrap());
                        let r = answer::pmp_map(0, internal, OPENED + 1, LIFETIME);
                        (600, r.to_vec())
                    }
                    (2, 60) if lifetime_of(&m) != 0 => {
                        let req: [u8; 60] = m[..].try_into().unwrap();
                        let r = answer::pcp(&req, 0, (PUBLIC.into(), OPENED), LIFETIME);
                        (400, r.to_vec())
                    }
                    _ => continue,
                };
                let gw = Arc::clone(&gw);
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(wait));
                    let _ = gw.send_to(&r, from);
                });
            }
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let m = pcp_only(Gateway::At(at), 40_000).await.unwrap();
            assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED)));
            drop(m);
            released().await;
        });
        rt.shutdown_background();
        let mut all = Vec::new();
        while let Ok(m) = rx.recv_timeout(Duration::from_millis(500)) {
            all.push(m);
        }
        assert!(
            all.iter()
                .any(|m| m[0] == 0 && m.len() == 12 && lifetime_of(m) != 0),
            "NAT-PMP was not asked, so the test shows nothing"
        );
        assert!(releases(&all, 0) > 0, "the NAT-PMP mapping stayed open");
    }

    #[tokio::test]
    async fn no_gateway_asks_no_one() {
        assert!(pcp_only(Gateway::Off, 40_000).await.is_none());
        // Nothing listens there: the ICMP error ends the attempt early.
        let closed = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let at = closed.local_addr().unwrap();
        drop(closed);
        let started = std::time::Instant::now();
        assert!(pcp_only(Gateway::At(at), 40_000).await.is_none());
        assert!(started.elapsed() < LIMIT);
        let v6 = Gateway::At("[::1]:5351".parse().unwrap());
        assert!(pcp_only(v6, 40_000).await.is_none());
        assert!(
            pcp_only(Gateway::At("127.0.0.1:5351".parse().unwrap()), 0)
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

    /// What the fake IGD does with `AddPortMapping`.
    #[derive(Clone, Copy)]
    enum Igd {
        Maps,
        /// Maps, but takes 60 ms over every HTTP answer, as a slow router
        /// does.
        Slow,
        /// UPnP error 725: only permanent leases.
        PermanentOnly,
        /// Its external address is behind another NAT.
        Private,
    }

    /// A UPnP gateway on loopback: an SSDP responder whose LOCATION points
    /// to its own small HTTP server. Every SOAP action it receives is passed
    /// on with its body.
    async fn igd(kind: Igd) -> (SocketAddr, mpsc::UnboundedReceiver<(String, String)>) {
        igd_at("127.0.0.1", kind).await
    }

    async fn igd_at(
        ip: &str,
        kind: Igd,
    ) -> (SocketAddr, mpsc::UnboundedReceiver<(String, String)>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let ssdp = UdpSocket::bind((ip, 0)).await.unwrap();
        let http = tokio::net::TcpListener::bind((ip, 0)).await.unwrap();
        let at = ssdp.local_addr().unwrap();
        let location = format!("http://{}/rootDesc.xml", http.local_addr().unwrap());
        tokio::spawn(async move {
            let mut buf = [0u8; 1500];
            loop {
                let (n, from) = match ssdp.recv_from(&mut buf).await {
                    Ok(got) => got,
                    Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => continue,
                    Err(_) => break,
                };
                if buf[..n].starts_with(b"M-SEARCH") {
                    let reply = format!(
                        "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=120\r\n\
                         ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
                         USN: uuid:test::urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
                         LOCATION: {location}\r\n\r\n"
                    );
                    let _ = ssdp.send_to(reply.as_bytes(), from).await;
                }
            }
        });
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Ok((mut c, _)) = http.accept().await {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut req = Vec::new();
                    let mut buf = [0u8; 4096];
                    let (head, body) = loop {
                        let n = c.read(&mut buf).await.unwrap_or(0);
                        if n == 0 {
                            return;
                        }
                        req.extend_from_slice(&buf[..n]);
                        let text = String::from_utf8_lossy(&req).to_string();
                        if let Some((head, rest)) = text.split_once("\r\n\r\n") {
                            let length = head
                                .lines()
                                .find_map(|l| {
                                    let (k, v) = l.split_once(':')?;
                                    k.eq_ignore_ascii_case("content-length")
                                        .then(|| v.trim().parse::<usize>().ok())?
                                })
                                .unwrap_or(0);
                            if rest.len() >= length {
                                break (head.to_string(), rest.to_string());
                            }
                        }
                    };
                    let action = head
                        .lines()
                        .find_map(|l| {
                            let (k, v) = l.split_once(':')?;
                            k.eq_ignore_ascii_case("soapaction").then(|| {
                                v.trim()
                                    .trim_matches('"')
                                    .rsplit('#')
                                    .next()
                                    .unwrap()
                                    .to_string()
                            })
                        })
                        .unwrap_or_default();
                    if matches!(kind, Igd::Slow) {
                        tokio::time::sleep(Duration::from_millis(60)).await;
                    }
                    let (status, xml) = respond(kind, &head, &action);
                    if !action.is_empty() {
                        let _ = tx.send((action, body));
                    }
                    let reply = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: text/xml\r\nContent-Length: {}\r\n\
                         Connection: close\r\n\r\n{xml}",
                        xml.len()
                    );
                    let _ = c.write_all(reply.as_bytes()).await;
                    let _ = c.shutdown().await;
                });
            }
        });
        (at, rx)
    }

    const WANIP: &str = "urn:schemas-upnp-org:service:WANIPConnection:1";

    fn respond(kind: Igd, head: &str, action: &str) -> (&'static str, String) {
        let envelope = |inner: String| {
            format!(
                "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
                 s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body>{inner}</s:Body></s:Envelope>"
            )
        };
        let ok = |name: &str, args: &str| {
            envelope(format!(
                "<u:{name}Response xmlns:u=\"{WANIP}\">{args}</u:{name}Response>"
            ))
        };
        if head.starts_with("GET /rootDesc.xml") {
            return (
                "200 OK",
                format!(
                    "<?xml version=\"1.0\"?><root xmlns=\"urn:schemas-upnp-org:device-1-0\"><device>\
                 <deviceType>urn:schemas-upnp-org:device:InternetGatewayDevice:1</deviceType>\
                 <deviceList><device><deviceType>urn:schemas-upnp-org:device:WANDevice:1</deviceType>\
                 <deviceList><device><deviceType>urn:schemas-upnp-org:device:WANConnectionDevice:1</deviceType>\
                 <serviceList><service><serviceType>{WANIP}</serviceType><serviceId>urn:upnp-org:serviceId:WANIPConn1</serviceId>\
                 <SCPDURL>/scpd.xml</SCPDURL><controlURL>/ctl</controlURL><eventSubURL>/evt</eventSubURL></service></serviceList>\
                 </device></deviceList></device></deviceList></device></root>"
                ),
            );
        }
        if head.starts_with("GET /scpd.xml") {
            let action = |name: &str, args: &[&str]| {
                let args: String = args
                    .iter()
                    .map(|a| {
                        format!("<argument><name>{a}</name><direction>in</direction></argument>")
                    })
                    .collect();
                format!("<action><name>{name}</name><argumentList>{args}</argumentList></action>")
            };
            return (
                "200 OK",
                format!(
                    "<?xml version=\"1.0\"?><scpd xmlns=\"urn:schemas-upnp-org:service-1-0\"><actionList>{}{}{}</actionList></scpd>",
                    action("GetExternalIPAddress", &[]),
                    action(
                        "AddPortMapping",
                        &[
                            "NewRemoteHost",
                            "NewExternalPort",
                            "NewProtocol",
                            "NewInternalPort",
                            "NewInternalClient",
                            "NewEnabled",
                            "NewPortMappingDescription",
                            "NewLeaseDuration"
                        ]
                    ),
                    action(
                        "DeletePortMapping",
                        &["NewRemoteHost", "NewExternalPort", "NewProtocol"]
                    ),
                ),
            );
        }
        match (action, kind) {
            ("GetExternalIPAddress", Igd::Private) => ("200 OK", ok(action, "<NewExternalIPAddress>192.168.100.2</NewExternalIPAddress>")),
            ("GetExternalIPAddress", _) => ("200 OK", ok(action, &format!("<NewExternalIPAddress>{PUBLIC}</NewExternalIPAddress>"))),
            ("AddPortMapping", Igd::PermanentOnly) => ("500 Internal Server Error", envelope(
                "<s:Fault><faultcode>s:Client</faultcode><faultstring>UPnPError</faultstring><detail>\
                 <UPnPError xmlns=\"urn:schemas-upnp-org:control-1-0\"><errorCode>725</errorCode>\
                 <errorDescription>OnlyPermanentLeasesSupported</errorDescription></UPnPError></detail></s:Fault>".to_string(),
            )),
            ("AddPortMapping" | "DeletePortMapping", _) => ("200 OK", ok(action, "")),
            _ => ("404 Not Found", String::new()),
        }
    }

    /// The value of one SOAP argument.
    fn arg(body: &str, name: &str) -> Option<String> {
        let (_, after) = body.split_once(&format!("<{name}>"))?;
        let (value, _) = after.split_once(&format!("</{name}>"))?;
        Some(value.to_string())
    }

    /// PCP and NAT-PMP get no answer; the UPnP gateway maps the port for
    /// [`LIFETIME`] and the mapping is deleted when dropped.
    #[tokio::test]
    async fn upnp_maps_when_pcp_and_nat_pmp_are_silent() {
        let (Gateway::At(silent), _) = router(Router::Silent).await else {
            unreachable!()
        };
        let (ssdp, mut seen) = igd(Igd::Maps).await;
        let started = std::time::Instant::now();
        let m = tokio::time::timeout(LIMIT, map_with(silent, Some(ssdp), 40_000, unseen(), None))
            .await
            .unwrap()
            .unwrap();
        assert!(started.elapsed() < LIMIT);
        assert_eq!(m.external().ip(), IpAddr::V4(PUBLIC));
        let (action, _) = seen.recv().await.unwrap();
        assert_eq!(action, "GetExternalIPAddress");
        let (action, body) = seen.recv().await.unwrap();
        assert_eq!(action, "AddPortMapping");
        assert_eq!(arg(&body, "NewProtocol").as_deref(), Some("UDP"));
        assert_eq!(arg(&body, "NewInternalPort").as_deref(), Some("40000"));
        assert_eq!(arg(&body, "NewLeaseDuration"), Some(LIFETIME.to_string()));
        assert_eq!(
            arg(&body, "NewInternalClient").as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(
            arg(&body, "NewExternalPort"),
            Some(m.external().port().to_string())
        );
        let port = m.external().port().to_string();
        drop(m);
        let (action, body) = seen.recv().await.unwrap();
        assert_eq!(action, "DeletePortMapping");
        assert_eq!(arg(&body, "NewExternalPort"), Some(port));
    }

    /// A process that exits right after dropping a UPnP mapping takes its
    /// runtime, and the spawned delete, with it: at home the port stayed
    /// open until its lease ran out (#406). `released` waits for the
    /// delete, so the CLI can call it before it shuts the runtime down.
    #[tokio::test]
    async fn released_waits_for_the_upnp_delete() {
        let (Gateway::At(silent), _) = router(Router::Silent).await else {
            unreachable!()
        };
        let (ssdp, mut seen) = igd(Igd::Maps).await;
        let m = tokio::time::timeout(LIMIT, map_with(silent, Some(ssdp), 40_000, unseen(), None))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(seen.recv().await.unwrap().0, "GetExternalIPAddress");
        assert_eq!(seen.recv().await.unwrap().0, "AddPortMapping");
        drop(m);
        released().await;
        assert_eq!(seen.try_recv().unwrap().0, "DeletePortMapping");
    }

    /// An attempt nobody waits for any more still counts: a process that
    /// gave up on the mapping and exits at once must not cut off the
    /// router mid-mapping, which would leave the port open until its lease
    /// ends. `released` waits for the mapping and its delete.
    #[tokio::test]
    async fn released_waits_for_an_attempt_nobody_waits_for() {
        let (Gateway::At(silent), _) = router(Router::Silent).await else {
            unreachable!()
        };
        let (ssdp, mut seen) = igd(Igd::Slow).await;
        let got = start_at(silent, Some(ssdp), 40_000)
            .wait(Duration::ZERO)
            .await;
        assert!(got.is_none());
        released().await;
        let mut actions = Vec::new();
        while let Ok((action, _)) = seen.try_recv() {
            actions.push(action);
        }
        assert_eq!(
            actions,
            [
                "GetExternalIPAddress",
                "AddPortMapping",
                "DeletePortMapping"
            ]
        );
    }

    /// A UPnP gateway takes several requests, so its mapping comes well
    /// after PCP's would. Once it has answered, `wait` waits for it rather
    /// than giving up when the grace ends. At home a mapping made a few
    /// milliseconds too late was released at once, and the router showed
    /// no port at all (#406).
    #[tokio::test]
    async fn a_slow_upnp_gateway_is_waited_for_once_it_answered() {
        let (Gateway::At(silent), _) = router(Router::Silent).await else {
            unreachable!()
        };
        let (ssdp, mut seen) = igd(Igd::Slow).await;
        let started = std::time::Instant::now();
        let m = start_at(silent, Some(ssdp), 40_000)
            .wait(Duration::from_millis(300))
            .await
            // `None` here: the mapping was not waited for.
            .unwrap();
        assert!(started.elapsed() < LIMIT, "{:?}", started.elapsed());
        assert_eq!(m.external().ip(), IpAddr::V4(PUBLIC));
        assert_eq!(seen.recv().await.unwrap().0, "GetExternalIPAddress");
        assert_eq!(seen.recv().await.unwrap().0, "AddPortMapping");
        drop(m);
    }

    /// A gateway that answers the search but gives no mapping, here
    /// because it sits behind another NAT, ends the wait as soon as UPnP
    /// gives up, not after the PCP and NAT-PMP retries.
    #[tokio::test]
    async fn a_upnp_gateway_that_gives_nothing_ends_the_wait() {
        let (Gateway::At(silent), _) = router(Router::Silent).await else {
            unreachable!()
        };
        let (ssdp, _seen) = igd(Igd::Private).await;
        let started = std::time::Instant::now();
        let got = start_at(silent, Some(ssdp), 40_000)
            .wait(Duration::from_millis(100))
            .await;
        assert!(got.is_none());
        let took = started.elapsed();
        // UPnP gives up right after 250 ms; the retries would run to ~1 s.
        assert!(took < Duration::from_millis(600), "{took:?}");
    }

    /// A gateway that answers nothing costs only the grace.
    #[tokio::test]
    async fn a_silent_gateway_costs_only_the_grace() {
        let (Gateway::At(silent), _) = router(Router::Silent).await else {
            unreachable!()
        };
        // Nothing listens on this SSDP port.
        let ssdp = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let at = ssdp.local_addr().unwrap();
        drop(ssdp);
        let started = std::time::Instant::now();
        let got = start_at(silent, Some(at), 40_000)
            .wait(Duration::from_millis(300))
            .await;
        assert!(got.is_none());
        let took = started.elapsed();
        assert!(took < Duration::from_millis(600), "{took:?}");
    }

    /// When PCP answers, UPnP maps nothing.
    #[tokio::test]
    async fn upnp_is_not_used_when_pcp_answers() {
        let (Gateway::At(pcp), _) = router(Router::Pcp).await else {
            unreachable!()
        };
        let (ssdp, mut seen) = igd(Igd::Maps).await;
        let m = map_with(pcp, Some(ssdp), 40_000, unseen(), None)
            .await
            .unwrap();
        assert_eq!(m.external(), SocketAddr::from((PUBLIC, OPENED)));
        tokio::time::sleep(RESEND * 4).await;
        assert!(seen.try_recv().is_err(), "UPnP was asked");
        drop(m);
    }

    /// A router that only grants permanent leases gets no mapping, and we
    /// never fall back to one.
    #[tokio::test]
    async fn upnp_permanent_leases_are_refused() {
        let (Gateway::At(silent), _) = router(Router::Silent).await else {
            unreachable!()
        };
        let (ssdp, mut seen) = igd(Igd::PermanentOnly).await;
        // As in `map`: what is not there within `LIMIT` is not there.
        let got =
            tokio::time::timeout(LIMIT, map_with(silent, Some(ssdp), 40_000, unseen(), None)).await;
        assert!(got.ok().flatten().is_none());
        let mut asked = false;
        while let Ok((action, body)) = seen.try_recv() {
            if action == "AddPortMapping" {
                asked = true;
                assert_eq!(arg(&body, "NewLeaseDuration"), Some(LIFETIME.to_string()));
            }
        }
        // UPnP did run: the router was asked and said no.
        assert!(asked);
    }

    /// A gateway behind another NAT is not asked to map at all.
    #[tokio::test]
    async fn upnp_behind_another_nat_maps_nothing() {
        let (Gateway::At(silent), _) = router(Router::Silent).await else {
            unreachable!()
        };
        let (ssdp, mut seen) = igd(Igd::Private).await;
        // As in `map`: what is not there within `LIMIT` is not there.
        let got =
            tokio::time::timeout(LIMIT, map_with(silent, Some(ssdp), 40_000, unseen(), None)).await;
        assert!(got.ok().flatten().is_none());
        let mut actions = Vec::new();
        while let Ok((action, _)) = seen.try_recv() {
            actions.push(action);
        }
        // UPnP did run: it learnt the address, then asked for nothing.
        assert_eq!(actions, ["GetExternalIPAddress"]);
    }

    /// A LAN host that answers the search and describes itself (its own
    /// address in LOCATION) is not used, even when it is a working IGD.
    /// `igd-next` would accept it as soon as its address is one it trusts;
    /// the test trusts it, as `igd-next` trusts any non-loopback host that
    /// describes itself, so only our own check stands in the way. Linux
    /// only: 127.0.0.2 is not on loopback elsewhere.
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn upnp_ignores_a_device_that_is_not_the_gateway() {
        // The IGD is at 127.0.0.2; the gateway we ask is at 127.0.0.1.
        let (elsewhere, mut seen) = igd_at("127.0.0.2", Igd::Maps).await;
        let forwarder = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let ssdp = forwarder.local_addr().unwrap();
        tokio::spawn(async move {
            let mut buf = [0u8; 1500];
            while let Ok((n, from)) = forwarder.recv_from(&mut buf).await {
                // Pass the search on so the other host answers the client.
                let relay = UdpSocket::bind("127.0.0.2:0").await.unwrap();
                relay.send_to(&buf[..n], elsewhere).await.unwrap();
                let mut answer = [0u8; 1500];
                if let Ok((m, _)) = relay.recv_from(&mut answer).await {
                    let _ = relay.send_to(&answer[..m], from).await;
                }
            }
        });
        let mut options = search_options(ssdp);
        options.allowed_gateway_ips.push(elsewhere.ip());
        let (_stop, stopped) = oneshot::channel();
        let local = SocketAddr::from(([127, 0, 0, 1], 40_000));
        let got =
            tokio::time::timeout(LIMIT, upnp_with(ssdp, options, local, stopped, &unseen())).await;
        assert!(got.unwrap().is_none());
        // The description was read, but nothing was asked of that device.
        assert!(seen.try_recv().is_err());
    }

    /// The lease is renewed at half its length with the same external port.
    #[tokio::test]
    async fn upnp_lease_is_renewed() {
        let (Gateway::At(silent), _) = router(Router::Silent).await else {
            unreachable!()
        };
        let (ssdp, mut seen) = igd(Igd::Maps).await;
        let m = tokio::time::timeout(LIMIT, map_with(silent, Some(ssdp), 40_000, unseen(), None))
            .await
            .unwrap()
            .unwrap();
        while seen.recv().await.unwrap().0 != "AddPortMapping" {}
        // The mapping renews only after a minute; a 2-second lease shows the
        // same loop in real time (a paused clock would also run out the
        // timeouts around the real HTTP requests).
        let gateway = igd_next::aio::tokio::search_gateway(search_options(ssdp))
            .await
            .unwrap();
        let local = SocketAddr::from(([127, 0, 0, 1], 40_000));
        let renew = tokio::spawn(renew_upnp(Arc::new(gateway), 41_000, local, 2));
        let (action, body) = tokio::time::timeout(Duration::from_secs(5), seen.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(action, "AddPortMapping");
        assert_eq!(arg(&body, "NewExternalPort").as_deref(), Some("41000"));
        assert_eq!(arg(&body, "NewLeaseDuration").as_deref(), Some("2"));
        renew.abort();
        drop(m);
    }
}
