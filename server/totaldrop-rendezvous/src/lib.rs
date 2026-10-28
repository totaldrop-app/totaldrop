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
//! - nothing is logged about clients: no addresses, no nameplates, no sizes.
//!
//! Serve it behind a TLS 1.3 terminator on port 443 (spec section 5).

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
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
            let Ok((stream, _addr)) = listener.accept().await else {
                continue;
            };
            let Ok(permit) = self.slots.clone().try_acquire_owned() else {
                // Over capacity: drop the connection without reading it.
                drop(stream);
                continue;
            };
            let server = self.clone();
            tokio::spawn(async move {
                server.handle(stream).await;
                drop(permit);
            });
        }
    }

    // The handshake callback's error type is fixed by tungstenite.
    #[allow(clippy::result_large_err)]
    async fn handle(&self, stream: TcpStream) {
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
        let Ok(Ok(ws)) = accepted else {
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
                if np.sides.is_empty() && token != Some(np.token) {
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
