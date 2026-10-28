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

fn transport_config() -> Arc<quinn::TransportConfig> {
    let mut t = quinn::TransportConfig::default();
    // One control stream; up to four data streams at a time (spec section 8),
    // with a little room for streams that are finishing.
    t.max_concurrent_bidi_streams(1u8.into());
    t.max_concurrent_uni_streams(8u8.into());
    t.keep_alive_interval(Some(Duration::from_secs(5)));
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
/// relay link (see [`crate::relay::spawn`]).
pub fn endpoint(
    socket: UdpSocket,
    own: &EndpointKey,
    peer: &[u8; 32],
) -> Result<(quinn::Endpoint, crate::path::RelayPipes)> {
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
    let (socket, pipes) = crate::path::PathSocket::new(runtime.wrap_udp_socket(socket)?);
    let mut endpoint = quinn::Endpoint::new_with_abstract_socket(
        quinn::EndpointConfig::default(),
        Some(server),
        socket,
        runtime,
    )?;
    endpoint.set_default_client_config(client);
    Ok((endpoint, pipes))
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
            .ok_or(Error::Connect("endpoint closed".into()))?;
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
        Err(_) => Err(Error::Connect("no connection to the other device".into())),
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
        _ => Err(Error::Connect("the receiver did not connect".into())),
    }
}

/// Try every path at once, a little staggered and the relay last, and keep
/// the first connection whose handshake completes.
pub async fn connect_any(endpoint: &quinn::Endpoint, paths: &Paths) -> Result<quinn::Connection> {
    if paths.is_empty() {
        return Err(Error::Connect("the other side announced no address".into()));
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
            let connecting = endpoint
                .connect(addr, SERVER_NAME)
                .map_err(|e| Error::Connect(e.to_string()))?;
            tokio::time::timeout(Duration::from_secs(10), connecting)
                .await
                .map_err(|_| Error::Connect("timed out".into()))?
                .map_err(Error::from)
        });
    }
    let mut last = Error::Connect("no address answered".into());
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
        let (a, _) = endpoint(sa, &ka, &kb.public()).unwrap();
        let (b, _) = endpoint(sb, &kb, &ka.public()).unwrap();
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
        let (mut a_out, a_in, mut b_out, b_in) = (a.outgoing, a.incoming, b.outgoing, b.incoming);
        let (ga2, gb2) = (ga.clone(), gb.clone());
        tokio::spawn(async move {
            while let Some(d) = a_out.recv().await {
                if ga.is_open() && gb.is_open() {
                    let _ = b_in.send(d).await;
                }
            }
        });
        tokio::spawn(async move {
            while let Some(d) = b_out.recv().await {
                if ga2.is_open() && gb2.is_open() {
                    let _ = a_in.send(d).await;
                }
            }
        });
    }

    /// Connect over the relay stand-in, with `direct` as each side's direct
    /// addresses, and return how long it took.
    async fn over_relay(direct: Vec<SocketAddr>) -> Duration {
        let (ka, kb) = (
            EndpointKey::generate().unwrap(),
            EndpointKey::generate().unwrap(),
        );
        let sa = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let sb = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        let (sender, pa) = endpoint(sa, &ka, &kb.public()).unwrap();
        let (receiver, pb) = endpoint(sb, &kb, &ka.public()).unwrap();
        let (send_gate, recv_gate) = (crate::relay::Gate::default(), crate::relay::Gate::default());
        cross(pa, pb, send_gate.clone(), recv_gate.clone());
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
        let wait = Duration::from_secs(10);
        let start = tokio::time::Instant::now();
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
        assert!(r.is_ok());
        assert_eq!(&s.unwrap(), b"relay");
        assert!(send_gate.is_open() && recv_gate.is_open());
        start.elapsed()
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
        let (sender, pa) = endpoint(sa, &ka, &kb.public()).unwrap();
        let (receiver, pb) = endpoint(sb, &kb, &ka.public()).unwrap();
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

    /// In privacy mode both sides contact the relay at once (#63).
    #[tokio::test]
    async fn privacy_mode_uses_the_relay_at_once() {
        // Before the sender's own dial even starts: its link did not wait for it.
        assert!(over_relay(Vec::new()).await < REVERSE_DELAY);
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
}
