// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! TotalDrop rendezvous server (spec section 5).
//!
//! Helps two devices that share a transfer code meet in a mailbox and pass
//! each other opaque messages. It never sees keys, file names or content:
//! after the two CPace messages everything it carries is encrypted end to end.
//!
//! Rules enforced here:
//! - a nameplate accepts exactly two claims; a third is refused, which is
//!   what limits an attacker to one guess per code;
//! - after a `scary` close by one of two sides that exchanged messages, the
//!   nameplate is retired for 24 hours;
//! - every request is checked against the mailbox a connection joined, so a
//!   connection whose nameplate expired cannot touch the number's next owner;
//! - unclaimed nameplates expire after 15 minutes, and every mailbox is
//!   deleted when both sides have left, or after one hour at the latest;
//! - each client (an IPv4 address or an IPv6 /64) may open 60 connections
//!   at once and one more every second; past that it is told
//!   `too many requests`. The counters are in memory only, under a keyed
//!   hash of the client, and are forgotten once the client is back at a
//!   full burst (threat model T9);
//! - nothing is logged about clients: no addresses, no nameplates, no sizes.
//!
//! The rate limit sees the address of the TCP peer. Behind a TLS
//! terminator or proxy on another address, every client is that address.
//!
//! Serve it behind a TLS 1.3 terminator on port 443 (spec section 5).

use std::collections::{HashMap, VecDeque};
use std::hash::{BuildHasher, RandomState};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Semaphore, mpsc};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use totaldrop_proto::wire::{ClientMessage, MAX_MAILBOX_BODY, Mood, ServerMessage};

/// Limits and timings. The defaults are the protocol's; tests shorten them.
#[derive(Debug, Clone)]
pub struct Config {
    /// How long an allocated nameplate waits for its second claim.
    pub unclaimed_ttl: Duration,
    /// Hard cap on the life of any mailbox.
    pub mailbox_ttl: Duration,
    /// How long a nameplate stays retired after a `scary` close.
    pub retire_for: Duration,
    /// How long an idle connection is kept.
    pub idle_timeout: Duration,
    /// How long a connection may wait before it allocates or claims. Pings
    /// do not extend it.
    pub claim_timeout: Duration,
    /// Most simultaneous connections.
    pub max_connections: usize,
    /// Most simultaneous nameplates.
    pub max_nameplates: usize,
    /// Most messages one side may post to a mailbox.
    pub max_messages_per_side: usize,
    /// Scary closes within [`Config::scary_window`] that count as an attack wave.
    pub scary_threshold: usize,
    /// Window for counting scary closes.
    pub scary_window: Duration,
    /// Delay added to every allocation during an attack wave.
    pub wave_delay: Duration,
    /// How often expired mailboxes are swept.
    pub sweep_interval: Duration,
    /// Connections one client may open at once.
    pub rate_burst: u32,
    /// After a burst, one more connection is allowed every `rate_interval`.
    pub rate_interval: Duration,
    /// Most clients counted at once. A client beyond this is let through
    /// uncounted: the global limits still hold, and refusing everyone new
    /// would let one attacker with many addresses shut the service.
    pub max_clients: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            unclaimed_ttl: Duration::from_secs(15 * 60),
            mailbox_ttl: Duration::from_secs(60 * 60),
            retire_for: Duration::from_secs(24 * 60 * 60),
            idle_timeout: Duration::from_secs(10 * 60),
            claim_timeout: Duration::from_secs(10),
            max_connections: 10_000,
            max_nameplates: 100_000,
            max_messages_per_side: 32,
            scary_threshold: 100,
            scary_window: Duration::from_secs(10 * 60),
            wave_delay: Duration::from_secs(2),
            sweep_interval: Duration::from_secs(5),
            rate_burst: 60,
            rate_interval: Duration::from_secs(1),
            max_clients: 100_000,
        }
    }
}

type ConnId = u64;
type Outbox = mpsc::Sender<ServerMessage>;

/// Most messages buffered for a side that has not arrived yet.
const MAX_PENDING_MESSAGES: usize = 8;
/// Most bytes buffered for a side that has not arrived yet. A real exchange
/// buffers at most one CPace message, one certificate and one set of hints.
const MAX_PENDING_BYTES: usize = 32 * 1024;
/// How long a freed nameplate rests before it is handed out again, so a
/// late receiver with an old code does not walk into a stranger's mailbox.
const COOLDOWN: Duration = Duration::from_secs(10 * 60);
/// How long the WebSocket handshake and each write may take.
const IO_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a refused connection is kept open for the client to close it.
/// The client sends its first message right after the handshake; closing
/// the socket with it unread resets the connection, and Windows then drops
/// the refusal before the client reads it (#442).
const REFUSAL_LINGER: Duration = Duration::from_secs(2);
/// What a client over its rate is told.
pub const TOO_MANY: &str = "too many requests";

/// Whether a claim carries the nameplate's allocation token, compared in
/// constant time, as the Cloudflare Worker does (`sameBytes`; #428).
fn token_matches(given: Option<[u8; 16]>, ours: &[u8; 16]) -> bool {
    use subtle::ConstantTimeEq as _;
    given.is_some_and(|t| bool::from(t.ct_eq(ours)))
}

/// The part of an address that names one client: an IPv4 address (also
/// when it arrives as an IPv4-mapped IPv6 address) or the first 64 bits of
/// an IPv6 address, the block one subscriber usually gets.
fn client_of(ip: IpAddr) -> [u16; 4] {
    match ip {
        IpAddr::V4(v4) => client_v4(v4.octets()),
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => client_v4(v4.octets()),
            None => {
                let [a, b, c, d, ..] = v6.segments();
                [a, b, c, d]
            }
        },
    }
}

/// An IPv4 client, kept apart from every IPv6 /64 by a marker no global
/// IPv6 prefix starts with.
fn client_v4([a, b, c, d]: [u8; 4]) -> [u16; 4] {
    [
        0,
        0xffff,
        u16::from_be_bytes([a, b]),
        u16::from_be_bytes([c, d]),
    ]
}

/// Per-client counters (threat model T9), the same rule as the Worker's
/// (`server/rendezvous-worker/src/limit.ts`): the generic cell rate
/// algorithm, one time per client at which its allowance is next fully
/// spent. Clients are kept under a hash keyed at random when the server
/// starts, never under their address, and only in memory.
#[derive(Debug, Default)]
struct Limiter {
    due: HashMap<u64, Instant>,
    key: RandomState,
}

impl Limiter {
    /// Whether `ip` may open one more connection now. An allowed
    /// connection is counted; a refused one is not.
    fn admit(&mut self, ip: IpAddr, now: Instant, cfg: &Config) -> bool {
        let id = self.key.hash_one(client_of(ip));
        let known = self.due.get(&id).copied();
        if known.is_none() && self.due.len() >= cfg.max_clients {
            return true;
        }
        let start = known.map_or(now, |due| due.max(now));
        let Some(next) = start.checked_add(cfg.rate_interval) else {
            return false;
        };
        let window = cfg.rate_interval.saturating_mul(cfg.rate_burst);
        if next.saturating_duration_since(now) > window {
            return false;
        }
        self.due.insert(id, next);
        true
    }

    /// Forget clients that are back at a full burst.
    fn sweep(&mut self, now: Instant) {
        self.due.retain(|_, due| *due > now);
    }
}

/// Which endpoint a connection opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    /// `/v0/allocate`: one `allocate`, one reply.
    Allocate,
    /// `/v0/mailbox/<nameplate>`.
    Mailbox(u32),
}

/// What a connection holds: a nameplate number and the mailbox it pointed to
/// when this connection got it. A number can be reused after expiry; the
/// mailbox ID cannot, so every request is checked against both.
type Held = Option<(u32, [u8; 16])>;

#[derive(Debug)]
struct Side {
    conn: ConnId,
    outbox: Outbox,
    posted: usize,
    closed: bool,
}

#[derive(Debug)]
struct Nameplate {
    created: Instant,
    /// Secret given to the allocator; the first claim must present it.
    token: [u8; 16],
    mailbox: [u8; 16],
    sides: Vec<Side>,
    /// Messages posted before the other side arrived, by poster.
    pending: Vec<(ConnId, String, Vec<u8>)>,
    pending_bytes: usize,
    /// Set by the first counted scary close: the other side's close counts
    /// for nothing more.
    scared: bool,
}

#[derive(Debug, Default)]
struct State {
    nameplates: HashMap<u32, Nameplate>,
    retired: HashMap<u32, Instant>,
    cooling: HashMap<u32, Instant>,
    scary: VecDeque<Instant>,
    next_conn: ConnId,
    limiter: Limiter,
}

impl State {
    fn is_retired(&self, n: u32, now: Instant, cfg: &Config) -> bool {
        self.retired
            .get(&n)
            .is_some_and(|t| now.saturating_duration_since(*t) < cfg.retire_for)
    }

    fn is_cooling(&self, n: u32, now: Instant) -> bool {
        self.cooling
            .get(&n)
            .is_some_and(|t| now.saturating_duration_since(*t) < COOLDOWN)
    }

    fn free_nameplate(&self, now: Instant, cfg: &Config) -> Option<u32> {
        let range = cfg.max_nameplates.saturating_mul(4);
        let usable = |n: &u32| !self.nameplates.contains_key(n) && !self.is_retired(*n, now, cfg);
        (1u32..=u32::MAX)
            .take(range)
            .find(|n| usable(n) && !self.is_cooling(*n, now))
            .or_else(|| (1u32..=u32::MAX).take(range).find(usable))
    }

    fn remove(&mut self, n: u32, now: Instant) {
        if self.nameplates.remove(&n).is_some() {
            self.cooling.insert(n, now);
        }
    }

    fn attack_wave(&mut self, now: Instant, cfg: &Config) -> bool {
        while self
            .scary
            .front()
            .is_some_and(|t| now.saturating_duration_since(*t) > cfg.scary_window)
        {
            self.scary.pop_front();
        }
        self.scary.len() >= cfg.scary_threshold
    }

    /// Remove expired nameplates, retirements and cooldowns. Returns the
    /// outboxes of every client that still held an expired nameplate, so
    /// they can be told and disconnected.
    fn sweep(&mut self, now: Instant, cfg: &Config) -> Vec<Outbox> {
        let mut told = Vec::new();
        let mut expired_numbers = Vec::new();
        for (n, np) in &self.nameplates {
            let age = now.saturating_duration_since(np.created);
            if (np.sides.len() < 2 && age > cfg.unclaimed_ttl) || age > cfg.mailbox_ttl {
                expired_numbers.push(*n);
                told.extend(
                    np.sides
                        .iter()
                        .filter(|s| !s.closed)
                        .map(|s| s.outbox.clone()),
                );
            }
        }
        for n in expired_numbers {
            self.remove(n, now);
        }
        self.retired
            .retain(|_, t| now.saturating_duration_since(*t) < cfg.retire_for);
        self.cooling
            .retain(|_, t| now.saturating_duration_since(*t) < COOLDOWN);
        self.limiter.sweep(now);
        told
    }

    /// The nameplate `held` points to, if it is still the same mailbox.
    fn current(&mut self, held: Held) -> Option<(u32, &mut Nameplate)> {
        let (n, id) = held?;
        self.nameplates
            .get_mut(&n)
            .filter(|np| np.mailbox == id)
            .map(|np| (n, np))
    }
}

/// A running rendezvous server.
#[derive(Debug, Clone)]
pub struct Server {
    state: Arc<Mutex<State>>,
    config: Arc<Config>,
    slots: Arc<Semaphore>,
}

impl Server {
    /// A server with the given configuration.
    pub fn new(config: Config) -> Self {
        let slots = Arc::new(Semaphore::new(config.max_connections));
        Self {
            state: Arc::new(Mutex::new(State::default())),
            config: Arc::new(config),
            slots,
        }
    }

    /// Accept connections on `listener` until the task is dropped.
    pub async fn serve(self, listener: TcpListener) {
        let sweeper = self.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(sweeper.config.sweep_interval);
            loop {
                tick.tick().await;
                let told = sweeper
                    .state
                    .lock()
                    .await
                    .sweep(Instant::now(), &sweeper.config);
                for outbox in told {
                    let _ = outbox.try_send(ServerMessage::Error {
                        reason: "mailbox expired".into(),
                    });
                }
            }
        });
        loop {
            let Ok((stream, addr)) = listener.accept().await else {
                continue;
            };
            let Ok(permit) = self.slots.clone().try_acquire_owned() else {
                // Over capacity: drop the connection without reading it.
                drop(stream);
                continue;
            };
            let server = self.clone();
            tokio::spawn(async move {
                server.handle(stream, addr.ip()).await;
                drop(permit);
            });
        }
    }

    // The handshake callback's error type is fixed by tungstenite.
    #[allow(clippy::result_large_err)]
    async fn handle(&self, stream: TcpStream, ip: IpAddr) {
        let _ = stream.set_nodelay(true);
        let mut ws_config = WebSocketConfig::default();
        ws_config.max_message_size = Some(MAX_MAILBOX_BODY.saturating_add(1024));
        ws_config.max_frame_size = Some(MAX_MAILBOX_BODY.saturating_add(1024));
        let mut path = String::new();
        let callback = |req: &Request, resp: Response| {
            path = req.uri().path().to_owned();
            Ok(resp)
        };
        let accepted = tokio::time::timeout(
            IO_TIMEOUT,
            tokio_tungstenite::accept_hdr_async_with_config(stream, callback, Some(ws_config)),
        )
        .await;
        let Ok(Ok(mut ws)) = accepted else {
            return;
        };
        let route = match path.as_str() {
            "/v0/allocate" => Route::Allocate,
            p => match p
                .strip_prefix("/v0/mailbox/")
                .and_then(|n| n.parse::<u32>().ok())
            {
                Some(n) if n > 0 => Route::Mailbox(n),
                _ => return,
            },
        };
        // Every allocation and every mailbox opening counts, as in the
        // Worker; a client over its rate gets a reason, not a dropped socket.
        let admitted = self
            .state
            .lock()
            .await
            .limiter
            .admit(ip, Instant::now(), &self.config);
        if !admitted {
            let refusal = ServerMessage::Error {
                reason: TOO_MANY.into(),
            };
            let _ = tokio::time::timeout(IO_TIMEOUT, async {
                let _ = ws.send(Message::Binary(refusal.encode().into())).await;
                let _ = ws.close(None).await;
            })
            .await;
            // Read what the client sent until it closes too (RFC 6455,
            // section 7.1.1: the server closes the TCP connection after the
            // closing handshake), so that nothing is left unread.
            let _ = tokio::time::timeout(REFUSAL_LINGER, async {
                while let Some(Ok(_)) = ws.next().await {}
            })
            .await;
            return;
        }
        let (mut sink, mut source) = ws.split();
        let (outbox, mut inbox) = mpsc::channel::<ServerMessage>(64);

        let conn = {
            let mut st = self.state.lock().await;
            st.next_conn = st.next_conn.wrapping_add(1);
            st.next_conn
        };

        // Writer: forwards queued server messages; an error, a failed write
        // or a client that stops reading ends the connection.
        let writer = tokio::spawn(async move {
            while let Some(msg) = inbox.recv().await {
                let fatal = matches!(msg, ServerMessage::Error { .. });
                let sent = tokio::time::timeout(
                    IO_TIMEOUT,
                    sink.send(Message::Binary(msg.encode().into())),
                )
                .await;
                if !matches!(sent, Ok(Ok(()))) || fatal {
                    break;
                }
            }
            let _ = tokio::time::timeout(IO_TIMEOUT, sink.close()).await;
        });

        let mut held: Held = None;
        // Until it allocates or claims, a connection has a fixed deadline
        // that pings do not move, so idle sockets cannot use up the slots.
        let claim_deadline = tokio::time::Instant::now().checked_add(self.config.claim_timeout);
        loop {
            let wait = match (held, claim_deadline) {
                (None, Some(d)) => {
                    let left = d.saturating_duration_since(tokio::time::Instant::now());
                    // A ping flood must not keep a message ready forever.
                    if left.is_zero() {
                        break;
                    }
                    left
                }
                (None, None) => break,
                (Some(_), _) => self.config.idle_timeout,
            };
            let next = tokio::select! {
                n = tokio::time::timeout(wait, source.next()) => n,
                // The writer ended (expiry, error, or a client that stopped reading).
                () = async { while !writer.is_finished() { tokio::time::sleep(Duration::from_millis(200)).await } } => break,
            };
            let frame = match next {
                Ok(Some(Ok(frame))) => frame,
                _ => break,
            };
            let data = match frame {
                Message::Binary(b) => b,
                // After a claim, pings keep a connection open, but never a
                // mailbox: expiry runs on the mailbox's own clock and
                // disconnects its holders. Before a claim they change nothing.
                Message::Ping(_) | Message::Pong(_) => continue,
                _ => break,
            };
            let reply = match ClientMessage::decode(&data) {
                Ok(msg) => self.on_message(conn, route, &outbox, &mut held, msg).await,
                Err(_) => Err("malformed message"),
            };
            if let Err(reason) = reply {
                if reason != "done" {
                    let _ = outbox.try_send(ServerMessage::Error {
                        reason: reason.into(),
                    });
                }
                break;
            }
        }
        // A client that disappears without CLOSE counts as a lonely close.
        if held.is_some() {
            self.close(conn, held, Mood::Lonely).await;
        }
        drop(outbox);
        let _ = writer.await;
    }

    async fn on_message(
        &self,
        conn: ConnId,
        route: Route,
        outbox: &Outbox,
        held: &mut Held,
        msg: ClientMessage,
    ) -> Result<(), &'static str> {
        match msg {
            ClientMessage::Allocate => {
                if route != Route::Allocate {
                    return Err("allocate on /v0/allocate");
                }
                let wave = self
                    .state
                    .lock()
                    .await
                    .attack_wave(Instant::now(), &self.config);
                if wave {
                    tokio::time::sleep(self.config.wave_delay).await;
                }
                let mut st = self.state.lock().await;
                let now = Instant::now();
                if st.nameplates.len() >= self.config.max_nameplates {
                    return Err("server busy");
                }
                let n = st.free_nameplate(now, &self.config).ok_or("server busy")?;
                let mut mailbox = [0u8; 16];
                getrandom::fill(&mut mailbox).map_err(|_| "server error")?;
                let mut token = [0u8; 16];
                getrandom::fill(&mut token).map_err(|_| "server error")?;
                st.nameplates.insert(
                    n,
                    Nameplate {
                        created: now,
                        token,
                        mailbox,
                        sides: Vec::new(),
                        pending: Vec::new(),
                        pending_bytes: 0,
                        scared: false,
                    },
                );
                st.cooling.remove(&n);
                drop(st);
                outbox
                    .send(ServerMessage::Allocated {
                        nameplate: n,
                        token,
                    })
                    .await
                    .map_err(|_| "closed")?;
                // An allocation connection does nothing else.
                Err("done")
            }
            ClientMessage::Claim { nameplate, token } => {
                if route != Route::Mailbox(nameplate) || held.is_some() {
                    return Err("claim the mailbox of this address once");
                }
                let mut st = self.state.lock().await;
                let np = st
                    .nameplates
                    .get_mut(&nameplate)
                    .ok_or("unknown nameplate")?;
                if np.sides.len() >= 2 {
                    // The rule that limits an attacker to one guess.
                    return Err("nameplate full");
                }
                if np.sides.is_empty() && !token_matches(token, &np.token) {
                    // The allocator always claims first, with its token;
                    // anyone else claiming an empty nameplate is guessing.
                    return Err("unknown nameplate");
                }
                np.sides.push(Side {
                    conn,
                    outbox: outbox.clone(),
                    posted: 0,
                    closed: false,
                });
                let mailbox = np.mailbox;
                *held = Some((nameplate, mailbox));
                // Hand over anything the other side posted before we came.
                let mut deliver = Vec::new();
                for (from, phase, body) in np.pending.drain(..) {
                    if from != conn {
                        deliver.push(ServerMessage::Message { phase, body });
                    }
                }
                np.pending_bytes = 0;
                drop(st);
                outbox
                    .send(ServerMessage::Claimed { mailbox })
                    .await
                    .map_err(|_| "closed")?;
                for m in deliver {
                    outbox.send(m).await.map_err(|_| "closed")?;
                }
                Ok(())
            }
            ClientMessage::Add { phase, body } => {
                let mut st = self.state.lock().await;
                let (_, np) = st.current(*held).ok_or("claim first")?;
                let me = np
                    .sides
                    .iter_mut()
                    .find(|s| s.conn == conn)
                    .ok_or("claim first")?;
                if me.closed {
                    return Err("mailbox closed");
                }
                me.posted = me.posted.saturating_add(1);
                if me.posted > self.config.max_messages_per_side {
                    return Err("too many messages");
                }
                let peer = np
                    .sides
                    .iter()
                    .find(|s| s.conn != conn && !s.closed)
                    .map(|s| s.outbox.clone());
                match peer {
                    Some(peer) => {
                        drop(st);
                        // A slow or vanished peer must not stall us.
                        let _ = peer.try_send(ServerMessage::Message { phase, body });
                    }
                    None => {
                        let bytes = np.pending_bytes.saturating_add(body.len());
                        if np.pending.len() >= MAX_PENDING_MESSAGES || bytes > MAX_PENDING_BYTES {
                            return Err("mailbox full");
                        }
                        np.pending_bytes = bytes;
                        np.pending.push((conn, phase, body));
                    }
                }
                Ok(())
            }
            ClientMessage::Close { mood } => {
                self.close(conn, *held, mood).await;
                *held = None;
                Err("closed")
            }
        }
    }

    async fn close(&self, conn: ConnId, held: Held, mood: Mood) {
        let mut st = self.state.lock().await;
        let now = Instant::now();
        let Some((n, np)) = st.current(held) else {
            return;
        };
        let is_side = np.sides.iter().any(|s| s.conn == conn);
        if !is_side {
            // Only the two people in a mailbox can close it.
            return;
        }
        let mut notify = Vec::new();
        for side in &mut np.sides {
            if side.conn == conn {
                side.closed = true;
            } else if !side.closed {
                notify.push(side.outbox.clone());
            }
        }
        let all_closed = np.sides.iter().all(|s| s.closed);
        // A scary close only counts when two sides really exchanged
        // messages; otherwise anyone could retire nameplates at will.
        let exchanged = np.sides.len() == 2 && np.sides.iter().all(|s| s.posted > 0);
        let scary = mood == Mood::Scary && is_side && exchanged && !np.scared;
        if scary {
            np.scared = true;
        }
        if all_closed || scary {
            // Contents are forgotten as soon as nobody needs them.
            np.pending.clear();
            np.pending_bytes = 0;
        }
        if all_closed {
            st.remove(n, now);
        }
        if scary {
            st.retired.insert(n, now);
            st.scary.push_back(now);
        }
        drop(st);
        for peer in notify {
            let _ = peer.try_send(ServerMessage::PeerClosed { mood });
        }
    }

    /// Scary closes in the current attack-wave window, for tests and health
    /// checks.
    pub async fn scary_count(&self) -> usize {
        self.state.lock().await.scary.len()
    }

    /// Number of live nameplates, for tests and health checks.
    pub async fn nameplate_count(&self) -> usize {
        self.state.lock().await.nameplates.len()
    }
}

/// Bind `addr` and serve with `config`. Returns the bound address and the
/// server task.
pub async fn spawn(
    addr: SocketAddr,
    config: Config,
) -> std::io::Result<(SocketAddr, Server, tokio::task::JoinHandle<()>)> {
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    let server = Server::new(config);
    let task = tokio::spawn(server.clone().serve(listener));
    Ok((local, server, task))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::arithmetic_side_effects)]
mod tests {
    // The same cases as the Worker's `test/limit.test.ts`.
    use super::*;

    fn config() -> Config {
        Config {
            rate_burst: 4,
            rate_interval: Duration::from_secs(2),
            max_clients: 3,
            ..Config::default()
        }
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn a_burst_is_allowed_then_one_more_each_interval() {
        let cfg = config();
        let mut l = Limiter::default();
        let t0 = Instant::now();
        let a = ip("203.0.113.7");
        for i in 0..4 {
            assert!(l.admit(a, t0, &cfg), "request {i}");
        }
        assert!(!l.admit(a, t0, &cfg));
        assert!(!l.admit(a, t0 + Duration::from_millis(1_999), &cfg));
        assert!(l.admit(a, t0 + Duration::from_secs(2), &cfg));
        assert!(!l.admit(a, t0 + Duration::from_secs(2), &cfg));
    }

    #[test]
    fn refused_requests_are_not_counted() {
        let cfg = config();
        let mut l = Limiter::default();
        let t0 = Instant::now();
        let a = ip("203.0.113.7");
        for _ in 0..4 {
            l.admit(a, t0, &cfg);
        }
        for _ in 0..100 {
            assert!(!l.admit(a, t0, &cfg));
        }
        assert!(l.admit(a, t0 + Duration::from_secs(2), &cfg));
    }

    #[test]
    fn clients_are_counted_apart() {
        let cfg = config();
        let mut l = Limiter::default();
        let t0 = Instant::now();
        for _ in 0..4 {
            l.admit(ip("203.0.113.7"), t0, &cfg);
        }
        assert!(!l.admit(ip("203.0.113.7"), t0, &cfg));
        assert!(l.admit(ip("203.0.113.8"), t0, &cfg));
    }

    #[test]
    fn a_quiet_client_is_forgotten_once_its_burst_is_full_again() {
        let cfg = config();
        let mut l = Limiter::default();
        let t0 = Instant::now();
        for _ in 0..4 {
            l.admit(ip("203.0.113.7"), t0, &cfg);
        }
        l.admit(ip("203.0.113.8"), t0, &cfg);
        l.sweep(t0 + Duration::from_secs(2));
        assert_eq!(l.due.len(), 1);
        l.sweep(t0 + Duration::from_secs(8));
        assert!(l.due.is_empty());
    }

    #[test]
    fn beyond_the_most_clients_a_new_client_is_let_through_uncounted() {
        let cfg = config();
        let mut l = Limiter::default();
        let t0 = Instant::now();
        for c in ["192.0.2.1", "192.0.2.2", "192.0.2.3"] {
            l.admit(ip(c), t0, &cfg);
        }
        for _ in 0..10 {
            assert!(l.admit(ip("192.0.2.4"), t0, &cfg));
        }
        assert_eq!(l.due.len(), 3);
    }

    /// #428: only the very token lets a claim in, now compared in
    /// constant time; one bit off, or none at all, does not.
    #[test]
    fn only_the_allocation_token_matches() {
        let ours = [0x5a; 16];
        assert!(token_matches(Some(ours), &ours));
        assert!(!token_matches(None, &ours));
        for byte in 0..ours.len() {
            let mut near = ours;
            if let Some(b) = near.get_mut(byte) {
                *b ^= 1;
            }
            assert!(!token_matches(Some(near), &ours), "byte {byte}");
        }
    }

    #[test]
    fn a_client_is_an_ipv4_address_or_an_ipv6_64() {
        assert_eq!(
            client_of(ip("203.0.113.7")),
            client_of(ip("::ffff:203.0.113.7"))
        );
        assert_ne!(client_of(ip("203.0.113.7")), client_of(ip("203.0.113.8")));
        assert_eq!(
            client_of(ip("2001:db8:1:2:3:4:5:6")),
            client_of(ip("2001:db8:1:2::9"))
        );
        assert_ne!(
            client_of(ip("2001:db8:1:2::9")),
            client_of(ip("2001:db8:1:3::9"))
        );
        // An IPv4 client never shares a counter with an IPv6 /64.
        assert_ne!(client_of(ip("0.0.0.1")), client_of(ip("::1")));
    }
}
