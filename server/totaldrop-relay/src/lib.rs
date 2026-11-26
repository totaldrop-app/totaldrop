// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! The TotalDrop relay (spec section 6).
//!
//! It does two jobs, both described in [`totaldrop_proto::relay`]:
//!
//! - **address discovery**: a STUN responder (Binding requests only) that
//!   tells a device the address and port it saw, for relays that are run
//!   without a separate STUN server;
//! - **forwarding** on WebSocket: devices sign in with a single-use endpoint
//!   key, and the relay passes QUIC datagrams between keys.
//!
//! What crosses the relay is already encrypted end to end. The relay keeps
//! nothing on disk and logs nothing about clients: no addresses, no keys, no
//! traffic. Each connection is capped in rate and idle time, and each pair is
//! capped in total bytes per key and total lifetime across reconnects, kept in
//! memory under a random-keyed hash until the pair's window expires.

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{Mutex, Notify, Semaphore, mpsc};
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use totaldrop_proto::relay as wire;

/// The WebSocket path of forwarding.
pub const PATH: &str = "/v0/relay";

/// How long the WebSocket handshake, signing in and every write may take.
const IO_TIMEOUT: Duration = Duration::from_secs(10);

/// Limits. The defaults suit one small server.
#[derive(Debug, Clone)]
pub struct Config {
    /// Devices connected at once.
    pub max_clients: usize,
    /// Pairs whose byte and time totals are kept in memory at once.
    pub max_pairs: usize,
    /// Bytes per second one device may send through the relay.
    pub rate: u64,
    /// Bytes one key may send on a pair in total, across reconnects.
    pub session_bytes: u64,
    /// How long one pair may use the relay, from its first sign-in.
    pub session_time: Duration,
    /// A connection that sends no datagram for this long is closed; pings do
    /// not count.
    pub idle: Duration,
    /// Datagrams queued for one device before more are dropped.
    pub queue: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_clients: 2_000,
            max_pairs: 100_000,
            rate: 8 * 1024 * 1024,
            session_bytes: 32 * 1024 * 1024 * 1024,
            session_time: Duration::from_secs(6 * 60 * 60),
            idle: Duration::from_secs(60),
            queue: 1_024,
        }
    }
}

type Outbox = mpsc::Sender<Vec<u8>>;

/// The two endpoint keys a pair path names.
type Pair = ([u8; 32], [u8; 32]);

/// Random-keyed HMAC digest of a pair or endpoint key, so raw public keys do
/// not stay in memory after their sockets close.
type Tag = [u8; 32];

fn hmac_key() -> std::io::Result<ring::hmac::Key> {
    let mut secret = [0u8; 32];
    getrandom::fill(&mut secret)
        .map_err(|e| std::io::Error::other(format!("OS randomness unavailable: {e}")))?;
    Ok(ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &secret))
}

fn pair_tag(secret: &ring::hmac::Key, pair: &Pair) -> Tag {
    let mut ctx = ring::hmac::Context::with_key(secret);
    ctx.update(b"totaldrop relay pair v0");
    ctx.update(&pair.0);
    ctx.update(&pair.1);
    let mut out = [0u8; 32];
    if let Some(b) = ctx.sign().as_ref().get(..32) {
        out.copy_from_slice(b);
    }
    out
}

fn key_tag(secret: &ring::hmac::Key, key: &[u8; 32]) -> Tag {
    let mut ctx = ring::hmac::Context::with_key(secret);
    ctx.update(b"totaldrop relay key v0");
    ctx.update(key);
    let mut out = [0u8; 32];
    if let Some(b) = ctx.sign().as_ref().get(..32) {
        out.copy_from_slice(b);
    }
    out
}

/// One signed-in connection.
#[derive(Debug)]
struct Client {
    /// Connection number, so a connection removes only its own entry.
    id: u64,
    /// The pair it signed in on: a datagram reaches a key only on the same
    /// pair.
    pair: Pair,
    pair_tag: Tag,
    pair_gen: u64,
    outbox: Outbox,
    /// Ends the connection: it was replaced, or the other side left.
    end: Arc<Notify>,
}

/// Running totals for one pair across reconnects (#190).
#[derive(Debug)]
struct PairRecord {
    generation: u64,
    started: Instant,
    keys: (Tag, Tag),
    /// Bytes sent by `(pair.0, pair.1)`.
    sent: (u64, u64),
    /// Set once either key exceeds `Config::session_bytes`.
    over: bool,
    /// Signed-in sockets currently open on this pair.
    live: u8,
    /// Non-zero stamp matching the latest entry in `State::inactive` while
    /// `live == 0`; `0` while at least one socket is open.
    inactive_stamp: u64,
}

type OrderEntry = (Tag, u64, Instant, (Tag, Tag));

#[derive(Debug, Default)]
struct State {
    clients: HashMap<[u8; 32], Client>,
    pairs: HashMap<Tag, PairRecord>,
    /// Pairs in creation order, so expired entries sit at the front.
    order: VecDeque<OrderEntry>,
    /// Inactive pairs (`live == 0`) in FIFO disconnect order for amortized
    /// `O(1)` eviction without scanning `order` (#190).
    inactive: VecDeque<(Tag, u64, u64)>,
    /// Which pair each single-use key signed in on: `(pair_tag, gen, started)`.
    key_pair: HashMap<Tag, (Tag, u64, Instant)>,
    next_gen: u64,
    inactive_seq: u64,
}

impl State {
    fn drop_entry(&mut self, (ptag, generation, _, ktags): OrderEntry) {
        if self
            .pairs
            .get(&ptag)
            .is_some_and(|r| r.generation == generation)
        {
            self.pairs.remove(&ptag);
        }
        for kt in [ktags.0, ktags.1] {
            if self
                .key_pair
                .get(&kt)
                .is_some_and(|&(_, g, _)| g == generation)
            {
                self.key_pair.remove(&kt);
            }
        }
    }

    fn note_inactive(&mut self, ptag: Tag) {
        self.inactive_seq = self.inactive_seq.wrapping_add(1).max(1);
        let stamp = self.inactive_seq;
        if let Some(rec) = self.pairs.get_mut(&ptag)
            && rec.live == 0
        {
            rec.inactive_stamp = stamp;
            self.inactive.push_back((ptag, rec.generation, stamp));
        }
        if self.inactive.len() > self.pairs.len().saturating_mul(2).max(64) {
            let pairs = &self.pairs;
            self.inactive.retain(|&(pt, g, st)| {
                pairs
                    .get(&pt)
                    .is_some_and(|r| r.generation == g && r.live == 0 && r.inactive_stamp == st)
            });
        }
    }

    fn trim_stale_inactive(&mut self) {
        while let Some(&(ptag, generation, stamp)) = self.inactive.front() {
            let valid = self.pairs.get(&ptag).is_some_and(|r| {
                r.generation == generation && r.live == 0 && r.inactive_stamp == stamp
            });
            if valid {
                break;
            }
            self.inactive.pop_front();
        }
    }

    fn sweep(&mut self, now: Instant, session_time: Duration) {
        while let Some(&(_, _, started, _)) = self.order.front() {
            if now.saturating_duration_since(started) < session_time {
                break;
            }
            if let Some(entry) = self.order.pop_front() {
                self.drop_entry(entry);
            }
        }
        self.trim_stale_inactive();
    }

    fn make_room(&mut self, max_pairs: usize) {
        while self.pairs.len() >= max_pairs {
            let Some((ptag, generation, stamp)) = self.inactive.pop_front() else {
                break;
            };
            let evict = self.pairs.get(&ptag).is_some_and(|r| {
                r.generation == generation && r.live == 0 && r.inactive_stamp == stamp
            });
            if evict && let Some(rec) = self.pairs.remove(&ptag) {
                for kt in [rec.keys.0, rec.keys.1] {
                    if self
                        .key_pair
                        .get(&kt)
                        .is_some_and(|&(_, g, _)| g == generation)
                    {
                        self.key_pair.remove(&kt);
                    }
                }
            }
        }
        if self.order.len() > max_pairs.saturating_mul(2).max(64) {
            let pairs = &self.pairs;
            self.order
                .retain(|&(pt, g, _, _)| pairs.get(&pt).is_some_and(|r| r.generation == g));
        }
    }
}

/// A running relay.
#[derive(Debug, Clone)]
pub struct Relay {
    state: Arc<Mutex<State>>,
    secret: Arc<ring::hmac::Key>,
    next: Arc<std::sync::atomic::AtomicU64>,
    accepted: Arc<std::sync::atomic::AtomicU64>,
    forwarded: Arc<std::sync::atomic::AtomicU64>,
    slots: Arc<Semaphore>,
    config: Arc<Config>,
}

impl Relay {
    /// A relay with the given limits. Fails closed if OS randomness for the
    /// per-process HMAC key is unavailable.
    pub fn new(config: Config) -> std::io::Result<Self> {
        Ok(Self {
            state: Arc::default(),
            secret: Arc::new(hmac_key()?),
            next: Arc::default(),
            accepted: Arc::default(),
            forwarded: Arc::default(),
            slots: Arc::new(Semaphore::new(config.max_clients)),
            config: Arc::new(config),
        })
    }

    /// Devices signed in right now, for tests and health checks.
    pub async fn client_count(&self) -> usize {
        self.state.lock().await.clients.len()
    }

    /// Pairs whose running totals are currently held in memory, for tests.
    pub async fn pair_count(&self) -> usize {
        self.state.lock().await.pairs.len()
    }

    /// TCP connections accepted since start: one total, for tests.
    pub fn connections_accepted(&self) -> u64 {
        self.accepted.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Bytes of datagrams forwarded since start: one total, nothing per
    /// client, for tests and health checks.
    pub fn forwarded_bytes(&self) -> u64 {
        self.forwarded.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Periodically forget expired pair records even when no new traffic
    /// arrives.
    async fn sweep_loop(&self) {
        let step = self
            .config
            .session_time
            .checked_div(2)
            .unwrap_or(Duration::from_secs(60))
            .clamp(Duration::from_millis(50), Duration::from_secs(60));
        let mut ticker = tokio::time::interval(step);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            self.state
                .lock()
                .await
                .sweep(Instant::now(), self.config.session_time);
        }
    }

    /// Answer STUN Binding requests on `socket` until the task is dropped.
    pub async fn discovery(socket: UdpSocket) {
        let mut buf = [0u8; 548];
        loop {
            let Ok((n, from)) = socket.recv_from(&mut buf).await else {
                continue;
            };
            if let Some(id) = buf.get(..n).and_then(wire::parse_stun_request) {
                let _ = socket.send_to(&wire::stun_response(&id, from), from).await;
            }
        }
    }

    /// Accept WebSocket connections on `listener` and sweep expired pair
    /// records until the task is dropped.
    pub async fn serve(self, listener: TcpListener) {
        let sweeping = self.clone();
        tokio::join!(self.accept_loop(listener), sweeping.sweep_loop());
    }

    async fn accept_loop(self, listener: TcpListener) {
        loop {
            let Ok((stream, _addr)) = listener.accept().await else {
                continue;
            };
            self.accepted
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let Ok(permit) = self.slots.clone().try_acquire_owned() else {
                // Over capacity: drop the connection without reading it.
                drop(stream);
                continue;
            };
            let relay = self.clone();
            tokio::spawn(async move {
                relay.handle(stream).await;
                drop(permit);
            });
        }
    }

    // The handshake callback's error type is fixed by tungstenite.
    #[allow(clippy::result_large_err)]
    async fn handle(&self, stream: TcpStream) {
        let _ = stream.set_nodelay(true);
        let mut ws_config = WebSocketConfig::default();
        ws_config.max_message_size = Some(wire::MAX_MESSAGE);
        ws_config.max_frame_size = Some(wire::MAX_MESSAGE);
        let mut path = String::new();
        let callback = |req: &Request, resp: Response| {
            path = req.uri().path().to_owned();
            refuse_browsers(req, resp)
        };
        let Ok(Ok(mut ws)) = tokio::time::timeout(
            IO_TIMEOUT,
            tokio_tungstenite::accept_hdr_async_with_config(stream, callback, Some(ws_config)),
        )
        .await
        else {
            return;
        };
        // `/v0/relay/<pair>` names the only two keys that may sign in here,
        // and the only two that datagrams may pass between (as on the
        // Cloudflare relay). Any other path is refused.
        let Some(pair) = path
            .strip_prefix(PATH)
            .and_then(|rest| rest.strip_prefix('/'))
            .and_then(wire::parse_pair_name)
        else {
            return;
        };

        // Sign in: the device proves it holds the endpoint key it claims.
        let mut challenge = [0u8; 32];
        if getrandom::fill(&mut challenge).is_err() {
            return;
        }
        let key = tokio::time::timeout(IO_TIMEOUT, async {
            ws.send(Message::Binary(wire::challenge(&challenge).into()))
                .await
                .ok()?;
            let hello = match ws.next().await {
                Some(Ok(Message::Binary(b))) => wire::parse_hello(&b)?,
                _ => return None,
            };
            let (key, signature) = hello;
            if key != pair.0 && key != pair.1 {
                return None;
            }
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, key)
                .verify(&wire::signed_bytes(&challenge), &signature)
                .ok()?;
            Some(key)
        })
        .await;
        let Ok(Some(key)) = key else { return };
        let second_side = key != pair.0;
        let other = if second_side { pair.0 } else { pair.1 };

        let (outbox, mut inbox) = mpsc::channel::<Vec<u8>>(self.config.queue);
        let id = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let end = Arc::new(Notify::new());
        let ptag = pair_tag(&self.secret, &pair);
        let ktags = (
            key_tag(&self.secret, &pair.0),
            key_tag(&self.secret, &pair.1),
        );
        let ktag = if second_side { ktags.1 } else { ktags.0 };

        let admitted = {
            let now = Instant::now();
            let mut state = self.state.lock().await;
            state.sweep(now, self.config.session_time);

            // One pair per key: keys are single-use, so a key on a second
            // pair within its window is refused even after disconnecting
            // (#190). (The Cloudflare relay has one object per pair and
            // cannot see this.)
            if state.clients.get(&key).is_some_and(|old| old.pair != pair) {
                return;
            }
            if let Some(&(bound, _, started)) = state.key_pair.get(&ktag)
                && now.saturating_duration_since(started) < self.config.session_time
                && bound != ptag
            {
                return;
            }

            let (pair_gen, started) = if let Some(rec) = state.pairs.get(&ptag)
                && now.saturating_duration_since(rec.started) < self.config.session_time
            {
                if rec.over {
                    return;
                }
                (rec.generation, rec.started)
            } else {
                // Refuse sign-in when no pair record can be created so live
                // load can never bypass per-pair reconnect limits (#190).
                if self.config.max_pairs == 0 {
                    return;
                }
                state.make_room(self.config.max_pairs);
                if state.pairs.len() >= self.config.max_pairs {
                    return;
                }
                let generation = state.next_gen;
                state.next_gen = state.next_gen.wrapping_add(1);
                state.pairs.insert(
                    ptag,
                    PairRecord {
                        generation,
                        started: now,
                        keys: ktags,
                        sent: (0, 0),
                        over: false,
                        live: 0,
                        inactive_stamp: 0,
                    },
                );
                state.order.push_back((ptag, generation, now, ktags));
                (generation, now)
            };

            let Some(deadline) = started
                .checked_add(self.config.session_time)
                .filter(|&d| now < d)
            else {
                return;
            };
            state.key_pair.insert(ktag, (ptag, pair_gen, started));
            // The holder of the key signed in again on its pair: its old
            // socket died without this relay noticing (a Wi-Fi switch, NAT
            // rebinding). The new one takes over at once, before QUIC gives
            // up (#190). Only the key's holder can sign in.
            let replaced = state.clients.insert(
                key,
                Client {
                    id,
                    pair,
                    pair_tag: ptag,
                    pair_gen,
                    outbox,
                    end: end.clone(),
                },
            );
            if let Some(old) = replaced {
                let became_inactive = state
                    .pairs
                    .get_mut(&old.pair_tag)
                    .filter(|r| r.generation == old.pair_gen)
                    .is_some_and(|old_rec| {
                        old_rec.live = old_rec.live.saturating_sub(1);
                        old_rec.live == 0
                    });
                if became_inactive {
                    state.note_inactive(old.pair_tag);
                }
                old.end.notify_one();
            }
            if let Some(rec) = state
                .pairs
                .get_mut(&ptag)
                .filter(|r| r.generation == pair_gen)
            {
                rec.live = rec.live.saturating_add(1);
                rec.inactive_stamp = 0;
            }
            (pair_gen, deadline)
        };
        let (pair_gen, deadline) = admitted;

        if !matches!(
            tokio::time::timeout(IO_TIMEOUT, ws.send(Message::Binary(wire::ok().into()))).await,
            Ok(Ok(()))
        ) {
            self.leave(&key, id).await;
            return;
        }

        let (mut sink, mut source) = ws.split();
        let writer = tokio::spawn(async move {
            while let Some(frame) = inbox.recv().await {
                let sent =
                    tokio::time::timeout(IO_TIMEOUT, sink.send(Message::Binary(frame.into())))
                        .await;
                if !matches!(sent, Ok(Ok(()))) {
                    break;
                }
            }
            let _ = tokio::time::timeout(IO_TIMEOUT, sink.close()).await;
        });

        // Idle time counts from the last datagram: pings keep TCP alive but
        // do not keep a connection that carries nothing.
        let mut last_data = Instant::now();
        let mut budget = Budget::new(self.config.rate);
        loop {
            if writer.is_finished() {
                break;
            }
            let now = Instant::now();
            let Some(until_end) = deadline
                .checked_duration_since(now)
                .filter(|w| !w.is_zero())
            else {
                break;
            };
            let Some(until_idle) = self
                .config
                .idle
                .checked_sub(last_data.elapsed())
                .filter(|w| !w.is_zero())
            else {
                break;
            };
            let wait = until_end.min(until_idle);
            let next = tokio::select! {
                () = end.notified() => break,
                next = tokio::time::timeout(wait, source.next()) => match next {
                    Ok(Some(Ok(m))) => m,
                    _ => break,
                },
            };
            let frame = match next {
                Message::Binary(b) => b,
                Message::Ping(_) | Message::Pong(_) => continue,
                _ => break,
            };
            let Some((to, datagrams)) = wire::parse_frame(&frame) else {
                break;
            };
            // Only the other key of this pair is a destination.
            if to != other {
                break;
            }
            last_data = Instant::now();
            let payload: usize = datagrams.iter().map(|d| d.len()).sum();
            let size = u64::try_from(frame.len()).unwrap_or(u64::MAX);
            // Charge the sender's allowance and look up the peer's outbox
            // under a single state lock (#190).
            let Some(target) = self
                .spend_and_lookup(&ptag, pair_gen, second_side, size, &to, &pair)
                .await
            else {
                break;
            };
            // Slow the sender down instead of dropping: TCP pushes back.
            budget.spend(size).await;
            if let Some(target) = target {
                // A full queue drops, like a busy network; QUIC recovers.
                // Same message with the source key in place of the target's.
                let mut out = frame.to_vec();
                if let Some(head) = out.get_mut(..32) {
                    head.copy_from_slice(&key);
                }
                if target.try_send(out).is_ok() {
                    let n = u64::try_from(payload).unwrap_or(0);
                    self.forwarded
                        .fetch_add(n, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
        writer.abort();
        self.leave(&key, id).await;
    }

    /// Charge `size` bytes to `ptag`/`pair_gen` and resolve `to`'s outbox
    /// under one lock acquisition (#190). Returns `None` once the byte limit
    /// is exceeded (or the pair record has expired).
    async fn spend_and_lookup(
        &self,
        ptag: &Tag,
        pair_gen: u64,
        second_side: bool,
        size: u64,
        to: &[u8; 32],
        pair: &Pair,
    ) -> Option<Option<Outbox>> {
        let mut state = self.state.lock().await;
        let rec = state
            .pairs
            .get_mut(ptag)
            .filter(|r| r.generation == pair_gen)?;
        let slot = if second_side {
            &mut rec.sent.1
        } else {
            &mut rec.sent.0
        };
        *slot = slot.saturating_add(size);
        if *slot > self.config.session_bytes || rec.over {
            rec.over = true;
            return None;
        }
        // The target must have signed in on this same pair: a key that is
        // part of two pairs is reachable only from the one it chose.
        let target = state
            .clients
            .get(to)
            .filter(|c| c.pair == *pair)
            .map(|c| c.outbox.clone());
        Some(target)
    }

    /// Remove the connection `id` of `key`. When it was still the key's
    /// connection (not replaced), the transfer is over, however it ended:
    /// the other signed-in side of its pair is ended too, as on the
    /// Cloudflare relay (protocol notes, item 20).
    async fn leave(&self, key: &[u8; 32], id: u64) {
        let mut state = self.state.lock().await;
        if !state.clients.get(key).is_some_and(|c| c.id == id) {
            return;
        }
        let Some(gone) = state.clients.remove(key) else {
            return;
        };
        let became_inactive = state
            .pairs
            .get_mut(&gone.pair_tag)
            .filter(|r| r.generation == gone.pair_gen)
            .is_some_and(|rec| {
                rec.live = rec.live.saturating_sub(1);
                rec.live == 0
            });
        if became_inactive {
            state.note_inactive(gone.pair_tag);
        }
        let other = if *key == gone.pair.0 {
            gone.pair.1
        } else {
            gone.pair.0
        };
        if let Some(peer) = state.clients.get(&other).filter(|c| c.pair == gone.pair) {
            peer.end.notify_one();
        }
    }
}

/// A token bucket holding at most one second of traffic.
struct Budget {
    rate: u64,
    tokens: u64,
    last: Instant,
}

impl Budget {
    fn new(rate: u64) -> Self {
        Self {
            rate: rate.max(1),
            tokens: rate,
            last: Instant::now(),
        }
    }

    async fn spend(&mut self, n: u64) {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last);
        self.last = now;
        let refill = u64::try_from(
            elapsed
                .as_micros()
                .saturating_mul(u128::from(self.rate))
                .checked_div(1_000_000)
                .unwrap_or(0),
        )
        .unwrap_or(u64::MAX);
        self.tokens = self.tokens.saturating_add(refill).min(self.rate);
        if self.tokens >= n {
            self.tokens = self.tokens.saturating_sub(n);
            return;
        }
        let missing = n.saturating_sub(self.tokens);
        self.tokens = 0;
        let micros = missing
            .saturating_mul(1_000_000)
            .checked_div(self.rate)
            .unwrap_or(0);
        tokio::time::sleep(Duration::from_micros(micros)).await;
        self.last = Instant::now();
    }
}

/// Serve WebSocket forwarding on `ws` and discovery on `udp`. Returns the
/// bound addresses, the relay and its task.
pub async fn spawn(
    ws: SocketAddr,
    udp: SocketAddr,
    config: Config,
) -> std::io::Result<(SocketAddr, SocketAddr, Relay, tokio::task::JoinHandle<()>)> {
    let listener = TcpListener::bind(ws).await?;
    let socket = UdpSocket::bind(udp).await?;
    let (ws, udp) = (listener.local_addr()?, socket.local_addr()?);
    let relay = Relay::new(config)?;
    let serving = relay.clone();
    let task = tokio::spawn(async move {
        tokio::join!(serving.serve(listener), Relay::discovery(socket));
    });
    Ok((ws, udp, relay, task))
}

/// For tests: forwarding on `ws://127.0.0.1:<port>/v0/relay` (clients add
/// `/<key A>-<key B>`) and STUN on UDP `127.0.0.1:<other port>`. Returns
/// both, the relay and its task.
///
/// Each gets its own free port from the system. Taking the TCP port's
/// number for UDP as well kept failing on Windows runners (#320): a port
/// free for one protocol need not be free for the other.
pub async fn spawn_local(
    config: Config,
) -> std::io::Result<(String, String, Relay, tokio::task::JoinHandle<()>)> {
    let any = SocketAddr::from(([127, 0, 0, 1], 0));
    let (ws, udp, relay, task) = spawn(any, any, config).await?;
    Ok((format!("ws://{ws}{PATH}"), udp.to_string(), relay, task))
}

/// #450: a WebSocket opened by a web page always carries `Origin`; the
/// apps and the command line never send it. Refusing those keeps a page
/// from opening connections from its visitors' browsers.
#[allow(clippy::result_large_err)] // the type tungstenite asks for
fn refuse_browsers(req: &Request, resp: Response) -> Result<Response, ErrorResponse> {
    if req.headers().contains_key("origin") {
        let mut refused = ErrorResponse::new(Some("browsers are not served\n".to_owned()));
        *refused.status_mut() = tokio_tungstenite::tungstenite::http::StatusCode::FORBIDDEN;
        return Err(refused);
    }
    Ok(resp)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

    use super::*;

    #[tokio::test]
    async fn discovery_answers_with_the_seen_address() {
        let (_url, stun, _relay, _task) = spawn_local(Config::default()).await.unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let nonce = [5u8; 12];
        client
            .send_to(&wire::stun_request(&nonce), stun.as_str())
            .await
            .unwrap();
        let mut buf = [0u8; 128];
        let (n, _) = tokio::time::timeout(Duration::from_secs(2), client.recv_from(&mut buf))
            .await
            .unwrap()
            .unwrap();
        let seen = wire::parse_stun_response(&buf[..n], &nonce).unwrap();
        assert_eq!(seen, client.local_addr().unwrap());
        // Garbage gets no answer.
        client.send_to(b"hello", stun.as_str()).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(300), client.recv_from(&mut buf))
                .await
                .is_err()
        );
    }

    type Ws = tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >;

    fn keypair() -> (ring::signature::Ed25519KeyPair, [u8; 32]) {
        use ring::signature::KeyPair;
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let kp = ring::signature::Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let public: [u8; 32] = kp.public_key().as_ref().try_into().unwrap();
        (kp, public)
    }

    /// Connect to `url` and sign in with `kp`; `None` when the relay refuses.
    async fn sign_in(
        url: &str,
        kp: &ring::signature::Ed25519KeyPair,
        public: &[u8; 32],
    ) -> Option<Ws> {
        let (mut ws, _) = tokio_tungstenite::connect_async(url).await.ok()?;
        let challenge = match tokio::time::timeout(Duration::from_secs(2), ws.next()).await {
            Ok(Some(Ok(Message::Binary(b)))) => wire::parse_challenge(&b)?,
            _ => return None,
        };
        let sig: [u8; 64] = kp
            .sign(&wire::signed_bytes(&challenge))
            .as_ref()
            .try_into()
            .ok()?;
        ws.send(Message::Binary(wire::hello(public, &sig).into()))
            .await
            .ok()?;
        match tokio::time::timeout(Duration::from_secs(2), ws.next()).await {
            Ok(Some(Ok(Message::Binary(b)))) if wire::is_ok(&b) => Some(ws),
            _ => None,
        }
    }

    /// The next datagram frame on `ws` within `wait`, skipping pings.
    async fn next_frame(ws: &mut Ws, wait: Duration) -> Option<Vec<u8>> {
        let deadline = tokio::time::Instant::now().checked_add(wait).unwrap();
        loop {
            match tokio::time::timeout_at(deadline, ws.next()).await {
                Ok(Some(Ok(Message::Binary(b)))) => return Some(b.to_vec()),
                Ok(Some(Ok(Message::Ping(_) | Message::Pong(_)))) => {}
                _ => return None,
            }
        }
    }

    /// Whether the relay closed `ws` within `wait`.
    async fn closed(ws: &mut Ws, wait: Duration) -> bool {
        let deadline = tokio::time::Instant::now().checked_add(wait).unwrap();
        loop {
            match tokio::time::timeout_at(deadline, ws.next()).await {
                Err(_) => return false,
                Ok(None | Some(Err(_)) | Some(Ok(Message::Close(_)))) => return true,
                Ok(Some(Ok(_))) => {}
            }
        }
    }

    #[tokio::test]
    async fn plain_path_is_refused() {
        let (url, _stun, _relay, _task) = spawn_local(Config::default()).await.unwrap();
        let (kp, public) = keypair();
        assert!(sign_in(&url, &kp, &public).await.is_none());
    }

    /// A device can reach only the other key of its own pair; trying another
    /// pair's key ends its connection and delivers nothing.
    #[tokio::test]
    async fn forwards_only_within_the_pair() {
        let (url, _stun, relay, _task) = spawn_local(Config::default()).await.unwrap();
        let (ka, a) = keypair();
        let (kb, b) = keypair();
        let (kc, c) = keypair();
        let (kd, d) = keypair();
        let ab = format!("{url}/{}", wire::pair_name(&a, &b));
        let cd = format!("{url}/{}", wire::pair_name(&c, &d));
        let mut wa = sign_in(&ab, &ka, &a).await.unwrap();
        let mut wb = sign_in(&ab, &kb, &b).await.unwrap();
        let mut wc = sign_in(&cd, &kc, &c).await.unwrap();
        let _wd = sign_in(&cd, &kd, &d).await.unwrap();

        // C, signed in on the C–D pair, aims at A.
        wc.send(Message::Binary(wire::frame(&a, b"injected").into()))
            .await
            .unwrap();
        assert!(closed(&mut wc, Duration::from_secs(2)).await);
        assert!(
            next_frame(&mut wa, Duration::from_millis(300))
                .await
                .is_none()
        );
        assert_eq!(relay.forwarded_bytes(), 0);

        // Within the pair it works.
        wb.send(Message::Binary(wire::frame(&a, b"hello").into()))
            .await
            .unwrap();
        let got = next_frame(&mut wa, Duration::from_secs(2)).await.unwrap();
        let (from, datagrams) = wire::parse_frame(&got).unwrap();
        assert_eq!(from, b);
        assert_eq!(datagrams, vec![&b"hello"[..]]);
    }

    /// A key in two pairs is reachable only on the pair it signed in on: C,
    /// on its own pair with A, cannot reach A, who is on the A–B pair.
    #[tokio::test]
    async fn a_shared_key_is_reachable_only_on_its_own_pair() {
        let (url, _stun, relay, _task) = spawn_local(Config::default()).await.unwrap();
        let (ka, a) = keypair();
        let (_kb, b) = keypair();
        let (kc, c) = keypair();
        let mut wa = sign_in(&format!("{url}/{}", wire::pair_name(&a, &b)), &ka, &a)
            .await
            .unwrap();
        let mut wc = sign_in(&format!("{url}/{}", wire::pair_name(&a, &c)), &kc, &c)
            .await
            .unwrap();
        wc.send(Message::Binary(wire::frame(&a, b"flood").into()))
            .await
            .unwrap();
        assert!(
            next_frame(&mut wa, Duration::from_millis(400))
                .await
                .is_none()
        );
        assert_eq!(relay.forwarded_bytes(), 0);
    }

    #[tokio::test]
    async fn a_key_outside_the_pair_cannot_sign_in() {
        let (url, _stun, _relay, _task) = spawn_local(Config::default()).await.unwrap();
        let (_ka, a) = keypair();
        let (_kb, b) = keypair();
        let (kc, c) = keypair();
        let ab = format!("{url}/{}", wire::pair_name(&a, &b));
        assert!(sign_in(&ab, &kc, &c).await.is_none());
    }

    /// Pings do not keep a connection that carries no datagrams.
    #[tokio::test]
    async fn pings_do_not_keep_an_idle_connection() {
        let config = Config {
            idle: Duration::from_millis(400),
            ..Config::default()
        };
        let (url, _stun, _relay, _task) = spawn_local(config).await.unwrap();
        let (ka, a) = keypair();
        let (_kb, b) = keypair();
        let mut wa = sign_in(&format!("{url}/{}", wire::pair_name(&a, &b)), &ka, &a)
            .await
            .unwrap();
        let start = Instant::now();
        let mut was_closed = false;
        while start.elapsed() < Duration::from_secs(3) {
            if wa.send(Message::Ping(Vec::new().into())).await.is_err() {
                was_closed = true;
                break;
            }
            if closed(&mut wa, Duration::from_millis(100)).await {
                was_closed = true;
                break;
            }
        }
        assert!(
            was_closed,
            "a ping-only connection must be closed when idle"
        );
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn budget_limits_the_rate() {
        let mut b = Budget::new(100_000);
        let start = Instant::now();
        for _ in 0..30 {
            b.spend(10_000).await;
        }
        // 300 KB at 100 KB/s with one second of burst: about two seconds.
        assert!(start.elapsed() >= Duration::from_millis(1_800));
    }

    /// #190: reconnecting on the same pair does not reset a key's byte limit,
    /// while the other key of the pair keeps its own allowance.
    #[tokio::test]
    async fn reconnecting_does_not_reset_the_byte_limit() {
        let config = Config {
            session_bytes: 2_000,
            ..Config::default()
        };
        let (url, _stun, _relay, _task) = spawn_local(config).await.unwrap();
        let (ka, a) = keypair();
        let (kb, b) = keypair();
        let ab = format!("{url}/{}", wire::pair_name(&a, &b));

        let mut wa = sign_in(&ab, &ka, &a).await.unwrap();
        let mut wb = sign_in(&ab, &kb, &b).await.unwrap();
        wa.send(Message::Binary(wire::frame(&b, &[1u8; 1_400]).into()))
            .await
            .unwrap();
        assert!(next_frame(&mut wb, Duration::from_secs(2)).await.is_some());
        wa.close(None).await.unwrap();
        assert!(closed(&mut wb, Duration::from_secs(2)).await);

        // Both reconnect on the same pair.
        let mut wa = sign_in(&ab, &ka, &a).await.unwrap();
        let mut wb = sign_in(&ab, &kb, &b).await.unwrap();

        // Key B has sent nothing yet, so its own 2 KB allowance still works.
        wb.send(Message::Binary(wire::frame(&a, &[2u8; 1_400]).into()))
            .await
            .unwrap();
        assert!(next_frame(&mut wa, Duration::from_secs(2)).await.is_some());

        // Key A already spent >1.4 KB; another 1 KB crosses 2 KB and ends the pair.
        let _ = wa
            .send(Message::Binary(wire::frame(&b, &[3u8; 1_000]).into()))
            .await;
        assert!(
            next_frame(&mut wb, Duration::from_millis(300))
                .await
                .is_none(),
            "frame past the pair's per-key byte limit must not be forwarded"
        );
        assert!(closed(&mut wa, Duration::from_secs(2)).await);
        assert!(closed(&mut wb, Duration::from_secs(2)).await);
        assert!(
            sign_in(&ab, &ka, &a).await.is_none(),
            "a pair over its byte limit must refuse further sign-ins until expiry"
        );
    }

    /// #190: reconnecting on the same pair does not reset the pair's session
    /// lifetime, and once expired the pair is forgotten.
    #[tokio::test]
    async fn reconnecting_does_not_reset_the_time_limit() {
        let config = Config {
            session_time: Duration::from_millis(900),
            ..Config::default()
        };
        let (url, _stun, relay, _task) = spawn_local(config).await.unwrap();
        let (ka, a) = keypair();
        let (_kb, b) = keypair();
        let ab = format!("{url}/{}", wire::pair_name(&a, &b));

        let mut wa = sign_in(&ab, &ka, &a).await.unwrap();
        tokio::time::sleep(Duration::from_millis(450)).await;
        wa.close(None).await.unwrap();

        // Reconnect roughly halfway through the 900 ms pair lifetime.
        let mut wa2 = sign_in(&ab, &ka, &a).await.unwrap();
        assert!(
            closed(&mut wa2, Duration::from_millis(700)).await,
            "second connection must end at the pair's original 900 ms deadline"
        );

        // Even with no new connections, the background sweep forgets expired pairs.
        let purge_by = Instant::now().checked_add(Duration::from_secs(2)).unwrap();
        while relay.pair_count().await > 0 && Instant::now() < purge_by {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert_eq!(relay.pair_count().await, 0, "expired pair was not swept");
    }

    /// #190: `Relay::serve` itself sweeps expired pairs even when idle.
    #[tokio::test]
    async fn serve_sweeps_expired_pairs_while_idle() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let relay = Relay::new(Config {
            session_time: Duration::from_millis(120),
            ..Config::default()
        })
        .unwrap();
        let serving = relay.clone();
        let _task = tokio::spawn(async move { serving.serve(listener).await });

        let (ka, a) = keypair();
        let (_kb, b) = keypair();
        let ab = format!("ws://{addr}/v0/relay/{}", wire::pair_name(&a, &b));
        let mut wa = sign_in(&ab, &ka, &a).await.unwrap();
        wa.close(None).await.unwrap();
        assert_eq!(relay.pair_count().await, 1);

        let purge_by = Instant::now().checked_add(Duration::from_secs(2)).unwrap();
        while relay.pair_count().await > 0 && Instant::now() < purge_by {
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
        assert_eq!(relay.pair_count().await, 0, "Relay::serve did not sweep");
    }

    /// #190: if a client replaces a lingering entry from an earlier generation
    /// of the same pair, the new generation's `live` count must still be `1`
    /// so `make_room` never evicts the active pair.
    #[tokio::test]
    async fn replacing_across_generation_rollover_keeps_pair_live() {
        let config = Config {
            max_pairs: 1,
            session_bytes: 2_000,
            session_time: Duration::from_millis(500),
            ..Config::default()
        };
        let (url, _stun, relay, _task) = spawn_local(config).await.unwrap();
        let (ka, a) = keypair();
        let (kb, b) = keypair();
        let pair = if a < b { (a, b) } else { (b, a) };
        let ptag = pair_tag(&relay.secret, &pair);

        // Seed a lingering generation-0 client record whose pair record has
        // already expired and been swept before `leave()` ran.
        {
            let (tx, _rx) = mpsc::channel(1);
            let mut state = relay.state.lock().await;
            state.next_gen = 1;
            state.clients.insert(
                a,
                Client {
                    id: 999,
                    pair,
                    pair_tag: ptag,
                    pair_gen: 0,
                    outbox: tx,
                    end: Arc::new(Notify::new()),
                },
            );
        }

        // Key A signs in now on generation 1, replacing the lingering client:
        // even before Key B connects, `live` on generation 1 must be 1.
        let ab = format!("{url}/{}", wire::pair_name(&a, &b));
        let mut wa = sign_in(&ab, &ka, &a).await.unwrap();
        assert_eq!(
            relay.state.lock().await.pairs.get(&ptag).map(|r| r.live),
            Some(1),
            "generation 1 must record live == 1 after replacing generation 0"
        );

        // Another pair tries to sign in while only Key A is live on (a, b):
        // because max_pairs is 1 and (a, b) is live, (c, d) is refused and
        // (a, b) is NOT evicted.
        let (kc, c) = keypair();
        let (_kd, d) = keypair();
        let cd = format!("{url}/{}", wire::pair_name(&c, &d));
        assert!(
            sign_in(&cd, &kc, &c).await.is_none(),
            "full live pair table must refuse new pairs instead of admitting untracked"
        );
        assert_eq!(
            relay
                .state
                .lock()
                .await
                .pairs
                .get(&ptag)
                .map(|r| r.generation),
            Some(1),
            "live pair was evicted across generation rollover"
        );

        let mut wb = sign_in(&ab, &kb, &b).await.unwrap();
        wa.send(Message::Binary(wire::frame(&b, &[1u8; 1_400]).into()))
            .await
            .unwrap();
        assert!(next_frame(&mut wb, Duration::from_secs(2)).await.is_some());
        let _ = wa
            .send(Message::Binary(wire::frame(&b, &[2u8; 1_000]).into()))
            .await;
        assert!(
            next_frame(&mut wb, Duration::from_millis(200))
                .await
                .is_none(),
            "live pair's byte counter was evicted across generation rollover"
        );
    }

    /// #190: a single-use key that signed in on one pair cannot hop to a
    /// second pair after disconnecting while its window is still active.
    #[tokio::test]
    async fn a_key_cannot_switch_to_another_pair_before_expiry() {
        let config = Config {
            session_time: Duration::from_millis(500),
            ..Config::default()
        };
        let (url, _stun, _relay, _task) = spawn_local(config).await.unwrap();
        let (ka, a) = keypair();
        let (_kb, b) = keypair();
        let (_kc, c) = keypair();
        let ab = format!("{url}/{}", wire::pair_name(&a, &b));
        let ac = format!("{url}/{}", wire::pair_name(&a, &c));

        let mut wa = sign_in(&ab, &ka, &a).await.unwrap();
        wa.close(None).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            sign_in(&ac, &ka, &a).await.is_none(),
            "key must stay bound to its first pair until the window expires"
        );
    }

    /// #190: the in-memory pair map never grows past `Config::max_pairs`, and
    /// eviction skips live pairs while evicting inactive ones in FIFO order.
    #[tokio::test]
    async fn pair_map_is_bounded_by_max_pairs() {
        let config = Config {
            max_pairs: 2,
            session_bytes: 2_000,
            ..Config::default()
        };
        let (url, _stun, relay, _task) = spawn_local(config).await.unwrap();

        // Pair 1 stays live and spends 1.4 KB.
        let (ka1, a1) = keypair();
        let (kb1, b1) = keypair();
        let ab1 = format!("{url}/{}", wire::pair_name(&a1, &b1));
        let mut wa1 = sign_in(&ab1, &ka1, &a1).await.unwrap();
        let mut wb1 = sign_in(&ab1, &kb1, &b1).await.unwrap();
        wa1.send(Message::Binary(wire::frame(&b1, &[1u8; 1_400]).into()))
            .await
            .unwrap();
        assert!(next_frame(&mut wb1, Duration::from_secs(2)).await.is_some());

        // Pairs 2, 3, 4 each sign in and disconnect; they churn the single
        // remaining inactive slot without touching live Pair 1.
        for _ in 0..3 {
            let (ka, a) = keypair();
            let (_kb, b) = keypair();
            let ab = format!("{url}/{}", wire::pair_name(&a, &b));
            let mut wa = sign_in(&ab, &ka, &a).await.unwrap();
            wa.close(None).await.unwrap();
            let deadline = Instant::now().checked_add(Duration::from_secs(1)).unwrap();
            while relay.client_count().await > 2 && Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
        assert_eq!(relay.pair_count().await, 2);

        // Live Pair 1 still enforces its 2 KB byte cap.
        let _ = wa1
            .send(Message::Binary(wire::frame(&b1, &[2u8; 1_000]).into()))
            .await;
        assert!(
            next_frame(&mut wb1, Duration::from_millis(200))
                .await
                .is_none()
        );
    }

    /// #190: repeated reconnect churn on a second pair while an earlier
    /// inactive pair sits at the front compacts `State::inactive` in place.
    #[test]
    fn inactive_queue_stays_bounded_under_single_pair_churn() {
        let now = Instant::now();
        let mut state = State::default();
        let p1: Tag = [1u8; 32];
        let p2: Tag = [2u8; 32];
        state.pairs.insert(
            p1,
            PairRecord {
                generation: 0,
                started: now,
                keys: ([10u8; 32], [11u8; 32]),
                sent: (0, 0),
                over: false,
                live: 0,
                inactive_stamp: 0,
            },
        );
        state.note_inactive(p1);
        state.pairs.insert(
            p2,
            PairRecord {
                generation: 1,
                started: now,
                keys: ([20u8; 32], [21u8; 32]),
                sent: (0, 0),
                over: false,
                live: 0,
                inactive_stamp: 0,
            },
        );
        for _ in 0..200 {
            if let Some(rec) = state.pairs.get_mut(&p2) {
                rec.live = 1;
                rec.inactive_stamp = 0;
            }
            if let Some(rec) = state.pairs.get_mut(&p2) {
                rec.live = 0;
            }
            state.note_inactive(p2);
        }
        assert!(
            state.inactive.len() <= 64,
            "inactive queue grew unbounded: {}",
            state.inactive.len()
        );
        assert_eq!(state.inactive.front().map(|e| e.0), Some(p1));
    }

    /// #190: churning many short-lived pairs with `make_room` compacts
    /// `State::order` in place so memory stays bounded by `max_pairs` even
    /// before `session_time` expires.
    #[test]
    fn order_queue_stays_bounded_under_pair_churn() {
        let now = Instant::now();
        let max_pairs = 2;
        let mut state = State::default();
        for i in 0u64..200 {
            state.make_room(max_pairs);
            let mut ptag = [0u8; 32];
            ptag[..8].copy_from_slice(&i.to_le_bytes());
            let ktags = (ptag, [i as u8; 32]);
            state.pairs.insert(
                ptag,
                PairRecord {
                    generation: i,
                    started: now,
                    keys: ktags,
                    sent: (0, 0),
                    over: false,
                    live: 0,
                    inactive_stamp: 0,
                },
            );
            state.order.push_back((ptag, i, now, ktags));
            state.note_inactive(ptag);
        }
        assert!(
            state.pairs.len() <= max_pairs,
            "pairs grew past max_pairs: {}",
            state.pairs.len()
        );
        assert!(
            state.order.len() <= 65,
            "order queue grew unbounded under churn: {}",
            state.order.len()
        );
    }

    /// #190: when `max_pairs == 0` or every pair slot holds a live pair, new
    /// sign-ins are refused rather than admitted untracked.
    #[tokio::test]
    async fn full_live_pair_table_and_zero_max_pairs_refuse_sign_in() {
        let (url0, _stun0, _relay0, _task0) = spawn_local(Config {
            max_pairs: 0,
            ..Config::default()
        })
        .await
        .unwrap();
        let (ka, a) = keypair();
        let (_kb, b) = keypair();
        let ab0 = format!("{url0}/{}", wire::pair_name(&a, &b));
        assert!(
            sign_in(&ab0, &ka, &a).await.is_none(),
            "max_pairs == 0 must refuse sign-in"
        );

        let (url, _stun, relay, _task) = spawn_local(Config {
            max_pairs: 1,
            ..Config::default()
        })
        .await
        .unwrap();
        let ab = format!("{url}/{}", wire::pair_name(&a, &b));
        let mut wa = sign_in(&ab, &ka, &a).await.unwrap();
        let (kc, c) = keypair();
        let (_kd, d) = keypair();
        let cd = format!("{url}/{}", wire::pair_name(&c, &d));
        assert!(
            sign_in(&cd, &kc, &c).await.is_none(),
            "new pair must be refused while all max_pairs slots are live"
        );
        // Once the live pair disconnects, its inactive slot can be evicted for a new pair.
        wa.close(None).await.unwrap();
        let deadline = Instant::now().checked_add(Duration::from_secs(1)).unwrap();
        while relay.client_count().await > 0 && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            sign_in(&cd, &kc, &c).await.is_some(),
            "new pair must succeed once an inactive slot can be evicted"
        );
    }
}
