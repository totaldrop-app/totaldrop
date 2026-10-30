//! The Dart-facing API of TotalDrop, generated into Dart by
//! flutter_rust_bridge. Everything here is a thin layer over
//! `totaldrop-core`; no protocol logic lives in this file.
//!
//! Types are kept plain (structs and fieldless enums, `i64` numbers) so the
//! generated Dart needs no extra code generators.

use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

use flutter_rust_bridge::frb;
use totaldrop_core::{
    Event, IncomingOffer, PathOptions, PreparedSend, ReceiveOptions, SendOptions, Stop,
};

use crate::frb_generated::StreamSink;

/// One runtime for all network work, independent of how the bridge runs
/// its own async functions.
static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("totaldrop")
        .build()
        .unwrap_or_else(|_| std::process::abort())
});

async fn on_runtime<F, T>(fut: F) -> Result<T, TransferError>
where
    F: std::future::Future<Output = Result<T, TransferError>> + Send + 'static,
    T: Send + 'static,
{
    RUNTIME
        .spawn(fut)
        .await
        .unwrap_or_else(|_| Err(TransferError::other("internal task failed")))
}

/// Stops one transfer, at any stage: preparing, waiting, connecting or
/// moving files. Dart makes one per transfer and passes it to every call
/// of that transfer. Once the session is up, the other side is told and
/// sees "the other side stopped"; partial files are removed.
#[frb(opaque)]
pub struct Stopper {
    stop: Stop,
}

/// A new, unstopped stopper.
#[frb(sync)]
pub fn new_stopper() -> Stopper {
    Stopper { stop: Stop::new() }
}

/// Stop the transfer `stopper` belongs to. Safe to call more than once,
/// and before the transfer starts.
#[frb(sync)]
pub fn stop_transfer(stopper: &Stopper) {
    stopper.stop.stop();
}

fn stopped() -> TransferError {
    TransferError::from(totaldrop_core::Error::Stopped)
}

fn n(v: usize) -> i64 {
    i64::try_from(v).unwrap_or(i64::MAX)
}

fn b(v: u64) -> i64 {
    i64::try_from(v).unwrap_or(i64::MAX)
}

/// The default rendezvous server.
#[frb(sync)]
pub fn default_server() -> String {
    totaldrop_core::rendezvous::DEFAULT_SERVER.to_owned()
}

/// Kinds of failure. Dart shows a translated message for each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The code did not match: mistyped, or someone tried to intercept.
    WrongCode,
    /// The other side left.
    PeerLeft,
    /// The receiver declined.
    Declined,
    /// The other side cancelled.
    Cancelled,
    /// Stopped on this device.
    Stopped,
    /// The code could not be read.
    InvalidCode,
    /// The rendezvous server could not be reached or refused.
    Server,
    /// No network path to the other device worked.
    Connect,
    /// The offer broke a safety rule.
    InvalidOffer,
    /// A local file problem.
    Storage,
    /// Anything else.
    Other,
}

/// Why a transfer ended.
#[derive(Debug, Clone)]
pub struct TransferError {
    /// What went wrong.
    pub kind: ErrorKind,
    /// Technical detail, in English, for "more information".
    pub detail: String,
}

impl TransferError {
    fn other(detail: &str) -> Self {
        Self {
            kind: ErrorKind::Other,
            detail: detail.to_owned(),
        }
    }
}

impl From<totaldrop_core::Error> for TransferError {
    fn from(e: totaldrop_core::Error) -> Self {
        use totaldrop_core::Error as E;
        let detail = e.to_string();
        let kind = match e {
            E::WrongCode => ErrorKind::WrongCode,
            E::PeerLeft => ErrorKind::PeerLeft,
            E::Rejected(_) => ErrorKind::Declined,
            E::Cancelled(_) => ErrorKind::Cancelled,
            E::Stopped => ErrorKind::Stopped,
            E::Code(_) => ErrorKind::InvalidCode,
            E::Rendezvous(_) => ErrorKind::Server,
            E::Connect(_) => ErrorKind::Connect,
            E::InvalidOffer(_) => ErrorKind::InvalidOffer,
            E::Io(_) => ErrorKind::Storage,
            _ => ErrorKind::Other,
        };
        Self { kind, detail }
    }
}

/// Summary of a finished transfer.
#[derive(Debug, Clone, Default)]
pub struct TransferSummary {
    /// Files verified on arrival.
    pub files_ok: i64,
    /// Files discarded because they did not arrive intact.
    pub files_failed: i64,
    /// Bytes moved.
    pub bytes: i64,
    /// Where the receiver saved the top-level items (empty for the sender).
    pub saved: Vec<String>,
}

/// Kinds of progress event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// Waiting for the receiver.
    Waiting,
    /// The secure session is up; `words` holds the verification words.
    Connected,
    /// `done` of `total` bytes have moved.
    Progress,
    /// Finished; `summary` is set. Always the last event on success.
    Finished,
    /// Failed; `error` is set. Always the last event on failure.
    Failed,
}

/// Something that happened during a transfer.
#[derive(Debug, Clone)]
pub struct TransferEvent {
    /// What happened.
    pub kind: EventKind,
    /// Verification words, for `Connected`.
    pub words: Vec<String>,
    /// Bytes done, for `Progress`.
    pub done: i64,
    /// Bytes in total, for `Progress`.
    pub total: i64,
    /// For `Finished`.
    pub summary: Option<TransferSummary>,
    /// For `Failed`.
    pub error: Option<TransferError>,
}

impl TransferEvent {
    fn of(kind: EventKind) -> Self {
        Self {
            kind,
            words: Vec::new(),
            done: 0,
            total: 0,
            summary: None,
            error: None,
        }
    }
}

type Sink = Arc<Mutex<StreamSink<TransferEvent>>>;

fn emit(sink: &Sink, event: TransferEvent) {
    if let Ok(s) = sink.lock() {
        let _ = s.add(event);
    }
}

fn core_events(sink: Sink) -> totaldrop_core::Events {
    Arc::new(move |event| {
        let e = match event {
            Event::Waiting => TransferEvent::of(EventKind::Waiting),
            Event::Connected { words } => TransferEvent {
                words,
                ..TransferEvent::of(EventKind::Connected)
            },
            Event::Progress { done, total } => TransferEvent {
                done: b(done),
                total: b(total),
                ..TransferEvent::of(EventKind::Progress)
            },
        };
        emit(&sink, e);
    })
}

fn finish(sink: &Sink, result: Result<TransferSummary, TransferError>) {
    match result {
        Ok(summary) => emit(
            sink,
            TransferEvent {
                summary: Some(summary),
                ..TransferEvent::of(EventKind::Finished)
            },
        ),
        Err(error) => emit(
            sink,
            TransferEvent {
                error: Some(error),
                ..TransferEvent::of(EventKind::Failed)
            },
        ),
    }
}

/// A send with a code, waiting to be run.
#[frb(opaque)]
pub struct SendSession {
    inner: Mutex<Option<PreparedSend>>,
    code: String,
    items: i64,
    total_bytes: i64,
}

impl SendSession {
    /// The code to tell the receiver, for example `7-harbor-rocket`.
    #[frb(sync, getter)]
    pub fn code(&self) -> String {
        self.code.clone()
    }

    /// Number of files and folders offered.
    #[frb(sync, getter)]
    pub fn items(&self) -> i64 {
        self.items
    }

    /// Total bytes offered.
    #[frb(sync, getter)]
    pub fn total_bytes(&self) -> i64 {
        self.total_bytes
    }
}

/// The paths a transfer may take. With `privacy`, only the relay: the
/// other device never learns this one's IP address.
fn route(privacy: bool) -> PathOptions {
    PathOptions {
        relay_only: privacy,
        ..PathOptions::default()
    }
}

/// Scan and hash `paths`, reserve a code. `strong` adds a third word;
/// `privacy` sends through the relay only.
pub async fn prepare_send(
    server: String,
    paths: Vec<String>,
    strong: bool,
    privacy: bool,
    stopper: &Stopper,
) -> Result<SendSession, TransferError> {
    let stop = stopper.stop.clone();
    on_runtime(async move {
        let opts = SendOptions {
            server,
            words: if strong { 3 } else { 2 },
            paths: route(privacy),
        };
        let paths = paths.into_iter().map(PathBuf::from).collect();
        // Dropping the preparation releases the reserved code at once.
        let prepared = tokio::select! {
            biased;
            () = stop.stopped() => return Err(stopped()),
            r = PreparedSend::new(&opts, paths) => r?,
        };
        let (items, total_bytes) = prepared.totals();
        Ok(SendSession {
            code: prepared.code().to_string(),
            items: n(items),
            total_bytes: b(total_bytes),
            inner: Mutex::new(Some(prepared)),
        })
    })
    .await
}

/// Wait for the receiver and send, until `stopper` is stopped. Events end
/// with `Finished` or `Failed`.
pub async fn run_send(session: &SendSession, stopper: &Stopper, sink: StreamSink<TransferEvent>) {
    let sink: Sink = Arc::new(Mutex::new(sink));
    let prepared = session.inner.lock().ok().and_then(|mut g| g.take());
    let Some(prepared) = prepared else {
        return finish(&sink, Err(TransferError::other("this send already ran")));
    };
    let events = core_events(sink.clone());
    let stop = stopper.stop.clone();
    let result = on_runtime(async move {
        let r = prepared.run_with(events, stop).await?;
        Ok(TransferSummary {
            files_ok: n(r.files_ok),
            files_failed: n(r.files_failed),
            bytes: b(r.bytes),
            saved: Vec::new(),
        })
    })
    .await;
    finish(&sink, result);
}

/// An offer that arrived, waiting for the person to decide.
#[frb(opaque)]
pub struct Offer {
    inner: Mutex<Option<IncomingOffer>>,
    files: i64,
    folders: i64,
    total_bytes: i64,
    names: Vec<String>,
    words: Vec<String>,
}

impl Offer {
    /// Number of files.
    #[frb(sync, getter)]
    pub fn files(&self) -> i64 {
        self.files
    }

    /// Number of folders.
    #[frb(sync, getter)]
    pub fn folders(&self) -> i64 {
        self.folders
    }

    /// Total size in bytes.
    #[frb(sync, getter)]
    pub fn total_bytes(&self) -> i64 {
        self.total_bytes
    }

    /// Top-level names.
    #[frb(sync, getter)]
    pub fn names(&self) -> Vec<String> {
        self.names.clone()
    }

    /// The four verification words; the sender sees the same.
    #[frb(sync, getter)]
    pub fn words(&self) -> Vec<String> {
        self.words.clone()
    }
}

/// Use `code` to connect to the sender and receive its offer, until
/// `stopper` is stopped. `privacy` receives through the relay only.
pub async fn connect_receive(
    server: String,
    code: String,
    privacy: bool,
    stopper: &Stopper,
) -> Result<Offer, TransferError> {
    let stop = stopper.stop.clone();
    on_runtime(async move {
        let offer = IncomingOffer::connect_with(
            &ReceiveOptions {
                server,
                paths: route(privacy),
            },
            &code,
            &stop,
        )
        .await?;
        let s = offer.summary().clone();
        Ok(Offer {
            files: n(s.files),
            folders: n(s.dirs),
            total_bytes: b(s.total_size),
            names: s.top_level,
            words: offer.words().to_vec(),
            inner: Mutex::new(Some(offer)),
        })
    })
    .await
}

/// Accept the offer and save everything beneath `destination`, which is
/// created if missing, until `stopper` is stopped. Events end with
/// `Finished` or `Failed`.
pub async fn accept_offer(
    offer: &Offer,
    destination: String,
    stopper: &Stopper,
    sink: StreamSink<TransferEvent>,
) {
    let sink: Sink = Arc::new(Mutex::new(sink));
    let incoming = offer.inner.lock().ok().and_then(|mut g| g.take());
    let Some(incoming) = incoming else {
        return finish(
            &sink,
            Err(TransferError::other("this offer was already answered")),
        );
    };
    let events = core_events(sink.clone());
    let stop = stopper.stop.clone();
    let result = on_runtime(async move {
        let dest = PathBuf::from(destination);
        std::fs::create_dir_all(&dest).map_err(|e| TransferError {
            kind: ErrorKind::Storage,
            detail: e.to_string(),
        })?;
        let r = incoming.accept_with(&dest, events, &stop).await?;
        Ok(TransferSummary {
            files_ok: n(r.files_ok),
            files_failed: n(r.files_failed),
            bytes: b(r.bytes),
            saved: r.placed.iter().map(|p| p.display().to_string()).collect(),
        })
    })
    .await;
    finish(&sink, result);
}

/// Decline the offer.
pub async fn decline_offer(offer: &Offer) -> Result<(), TransferError> {
    let incoming = offer
        .inner
        .lock()
        .ok()
        .and_then(|mut g| g.take())
        .ok_or_else(|| TransferError::other("this offer was already answered"))?;
    on_runtime(async move { Ok(incoming.decline().await?) }).await
}

/// Normalize a code as typed: lowercase, spaces to dashes, invisible format
/// characters such as direction marks dropped. Returns `None` if it cannot
/// be a code, so the UI can say so before connecting.
#[frb(sync)]
pub fn normalize_code(input: String) -> Option<String> {
    let cleaned = totaldrop_proto::strip_invisible_format(&input);
    let cleaned = cleaned
        .trim()
        .to_lowercase()
        .split(|c: char| c.is_whitespace() || c == '-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    totaldrop_proto::TransferCode::parse(&cleaned)
        .ok()
        .map(|c| c.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_as_typed() {
        assert_eq!(
            normalize_code(" 7 Abandon  ability ".into()),
            Some("7-abandon-ability".into())
        );
        assert_eq!(normalize_code("seven words".into()), None);
    }

    #[test]
    fn codes_typed_with_arabic_indic_digits() {
        // Arabic, Persian and Urdu keyboards; the field then shows ASCII.
        for code in ["\u{0667}-abandon-ability", "\u{06F7} abandon ability"] {
            assert_eq!(
                normalize_code(code.into()),
                Some("7-abandon-ability".into()),
                "{code:?}"
            );
        }
    }

    #[test]
    fn codes_pasted_from_a_right_to_left_message() {
        // The share text isolates the code (U+2068 … U+2069); a copy of the
        // code alone can carry the isolates or other direction marks.
        for code in [
            "\u{2068}7-abandon-ability\u{2069}",
            "\u{2066}7 abandon ability\u{2069}",
            "\u{2067}7-abandon-ability\u{2069}\u{200F}",
            "\u{200E}7-\u{202D}abandon\u{202C}-ability\u{061C}",
            " \u{2068} 7-abandon-ability \u{2069} ",
            "\u{FEFF}7-aban\u{200B}don-ability\u{2060}",
        ] {
            assert_eq!(
                normalize_code(code.into()),
                Some("7-abandon-ability".into()),
                "{code:?}"
            );
        }
        assert_eq!(normalize_code("\u{2068}\u{2069}".into()), None);
    }
}
