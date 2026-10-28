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
//! traffic. Each connection is capped in rate, total bytes and lifetime.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{Mutex, Semaphore, mpsc};
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
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
    /// Bytes per second one device may send through the relay.
    pub rate: u64,
    /// Bytes one connection may send in total.
    pub session_bytes: u64,
    /// How long one connection may last.
    pub session_time: Duration,
    /// A connection with no traffic for this long is closed.
    pub idle: Duration,
    /// Datagrams queued for one device before more are dropped.
    pub queue: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_clients: 2_000,
            rate: 8 * 1024 * 1024,
            session_bytes: 32 * 1024 * 1024 * 1024,
            session_time: Duration::from_secs(6 * 60 * 60),
            idle: Duration::from_secs(60),
            queue: 1_024,
        }
    }
}

type Outbox = mpsc::Sender<Vec<u8>>;

/// Signed-in devices: endpoint key, connection number, outbox.
type Clients = HashMap<[u8; 32], (u64, Outbox)>;

/// A running relay.
#[derive(Debug, Clone)]
pub struct Relay {
    clients: Arc<Mutex<Clients>>,
    next: Arc<std::sync::atomic::AtomicU64>,
    accepted: Arc<std::sync::atomic::AtomicU64>,
    forwarded: Arc<std::sync::atomic::AtomicU64>,
    slots: Arc<Semaphore>,
    config: Arc<Config>,
}

impl Relay {
    /// A relay with the given limits.
    pub fn new(config: Config) -> Self {
        Self {
            clients: Arc::default(),
            next: Arc::default(),
            accepted: Arc::default(),
            forwarded: Arc::default(),
            slots: Arc::new(Semaphore::new(config.max_clients)),
            config: Arc::new(config),
        }
    }

    /// Devices signed in right now, for tests and health checks.
    pub async fn client_count(&self) -> usize {
        self.clients.lock().await.len()
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

    /// Accept WebSocket connections on `listener` until the task is dropped.
    pub async fn serve(self, listener: TcpListener) {
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
            Ok(resp)
        };
        let Ok(Ok(mut ws)) = tokio::time::timeout(
            IO_TIMEOUT,
            tokio_tungstenite::accept_hdr_async_with_config(stream, callback, Some(ws_config)),
        )
        .await
        else {
            return;
        };
        // `/v0/relay/<pair>` names the only two keys that may sign in here;
        // plain `/v0/relay` takes any key.
        let pair = match path.strip_prefix(PATH) {
            Some("") => None,
            Some(rest) => match rest.strip_prefix('/').and_then(wire::parse_pair_name) {
                Some(p) => Some(p),
                None => return,
            },
            None => return,
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
            if pair.is_some_and(|(a, b)| key != a && key != b) {
                return None;
            }
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, key)
                .verify(&wire::signed_bytes(&challenge), &signature)
                .ok()?;
            Some(key)
        })
        .await;
        let Ok(Some(key)) = key else { return };

        let (outbox, mut inbox) = mpsc::channel::<Vec<u8>>(self.config.queue);
        let id = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        {
            let mut clients = self.clients.lock().await;
            // One connection per key: keys are single-use, so a second one is
            // either a bug or someone replaying a sign-in.
            if clients.contains_key(&key) {
                return;
            }
            clients.insert(key, (id, outbox));
        }
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

        let started = Instant::now();
        let mut budget = Budget::new(self.config.rate);
        let mut total: u64 = 0;
        loop {
            if started.elapsed() > self.config.session_time || writer.is_finished() {
                break;
            }
            let next = match tokio::time::timeout(self.config.idle, source.next()).await {
                Ok(Some(Ok(m))) => m,
                _ => break,
            };
            let frame = match next {
                Message::Binary(b) => b,
                Message::Ping(_) | Message::Pong(_) => continue,
                _ => break,
            };
            let Some((to, datagrams)) = wire::parse_frame(&frame) else {
                break;
            };
            let payload: usize = datagrams.iter().map(|d| d.len()).sum();
            let size = u64::try_from(frame.len()).unwrap_or(u64::MAX);
            total = total.saturating_add(size);
            if total > self.config.session_bytes {
                break;
            }
            // Slow the sender down instead of dropping: TCP pushes back.
            budget.spend(size).await;
            let target = self.clients.lock().await.get(&to).map(|(_, o)| o.clone());
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

    async fn leave(&self, key: &[u8; 32], id: u64) {
        let mut clients = self.clients.lock().await;
        if clients.get(key).is_some_and(|(i, _)| *i == id) {
            clients.remove(key);
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
    let relay = Relay::new(config);
    let serving = relay.clone();
    let task = tokio::spawn(async move {
        tokio::join!(serving.serve(listener), Relay::discovery(socket));
    });
    Ok((ws, udp, relay, task))
}

/// For tests: forwarding on `ws://127.0.0.1:<port>/v0/relay` and STUN on UDP
/// `127.0.0.1:<port>`. Returns both, the relay and its task.
pub async fn spawn_local(
    config: Config,
) -> std::io::Result<(String, String, Relay, tokio::task::JoinHandle<()>)> {
    // Find a port free for both TCP and UDP.
    for _ in 0..20 {
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).await?;
        let port = listener.local_addr()?.port();
        let Ok(socket) = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], port))).await else {
            continue;
        };
        let relay = Relay::new(config);
        let serving = relay.clone();
        let task = tokio::spawn(async move {
            tokio::join!(serving.serve(listener), Relay::discovery(socket));
        });
        return Ok((
            format!("ws://127.0.0.1:{port}{PATH}"),
            format!("127.0.0.1:{port}"),
            relay,
            task,
        ));
    }
    Err(std::io::Error::other("no free port"))
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
}
