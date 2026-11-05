// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! QUIC with raw public keys (spec section 6).
//!
//! Every transfer uses a fresh, single-use Ed25519 endpoint key on each side.
//! TLS 1.3 inside QUIC presents these keys as raw public keys (RFC 7250), and
//! each side accepts only the exact key the other announced in its
//! authenticated `HINTS`. This keeps relays and observers from linking
//! sessions and from reading transport metadata; the source of trust is
//! still the Noise handshake that runs inside.

use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use ring::signature::KeyPair;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, WebPkiSupportedAlgorithms};
use rustls::pki_types::{
    CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, SubjectPublicKeyInfoDer,
    UnixTime,
};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::sign::CertifiedKey;
use rustls::{DigitallySignedStruct, DistinguishedName, SignatureScheme};

use crate::error::{Error, Result};

/// ALPN for protocol version 0.
pub const ALPN: &[u8] = b"totaldrop/0";

/// Name sent in TLS; every TotalDrop connection uses the same one, so it
/// says nothing about who is connecting.
const SERVER_NAME: &str = "totaldrop";

/// DER prefix of an Ed25519 SubjectPublicKeyInfo (RFC 8410).
const ED25519_SPKI_PREFIX: [u8; 12] = [
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

fn spki(public: &[u8; 32]) -> Vec<u8> {
    let mut der = ED25519_SPKI_PREFIX.to_vec();
    der.extend_from_slice(public);
    der
}

fn provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// A single-use Ed25519 key identifying this side's QUIC endpoint.
pub struct EndpointKey {
    pkcs8: Vec<u8>,
    public: [u8; 32],
}

impl core::fmt::Debug for EndpointKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("EndpointKey(..)")
    }
}

impl Drop for EndpointKey {
    fn drop(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.pkcs8);
    }
}

impl EndpointKey {
    /// Generate a fresh key.
    pub fn generate() -> Result<Self> {
        let rng = ring::rand::SystemRandom::new();
        let doc =
            ring::signature::Ed25519KeyPair::generate_pkcs8(&rng).map_err(|_| Error::Random)?;
        let pair =
            ring::signature::Ed25519KeyPair::from_pkcs8(doc.as_ref()).map_err(|_| Error::Random)?;
        let public: [u8; 32] = pair
            .public_key()
            .as_ref()
            .try_into()
            .map_err(|_| Error::Random)?;
        Ok(Self {
            pkcs8: doc.as_ref().to_vec(),
            public,
        })
    }

    /// The 32-byte public key announced in `HINTS`.
    pub fn public(&self) -> [u8; 32] {
        self.public
    }

    /// Sign `message`, to prove to a relay that this side holds the key.
    pub fn sign(&self, message: &[u8]) -> Result<[u8; 64]> {
        let pair =
            ring::signature::Ed25519KeyPair::from_pkcs8(&self.pkcs8).map_err(|_| Error::Random)?;
        pair.sign(message)
            .as_ref()
            .try_into()
            .map_err(|_| Error::Random)
    }

    fn certified(&self, provider: &CryptoProvider) -> Result<Arc<CertifiedKey>> {
        let der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.pkcs8.clone()));
        let signer = provider
            .key_provider
            .load_private_key(der)
            .map_err(|e| Error::Connect(e.to_string()))?;
        Ok(Arc::new(CertifiedKey::new(
            vec![CertificateDer::from(spki(&self.public))],
            signer,
        )))
    }
}

/// Accepts exactly one raw public key.
#[derive(Debug)]
struct Pinned {
    expected: Vec<u8>,
    algorithms: WebPkiSupportedAlgorithms,
}

impl Pinned {
    fn new(peer: &[u8; 32], provider: &CryptoProvider) -> Self {
        Self {
            expected: spki(peer),
            algorithms: provider.signature_verification_algorithms,
        }
    }

    fn check(&self, presented: &CertificateDer<'_>, others: &[CertificateDer<'_>]) -> bool {
        others.is_empty() && presented.as_ref() == self.expected.as_slice()
    }

    fn verify13(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> core::result::Result<HandshakeSignatureValid, rustls::Error> {
        if dss.scheme != SignatureScheme::ED25519 || cert.as_ref() != self.expected.as_slice() {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::BadSignature,
            ));
        }
        rustls::crypto::verify_tls13_signature_with_raw_key(
            message,
            &SubjectPublicKeyInfoDer::from(cert.as_ref()),
            dss,
            &self.algorithms,
        )
    }
}

fn wrong_key() -> rustls::Error {
    rustls::Error::InvalidCertificate(rustls::CertificateError::ApplicationVerificationFailure)
}

fn no_tls12() -> rustls::Error {
    rustls::Error::PeerIncompatible(rustls::PeerIncompatible::Tls12NotOffered)
}

impl ServerCertVerifier for Pinned {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> core::result::Result<ServerCertVerified, rustls::Error> {
        if self.check(end_entity, intermediates) {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(wrong_key())
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> core::result::Result<HandshakeSignatureValid, rustls::Error> {
        Err(no_tls12())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> core::result::Result<HandshakeSignatureValid, rustls::Error> {
        self.verify13(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }

    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

impl ClientCertVerifier for Pinned {
    fn offer_client_auth(&self) -> bool {
        true
    }

    fn client_auth_mandatory(&self) -> bool {
        true
    }

    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> core::result::Result<ClientCertVerified, rustls::Error> {
        if self.check(end_entity, intermediates) {
            Ok(ClientCertVerified::assertion())
        } else {
            Err(wrong_key())
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> core::result::Result<HandshakeSignatureValid, rustls::Error> {
        Err(no_tls12())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> core::result::Result<HandshakeSignatureValid, rustls::Error> {
        self.verify13(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }

    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

/// How much one stream may have in flight: 8 MiB.
const STREAM_WINDOW: u32 = 8 * 1024 * 1024;

/// How much one connection may have in flight, all streams together: 16 MiB.
pub(crate) const CONNECTION_WINDOW: u32 = 16 * 1024 * 1024;

fn transport_config() -> Arc<quinn::TransportConfig> {
    let mut t = quinn::TransportConfig::default();
    // One control stream; up to four data streams at a time (spec section 8),
    // with a little room for streams that are finishing.
    t.max_concurrent_bidi_streams(1u8.into());
    t.max_concurrent_uni_streams(8u8.into());
    // Room for several megabytes in flight, so a path with a long round
    // trip is not held to window / RTT: quinn's default of 1.25 MB per
    // stream allows one stream only about 4 MB/s at 300 ms over the relay
    // (#185). An honest client never has more than these windows in flight,
    // which also bounds what the relay holds for its connection; the
    // Cloudflare relay enforces a bound of its own with probes (#190).
    t.stream_receive_window(STREAM_WINDOW.into());
    t.receive_window(CONNECTION_WINDOW.into());
    t.send_window(u64::from(CONNECTION_WINDOW));
    // Often enough that a direct path that goes quiet stands out within
    // FALLBACK_SILENCE even with a few keep-alives lost (#185).
    t.keep_alive_interval(Some(Duration::from_secs(2)));
    if let Ok(idle) = quinn::IdleTimeout::try_from(Duration::from_secs(30)) {
        t.max_idle_timeout(Some(idle));
    }
    Arc::new(t)
}

/// A UDP socket bound before `HINTS` are sent, so its port can be announced.
pub fn bind_socket() -> Result<UdpSocket> {
    Ok(UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0)))?)
}

/// The addresses at which `socket` can likely be reached: loopback and the
/// address of the interface that holds the default route. Finding the latter
/// with an unconnected UDP `connect` sends no packet.
pub fn local_addresses(socket: &UdpSocket) -> Result<Vec<SocketAddr>> {
    let port = socket.local_addr()?.port();
    let mut out = vec![SocketAddr::from(([127, 0, 0, 1], port))];
    if let Ok(probe) = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0))) {
        // 192.0.2.1 is TEST-NET-1 (RFC 5737); nothing is sent to it.
        if probe.connect(SocketAddr::from(([192, 0, 2, 1], 9))).is_ok()
            && let Ok(addr) = probe.local_addr()
            && !addr.ip().is_unspecified()
            && !addr.ip().is_loopback()
        {
            out.push(SocketAddr::new(addr.ip(), port));
        }
    }
    Ok(out)
}

/// One QUIC endpoint per side that both listens and dials. Each direction
/// accepts only the peer's announced endpoint key, and each side presents its
/// own, so the connection is equally authenticated whichever side opened it.
/// A firewall in front of either device then still leaves one way through.
///
/// The endpoint can also reach the peer through a relay at
/// [`RELAY_PEER`](crate::path::RELAY_PEER); the returned pipes connect it to a
/// relay link (see [`crate::relay::spawn`]), and the returned
/// [`Fallback`](crate::path::Fallback) moves a direct connection onto it.
pub fn endpoint(
    socket: UdpSocket,
    own: &EndpointKey,
    peer: &[u8; 32],
) -> Result<(
    quinn::Endpoint,
    crate::path::RelayPipes,
    crate::path::Fallback,
)> {
    let runtime = quinn::default_runtime().ok_or(Error::Connect("no async runtime".into()))?;
    let inner = runtime.wrap_udp_socket(socket)?;
    endpoint_on(inner, own, peer)
}

/// [`endpoint`] on a socket that is already wrapped for quinn.
fn endpoint_on(
    inner: Arc<dyn quinn::AsyncUdpSocket>,
    own: &EndpointKey,
    peer: &[u8; 32],
) -> Result<(
    quinn::Endpoint,
    crate::path::RelayPipes,
    crate::path::Fallback,
)> {
    let provider = provider();

    let mut server_tls = rustls::ServerConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| Error::Connect(e.to_string()))?
        .with_client_cert_verifier(Arc::new(Pinned::new(peer, &provider)))
        .with_cert_resolver(Arc::new(
            rustls::server::AlwaysResolvesServerRawPublicKeys::new(own.certified(&provider)?),
        ));
    server_tls.alpn_protocols = vec![ALPN.to_vec()];
    let server_crypto =
        QuicServerConfig::try_from(server_tls).map_err(|e| Error::Connect(e.to_string()))?;
    let mut server = quinn::ServerConfig::with_crypto(Arc::new(server_crypto));
    server.transport_config(transport_config());

    let mut client_tls = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| Error::Connect(e.to_string()))?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(Pinned::new(peer, &provider)))
        .with_client_cert_resolver(Arc::new(
            rustls::client::AlwaysResolvesClientRawPublicKeys::new(own.certified(&provider)?),
        ));
    client_tls.alpn_protocols = vec![ALPN.to_vec()];
    // No session tickets or 0-RTT: every connection is fresh and unlinkable.
    client_tls.resumption = rustls::client::Resumption::disabled();
    let client_crypto =
        QuicClientConfig::try_from(client_tls).map_err(|e| Error::Connect(e.to_string()))?;
    let mut client = quinn::ClientConfig::new(Arc::new(client_crypto));
    client.transport_config(transport_config());

    let runtime = quinn::default_runtime().ok_or(Error::Connect("no async runtime".into()))?;
    let (socket, pipes, fallback) = crate::path::PathSocket::new(inner);
    let mut endpoint = quinn::Endpoint::new_with_abstract_socket(
        quinn::EndpointConfig::default(),
        Some(server),
        socket,
        runtime,
    )?;
    endpoint.set_default_client_config(client);
    Ok((endpoint, pipes, fallback))
}

/// The next incoming connection whose handshake completes; the pinned key
/// check inside TLS turns away anyone else. With `relay_only`, a connection
/// that did not come through the relay is closed too: in privacy mode this
/// side announced no address, and a peer that dials one anyway must not get
/// a direct path that confirms it.
async fn accept_one(endpoint: &quinn::Endpoint, relay_only: bool) -> Result<quinn::Connection> {
    loop {
        let incoming = endpoint
            .accept()
            .await
            .ok_or(Error::Connect(crate::error::msg::ENDPOINT_CLOSED.into()))?;
        if relay_only && incoming.remote_address() != crate::path::RELAY_PEER {
            incoming.refuse();
            continue;
        }
        if let Ok(conn) = incoming.await {
            return Ok(conn);
        }
    }
}

/// How long the side that normally waits holds back before it also dials,
/// so the usual direction wins when both work.
const REVERSE_DELAY: Duration = Duration::from_millis(400);

/// How long direct addresses get before the relay is contacted and tried as
/// well, so a working direct path wins and the relay never hears of it.
const RELAY_DELAY: Duration = Duration::from_millis(1200);

/// Where the other device can be reached.
#[derive(Debug, Clone, Default)]
pub struct Paths {
    /// Addresses from its `HINTS`, tried directly.
    pub direct: Vec<SocketAddr>,
    /// The relay link to it, if there is one. It is opened only when the
    /// relay attempt starts.
    pub relay: Option<crate::relay::Gate>,
    /// Privacy mode on this side: accept only connections through the relay.
    pub relay_only: bool,
}

impl Paths {
    fn is_empty(&self) -> bool {
        self.direct.is_empty() && self.relay.is_none()
    }
}

/// The receiver's connection: it dials the sender and, at the same time,
/// accepts the sender dialling it. The first connection that comes up is the
/// one; the receiver then opens the control stream on it, which tells the
/// sender which connection was chosen.
pub async fn receiver_connection(
    endpoint: &quinn::Endpoint,
    sender: &Paths,
    wait: Duration,
) -> Result<quinn::Connection> {
    let dial = async {
        match connect_any(endpoint, sender).await {
            Ok(conn) => Ok(conn),
            // Keep waiting for the other direction.
            Err(e) => {
                tokio::time::sleep(wait).await;
                Err(e)
            }
        }
    };
    let result = tokio::time::timeout(wait, async {
        tokio::select! {
            r = dial => r,
            r = accept_one(endpoint, sender.relay_only) => r,
        }
    })
    .await;
    match result {
        Ok(r) => r,
        Err(_) => Err(Error::Connect(crate::error::msg::NO_CONNECTION.into())),
    }
}

/// The sender's connection and the control stream the receiver opened on it.
/// The sender accepts the receiver dialling in and, a moment later, also
/// dials the receiver. Whichever connection carries the receiver's control
/// stream is the one; the others are closed.
pub async fn sender_connection(
    endpoint: &quinn::Endpoint,
    receiver: &Paths,
    wait: Duration,
) -> Result<(quinn::Connection, quinn::SendStream, quinn::RecvStream)> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<quinn::Connection>();
    let mut sources = tokio::task::JoinSet::new();
    let connected = Arc::new(std::sync::atomic::AtomicBool::new(false));
    // The relay link opens on the sender's own clock: at once with no direct
    // address, otherwise after RELAY_DELAY, and not at all once a
    // connection has come up and been chosen. The dial below only tries the
    // relay path.
    let mut dial = receiver.clone();
    if let Some(gate) = receiver.relay.clone() {
        let delay = if receiver.direct.is_empty() {
            Duration::ZERO
        } else {
            RELAY_DELAY
        };
        let connected = connected.clone();
        sources.spawn(async move {
            tokio::time::sleep(delay).await;
            // A connection came up but was not chosen yet: give it time, then
            // fall back to the relay if it still has not carried the control
            // stream. This task is aborted as soon as one is chosen.
            if connected.load(std::sync::atomic::Ordering::SeqCst) {
                tokio::time::sleep(RELAY_DELAY).await;
            }
            gate.open();
        });
        dial.relay = Some(crate::relay::Gate::default());
    }
    {
        let (endpoint, tx) = (endpoint.clone(), tx.clone());
        let relay_only = receiver.relay_only;
        sources.spawn(async move {
            while let Ok(conn) = accept_one(&endpoint, relay_only).await {
                if tx.send(conn).is_err() {
                    break;
                }
            }
        });
    }
    if !receiver.is_empty() {
        let endpoint = endpoint.clone();
        sources.spawn(async move {
            tokio::time::sleep(REVERSE_DELAY).await;
            if let Ok(conn) = connect_any(&endpoint, &dial).await {
                let _ = tx.send(conn);
            }
        });
    } else {
        drop(tx);
    }

    let mut waiting = tokio::task::JoinSet::new();
    let mut open: Vec<quinn::Connection> = Vec::new();
    let chosen = tokio::time::timeout(wait, async {
        loop {
            tokio::select! {
                Some(conn) = rx.recv() => {
                    connected.store(true, std::sync::atomic::Ordering::SeqCst);
                    open.push(conn.clone());
                    waiting.spawn(async move {
                        conn.accept_bi().await.map(|(s, r)| (conn, s, r))
                    });
                }
                Some(done) = waiting.join_next(), if !waiting.is_empty() => {
                    if let Ok(Ok(found)) = done {
                        return Some(found);
                    }
                }
                else => return None,
            }
        }
    })
    .await;
    sources.abort_all();
    waiting.abort_all();
    match chosen {
        Ok(Some((conn, send, recv))) => {
            for other in open.iter().filter(|c| c.stable_id() != conn.stable_id()) {
                other.close(0u32.into(), b"another path won");
            }
            Ok((conn, send, recv))
        }
        _ => Err(Error::Connect(
            crate::error::msg::RECEIVER_DID_NOT_CONNECT.into(),
        )),
    }
}

/// How long one handshake attempt may take.
const HANDSHAKE: Duration = Duration::from_secs(10);

/// How often a fresh handshake attempt starts on a path that has not
/// answered yet, while the earlier ones go on.
const ATTEMPT: Duration = Duration::from_secs(4);

/// Try every path at once, a little staggered and the relay last, and keep
/// the first connection whose handshake completes. Each path is tried again
/// and again until the caller gives up: QUIC waits longer and longer
/// between resending an unanswered handshake packet, so a path that starts
/// working late (a relay link that connects on its second try, a NAT
/// mapping that opens) would otherwise go unnoticed for many seconds
/// (#185). A fresh attempt sends its first packet at once.
pub async fn connect_any(endpoint: &quinn::Endpoint, paths: &Paths) -> Result<quinn::Connection> {
    if paths.is_empty() {
        return Err(Error::Connect(
            crate::error::msg::NO_ADDRESS_ANNOUNCED.into(),
        ));
    }
    let mut targets: Vec<(SocketAddr, Duration, Option<crate::relay::Gate>)> = paths
        .direct
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let i = u64::try_from(i).unwrap_or(0);
            (*a, Duration::from_millis(150u64.saturating_mul(i)), None)
        })
        .collect();
    if let Some(gate) = &paths.relay {
        let delay = if paths.direct.is_empty() {
            Duration::ZERO
        } else {
            RELAY_DELAY
        };
        targets.push((crate::path::RELAY_PEER, delay, Some(gate.clone())));
    }
    let mut attempts = tokio::task::JoinSet::new();
    for (addr, delay, gate) in targets {
        let endpoint = endpoint.clone();
        attempts.spawn(async move {
            tokio::time::sleep(delay).await;
            // Only now, with no direct connection yet, contact the relay.
            // Datagrams wait in the pipes until the link is up.
            if let Some(gate) = gate {
                gate.open();
            }
            keep_dialling(&endpoint, addr).await
        });
    }
    let mut last = Error::Connect(crate::error::msg::NO_ADDRESS_ANSWERED.into());
    while let Some(joined) = attempts.join_next().await {
        match joined {
            Ok(Ok(conn)) => {
                attempts.abort_all();
                return Ok(conn);
            }
            Ok(Err(e)) => last = e,
            Err(_) => {}
        }
    }
    Err(last)
}

/// Dial `addr`, starting a fresh attempt every [`ATTEMPT`] until one
/// completes. Returns an error only when the address cannot be dialled at
/// all, for example an IPv6 address on an IPv4 socket.
async fn keep_dialling(endpoint: &quinn::Endpoint, addr: SocketAddr) -> Result<quinn::Connection> {
    let mut tries = tokio::task::JoinSet::new();
    let mut next = tokio::time::Instant::now();
    loop {
        tokio::select! {
            () = tokio::time::sleep_until(next) => {
                let connecting = endpoint
                    .connect(addr, SERVER_NAME)
                    .map_err(|e| Error::Connect(e.to_string()))?;
                tries.spawn(async move {
                    tokio::time::timeout(HANDSHAKE, connecting).await.ok()?.ok()
                });
                next = next.checked_add(ATTEMPT).unwrap_or(next);
            }
            // Match inside: a pattern here that did not match would switch
            // this branch off until the next fresh try, and a later try's
            // success would wait for it.
            Some(done) = tries.join_next(), if !tries.is_empty() => {
                if let Ok(Some(conn)) = done {
                    return Ok(conn);
                }
            }
        }
    }
}

/// Connections for tests in other modules.
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
pub(crate) mod testing {
    use super::*;

    /// A sender's and a receiver's connection over a relay stand-in that
    /// carries the sender's datagrams one every `gap`: a slow link, where
    /// data sent stays on the way for a while (#193). The receiver's
    /// datagrams (acknowledgements) go through at once.
    pub(crate) async fn over_slow_relay(gap: Duration) -> (quinn::Connection, quinn::Connection) {
        let (ka, kb) = (
            EndpointKey::generate().unwrap(),
            EndpointKey::generate().unwrap(),
        );
        let sa = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let sb = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let (sender, pa, _) = endpoint(sa, &ka, &kb.public()).unwrap();
        let (receiver, pb, _) = endpoint(sb, &kb, &ka.public()).unwrap();
        let (mut a_out, a_in, mut b_out, b_in) =
            (pa.outgoing, pa.incoming, pb.outgoing, pb.incoming);
        tokio::spawn(async move {
            while let Some(d) = a_out.recv().await {
                tokio::time::sleep(gap).await;
                let _ = b_in.send(d).await;
            }
        });
        tokio::spawn(async move {
            while let Some(d) = b_out.recv().await {
                let _ = a_in.send(d).await;
            }
        });
        let gate = crate::relay::Gate::default();
        gate.open();
        let paths = Paths {
            direct: Vec::new(),
            relay: Some(gate),
            relay_only: true,
        };
        let wait = Duration::from_secs(10);
        let recv_side = async {
            let conn = receiver_connection(&receiver, &paths, wait).await.unwrap();
            let (mut send, _recv) = conn.open_bi().await.unwrap();
            send.write_all(b"x").await.unwrap();
            (conn, send)
        };
        let send_side = async { sender_connection(&sender, &paths, wait).await.unwrap() };
        let ((receiver_conn, _ctl), (sender_conn, _send, _recv)) =
            tokio::join!(recv_side, send_side);
        (sender_conn, receiver_conn)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn pair() -> (quinn::Endpoint, SocketAddr, quinn::Endpoint, SocketAddr) {
        let (ka, kb) = (
            EndpointKey::generate().unwrap(),
            EndpointKey::generate().unwrap(),
        );
        let sa = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let sb = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let (aa, ab) = (sa.local_addr().unwrap(), sb.local_addr().unwrap());
        let (a, _, _) = endpoint(sa, &ka, &kb.public()).unwrap();
        let (b, _, _) = endpoint(sb, &kb, &ka.public()).unwrap();
        (a, aa, b, ab)
    }

    /// A closed UDP port: what a firewall in front of the sender looks like.
    fn unreachable() -> SocketAddr {
        let s = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let addr = s.local_addr().unwrap();
        drop(s);
        addr
    }

    async fn run(sender_reachable: bool, receiver_reachable: bool) -> bool {
        let (sender, sender_addr, receiver, receiver_addr) = pair();
        let to_sender = if sender_reachable {
            sender_addr
        } else {
            unreachable()
        };
        let to_receiver = if receiver_reachable {
            receiver_addr
        } else {
            unreachable()
        };
        let wait = Duration::from_secs(8);
        let recv_side = async {
            let paths = Paths {
                direct: vec![to_sender],
                relay: None,
                relay_only: false,
            };
            let conn = receiver_connection(&receiver, &paths, wait).await?;
            let (mut send, _recv) = conn.open_bi().await?;
            send.write_all(b"hello")
                .await
                .map_err(|e| Error::Connect(e.to_string()))?;
            Ok::<_, Error>(conn)
        };
        let send_side = async {
            let paths = Paths {
                direct: vec![to_receiver],
                relay: None,
                relay_only: false,
            };
            let (_conn, _send, mut recv) = sender_connection(&sender, &paths, wait).await?;
            let mut buf = [0u8; 5];
            recv.read_exact(&mut buf)
                .await
                .map_err(|e| Error::Connect(e.to_string()))?;
            Ok::<_, Error>(buf)
        };
        // A firewall drops packets instead of refusing them; a closed port on
        // loopback refuses. Either way the dial to it fails.
        let (r, s) = tokio::join!(recv_side, send_side);
        matches!((r, s), (Ok(_), Ok(buf)) if &buf == b"hello")
    }

    #[tokio::test]
    async fn receiver_dials_sender() {
        assert!(run(true, false).await);
    }

    #[tokio::test]
    async fn sender_dials_receiver_when_sender_is_unreachable() {
        assert!(run(false, true).await);
    }

    #[tokio::test]
    async fn both_directions_open() {
        assert!(run(true, true).await);
    }

    /// Carry datagrams between two endpoints' relay pipes, as a relay would:
    /// only while both sides' links are open, dropping the rest, as the relay
    /// drops frames for a peer that is not there yet.
    fn cross(
        a: crate::path::RelayPipes,
        b: crate::path::RelayPipes,
        ga: crate::relay::Gate,
        gb: crate::relay::Gate,
    ) {
        cross_from(a, b, ga, gb, Duration::ZERO);
    }

    /// Like [`cross`], but everything sent in the first `after` is lost: a
    /// relay that is slow to get going, or one side's link that only gets
    /// through on a later try (#185).
    fn cross_from(
        a: crate::path::RelayPipes,
        b: crate::path::RelayPipes,
        ga: crate::relay::Gate,
        gb: crate::relay::Gate,
        after: Duration,
    ) {
        let ready = tokio::time::Instant::now().checked_add(after).unwrap();
        let (mut a_out, a_in, mut b_out, b_in) = (a.outgoing, a.incoming, b.outgoing, b.incoming);
        let (ga2, gb2) = (ga.clone(), gb.clone());
        tokio::spawn(async move {
            while let Some(d) = a_out.recv().await {
                if ga.is_open() && gb.is_open() && tokio::time::Instant::now() >= ready {
                    let _ = b_in.send(d).await;
                }
            }
        });
        tokio::spawn(async move {
            while let Some(d) = b_out.recv().await {
                if ga2.is_open() && gb2.is_open() && tokio::time::Instant::now() >= ready {
                    let _ = a_in.send(d).await;
                }
            }
        });
    }

    /// Connect over the relay stand-in, with `direct` as each side's direct
    /// addresses.
    async fn over_relay(direct: Vec<SocketAddr>) {
        over_late_relay(direct, Duration::ZERO, Duration::from_secs(10)).await;
    }

    /// Connect over a relay stand-in that loses everything for `after`,
    /// within `wait`.
    async fn over_late_relay(direct: Vec<SocketAddr>, after: Duration, wait: Duration) {
        let (ka, kb) = (
            EndpointKey::generate().unwrap(),
            EndpointKey::generate().unwrap(),
        );
        let sa = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let sb = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let (sender, pa, _) = endpoint(sa, &ka, &kb.public()).unwrap();
        let (receiver, pb, _) = endpoint(sb, &kb, &ka.public()).unwrap();
        let (send_gate, recv_gate) = (crate::relay::Gate::default(), crate::relay::Gate::default());
        cross_from(pa, pb, send_gate.clone(), recv_gate.clone(), after);
        let to_sender = Paths {
            direct: direct.clone(),
            relay: Some(recv_gate.clone()),
            relay_only: false,
        };
        let to_receiver = Paths {
            direct,
            relay: Some(send_gate.clone()),
            relay_only: false,
        };
        let recv_side = async {
            let conn = receiver_connection(&receiver, &to_sender, wait).await?;
            assert_eq!(conn.remote_address(), crate::path::RELAY_PEER);
            let (mut send, _recv) = conn.open_bi().await?;
            send.write_all(b"relay")
                .await
                .map_err(|e| Error::Connect(e.to_string()))?;
            Ok::<_, Error>(conn)
        };
        let send_side = async {
            let (conn, _send, mut recv) = sender_connection(&sender, &to_receiver, wait).await?;
            assert_eq!(conn.remote_address(), crate::path::RELAY_PEER);
            let mut buf = [0u8; 5];
            recv.read_exact(&mut buf)
                .await
                .map_err(|e| Error::Connect(e.to_string()))?;
            Ok::<_, Error>(buf)
        };
        let (r, s) = tokio::join!(recv_side, send_side);
        assert!(r.is_ok(), "receiver: {:?}", r.err());
        assert_eq!(&s.unwrap(), b"relay");
        assert!(send_gate.is_open() && recv_gate.is_open());
    }

    /// A privacy-mode side refuses a direct connection even from the
    /// paired device, which dials an address it learned some other way,
    /// and the transfer goes through the relay.
    #[tokio::test]
    async fn privacy_mode_refuses_a_direct_dial() {
        let (ka, kb) = (
            EndpointKey::generate().unwrap(),
            EndpointKey::generate().unwrap(),
        );
        let sa = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let sb = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let receiver_addr = sb.local_addr().unwrap();
        let (sender, pa, _) = endpoint(sa, &ka, &kb.public()).unwrap();
        let (receiver, pb, _) = endpoint(sb, &kb, &ka.public()).unwrap();
        let (send_gate, recv_gate) = (crate::relay::Gate::default(), crate::relay::Gate::default());
        cross(pa, pb, send_gate.clone(), recv_gate.clone());
        // The receiver is in privacy mode and announced nothing; the sender
        // dials its real address anyway.
        let to_sender = Paths {
            direct: Vec::new(),
            relay: Some(recv_gate.clone()),
            relay_only: true,
        };
        let to_receiver = Paths {
            direct: vec![receiver_addr],
            relay: Some(send_gate.clone()),
            relay_only: false,
        };
        let wait = Duration::from_secs(10);
        let recv_side = async {
            let conn = receiver_connection(&receiver, &to_sender, wait).await?;
            let (mut send, _recv) = conn.open_bi().await?;
            send.write_all(b"relay")
                .await
                .map_err(|e| Error::Connect(e.to_string()))?;
            Ok::<_, Error>(conn)
        };
        let send_side = async {
            let (conn, _send, mut recv) = sender_connection(&sender, &to_receiver, wait).await?;
            let mut buf = [0u8; 5];
            recv.read_exact(&mut buf)
                .await
                .map_err(|e| Error::Connect(e.to_string()))?;
            Ok::<_, Error>((conn, buf))
        };
        let (r, s) = tokio::join!(recv_side, send_side);
        assert_eq!(r.unwrap().remote_address(), crate::path::RELAY_PEER);
        let (conn, buf) = s.unwrap();
        assert_eq!(&buf, b"relay");
        assert_eq!(conn.remote_address(), crate::path::RELAY_PEER);
    }

    #[tokio::test]
    async fn relay_path_when_no_direct_path_works() {
        over_relay(vec![unreachable()]).await;
    }

    /// In privacy mode both sides contact the relay at once (#63): the
    /// connection over the relay comes up before the sender's own dial
    /// starts, so the sender's link did not wait for that dial.
    ///
    /// The clock is paused, so it moves only when every task is waiting on
    /// a timer. The marker below is a timer of REVERSE_DELAY, the one the
    /// sender's dial waits on, started at the same instant; it can fire only
    /// once nothing else can make progress, however slow the machine is
    /// (#126). Had the sender's link waited for its dial, the receiver's
    /// first packets would be dropped and nothing could connect before the
    /// marker.
    ///
    /// Stricter still, and independent of quinn's retransmission timers:
    /// no time may pass at all. Both links open at the first instant and
    /// every packet goes through the stand-in at once, so the connection is
    /// up before any timer fires; a dropped packet would have to wait for a
    /// retransmission, and a late link for its own timer.
    #[tokio::test(start_paused = true)]
    async fn privacy_mode_uses_the_relay_at_once() {
        let start = tokio::time::Instant::now();
        let connected_first = tokio::select! {
            biased;
            () = over_relay(Vec::new()) => true,
            () = tokio::time::sleep(REVERSE_DELAY) => false,
        };
        assert!(
            connected_first,
            "the relay link waited for the sender's own dial"
        );
        assert_eq!(start.elapsed(), Duration::ZERO, "a timer had to fire");
    }

    /// When a direct path works, neither side contacts the relay (#63).
    #[tokio::test]
    async fn direct_path_never_opens_the_relay() {
        let (sender, sender_addr, receiver, receiver_addr) = pair();
        let (recv_gate, send_gate) = (crate::relay::Gate::default(), crate::relay::Gate::default());
        let wait = Duration::from_secs(8);
        let recv_side = async {
            let paths = Paths {
                direct: vec![sender_addr],
                relay: Some(recv_gate.clone()),
                relay_only: false,
            };
            let conn = receiver_connection(&receiver, &paths, wait).await?;
            let (mut send, _recv) = conn.open_bi().await?;
            send.write_all(b"hello")
                .await
                .map_err(|e| Error::Connect(e.to_string()))?;
            Ok::<_, Error>(conn)
        };
        let send_side = async {
            let paths = Paths {
                direct: vec![receiver_addr],
                relay: Some(send_gate.clone()),
                relay_only: false,
            };
            let (conn, _send, mut recv) = sender_connection(&sender, &paths, wait).await?;
            let mut buf = [0u8; 5];
            recv.read_exact(&mut buf)
                .await
                .map_err(|e| Error::Connect(e.to_string()))?;
            Ok::<_, Error>((conn, buf))
        };
        let (r, s) = tokio::join!(recv_side, send_side);
        let (conn, buf) = s.unwrap();
        assert_ne!(r.unwrap().remote_address(), crate::path::RELAY_PEER);
        assert_ne!(conn.remote_address(), crate::path::RELAY_PEER);
        assert_eq!(&buf, b"hello");
        assert!(!recv_gate.is_open());
        assert!(!send_gate.is_open());
    }

    /// A relay path that only starts carrying packets after a while is
    /// still used, as long as it works before the receiver gives up (#185).
    /// QUIC waits longer and longer between resending a handshake packet
    /// that got no answer (about 1, 3, 7 and 15 seconds after the first),
    /// so a single attempt limited to HANDSHAKE (10 seconds) never resends
    /// after a path that starts working at 7.5 seconds; fresh attempts do.
    #[tokio::test(start_paused = true)]
    async fn a_relay_path_that_starts_working_late_is_still_used() {
        for after in [7_500, 12_000] {
            let start = tokio::time::Instant::now();
            over_late_relay(
                Vec::new(),
                Duration::from_millis(after),
                crate::CONNECT_TIMEOUT,
            )
            .await;
            assert!(start.elapsed() < crate::CONNECT_TIMEOUT);
        }
        over_late_relay(
            vec![unreachable()],
            Duration::from_millis(9_500),
            crate::CONNECT_TIMEOUT,
        )
        .await;
    }

    /// A stream may have several megabytes in flight, so a path with a long
    /// round trip, such as the relay, is not held to the window divided by
    /// the round trip: about 4 MB/s with quinn's default 1.25 MB at 300 ms
    /// (#185). The receiver here never reads, so the writer gets exactly as
    /// far as the receiver's flow-control window lets it.
    #[tokio::test]
    async fn a_stream_can_have_several_megabytes_in_flight() {
        let (a, _, b, b_addr) = pair();
        let accept = tokio::spawn(async move {
            let conn = b.accept().await.unwrap().await.unwrap();
            // Hold the connection open without reading anything.
            tokio::time::sleep(Duration::from_secs(30)).await;
            drop(conn);
        });
        let conn = a.connect(b_addr, SERVER_NAME).unwrap().await.unwrap();
        let mut send = conn.open_uni().await.unwrap();
        let chunk = vec![0u8; 64 * 1024];
        let mut written = 0usize;
        let goal = 6 * 1024 * 1024;
        while written < goal {
            match tokio::time::timeout(Duration::from_secs(2), send.write_all(&chunk)).await {
                Ok(Ok(())) => written += chunk.len(),
                _ => break,
            }
        }
        accept.abort();
        assert!(
            written >= goal,
            "only {written} bytes fit in the receiver's window"
        );
    }

    /// Which ways a [`Cuttable`] socket's traffic is cut off.
    #[derive(Debug, Default)]
    struct Cut {
        outgoing: std::sync::atomic::AtomicBool,
        incoming: std::sync::atomic::AtomicBool,
    }

    impl Cut {
        fn set(&self, outgoing: bool, incoming: bool) {
            self.outgoing
                .store(outgoing, std::sync::atomic::Ordering::SeqCst);
            self.incoming
                .store(incoming, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// A UDP socket whose traffic can be cut off while the connection over
    /// it stays up: a Wi-Fi that drops, a NAT mapping that expires, a
    /// network that lets the handshake through and then stops carrying
    /// anything.
    #[derive(Debug)]
    struct Cuttable {
        inner: Arc<dyn quinn::AsyncUdpSocket>,
        cut: Arc<Cut>,
    }

    impl quinn::AsyncUdpSocket for Cuttable {
        fn create_io_poller(self: Arc<Self>) -> std::pin::Pin<Box<dyn quinn::UdpPoller>> {
            self.inner.clone().create_io_poller()
        }

        fn try_send(&self, transmit: &quinn::udp::Transmit) -> std::io::Result<()> {
            if self.cut.outgoing.load(std::sync::atomic::Ordering::SeqCst) {
                Ok(())
            } else {
                self.inner.try_send(transmit)
            }
        }

        fn poll_recv(
            &self,
            cx: &mut std::task::Context,
            bufs: &mut [std::io::IoSliceMut<'_>],
            meta: &mut [quinn::udp::RecvMeta],
        ) -> std::task::Poll<std::io::Result<usize>> {
            loop {
                match self.inner.poll_recv(cx, bufs, meta) {
                    std::task::Poll::Ready(Ok(_))
                        if self.cut.incoming.load(std::sync::atomic::Ordering::SeqCst) => {}
                    other => return other,
                }
            }
        }

        fn local_addr(&self) -> std::io::Result<SocketAddr> {
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

    /// An endpoint on loopback whose direct traffic is cut as `cut` says.
    fn cuttable_endpoint(
        own: &EndpointKey,
        peer: &EndpointKey,
        cut: &Arc<Cut>,
    ) -> (
        quinn::Endpoint,
        SocketAddr,
        crate::path::RelayPipes,
        crate::path::Fallback,
    ) {
        let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let addr = socket.local_addr().unwrap();
        let inner = quinn::default_runtime()
            .unwrap()
            .wrap_udp_socket(socket)
            .unwrap();
        let wrapped = Arc::new(Cuttable {
            inner,
            cut: cut.clone(),
        });
        let (endpoint, pipes, fallback) = endpoint_on(wrapped, own, &peer.public()).unwrap();
        (endpoint, addr, pipes, fallback)
    }

    /// Two sides connected directly over cuttable sockets, with a relay
    /// stand-in between their pipes that is not contacted yet.
    struct Direct {
        sender: quinn::Connection,
        receiver: quinn::Connection,
        sender_fallback: crate::path::Fallback,
        receiver_fallback: crate::path::Fallback,
        send_gate: crate::relay::Gate,
        recv_gate: crate::relay::Gate,
        sender_cut: Arc<Cut>,
        receiver_cut: Arc<Cut>,
        _held: (quinn::Endpoint, quinn::Endpoint, quinn::SendStream),
    }

    async fn direct() -> Direct {
        let (ka, kb) = (
            EndpointKey::generate().unwrap(),
            EndpointKey::generate().unwrap(),
        );
        let (sender_cut, receiver_cut) = (Arc::new(Cut::default()), Arc::new(Cut::default()));
        let (sender, sender_addr, pa, sender_fallback) = cuttable_endpoint(&ka, &kb, &sender_cut);
        let (receiver, receiver_addr, pb, receiver_fallback) =
            cuttable_endpoint(&kb, &ka, &receiver_cut);
        let (send_gate, recv_gate) = (crate::relay::Gate::default(), crate::relay::Gate::default());
        cross(pa, pb, send_gate.clone(), recv_gate.clone());
        let wait = Duration::from_secs(8);
        let to_sender = Paths {
            direct: vec![sender_addr],
            relay: Some(recv_gate.clone()),
            relay_only: false,
        };
        let to_receiver = Paths {
            direct: vec![receiver_addr],
            relay: Some(send_gate.clone()),
            relay_only: false,
        };
        let (r, s) = tokio::join!(
            async {
                let conn = receiver_connection(&receiver, &to_sender, wait)
                    .await
                    .unwrap();
                let (mut ctl, _recv) = conn.open_bi().await.unwrap();
                ctl.write_all(b"x").await.unwrap();
                (conn, ctl)
            },
            sender_connection(&sender, &to_receiver, wait),
        );
        let (receiver_conn, ctl) = r;
        let (sender_conn, _send, _recv) = s.unwrap();
        assert_eq!(sender_conn.remote_address(), receiver_addr);
        assert_eq!(receiver_conn.remote_address(), sender_addr);
        assert!(!send_gate.is_open() && !recv_gate.is_open());
        Direct {
            sender: sender_conn,
            receiver: receiver_conn,
            sender_fallback,
            receiver_fallback,
            send_gate,
            recv_gate,
            sender_cut,
            receiver_cut,
            _held: (sender, receiver, ctl),
        }
    }

    impl Direct {
        /// A watch on each side, with `silence` as the wait.
        fn watch_both(&self, silence: Duration) -> (crate::path::Watch, crate::path::Watch) {
            (self.watch_sender(silence), self.watch(silence, false))
        }

        fn watch_sender(&self, silence: Duration) -> crate::path::Watch {
            self.watch(silence, true)
        }

        fn watch(&self, silence: Duration, sender: bool) -> crate::path::Watch {
            let (fallback, conn, gate) = if sender {
                (&self.sender_fallback, &self.sender, &self.send_gate)
            } else {
                (&self.receiver_fallback, &self.receiver, &self.recv_gate)
            };
            crate::path::Watch::spawn(fallback, conn.clone(), gate.clone(), silence)
        }
    }

    /// Send 4 MiB from the sender: the first half while the path works,
    /// then call `cut`, then the rest. True when everything arrived, intact,
    /// within `within`.
    async fn send_across(d: &Direct, cut: impl FnOnce(), within: Duration) -> bool {
        let data: Vec<u8> = (0..4 * 1024 * 1024u32).map(|i| (i % 251) as u8).collect();
        let (first, second) = data.split_at(data.len() / 2);
        let mut out = d.sender.open_uni().await.unwrap();
        out.write_all(first).await.unwrap();
        let mut incoming = d.receiver.accept_uni().await.unwrap();
        let mut got = vec![0u8; first.len()];
        incoming.read_exact(&mut got).await.unwrap();
        cut();
        let rest = second.to_vec();
        let write = tokio::spawn(async move {
            let _ = out.write_all(&rest).await;
            let _ = out.finish();
            out
        });
        let Ok(Ok(read)) = tokio::time::timeout(within, incoming.read_to_end(usize::MAX)).await
        else {
            return false;
        };
        got.extend_from_slice(&read);
        let _out = write.await;
        got == data
    }

    /// A direct connection whose path stops carrying anything after the
    /// handshake moves onto the relay, and the transfer goes on (#185).
    /// Before, nothing moved, and QUIC gave up after its 30-second idle
    /// timeout ("timed out") although the relay would have worked. Here the
    /// direct path dies both ways, as when a phone's Wi-Fi drops; the
    /// watch on each side notices on its own and the sides meet again on
    /// the relay.
    #[tokio::test]
    async fn a_direct_connection_that_goes_quiet_moves_to_the_relay() {
        let d = direct().await;
        let _watches = d.watch_both(Duration::from_secs(2));
        let addresses = (d.sender.remote_address(), d.receiver.remote_address());
        let arrived = send_across(
            &d,
            || {
                d.sender_cut.set(true, true);
                d.receiver_cut.set(true, true);
            },
            Duration::from_secs(25),
        )
        .await;
        assert!(
            arrived,
            "the transfer stalled once the direct path went quiet"
        );
        assert!(d.sender_fallback.moved() && d.receiver_fallback.moved());
        assert!(d.send_gate.is_open() && d.recv_gate.is_open());
        // QUIC still sees the direct address: nothing had to migrate.
        assert_eq!(
            (d.sender.remote_address(), d.receiver.remote_address()),
            addresses
        );
    }

    /// When only one direction dies, the side that stops hearing moves
    /// first, the other stops hearing it and follows, and the transfer
    /// goes on (#185 review).
    #[tokio::test]
    async fn a_direct_path_that_dies_one_way_moves_to_the_relay() {
        let d = direct().await;
        let _watches = d.watch_both(Duration::from_secs(2));
        // The sender's datagrams no longer reach the receiver.
        let arrived = send_across(
            &d,
            || d.sender_cut.set(true, false),
            Duration::from_secs(25),
        )
        .await;
        assert!(arrived, "the transfer stalled once one direction died");
        assert!(d.sender_fallback.moved() && d.receiver_fallback.moved());
    }

    /// A side that moved comes back as soon as the direct path carries
    /// datagrams again, so a side whose peer did not move (no shared relay,
    /// or a link closed during setup, or a peer that was only paused) does
    /// not lose a direct path that recovers (#185 review). Here only the
    /// sender watches; the path is gone for longer than its wait, then
    /// comes back.
    #[tokio::test]
    async fn a_side_that_moved_comes_back_when_the_direct_path_returns() {
        let d = direct().await;
        let _watch = d.watch_sender(Duration::from_secs(2));
        let (sender_cut, receiver_cut) = (d.sender_cut.clone(), d.receiver_cut.clone());
        let fallback = d.sender_fallback.clone();
        let healed = tokio::spawn(async move {
            // The cut below lands at once; the wait is 2 seconds.
            tokio::time::sleep(Duration::from_secs(5)).await;
            let moved = fallback.moved();
            sender_cut.set(false, false);
            receiver_cut.set(false, false);
            moved
        });
        let arrived = send_across(
            &d,
            || {
                d.sender_cut.set(true, true);
                d.receiver_cut.set(true, true);
            },
            Duration::from_secs(25),
        )
        .await;
        assert!(healed.await.unwrap(), "the sender did not move");
        assert!(
            arrived,
            "the transfer did not survive a direct path that came back"
        );
        assert!(!d.sender_fallback.moved(), "the sender stayed on the relay");
    }

    /// An idle direct connection stays direct: QUIC's keep-alives count as
    /// hearing from the other side, so a pause in the transfer (a person
    /// reading the offer, say) does not move it onto the relay (#185).
    #[tokio::test]
    async fn an_idle_direct_connection_stays_direct() {
        let d = direct().await;
        // Longer than one keep-alive interval, much shorter than the wait.
        let silence = Duration::from_secs(4);
        let _watch = d.watch_sender(silence);
        tokio::time::sleep(silence.saturating_mul(2)).await;
        assert!(
            !d.sender_fallback.moved(),
            "an idle connection moved to the relay"
        );
        assert!(!d.send_gate.is_open());
        assert!(d.sender.close_reason().is_none());
    }

    /// A try that ends without a connection does not hold up a later one
    /// that succeeds: its success is used at once, not at the next fresh
    /// try (#185 review). The path starts working just after the first try
    /// gives up at HANDSHAKE.
    #[tokio::test(start_paused = true)]
    async fn a_failed_try_does_not_delay_a_later_one() {
        let (ka, kb) = (
            EndpointKey::generate().unwrap(),
            EndpointKey::generate().unwrap(),
        );
        let sa = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let sb = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let (dialler, pa, _) = endpoint(sa, &ka, &kb.public()).unwrap();
        let (listener, pb, _) = endpoint(sb, &kb, &ka.public()).unwrap();
        let (ga, gb) = (crate::relay::Gate::default(), crate::relay::Gate::default());
        gb.open();
        let after = HANDSHAKE.checked_add(Duration::from_millis(200)).unwrap();
        cross_from(pa, pb, ga.clone(), gb, after);
        let accept = tokio::spawn(async move {
            let conn = listener.accept().await.unwrap().await.unwrap();
            tokio::time::sleep(Duration::from_secs(60)).await;
            drop(conn);
        });
        let paths = Paths {
            direct: Vec::new(),
            relay: Some(ga),
            relay_only: true,
        };
        let start = tokio::time::Instant::now();
        let conn = connect_any(&dialler, &paths).await.unwrap();
        let took = start.elapsed();
        accept.abort();
        assert_eq!(conn.remote_address(), crate::path::RELAY_PEER);
        // Fresh tries start at 0, 4, 8 and 12 seconds.
        assert!(
            took < ATTEMPT.saturating_mul(3),
            "connected only after {took:?}"
        );
    }
}
