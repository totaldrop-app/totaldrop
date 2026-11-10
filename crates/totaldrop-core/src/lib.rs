// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
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
pub mod portmap;
pub mod relay;
pub mod rendezvous;
pub mod secure;
mod stop;
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

pub use error::{Cause, Error, Result};
pub use stop::Stop;
pub use transfer::{
    MAX_TEXT, OfferSummary, ReceiveReport, SendReport, TEXT_FILE, check_destination,
};

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

/// How long `own_hints` waits for the router's port mapping once STUN
/// has answered (or when there is no STUN).
const MAPPING_GRACE: Duration = Duration::from_millis(300);

/// How this side can be reached, for its `HINTS`. With `relay_only`, no
/// address at all (spec section 6, privacy mode). Otherwise the local
/// addresses, the public address the STUN server saw, if it answered, and
/// the port the router opened for us, if it did (#406).
async fn own_hints(
    socket: &std::net::UdpSocket,
    key: &transport::EndpointKey,
    paths: &PathOptions,
) -> Result<(Hints, Option<portmap::Mapping>)> {
    let mut hints = Hints {
        versions: pairing::VERSIONS.to_vec(),
        endpoint_key: key.public(),
        public: Vec::new(),
        local: Vec::new(),
        relay: paths.relay_url().map(str::to_owned),
    };
    if paths.relay_only {
        return Ok((hints, None));
    }
    hints.local = transport::local_addresses(socket)?
        .into_iter()
        .map(totaldrop_proto::wire::Address)
        .collect();
    // The router is asked alongside STUN (#406). Once STUN is done the
    // mapping gets at most `MAPPING_GRACE` more, so a router that never
    // answers costs a fraction of a second, not `portmap::LIMIT`; a UPnP
    // gateway that has answered by then is waited for until the attempt
    // ends. A mapping that comes later is dropped, which releases it.
    let port = socket.local_addr()?.port();
    let asking = portmap::start(paths.gateway.clone(), port);
    let seen = if paths.stun.is_empty() {
        Vec::new()
    } else {
        relay::discover(socket, &paths.stun).await
    };
    let mapping = asking.wait(MAPPING_GRACE).await;
    for seen in seen
        .into_iter()
        .chain(mapping.as_ref().map(portmap::Mapping::external))
    {
        let hidden =
            matches!(seen.ip(), std::net::IpAddr::V6(ip) if !transport::announceable_v6(ip));
        if !hidden
            && !hints
                .local
                .iter()
                .chain(hints.public.iter())
                .any(|a| a.0 == seen)
        {
            hints.public.push(totaldrop_proto::wire::Address(seen));
        }
    }
    Ok((hints, mapping))
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
    /// Router to ask for a forwarded port (PCP or NAT-PMP) while the
    /// transfer runs; never asked in privacy mode (#406).
    pub gateway: portmap::Gateway,
}

impl Default for PathOptions {
    fn default() -> Self {
        Self {
            relay: relay::DEFAULT_RELAY.into(),
            stun: relay::DEFAULT_STUN.into(),
            relay_only: false,
            gateway: portmap::Gateway::Auto,
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
            return Err(Error::Connect(
                crate::error::msg::PRIVACY_NEEDS_RELAY.into(),
            ));
        }
        if own.relay_only || peer_addresses(hints).is_empty() {
            return Err(Error::Connect(crate::error::msg::NO_SHARED_RELAY.into()));
        }
    }
    Ok(relay)
}

/// Whether the sender can already tell that the receiver will find no path:
/// the two relays differ and one side announces no address. The receiver
/// stops at once in that case; the sender says why instead of timing out.
fn sender_check(receiver: &Hints, own: &PathOptions) -> Result<()> {
    let same = shares_relay(receiver, own);
    let no_direct = own.relay_only || peer_addresses(receiver).is_empty();
    if !same && no_direct {
        return Err(Error::Connect(crate::error::msg::NO_SHARED_RELAY.into()));
    }
    Ok(())
}

/// Whether the receiver will use the sender's relay too, as
/// [`receiver_relay`] decides on its side. The receiver announces its own
/// setting as it is, so its `check_url` runs here on that spelling too.
fn shares_relay(receiver: &Hints, own: &PathOptions) -> bool {
    match (own.relay_url(), receiver.relay.as_deref()) {
        (Some(mine), Some(theirs)) => {
            relay::check_url(mine).is_ok()
                && relay::check_url(theirs).is_ok()
                && same_relay(mine, theirs)
        }
        _ => false,
    }
}

/// The relay the sender links to: its own, and only when the receiver
/// will use it too ([`shares_relay`]). Otherwise the receiver never
/// arrives there, and a link would only show the relay this side's
/// address for nothing (#301).
fn sender_relay<'a>(receiver: &Hints, own: &'a PathOptions) -> Option<&'a str> {
    own.relay_url().filter(|_| shares_relay(receiver, own))
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
            relay_only,
        },
        link,
    )
}

/// What keeps the relay at hand once a connection was chosen. Held only
/// to be dropped with the connection: a watch never runs without its link.
#[derive(Debug)]
enum Standby {
    /// No relay link is kept.
    Nothing,
    /// The connection goes through the relay on this link.
    Link { _link: relay::Link },
    /// A direct connection with its link parked, and the watch that moves
    /// it onto the link if the direct path goes quiet. The link is dropped
    /// first, as fields are dropped in order.
    Fallback {
        _link: relay::Link,
        _watch: path::Watch,
    },
}

/// What the chosen connection keeps of the relay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keep {
    /// The connection goes through the relay: keep the link as it is.
    Link,
    /// A direct connection that can fall back: park the link and watch.
    Fallback,
    /// Nothing: close the link, if any.
    Nothing,
}

/// Decide [`Keep`] for a connection to `remote`. A direct connection can
/// move onto the relay only if this side has a link (`linked`) and both
/// sides use the same relay (`shared`); otherwise the other side never
/// arrives there, and moving would only lose a direct path that may come
/// back.
fn keep(remote: std::net::SocketAddr, linked: bool, shared: bool) -> Keep {
    if path::is_relay(remote) {
        Keep::Link
    } else if linked && shared {
        Keep::Fallback
    } else {
        Keep::Nothing
    }
}

/// Keep what the chosen connection `conn` may still need of the relay
/// (#185). A direct connection that can fall back keeps its link, parked:
/// a connection to the relay opened before the direct path won is closed
/// now, and the link connects again only if the watch moves the connection
/// onto the relay because its direct path went quiet.
fn standby(
    conn: &quinn::Connection,
    paths: &transport::Paths,
    link: Option<relay::Link>,
    fallback: &path::Fallback,
    shared: bool,
) -> Standby {
    let gate = paths.relay.clone();
    match (
        keep(conn.remote_address(), link.is_some(), shared),
        link,
        gate,
    ) {
        (Keep::Link, Some(link), _) => Standby::Link { _link: link },
        (Keep::Fallback, Some(link), Some(gate)) => {
            link.park(fallback);
            let watch = path::Watch::spawn(fallback, conn.clone(), gate, path::FALLBACK_SILENCE);
            Standby::Fallback {
                _link: link,
                _watch: watch,
            }
        }
        _ => Standby::Nothing,
    }
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
    text: bool,
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
        Self::ready(opts, entries, skipped, false).await
    }

    /// Like [`PreparedSend::new`], for a text (#313): the receiver shows it
    /// instead of saving it. A receiver that does not know texts saves it
    /// as a file called `name`, which should end in `.txt`. At most
    /// [`MAX_TEXT`] bytes.
    pub async fn text(opts: &SendOptions, text: &str, name: &str) -> Result<Self> {
        let entry = transfer::text_entry(text, name)?;
        Self::ready(opts, vec![entry], 0, true).await
    }

    async fn ready(
        opts: &SendOptions,
        entries: Vec<transfer::SourceEntry>,
        skipped: usize,
        text: bool,
    ) -> Result<Self> {
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
            text,
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
        self.run_with(events, Stop::new()).await
    }

    /// Like [`PreparedSend::run`], until `stop` is stopped. Before the
    /// secure session is up, stopping just leaves; after it, the receiver is
    /// told and the transfer ends with [`Error::Stopped`].
    pub async fn run_with(self, events: Events, stop: Stop) -> Result<SendReport> {
        events(Event::Waiting);
        let setup = async {
            let socket = transport::bind_socket()?;
            let endpoint_key = Arc::new(transport::EndpointKey::generate()?);
            let (hints, mapping) = own_hints(&socket, &endpoint_key, &self.paths).await?;
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

            let (endpoint, pipes, fallback) =
                transport::endpoint(socket, &endpoint_key, &paired.hints.endpoint_key)?;
            // The sender's own relay carries the transfer when nothing else
            // works, if the receiver uses it too.
            let (paths, link) = paths_to(
                &paired.hints,
                sender_relay(&paired.hints, &self.paths),
                self.paths.relay_only,
                &endpoint_key,
                pipes,
            );
            let (conn, send, recv) =
                transport::sender_connection(&endpoint, &paths, ACCEPT_TIMEOUT).await?;
            // The receiver meets us on the relay only if it is also its own.
            let shared = shares_relay(&paired.hints, &self.paths);
            let _link = standby(&conn, &paths, link, &fallback, shared);
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
            Ok::<_, Error>((endpoint, conn, chan, _link, mapping))
        };
        // A session that is already up wins over a stop at the same moment:
        // send_files then sees the stop and tells the receiver.
        // The router keeps the port open until the transfer ends.
        let (endpoint, conn, chan, _link, _mapping) = tokio::select! {
            biased;
            r = setup => r?,
            () = stop.stopped() => return Err(Error::Stopped),
        };
        events(Event::Connected {
            words: words(chan.verification),
        });
        let report = transfer::send_files(
            &conn,
            chan,
            &self.keys,
            self.entries,
            self.text,
            progress_from(&events),
            &stop,
        )
        .await;
        if report.is_err() {
            // Free the connection at once instead of waiting for it to idle.
            conn.close(0u32.into(), b"failed");
        }
        let _ = tokio::time::timeout(Duration::from_secs(5), endpoint.wait_idle()).await;
        report
    }
}

/// Where the other side said it can be reached, each address once and in
/// canonical form (#403). Left out: addresses no one can dial (unspecified,
/// or IPv6 link-local, which needs an interface scope `HINTS` do not carry),
/// broadcast and multicast addresses (#450),
/// and IPv6 addresses this device would dial from a source it never
/// announces itself (see [`transport::dialable`]).
fn peer_addresses(hints: &Hints) -> Vec<SocketAddr> {
    let mut out: Vec<SocketAddr> = Vec::new();
    for a in hints.local.iter().chain(hints.public.iter()) {
        let a = path::canonical(a.0);
        if !out.contains(&a) && transport::dialable(a) {
            out.push(a);
        }
    }
    out
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
    _link: Standby,
    _mapping: Option<portmap::Mapping>,
}

impl core::fmt::Debug for IncomingOffer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IncomingOffer")
            .field("summary", &self.offer.summary)
            .finish_non_exhaustive()
    }
}

impl IncomingOffer {
    /// Like [`IncomingOffer::connect`], until `stop` is stopped. A stop while
    /// connecting just leaves; if the offer arrives as the stop lands, the
    /// sender is told.
    pub async fn connect_with(opts: &ReceiveOptions, code: &str, stop: &Stop) -> Result<Self> {
        let offer = tokio::select! {
            biased;
            r = Self::connect(opts, code) => r?,
            () = stop.stopped() => return Err(Error::Stopped),
        };
        if stop.is_stopped() {
            return Err(offer.stop().await);
        }
        Ok(offer)
    }

    /// Tell the sender this side stopped and close; returns
    /// [`Error::Stopped`]. For a person who leaves while the offer is shown.
    pub async fn stop(mut self) -> Error {
        let id = self.offer.transfer_id();
        let e = transfer::tell_stopped(&self.conn, &mut self.chan.sealer, &mut self.chan.send, id)
            .await;
        let _ = tokio::time::timeout(Duration::from_secs(2), self.endpoint.wait_idle()).await;
        e
    }

    /// Use `code` to connect to the sender and receive its offer.
    pub async fn connect(opts: &ReceiveOptions, code: &str) -> Result<Self> {
        let code = TransferCode::parse(code).map_err(Error::Code)?;
        let keys = DeviceKeys::generate()?;
        let mailbox = rendezvous::Mailbox::claim(&opts.server, code.nameplate(), None).await?;
        let mailbox_id = mailbox.id;
        let socket = transport::bind_socket()?;
        let endpoint_key = Arc::new(transport::EndpointKey::generate()?);
        let (hints, mapping) = own_hints(&socket, &endpoint_key, &opts.paths).await?;
        let paired =
            pairing::exchange(mailbox, Role::Responder, &code, mailbox_id, &keys, &hints).await?;
        // Both sides meet on the sender's relay, if it is also ours (spec
        // section 6).
        let relay = receiver_relay(&paired.hints, &opts.paths)?;

        let (endpoint, pipes, fallback) =
            transport::endpoint(socket, &endpoint_key, &paired.hints.endpoint_key)?;
        let (paths, link) = paths_to(
            &paired.hints,
            relay,
            opts.paths.relay_only,
            &endpoint_key,
            pipes,
        );
        let conn = transport::receiver_connection(&endpoint, &paths, CONNECT_TIMEOUT).await?;
        // `relay` is set only when it is the sender's relay too.
        let link = standby(&conn, &paths, link, &fallback, relay.is_some());
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
            _mapping: mapping,
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
        self.accept_with(dest, events, &Stop::new()).await
    }

    /// Like [`IncomingOffer::accept`], until `stop` is stopped: then the
    /// sender is told, partial files are removed and the transfer ends with
    /// [`Error::Stopped`]. Files already placed stay.
    pub async fn accept_with(
        self,
        dest: &Path,
        events: Events,
        stop: &Stop,
    ) -> Result<ReceiveReport> {
        let report = transfer::accept(
            &self.conn,
            self.chan,
            self.offer,
            dest,
            progress_from(&events),
            stop,
        )
        .await;
        self.conn.close(0u32.into(), b"done");
        let _ = tokio::time::timeout(Duration::from_secs(2), self.endpoint.wait_idle()).await;
        report
    }

    /// Accept a text offer ([`OfferSummary::text`]) and keep it in memory:
    /// nothing is written to disk. The text is in [`ReceiveReport::text`];
    /// a text that fails its hash or is not UTF-8 is counted as failed.
    pub async fn accept_text(self) -> Result<ReceiveReport> {
        self.accept_text_with(&Stop::new()).await
    }

    /// Like [`IncomingOffer::accept_text`], until `stop` is stopped.
    pub async fn accept_text_with(self, stop: &Stop) -> Result<ReceiveReport> {
        let report = transfer::accept_text(&self.conn, self.chan, self.offer, stop).await;
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

    /// Which connections keep the relay, and how (#185 review). A
    /// connection through the relay keeps its link; a direct one falls back
    /// only with a link and a relay both sides use; otherwise the link
    /// goes.
    #[test]
    fn what_a_chosen_connection_keeps_of_the_relay() {
        let direct = std::net::SocketAddr::from(([192, 0, 2, 1], 4000));
        assert_eq!(keep(path::RELAY_PEER, true, true), Keep::Link);
        assert_eq!(keep(path::RELAY_PEER, true, false), Keep::Link);
        // On a dual-stack socket QUIC reports the stand-in mapped (#403).
        let mapped = path::in_family(path::RELAY_PEER, true);
        assert_eq!(keep(mapped, true, false), Keep::Link);
        assert_eq!(
            keep(path::in_family(direct, true), true, true),
            Keep::Fallback
        );
        assert_eq!(keep(direct, true, true), Keep::Fallback);
        assert_eq!(keep(direct, true, false), Keep::Nothing);
        assert_eq!(keep(direct, false, true), Keep::Nothing);
    }

    /// #403: the other side's addresses are dialled once each, in
    /// canonical form, and never at addresses no one can dial.
    #[test]
    fn peer_addresses_are_deduplicated_and_dialable() {
        use std::net::Ipv6Addr;
        let addr = |a: SocketAddr| totaldrop_proto::wire::Address(a);
        let v4 = SocketAddr::from(([192, 0, 2, 7], 4000));
        let mut h = hints(None);
        h.local = vec![
            addr(v4),
            addr(path::in_family(v4, true)),
            addr(SocketAddr::from((Ipv6Addr::UNSPECIFIED, 4000))),
            addr(SocketAddr::from(([0, 0, 0, 0], 4000))),
            // #450: broadcast and multicast, also IPv4-mapped, which
            // `canonical` turns back into IPv4 before `dialable` sees them.
            addr(SocketAddr::from(([255, 255, 255, 255], 4000))),
            addr("[::ffff:255.255.255.255]:4000".parse().unwrap()),
            addr("[::ffff:224.0.0.251]:4000".parse().unwrap()),
            addr(SocketAddr::from((
                Ipv6Addr::new(0xfe80, 0, 0, 0, 0x1c2d, 0x3e4f, 0x5a6b, 0x7c8d),
                4000,
            ))),
        ];
        h.public = vec![addr(v4)];
        assert_eq!(peer_addresses(&h), vec![v4]);
    }

    /// #403: what one side announces stays within the 8 addresses per
    /// list that `HINTS` accept, here without a STUN server.
    #[tokio::test]
    async fn own_hints_fit_in_hints() {
        let socket = transport::bind_socket().unwrap();
        let key = transport::EndpointKey::generate().unwrap();
        let paths = PathOptions {
            stun: String::new(),
            gateway: portmap::Gateway::Off,
            ..PathOptions::default()
        };
        let (h, _) = own_hints(&socket, &key, &paths).await.unwrap();
        assert!(h.local.len() <= 8 && h.public.len() <= 8, "{h:?}");
        assert!(
            h.local
                .iter()
                .all(|a| transport::dialable(a.0) || a.0.ip().is_loopback())
        );
    }

    /// The sender falls back only when the receiver will use its relay
    /// too, exactly when the receiver's own check gives it a relay path.
    #[test]
    fn sender_and_receiver_agree_on_a_shared_relay() {
        let ours = options("wss://relay.example/v0/relay", false);
        for (theirs, shared) in [
            (Some("wss://RELAY.example:443/v0/relay/"), true),
            (Some("wss://other.example/v0/relay"), false),
            (None, false),
        ] {
            let receiver = with_address(hints(theirs));
            assert_eq!(shares_relay(&receiver, &ours), shared, "{theirs:?}");
            if let Some(url) = theirs {
                let sender = with_address(hints(Some("wss://relay.example/v0/relay")));
                let mine = options(url, false);
                assert_eq!(
                    receiver_relay(&sender, &mine).unwrap().is_some(),
                    shared,
                    "{url}"
                );
            }
        }
    }

    /// Two spellings of one relay that only one side's `check_url`
    /// accepts: the sender links only if the receiver will come (#301
    /// review). The receiver checks its own spelling, which is the one it
    /// announces.
    #[test]
    fn sender_links_only_to_a_relay_the_receiver_accepts() {
        for (mine, theirs) in [
            (
                "wss://relay.example/v0/relay",
                "WSS://relay.example/v0/relay",
            ),
            (
                "wss://relay.example/v0/relay",
                "wss://u@relay.example/v0/relay",
            ),
            (
                "ws://localhost:4000/v0/relay",
                "ws://LOCALHOST:4000/v0/relay",
            ),
        ] {
            let receiver_uses =
                receiver_relay(&with_address(hints(Some(mine))), &options(theirs, false))
                    .unwrap()
                    .is_some();
            let sender_links =
                shares_relay(&with_address(hints(Some(theirs))), &options(mine, false));
            assert_eq!(sender_links, receiver_uses, "{mine} / {theirs}");
        }
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

    /// A sender whose receiver uses another relay, or none, never links to
    /// its own: the receiver would not come (#301).
    #[test]
    fn sender_links_to_its_relay_only_when_the_receiver_uses_it() {
        let own = options("wss://relay.example/v0/relay", false);
        for (theirs, expected) in [
            (Some("wss://RELAY.example:443/v0/relay/"), true),
            (Some("wss://other.example/v0/relay"), false),
            (None, false),
        ] {
            let receiver = with_address(hints(theirs));
            assert_eq!(
                sender_relay(&receiver, &own),
                expected.then_some("wss://relay.example/v0/relay"),
                "{theirs:?}"
            );
        }
        let unsafe_relay = options("ws://relay.example.org/v0/relay", false);
        let receiver = with_address(hints(Some("ws://relay.example.org/v0/relay")));
        assert_eq!(sender_relay(&receiver, &unsafe_relay), None);
        assert_eq!(sender_relay(&receiver, &options("", false)), None);
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
