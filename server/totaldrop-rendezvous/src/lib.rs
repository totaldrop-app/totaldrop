//! TotalDrop rendezvous server (spec section 5).
//!
//! Helps two devices that share a transfer code meet in a mailbox and pass
//! each other opaque messages. It never sees keys, file names or content:
//! after the two CPace messages everything it carries is encrypted end to end.
//!
//! Rules enforced here:
//! - a nameplate accepts exactly two claims; a third is refused, which is
//!   what limits an attacker to one guess per code;
//! - after a `scary` close the nameplate is retired for 24 hours;
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            unclaimed_ttl: Duration::from_secs(15 * 60),
            mailbox_ttl: Duration::from_secs(60 * 60),
            retire_for: Duration::from_secs(24 * 60 * 60),
            idle_timeout: Duration::from_secs(10 * 60),
            max_connections: 10_000,
            max_nameplates: 100_000,
            max_messages_per_side: 32,
            scary_threshold: 100,
            scary_window: Duration::from_secs(10 * 60),
            wave_delay: Duration::from_secs(2),
        }
    }
}

type ConnId = u64;
type Outbox = mpsc::Sender<ServerMessage>;

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
    allocated_by: ConnId,
    mailbox: [u8; 16],
    sides: Vec<Side>,
    /// Messages posted before the other side arrived, by poster.
    pending: Vec<(ConnId, String, Vec<u8>)>,
}

#[derive(Debug, Default)]
struct State {
    nameplates: HashMap<u32, Nameplate>,
    retired: HashMap<u32, Instant>,
    scary: VecDeque<Instant>,
    next_conn: ConnId,
}

impl State {
    fn is_retired(&self, n: u32, now: Instant, cfg: &Config) -> bool {
        self.retired
            .get(&n)
            .is_some_and(|t| now.saturating_duration_since(*t) < cfg.retire_for)
    }

    fn free_nameplate(&self, now: Instant, cfg: &Config) -> Option<u32> {
        (1u32..=u32::MAX)
            .take(cfg.max_nameplates.saturating_add(cfg.max_nameplates))
            .find(|n| !self.nameplates.contains_key(n) && !self.is_retired(*n, now, cfg))
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

    /// Remove expired nameplates and retirements. Returns the outboxes of
    /// clients whose mailbox disappeared, so they can be told.
    fn sweep(&mut self, now: Instant, cfg: &Config) -> Vec<Outbox> {
        let mut told = Vec::new();
        self.nameplates.retain(|_, np| {
            let age = now.saturating_duration_since(np.created);
            let expired = (np.sides.len() < 2 && age > cfg.unclaimed_ttl) || age > cfg.mailbox_ttl;
            if expired {
                told.extend(
                    np.sides
                        .iter()
                        .filter(|s| !s.closed)
                        .map(|s| s.outbox.clone()),
                );
            }
            !expired
        });
        self.retired
            .retain(|_, t| now.saturating_duration_since(*t) < cfg.retire_for);
        told
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
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            loop {
                tick.tick().await;
                let told = sweeper
                    .state
                    .lock()
                    .await
                    .sweep(Instant::now(), &sweeper.config);
                for outbox in told {
                    let _ = outbox
                        .send(ServerMessage::Error {
                            reason: "mailbox expired".into(),
                        })
                        .await;
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

    async fn handle(&self, stream: TcpStream) {
        let _ = stream.set_nodelay(true);
        let mut ws_config = WebSocketConfig::default();
        ws_config.max_message_size = Some(MAX_MAILBOX_BODY.saturating_add(1024));
        ws_config.max_frame_size = Some(MAX_MAILBOX_BODY.saturating_add(1024));
        let Ok(ws) = tokio_tungstenite::accept_async_with_config(stream, Some(ws_config)).await
        else {
            return;
        };
        let (mut sink, mut source) = ws.split();
        let (outbox, mut inbox) = mpsc::channel::<ServerMessage>(64);

        let conn = {
            let mut st = self.state.lock().await;
            st.next_conn = st.next_conn.wrapping_add(1);
            st.next_conn
        };

        // Writer: forwards queued server messages; an error ends the connection.
        let writer = tokio::spawn(async move {
            while let Some(msg) = inbox.recv().await {
                let fatal = matches!(msg, ServerMessage::Error { .. });
                if sink
                    .send(Message::Binary(msg.encode().into()))
                    .await
                    .is_err()
                {
                    break;
                }
                if fatal {
                    break;
                }
            }
            let _ = sink.close().await;
        });

        let mut held: Option<u32> = None;
        loop {
            let next = tokio::time::timeout(self.config.idle_timeout, source.next()).await;
            let frame = match next {
                Ok(Some(Ok(frame))) => frame,
                _ => break,
            };
            let data = match frame {
                Message::Binary(b) => b,
                Message::Ping(_) | Message::Pong(_) => continue,
                _ => break,
            };
            let reply = match ClientMessage::decode(&data) {
                Ok(msg) => self.on_message(conn, &outbox, &mut held, msg).await,
                Err(_) => Err("malformed message"),
            };
            if let Err(reason) = reply {
                let _ = outbox
                    .send(ServerMessage::Error {
                        reason: reason.into(),
                    })
                    .await;
                break;
            }
            if writer.is_finished() {
                break;
            }
        }
        // A client that disappears without CLOSE counts as a lonely close.
        if let Some(n) = held {
            self.close(conn, n, Mood::Lonely, false).await;
        }
        drop(outbox);
        let _ = writer.await;
    }

    async fn on_message(
        &self,
        conn: ConnId,
        outbox: &Outbox,
        held: &mut Option<u32>,
        msg: ClientMessage,
    ) -> Result<(), &'static str> {
        match msg {
            ClientMessage::Allocate => {
                if held.is_some() {
                    return Err("one nameplate per connection");
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
                st.nameplates.insert(
                    n,
                    Nameplate {
                        created: now,
                        allocated_by: conn,
                        mailbox,
                        sides: Vec::new(),
                        pending: Vec::new(),
                    },
                );
                *held = Some(n);
                drop(st);
                outbox
                    .send(ServerMessage::Allocated { nameplate: n })
                    .await
                    .map_err(|_| "closed")
            }
            ClientMessage::Claim { nameplate } => {
                if held.is_some_and(|h| h != nameplate) {
                    return Err("one nameplate per connection");
                }
                let mut st = self.state.lock().await;
                let np = st
                    .nameplates
                    .get_mut(&nameplate)
                    .ok_or("unknown nameplate")?;
                if np.sides.iter().any(|s| s.conn == conn) {
                    return Err("already claimed");
                }
                if np.sides.len() >= 2 {
                    // The rule that limits an attacker to one guess.
                    return Err("nameplate full");
                }
                if held.is_none() && np.sides.is_empty() && np.allocated_by != conn {
                    // The allocator always claims first; anyone else claiming
                    // an empty nameplate is guessing.
                    return Err("unknown nameplate");
                }
                np.sides.push(Side {
                    conn,
                    outbox: outbox.clone(),
                    posted: 0,
                    closed: false,
                });
                *held = Some(nameplate);
                let mailbox = np.mailbox;
                // Hand over anything the other side posted before we came.
                let mut deliver = Vec::new();
                np.pending.retain(|(from, phase, body)| {
                    if *from != conn {
                        deliver.push(ServerMessage::Message {
                            phase: phase.clone(),
                            body: body.clone(),
                        });
                        false
                    } else {
                        true
                    }
                });
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
                let n = held.ok_or("claim first")?;
                let mut st = self.state.lock().await;
                let np = st.nameplates.get_mut(&n).ok_or("mailbox expired")?;
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
                    None => np.pending.push((conn, phase, body)),
                }
                Ok(())
            }
            ClientMessage::Close { mood } => {
                let n = held.take().ok_or("claim first")?;
                self.close(conn, n, mood, true).await;
                Err("closed")
            }
        }
    }

    async fn close(&self, conn: ConnId, n: u32, mood: Mood, explicit: bool) {
        let mut st = self.state.lock().await;
        let now = Instant::now();
        let Some(np) = st.nameplates.get_mut(&n) else {
            return;
        };
        let mut notify = Vec::new();
        for side in &mut np.sides {
            if side.conn == conn {
                side.closed = true;
            } else if !side.closed {
                notify.push(side.outbox.clone());
            }
        }
        let allocator_only = np.sides.is_empty() && np.allocated_by == conn;
        let all_closed = allocator_only || np.sides.iter().all(|s| s.closed);
        let scary = explicit && mood == Mood::Scary;
        if all_closed || scary {
            // Contents are forgotten as soon as nobody needs them.
            np.pending.clear();
        }
        if all_closed {
            st.nameplates.remove(&n);
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
