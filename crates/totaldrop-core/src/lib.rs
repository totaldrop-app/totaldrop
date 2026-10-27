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

fn own_hints(socket: &std::net::UdpSocket, key: &transport::EndpointKey) -> Result<Hints> {
    let local = transport::local_addresses(socket)?
        .into_iter()
        .map(totaldrop_proto::wire::Address)
        .collect();
    Ok(Hints {
        versions: pairing::VERSIONS.to_vec(),
        endpoint_key: key.public(),
        public: Vec::new(),
        local,
        relay: None,
    })
}

/// Options for sending.
#[derive(Debug, Clone)]
pub struct SendOptions {
    /// Rendezvous server URL.
    pub server: String,
    /// Secret words in the code: 2 (default) or 3.
    pub words: usize,
}

impl Default for SendOptions {
    fn default() -> Self {
        Self {
            server: rendezvous::DEFAULT_SERVER.into(),
            words: 2,
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
        let endpoint_key = transport::EndpointKey::generate()?;
        let hints = own_hints(&socket, &endpoint_key)?;
        let paired = pairing::exchange(
            self.mailbox,
            Role::Initiator,
            &self.code,
            self.mailbox_id,
            &self.keys,
            &hints,
        )
        .await?;

        let endpoint =
            transport::server_endpoint(socket, &endpoint_key, &paired.hints.endpoint_key)?;
        let conn = tokio::time::timeout(ACCEPT_TIMEOUT, accept_first(&endpoint))
            .await
            .map_err(|_| Error::Connect("the receiver did not connect".into()))??;
        let chan = secure::handshake(
            &conn,
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

/// Accept the first incoming connection that completes its handshake; the
/// pinned key check inside TLS turns away anyone else.
async fn accept_first(endpoint: &quinn::Endpoint) -> Result<quinn::Connection> {
    loop {
        let incoming = endpoint
            .accept()
            .await
            .ok_or(Error::Connect("endpoint closed".into()))?;
        if let Ok(conn) = incoming.await {
            return Ok(conn);
        }
    }
}

/// Options for receiving.
#[derive(Debug, Clone)]
pub struct ReceiveOptions {
    /// Rendezvous server URL.
    pub server: String,
}

impl Default for ReceiveOptions {
    fn default() -> Self {
        Self {
            server: rendezvous::DEFAULT_SERVER.into(),
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
        let endpoint_key = transport::EndpointKey::generate()?;
        let hints = own_hints(&socket, &endpoint_key)?;
        let paired =
            pairing::exchange(mailbox, Role::Responder, &code, mailbox_id, &keys, &hints).await?;

        let endpoint =
            transport::client_endpoint(socket, &endpoint_key, &paired.hints.endpoint_key)?;
        let addresses: Vec<SocketAddr> = paired
            .hints
            .local
            .iter()
            .chain(paired.hints.public.iter())
            .map(|a| a.0)
            .collect();
        let conn = transport::connect_any(&endpoint, &addresses).await?;
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
