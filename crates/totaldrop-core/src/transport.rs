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

/// The sending side's QUIC endpoint: a server that accepts only the
/// receiver's announced endpoint key.
pub fn server_endpoint(
    socket: UdpSocket,
    own: &EndpointKey,
    peer: &[u8; 32],
) -> Result<quinn::Endpoint> {
    let provider = provider();
    let mut tls = rustls::ServerConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| Error::Connect(e.to_string()))?
        .with_client_cert_verifier(Arc::new(Pinned::new(peer, &provider)))
        .with_cert_resolver(Arc::new(
            rustls::server::AlwaysResolvesServerRawPublicKeys::new(own.certified(&provider)?),
        ));
    tls.alpn_protocols = vec![ALPN.to_vec()];
    let crypto = QuicServerConfig::try_from(tls).map_err(|e| Error::Connect(e.to_string()))?;
    let mut config = quinn::ServerConfig::with_crypto(Arc::new(crypto));
    config.transport_config(transport_config());
    let runtime = quinn::default_runtime().ok_or(Error::Connect("no async runtime".into()))?;
    Ok(quinn::Endpoint::new(
        quinn::EndpointConfig::default(),
        Some(config),
        socket,
        runtime,
    )?)
}

/// The receiving side's QUIC client configuration: it accepts only the
/// sender's announced endpoint key and presents its own.
pub fn client_endpoint(
    socket: UdpSocket,
    own: &EndpointKey,
    peer: &[u8; 32],
) -> Result<quinn::Endpoint> {
    let provider = provider();
    let mut tls = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| Error::Connect(e.to_string()))?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(Pinned::new(peer, &provider)))
        .with_client_cert_resolver(Arc::new(
            rustls::client::AlwaysResolvesClientRawPublicKeys::new(own.certified(&provider)?),
        ));
    tls.alpn_protocols = vec![ALPN.to_vec()];
    // No session tickets or 0-RTT: every connection is fresh and unlinkable.
    tls.resumption = rustls::client::Resumption::disabled();
    let crypto = QuicClientConfig::try_from(tls).map_err(|e| Error::Connect(e.to_string()))?;
    let mut config = quinn::ClientConfig::new(Arc::new(crypto));
    config.transport_config(transport_config());
    let runtime = quinn::default_runtime().ok_or(Error::Connect("no async runtime".into()))?;
    let mut endpoint =
        quinn::Endpoint::new(quinn::EndpointConfig::default(), None, socket, runtime)?;
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}

/// Try every address at once, a little staggered, and keep the first
/// connection whose handshake completes.
pub async fn connect_any(
    endpoint: &quinn::Endpoint,
    addresses: &[SocketAddr],
) -> Result<quinn::Connection> {
    if addresses.is_empty() {
        return Err(Error::Connect("the other side announced no address".into()));
    }
    let mut attempts = tokio::task::JoinSet::new();
    for (i, addr) in addresses.iter().copied().enumerate() {
        let endpoint = endpoint.clone();
        let delay = Duration::from_millis(150u64.saturating_mul(u64::try_from(i).unwrap_or(0)));
        attempts.spawn(async move {
            tokio::time::sleep(delay).await;
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
