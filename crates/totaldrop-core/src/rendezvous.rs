//! Client side of the rendezvous mailbox (spec section 5).

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};
use totaldrop_proto::wire::{ClientMessage, MAX_MAILBOX_BODY, Mood, ServerMessage};

use crate::error::{Error, Result};

/// The project's rendezvous server.
pub const DEFAULT_SERVER: &str = "wss://rendezvous.totaldrop.app";

/// Unencrypted `ws://` is accepted only for a server on this machine, for
/// development and tests. Everything else must be `wss://` (TLS 1.3).
fn check_url(url: &str) -> Result<()> {
    // Parse exactly as the WebSocket client will, so a URL such as
    // `ws://localhost@evil.example` cannot pass for a local one.
    let uri: tokio_tungstenite::tungstenite::http::Uri = url
        .parse()
        .map_err(|_| Error::Rendezvous("invalid server address".into()))?;
    let has_userinfo = uri.authority().is_some_and(|a| a.as_str().contains('@'));
    if has_userinfo {
        return Err(Error::Rendezvous(
            "the server address must not contain a user name".into(),
        ));
    }
    match uri.scheme_str() {
        Some("wss") => Ok(()),
        Some("ws") => {
            let host = uri.host().unwrap_or("");
            if matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "::1") {
                Ok(())
            } else {
                Err(Error::Rendezvous(
                    "unencrypted ws:// is only allowed for a server on this machine".into(),
                ))
            }
        }
        _ => Err(Error::Rendezvous(
            "the server address must start with wss://".into(),
        )),
    }
}

fn tls_config() -> Result<Arc<rustls::ClientConfig>> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| Error::Rendezvous(e.to_string()))?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// The rendezvous endpoint that hands out nameplates.
pub const ALLOCATE_PATH: &str = "/v0/allocate";

/// The rendezvous endpoint of one mailbox.
pub fn mailbox_path(nameplate: u32) -> String {
    format!("/v0/mailbox/{nameplate}")
}

/// How often a waiting client pings the server, so idle connections are not
/// dropped by proxies along the way.
const KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(20);

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

fn join(server: &str, path: &str) -> String {
    format!("{}{path}", server.trim_end_matches('/'))
}

async fn open(url: &str) -> Result<Ws> {
    check_url(url)?;
    let mut config = WebSocketConfig::default();
    config.max_message_size = Some(MAX_MAILBOX_BODY.saturating_add(1024));
    config.max_frame_size = Some(MAX_MAILBOX_BODY.saturating_add(1024));
    let connector = if url.starts_with("wss://") {
        Connector::Rustls(tls_config()?)
    } else {
        Connector::Plain
    };
    let (ws, _) =
        tokio_tungstenite::connect_async_tls_with_config(url, Some(config), true, Some(connector))
            .await
            .map_err(|e| Error::Rendezvous(e.to_string()))?;
    Ok(ws)
}

async fn send(ws: &mut Ws, msg: ClientMessage) -> Result<()> {
    ws.send(Message::Binary(msg.encode().into()))
        .await
        .map_err(|e| Error::Rendezvous(e.to_string()))
}

/// Wait for the next server message, pinging while idle.
async fn recv(ws: &mut Ws) -> Result<ServerMessage> {
    let mut ping = tokio::time::interval(KEEPALIVE);
    ping.tick().await;
    loop {
        tokio::select! {
            frame = ws.next() => match frame {
                Some(Ok(Message::Binary(b))) => {
                    let msg = ServerMessage::decode(&b)?;
                    if let ServerMessage::Error { reason } = msg {
                        return Err(Error::Rendezvous(reason));
                    }
                    return Ok(msg);
                }
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                Some(Ok(Message::Close(_))) | None => {
                    return Err(Error::Rendezvous("connection closed".into()));
                }
                Some(Ok(_)) => return Err(Error::Protocol("unexpected rendezvous frame")),
                Some(Err(e)) => return Err(Error::Rendezvous(e.to_string())),
            },
            _ = ping.tick() => {
                ws.send(Message::Ping(Vec::new().into()))
                    .await
                    .map_err(|e| Error::Rendezvous(e.to_string()))?;
            }
        }
    }
}

/// A reserved nameplate and the token that lets its holder claim it first.
#[derive(Clone, Copy)]
pub struct Allocation {
    /// The nameplate, the number at the start of the code.
    pub nameplate: u32,
    token: [u8; 16],
}

impl core::fmt::Debug for Allocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Allocation")
            .field("nameplate", &self.nameplate)
            .finish_non_exhaustive()
    }
}

/// `ALLOCATE`: reserve a nameplate on `server`.
pub async fn allocate(server: &str) -> Result<Allocation> {
    let mut ws = open(&join(server, ALLOCATE_PATH)).await?;
    send(&mut ws, ClientMessage::Allocate).await?;
    let reply = recv(&mut ws).await;
    let _ = ws.close(None).await;
    match reply? {
        ServerMessage::Allocated { nameplate, token } => Ok(Allocation { nameplate, token }),
        _ => Err(Error::Protocol("expected allocated")),
    }
}

/// A claimed mailbox on the rendezvous server.
pub struct Mailbox {
    ws: Ws,
    /// The mailbox ID (CPace session ID).
    pub id: [u8; 16],
}

impl core::fmt::Debug for Mailbox {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Mailbox(..)")
    }
}

impl Mailbox {
    /// Claim `nameplate` on `server`. The allocator passes its token; the
    /// other side passes `None`.
    pub async fn claim(server: &str, nameplate: u32, token: Option<&Allocation>) -> Result<Self> {
        let mut ws = open(&join(server, &mailbox_path(nameplate))).await?;
        send(
            &mut ws,
            ClientMessage::Claim {
                nameplate,
                token: token.map(|a| a.token),
            },
        )
        .await?;
        match recv(&mut ws).await? {
            ServerMessage::Claimed { mailbox } => Ok(Self { ws, id: mailbox }),
            _ => Err(Error::Protocol("expected claimed")),
        }
    }

    /// `ADD`: post a message to the other side.
    pub async fn add(&mut self, phase: &str, body: Vec<u8>) -> Result<()> {
        send(
            &mut self.ws,
            ClientMessage::Add {
                phase: phase.to_owned(),
                body,
            },
        )
        .await
    }

    /// Wait for the other side's next message, which must be of `phase`.
    pub async fn expect(&mut self, phase: &str) -> Result<Vec<u8>> {
        match recv(&mut self.ws).await? {
            ServerMessage::Message { phase: p, body } if p == phase => Ok(body),
            ServerMessage::Message { .. } => Err(Error::Protocol("unexpected mailbox phase")),
            ServerMessage::PeerClosed { mood: Mood::Scary } => Err(Error::WrongCode),
            ServerMessage::PeerClosed { .. } => Err(Error::PeerLeft),
            _ => Err(Error::Protocol("unexpected rendezvous message")),
        }
    }

    /// `CLOSE`: leave the mailbox.
    pub async fn close(mut self, mood: Mood) {
        let _ = send(&mut self.ws, ClientMessage::Close { mood }).await;
        let _ = self.ws.close(None).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_policy() {
        assert!(check_url("wss://rendezvous.totaldrop.app/v0").is_ok());
        assert!(check_url("ws://127.0.0.1:4000").is_ok());
        assert!(check_url("ws://localhost:4000/v0").is_ok());
        assert!(check_url("ws://[::1]:4000").is_ok());
        assert!(check_url("ws://example.com").is_err());
        assert!(check_url("ws://127.0.0.1.evil.com").is_err());
        assert!(check_url("http://127.0.0.1").is_err());
        assert!(check_url("ws://localhost:4000@evil.com/").is_err());
        assert!(check_url("ws://127.0.0.1@evil.com").is_err());
    }
}
