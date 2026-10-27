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
pub const DEFAULT_SERVER: &str = "wss://rendezvous.totaldrop.app/v0";

/// Unencrypted `ws://` is accepted only for a server on this machine, for
/// development and tests. Everything else must be `wss://` (TLS 1.3).
fn check_url(url: &str) -> Result<()> {
    if url.starts_with("wss://") {
        return Ok(());
    }
    if let Some(rest) = url.strip_prefix("ws://") {
        let host_port = rest.split('/').next().unwrap_or("");
        let host = if let Some(v6) = host_port.strip_prefix('[') {
            v6.split(']').next().unwrap_or("")
        } else {
            host_port.split(':').next().unwrap_or("")
        };
        if matches!(host, "localhost" | "127.0.0.1" | "::1") {
            return Ok(());
        }
        return Err(Error::Rendezvous(
            "unencrypted ws:// is only allowed for a server on this machine".into(),
        ));
    }
    Err(Error::Rendezvous(
        "the server address must start with wss://".into(),
    ))
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

/// A connection to the rendezvous server.
pub struct Mailbox {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl core::fmt::Debug for Mailbox {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Mailbox(..)")
    }
}

impl Mailbox {
    /// Connect to the server at `url`.
    pub async fn connect(url: &str) -> Result<Self> {
        check_url(url)?;
        let mut config = WebSocketConfig::default();
        config.max_message_size = Some(MAX_MAILBOX_BODY.saturating_add(1024));
        config.max_frame_size = Some(MAX_MAILBOX_BODY.saturating_add(1024));
        let connector = if url.starts_with("wss://") {
            Connector::Rustls(tls_config()?)
        } else {
            Connector::Plain
        };
        let (ws, _) = tokio_tungstenite::connect_async_tls_with_config(
            url,
            Some(config),
            true,
            Some(connector),
        )
        .await
        .map_err(|e| Error::Rendezvous(e.to_string()))?;
        Ok(Self { ws })
    }

    async fn send(&mut self, msg: ClientMessage) -> Result<()> {
        self.ws
            .send(Message::Binary(msg.encode().into()))
            .await
            .map_err(|e| Error::Rendezvous(e.to_string()))
    }

    async fn recv(&mut self) -> Result<ServerMessage> {
        loop {
            match self.ws.next().await {
                Some(Ok(Message::Binary(b))) => {
                    let msg = ServerMessage::decode(&b)?;
                    if let ServerMessage::Error { reason } = msg {
                        return Err(Error::Rendezvous(reason));
                    }
                    return Ok(msg);
                }
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => continue,
                Some(Ok(_)) => return Err(Error::Protocol("unexpected rendezvous frame")),
                Some(Err(e)) => return Err(Error::Rendezvous(e.to_string())),
                None => return Err(Error::Rendezvous("connection closed".into())),
            }
        }
    }

    /// `ALLOCATE`: reserve a nameplate.
    pub async fn allocate(&mut self) -> Result<u32> {
        self.send(ClientMessage::Allocate).await?;
        match self.recv().await? {
            ServerMessage::Allocated { nameplate } => Ok(nameplate),
            _ => Err(Error::Protocol("expected allocated")),
        }
    }

    /// `CLAIM`: join the mailbox; returns the mailbox ID.
    pub async fn claim(&mut self, nameplate: u32) -> Result<[u8; 16]> {
        self.send(ClientMessage::Claim { nameplate }).await?;
        match self.recv().await? {
            ServerMessage::Claimed { mailbox } => Ok(mailbox),
            // Our own messages are never echoed; nothing else may come first.
            _ => Err(Error::Protocol("expected claimed")),
        }
    }

    /// `ADD`: post a message to the other side.
    pub async fn add(&mut self, phase: &str, body: Vec<u8>) -> Result<()> {
        self.send(ClientMessage::Add {
            phase: phase.to_owned(),
            body,
        })
        .await
    }

    /// Wait for the other side's next message, which must be of `phase`.
    pub async fn expect(&mut self, phase: &str) -> Result<Vec<u8>> {
        match self.recv().await? {
            ServerMessage::Message { phase: p, body } if p == phase => Ok(body),
            ServerMessage::Message { .. } => Err(Error::Protocol("unexpected mailbox phase")),
            ServerMessage::PeerClosed { mood: Mood::Scary } => Err(Error::WrongCode),
            ServerMessage::PeerClosed { .. } => Err(Error::PeerLeft),
            _ => Err(Error::Protocol("unexpected rendezvous message")),
        }
    }

    /// `CLOSE`: leave the mailbox.
    pub async fn close(mut self, mood: Mood) {
        let _ = self.send(ClientMessage::Close { mood }).await;
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
    }
}
