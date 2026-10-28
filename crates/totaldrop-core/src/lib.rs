//! The TotalDrop client engine: everything between "here are some files"
//! and "they arrived intact", for a transfer by code.
//!
//! A code transfer runs in four steps:
//! 1. the sender reserves a nameplate on the rendezvous server and shows a
//!    code such as `7-harbor-rocket`;
//! 2. both sides run CPace on the code in a mailbox, confirm keys by
//!    exchanging device certificates, and swap encrypted connection hints;
//! 3. they open a QUIC connection pinned to single-use endpoint keys, and
//!    run Noise XXpsk3 plus the ML-KEM-768 hybrid step inside it;
//! 4. the sender offers a signed manifest, and the files travel as
//!    STREAM-encrypted chunks, verified by BLAKE3 before they are placed.
//!
//! Device keys are generated per transfer for now. Long-term identities
//! belong in the platform keystore and arrive with saved contacts.

pub mod error;
pub mod framing;
pub mod pairing;
pub mod path;
pub mod relay;
pub mod rendezvous;
pub mod secure;
pub mod transfer;
pub mod transport;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use totaldrop_crypto::cpace::Role;
use totaldrop_crypto::identity::DeviceKeys;
use totaldrop_proto::TransferCode;
use totaldrop_proto::wire::{Certificate, Hints};

pub use error::{Error, Result};
pub use transfer::{OfferSummary, ReceiveReport, SendReport};

/// Something worth telling the person while a transfer runs.
#[derive(Debug, Clone)]
pub enum Event {
    /// The code is ready and the sender waits for the receiver.
    Waiting,
    /// The secure session is up. Both sides show the same four words.
    Connected {
        /// The verification words (spec section 6).
        words: Vec<String>,
    },
    /// Bytes moved so far, of the total.
    Progress {
        /// Bytes done.
        done: u64,
        /// Bytes in total.
        total: u64,
    },
}

/// Receives [`Event`]s.
pub type Events = Arc<dyn Fn(Event) + Send + Sync>;

/// No events wanted.
pub fn no_events() -> Events {
    Arc::new(|_| {})
}

fn words(indices: [u16; 4]) -> Vec<String> {
    indices
        .iter()
        .map(|i| totaldrop_proto::english_word(*i).unwrap_or("?").to_owned())
        .collect()
}

fn progress_from(events: &Events) -> transfer::Progress {
    let events = events.clone();
    Arc::new(move |done, total| events(Event::Progress { done, total }))
}

/// How long the sender waits for the receiver's QUIC connection after the
/// mailbox phase.
const ACCEPT_TIMEOUT: Duration = Duration::from_secs(30);
/// How long the receiver tries both directions before giving up.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

/// How this side can be reached, for its `HINTS`. With `relay_only`, no
/// address at all (spec section 6, privacy mode). Otherwise the local
/// addresses, and the public address the STUN server saw, if it answered.
async fn own_hints(
    socket: &std::net::UdpSocket,
    key: &transport::EndpointKey,
    paths: &PathOptions,
) -> Result<Hints> {
    let mut hints = Hints {
        versions: pairing::VERSIONS.to_vec(),
        endpoint_key: key.public(),
        public: Vec::new(),
        local: Vec::new(),
        relay: paths.relay_url().map(str::to_owned),
    };
    if paths.relay_only {
        return Ok(hints);
    }
    hints.local = transport::local_addresses(socket)?
        .into_iter()
        .map(totaldrop_proto::wire::Address)
        .collect();
    if !paths.stun.is_empty()
        && let Some(seen) = relay::discover(socket, &paths.stun).await
        && !hints.local.iter().any(|a| a.0 == seen)
    {
        hints.public.push(totaldrop_proto::wire::Address(seen));
    }
    Ok(hints)
}

/// How a transfer may reach the other device.
#[derive(Debug, Clone)]
pub struct PathOptions {
    /// Relay URL; empty for none. When no direct path works, the transfer
    /// runs over the sender's relay, which a receiver uses only if it is
    /// this same relay.
    pub relay: String,
    /// STUN server (`host:port`) that tells this device its public address;
    /// empty for none.
    pub stun: String,
    /// Never connect directly and announce no address: everything goes
    /// through the relay (spec section 6, privacy mode).
    pub relay_only: bool,
}

impl Default for PathOptions {
    fn default() -> Self {
        Self {
            relay: relay::DEFAULT_RELAY.into(),
            stun: relay::DEFAULT_STUN.into(),
            relay_only: false,
        }
    }
}

impl PathOptions {
    fn relay_url(&self) -> Option<&str> {
        (!self.relay.is_empty()).then_some(self.relay.as_str())
    }
}

/// A relay URL in a form two spellings of the same relay share: scheme and
/// host in lower case, the default port filled in, trailing `/` removed.
fn relay_identity(url: &str) -> Option<(String, String, u16, String)> {
    let uri: tokio_tungstenite::tungstenite::http::Uri = url.parse().ok()?;
    let scheme = uri.scheme_str()?.to_ascii_lowercase();
    // A port that is written but not a valid number (`:65536`, `host:`)
    // matches nothing, rather than falling back to the default.
    let host_port = uri.authority()?.as_str().rsplit('@').next()?;
    let written_port = host_port.rsplit(']').next()?.contains(':');
    if written_port && uri.port_u16().is_none() {
        return None;
    }
    let port = match (uri.port_u16(), scheme.as_str()) {
        (Some(p), _) => p,
        (None, "wss") => 443,
        (None, "ws") => 80,
        _ => return None,
    };
    let host = uri.host()?.to_ascii_lowercase();
    let path = uri
        .path_and_query()
        .map_or("", |p| p.as_str())
        .trim_end_matches('/')
        .to_owned();
    Some((scheme, host, port, path))
}

/// Whether two relay URLs name the same relay.
fn same_relay(a: &str, b: &str) -> bool {
    relay_identity(a).is_some_and(|x| relay_identity(b).is_some_and(|y| x == y))
}

/// The relay the receiver uses: the one the sender's `HINTS` name, but only
/// if it is a relay this side may use (`wss://`, or `ws://` on this
/// machine) and also this side's own relay setting. A receiver never
/// contacts a relay only the sender chose, so a sender cannot learn the
/// receiver's address by naming a relay it runs (#68); an empty setting
/// turns the relay off on this side too. In privacy mode there is no other
/// path, so a different relay is an error rather than a silent fallback.
///
/// The URL returned is this side's own spelling, so what is dialled always
/// comes from the local setting. With no path left at all (the sender
/// announced no address and uses another relay), it fails at once instead
/// of waiting out the connection timeout.
fn receiver_relay<'a>(hints: &Hints, own: &'a PathOptions) -> Result<Option<&'a str>> {
    let relay = own.relay_url().filter(|mine| {
        relay::check_url(mine).is_ok()
            && hints
                .relay
                .as_deref()
                .is_some_and(|theirs| same_relay(theirs, mine))
    });
    if relay.is_none() {
        if own.relay_only && own.relay_url().is_none() {
            return Err(Error::Connect("privacy mode needs a relay".into()));
        }
        if own.relay_only || peer_addresses(hints).is_empty() {
            return Err(Error::Connect(
                "no usable path: the transfer needs a relay both devices use".into(),
            ));
        }
    }
    Ok(relay)
}

/// Whether the sender can already tell that the receiver will find no path:
/// the two relays differ and one side announces no address. The receiver
/// stops at once in that case; the sender says why instead of timing out.
fn sender_check(receiver: &Hints, own: &PathOptions) -> Result<()> {
    let same = match (own.relay_url(), receiver.relay.as_deref()) {
        (Some(mine), Some(theirs)) => relay::check_url(mine).is_ok() && same_relay(mine, theirs),
        _ => false,
    };
    let no_direct = own.relay_only || peer_addresses(receiver).is_empty();
    if !same && no_direct {
        return Err(Error::Connect(
            "no usable path: the transfer needs a relay both devices use".into(),
        ));
    }
    Ok(())
}

/// Start the relay link at `url`, if any, and say which paths to try.
fn paths_to(
    peer: &Hints,
    url: Option<&str>,
    relay_only: bool,
    own: &Arc<transport::EndpointKey>,
    pipes: path::RelayPipes,
) -> (transport::Paths, Option<relay::Link>) {
    let gate = relay::Gate::default();
    let link = url.map(|u| {
        relay::spawn(
            u.to_owned(),
            own.clone(),
            peer.endpoint_key,
            pipes,
            gate.clone(),
        )
    });
    let direct = if relay_only {
        Vec::new()
    } else {
        peer_addresses(peer)
    };
    (
        transport::Paths {
            direct,
            relay: link.is_some().then_some(gate),
        },
        link,
    )
}

/// Options for sending.
#[derive(Debug, Clone)]
pub struct SendOptions {
    /// Rendezvous server URL.
    pub server: String,
    /// Secret words in the code: 2 (default) or 3.
    pub words: usize,
    /// Relay and privacy settings.
    pub paths: PathOptions,
}

impl Default for SendOptions {
    fn default() -> Self {
        Self {
            server: rendezvous::DEFAULT_SERVER.into(),
            words: 2,
            paths: PathOptions::default(),
        }
    }
}

/// A send that has a code and waits for the receiver.
pub struct PreparedSend {
    mailbox: rendezvous::Mailbox,
    code: TransferCode,
    mailbox_id: [u8; 16],
    keys: DeviceKeys,
    entries: Vec<transfer::SourceEntry>,
    skipped: usize,
    paths: PathOptions,
}

impl core::fmt::Debug for PreparedSend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PreparedSend").finish_non_exhaustive()
    }
}

impl PreparedSend {
    /// Scan and hash `paths`, reserve a nameplate and make a code.
    pub async fn new(opts: &SendOptions, paths: Vec<PathBuf>) -> Result<Self> {
        let (entries, skipped) = tokio::task::spawn_blocking(move || transfer::scan(&paths))
            .await
            .map_err(|_| Error::Protocol("scan task failed"))??;
        let keys = DeviceKeys::generate()?;
        let allocation = rendezvous::allocate(&opts.server).await?;
        let nameplate = allocation.nameplate;
        let code = TransferCode::generate(nameplate, opts.words).map_err(Error::Code)?;
        let mailbox =
            rendezvous::Mailbox::claim(&opts.server, nameplate, Some(&allocation)).await?;
        let mailbox_id = mailbox.id;
        Ok(Self {
            mailbox,
            code,
            mailbox_id,
            keys,
            entries,
            skipped,
            paths: opts.paths.clone(),
        })
    }

    /// The code to give the receiver.
    pub fn code(&self) -> &TransferCode {
        &self.code
    }

    /// Symbolic links and special files that will not be sent.
    pub fn skipped(&self) -> usize {
        self.skipped
    }

    /// Number of entries and total bytes that will be offered.
    pub fn totals(&self) -> (usize, u64) {
        (
            self.entries.len(),
            self.entries.iter().map(|e| e.entry.size).sum(),
        )
    }

    /// Wait for the receiver and send.
    pub async fn run(self, events: Events) -> Result<SendReport> {
        events(Event::Waiting);
        let socket = transport::bind_socket()?;
        let endpoint_key = Arc::new(transport::EndpointKey::generate()?);
        let hints = own_hints(&socket, &endpoint_key, &self.paths).await?;
        let paired = pairing::exchange(
            self.mailbox,
            Role::Initiator,
            &self.code,
            self.mailbox_id,
            &self.keys,
            &hints,
        )
        .await?;
        sender_check(&paired.hints, &self.paths)?;

        let (endpoint, pipes) =
            transport::endpoint(socket, &endpoint_key, &paired.hints.endpoint_key)?;
        // The sender's own relay carries the transfer when nothing else works.
        let (paths, link) = paths_to(
            &paired.hints,
            self.paths.relay_url(),
            self.paths.relay_only,
            &endpoint_key,
            pipes,
        );
        let (conn, send, recv) =
            transport::sender_connection(&endpoint, &paths, ACCEPT_TIMEOUT).await?;
        // A relay link opened before a slower direct path won is closed now.
        let _link = link.filter(|_| conn.remote_address() == path::RELAY_PEER);
        let chan = secure::handshake_on(
            send,
            recv,
            secure::HandshakeInput {
                keys: &self.keys,
                peer: &paired.peer,
                psk: paired.psk,
                mailbox_id: paired.mailbox_id,
            },
            false,
        )
        .await?;
        events(Event::Connected {
            words: words(chan.verification),
        });
        let report = transfer::send_files(
            &conn,
            chan,
            &self.keys,
            self.entries,
            progress_from(&events),
        )
        .await;
        let _ = tokio::time::timeout(Duration::from_secs(5), endpoint.wait_idle()).await;
        report
    }
}

/// Where the other side said it can be reached.
fn peer_addresses(hints: &Hints) -> Vec<SocketAddr> {
    hints
        .local
        .iter()
        .chain(hints.public.iter())
        .map(|a| a.0)
        .collect()
}

/// Options for receiving.
#[derive(Debug, Clone)]
pub struct ReceiveOptions {
    /// Rendezvous server URL.
    pub server: String,
    /// Relay and privacy settings. The sender's relay is used only when it
    /// is also the relay set here (the case of scheme and host, a default
    /// port and a trailing `/` aside).
    pub paths: PathOptions,
}

impl Default for ReceiveOptions {
    fn default() -> Self {
        Self {
            server: rendezvous::DEFAULT_SERVER.into(),
            paths: PathOptions::default(),
        }
    }
}

/// An offer that arrived and waits for a decision.
pub struct IncomingOffer {
    endpoint: quinn::Endpoint,
    conn: quinn::Connection,
    chan: secure::SecureChannel,
    offer: transfer::VerifiedOffer,
    words: Vec<String>,
    _peer: Certificate,
    _link: Option<relay::Link>,
}

impl core::fmt::Debug for IncomingOffer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IncomingOffer")
            .field("summary", &self.offer.summary)
            .finish_non_exhaustive()
    }
}

impl IncomingOffer {
    /// Use `code` to connect to the sender and receive its offer.
    pub async fn connect(opts: &ReceiveOptions, code: &str) -> Result<Self> {
        let code = TransferCode::parse(code).map_err(Error::Code)?;
        let keys = DeviceKeys::generate()?;
        let mailbox = rendezvous::Mailbox::claim(&opts.server, code.nameplate(), None).await?;
        let mailbox_id = mailbox.id;
        let socket = transport::bind_socket()?;
        let endpoint_key = Arc::new(transport::EndpointKey::generate()?);
        let hints = own_hints(&socket, &endpoint_key, &opts.paths).await?;
        let paired =
            pairing::exchange(mailbox, Role::Responder, &code, mailbox_id, &keys, &hints).await?;
        // Both sides meet on the sender's relay, if it is also ours (spec
        // section 6).
        let relay = receiver_relay(&paired.hints, &opts.paths)?;

        let (endpoint, pipes) =
            transport::endpoint(socket, &endpoint_key, &paired.hints.endpoint_key)?;
        let (paths, link) = paths_to(
            &paired.hints,
            relay,
            opts.paths.relay_only,
            &endpoint_key,
            pipes,
        );
        let conn = transport::receiver_connection(&endpoint, &paths, CONNECT_TIMEOUT).await?;
        // A relay link opened before a slower direct path won is closed now.
        let link = link.filter(|_| conn.remote_address() == path::RELAY_PEER);
        let mut chan = secure::handshake(
            &conn,
            secure::HandshakeInput {
                keys: &keys,
                peer: &paired.peer,
                psk: paired.psk,
                mailbox_id: paired.mailbox_id,
            },
            true,
        )
        .await?;
        let offer = transfer::receive_offer(&mut chan, &paired.peer).await?;
        let words = words(chan.verification);
        Ok(Self {
            endpoint,
            conn,
            chan,
            offer,
            words,
            _peer: paired.peer.clone(),
            _link: link,
        })
    }

    /// What is offered.
    pub fn summary(&self) -> &OfferSummary {
        &self.offer.summary
    }

    /// The four verification words; the sender sees the same.
    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// Accept and write everything beneath `dest`.
    pub async fn accept(self, dest: &Path, events: Events) -> Result<ReceiveReport> {
        let report = transfer::accept(
            &self.conn,
            self.chan,
            self.offer,
            dest,
            progress_from(&events),
        )
        .await;
        self.conn.close(0u32.into(), b"done");
        let _ = tokio::time::timeout(Duration::from_secs(2), self.endpoint.wait_idle()).await;
        report
    }

    /// Decline the offer.
    pub async fn decline(mut self) -> Result<()> {
        transfer::decline(&mut self.chan, &self.offer).await?;
        let _ = tokio::time::timeout(Duration::from_secs(2), self.conn.closed()).await;
        self.conn.close(0u32.into(), b"declined");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn hints(relay: Option<&str>) -> Hints {
        Hints {
            versions: pairing::VERSIONS.to_vec(),
            endpoint_key: [7; 32],
            public: Vec::new(),
            local: Vec::new(),
            relay: relay.map(str::to_owned),
        }
    }

    fn options(relay: &str, relay_only: bool) -> PathOptions {
        PathOptions {
            relay: relay.into(),
            relay_only,
            ..PathOptions::default()
        }
    }

    fn with_address(mut h: Hints) -> Hints {
        h.local = vec![totaldrop_proto::wire::Address(SocketAddr::from((
            [192, 168, 1, 2],
            4000,
        )))];
        h
    }

    #[test]
    fn receiver_uses_its_own_spelling_of_the_same_relay() {
        let own = options("wss://relay.totaldrop.app/v0/relay", false);
        for theirs in [
            "wss://relay.totaldrop.app/v0/relay/",
            "WSS://Relay.TotalDrop.App:443/v0/relay",
        ] {
            let h = hints(Some(theirs));
            assert_eq!(
                receiver_relay(&h, &own).unwrap(),
                Some("wss://relay.totaldrop.app/v0/relay")
            );
        }
        let own_slash = options("wss://relay.totaldrop.app/v0/relay/", true);
        let h = hints(Some(relay::DEFAULT_RELAY));
        assert_eq!(
            receiver_relay(&h, &own_slash).unwrap(),
            Some("wss://relay.totaldrop.app/v0/relay/")
        );
    }

    /// A sender that names a relay of its own does not get the receiver to
    /// connect there (#68).
    #[test]
    fn receiver_ignores_a_relay_only_the_sender_chose() {
        let own = options(relay::DEFAULT_RELAY, false);
        for theirs in [
            "wss://relay.example.org/v0/relay",
            "wss://relay.totaldrop.app:8443/v0/relay",
            "wss://relay.totaldrop.app/v0/other",
            "wss://relay.totaldrop.app@relay.example.org/v0/relay",
            "wss://relay.totaldrop.app:65536/v0/relay",
            "wss://relay.totaldrop.app:/v0/relay",
        ] {
            let h = with_address(hints(Some(theirs)));
            assert_eq!(receiver_relay(&h, &own).unwrap(), None, "{theirs}");
        }
        let h = with_address(hints(Some("wss://relay.example.org/v0/relay")));
        let err = receiver_relay(&h, &options(relay::DEFAULT_RELAY, true)).unwrap_err();
        assert!(err.to_string().contains("no usable path"));
    }

    #[test]
    fn no_path_at_all_fails_at_once() {
        let h = hints(Some("wss://relay.example.org/v0/relay"));
        let err = receiver_relay(&h, &options(relay::DEFAULT_RELAY, false)).unwrap_err();
        assert!(err.to_string().contains("no usable path"));
    }

    #[test]
    fn an_empty_relay_setting_turns_the_relay_off_when_receiving() {
        let h = with_address(hints(Some(relay::DEFAULT_RELAY)));
        assert_eq!(receiver_relay(&h, &options("", false)).unwrap(), None);
        let err = receiver_relay(&h, &options("", true)).unwrap_err();
        assert!(err.to_string().contains("needs a relay"));
    }

    #[test]
    fn receiver_refuses_an_unsafe_relay_even_if_it_is_its_own() {
        let url = "ws://relay.example.org/v0/relay";
        let h = with_address(hints(Some(url)));
        assert_eq!(receiver_relay(&h, &options(url, false)).unwrap(), None);
    }

    #[test]
    fn no_relay_announced() {
        let own = options(relay::DEFAULT_RELAY, false);
        let h = with_address(hints(None));
        assert_eq!(receiver_relay(&h, &own).unwrap(), None);
    }

    #[test]
    fn sender_explains_a_relay_mismatch_it_can_see() {
        let other = "wss://relay.example.org/v0/relay";
        let private = options(relay::DEFAULT_RELAY, true);
        assert!(sender_check(&with_address(hints(Some(other))), &private).is_err());
        let normal = options(relay::DEFAULT_RELAY, false);
        // The receiver in privacy mode announces no address.
        assert!(sender_check(&hints(Some(other)), &normal).is_err());
        assert!(sender_check(&with_address(hints(Some(other))), &normal).is_ok());
        assert!(sender_check(&hints(Some(relay::DEFAULT_RELAY)), &private).is_ok());
        // A sender with no relay: fine with a normal receiver, not with a
        // receiver in privacy mode.
        let none = options("", false);
        assert!(sender_check(&with_address(hints(Some(relay::DEFAULT_RELAY))), &none).is_ok());
        assert!(sender_check(&hints(Some(relay::DEFAULT_RELAY)), &none).is_err());
        // Both on the same relay, but one this side may not use.
        let unsafe_relay = options("ws://relay.example.org/v0/relay", true);
        assert!(
            sender_check(
                &hints(Some("ws://relay.example.org/v0/relay")),
                &unsafe_relay
            )
            .is_err()
        );
    }
}
