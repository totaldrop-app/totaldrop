// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! `Transfer`, the QML object that sends and receives.
//!
//! A thin layer over `totaldrop-core`, like the Android bridge
//! (`app/rust/src/api/transfer.rs`): no protocol logic lives here. Work
//! runs on the background runtime; every event is queued onto the Qt
//! thread and lands in properties QML binds to.

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, Weak};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QList, QString, QStringList, QUrl};
use totaldrop_core::{
    Event, IncomingOffer, PathOptions, PreparedSend, ReceiveOptions, SendOptions, Stop,
};

use totaldrop_l10n::reveal_text;

use crate::bridge::stage::{Role, Stage};
use crate::desktop;
use crate::quit::TRANSFERS;
use crate::runtime::RUNTIME;
use crate::settings::Settings;

/// One folder is measured at a time, so dropping many folders does not
/// send many threads across the disk at once.
static MEASURING: LazyLock<tokio::sync::Semaphore> =
    LazyLock::new(|| tokio::sync::Semaphore::new(1));

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        /// Qt string list.
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        /// List of URLs, as drag and drop hands them over.
        type QList_QUrl = cxx_qt_lib::QList<cxx_qt_lib::QUrl>;
    }

    extern "RustQt" {
        /// One send or receive at a time.
        #[qobject]
        #[qml_element]
        // Read-only for QML: set only from `Stage` and `Role` (#343).
        #[qproperty(QString, state, READ, NOTIFY)]
        #[qproperty(QString, role, READ, NOTIFY)]
        #[qproperty(QString, code)]
        #[qproperty(QString, link)]
        #[qproperty(QString, qr)]
        #[qproperty(i32, qr_size, cxx_name = "qrSize")]
        #[qproperty(QStringList, words)]
        #[qproperty(f64, done)]
        #[qproperty(f64, total)]
        #[qproperty(i32, items)]
        #[qproperty(f64, size)]
        #[qproperty(i32, offer_files, cxx_name = "offerFiles")]
        #[qproperty(QStringList, offer_names, cxx_name = "offerNames")]
        #[qproperty(i32, files_ok, cxx_name = "filesOk")]
        #[qproperty(i32, files_failed, cxx_name = "filesFailed")]
        #[qproperty(QString, saved_to, cxx_name = "savedTo")]
        // The same folder as a `file://` URL, for "Open folder" (#427).
        #[qproperty(QString, saved_to_url, cxx_name = "savedToUrl")]
        #[qproperty(QString, error_kind, cxx_name = "errorKind")]
        #[qproperty(QString, error_detail, cxx_name = "errorDetail")]
        #[qproperty(QString, error_cause, cxx_name = "errorCause")]
        #[qproperty(QString, server)]
        #[qproperty(QString, default_server, cxx_name = "defaultServer")]
        #[qproperty(bool, privacy_mode, cxx_name = "privacyMode")]
        #[qproperty(QString, destination)]
        #[qproperty(bool, is_text, cxx_name = "isText")]
        #[qproperty(QString, received_text, cxx_name = "receivedText")]
        #[qproperty(QString, shown_text, cxx_name = "shownText")]
        #[qproperty(bool, shown_short, cxx_name = "shownShort")]
        #[qproperty(bool, shown_hidden, cxx_name = "shownHidden")]
        type Transfer = super::TransferRust;

        /// Scan `paths`, reserve a code, then wait for the receiver and send.
        #[qinvokable]
        #[cxx_name = "send"]
        fn send(self: Pin<&mut Transfer>, paths: &QStringList, strong: bool);

        /// Like `send`, for a text the receiver shows instead of saving.
        #[qinvokable]
        #[cxx_name = "sendText"]
        fn send_text(self: Pin<&mut Transfer>, text: &QString, strong: bool);

        /// The length of `text` in UTF-8 bytes, as the text limit counts.
        #[qinvokable]
        #[cxx_name = "textBytes"]
        fn text_bytes(self: &Transfer, text: &QString) -> f64;

        /// The largest text `sendText` sends, in bytes.
        #[qinvokable]
        #[cxx_name = "maxText"]
        fn max_text(self: &Transfer) -> f64;

        /// The most characters of a text a text box shows: Qt lays out a
        /// long line in a time that grows faster than its length.
        #[qinvokable]
        #[cxx_name = "shownChars"]
        fn shown_chars(self: &Transfer) -> i32;

        /// The clipboard's text exactly, or empty (#326).
        #[qinvokable]
        #[cxx_name = "clipboardText"]
        fn clipboard_text(self: &Transfer) -> QString;

        /// Put `text` on the clipboard exactly as it is.
        #[qinvokable]
        #[cxx_name = "copyText"]
        fn copy_text(self: &Transfer, text: &QString);

        /// Connect with a code and show the offer.
        #[qinvokable]
        #[cxx_name = "receive"]
        fn receive(self: Pin<&mut Transfer>, code: &QString);

        /// Accept the offer shown.
        #[qinvokable]
        fn accept(self: Pin<&mut Transfer>);

        /// Decline the offer shown.
        #[qinvokable]
        fn decline(self: Pin<&mut Transfer>);

        /// Stop whatever runs and return to idle.
        #[qinvokable]
        fn cancel(self: Pin<&mut Transfer>);

        /// Back to idle after a result.
        #[qinvokable]
        fn reset(self: Pin<&mut Transfer>);

        /// The code as typed, normalized, or empty if it cannot be a code.
        #[qinvokable]
        #[cxx_name = "normalizeCode"]
        fn normalize_code(self: &Transfer, input: &QString) -> QString;

        /// Code words starting with `prefix`, at most `limit`.
        #[qinvokable]
        fn completions(self: &Transfer, prefix: &QString, limit: i32) -> QStringList;

        /// A code inside a TotalDrop link or QR payload, or empty.
        #[qinvokable]
        #[cxx_name = "codeFromLink"]
        fn code_from_link(self: &Transfer, text: &QString) -> QString;

        /// `name \t bytes \t is-folder` for a local path or file URL; a
        /// folder is -1 bytes until `measure` answers.
        #[qinvokable]
        fn describe(self: &Transfer, path: &QString) -> QString;

        /// Measure a folder's size off the GUI thread; answers with
        /// `measured`.
        #[qinvokable]
        fn measure(self: Pin<&mut Transfer>, path: &QString);

        /// Local paths for dropped URLs.
        #[qinvokable]
        #[cxx_name = "localPaths"]
        fn local_paths(self: &Transfer, urls: &QList_QUrl) -> QStringList;

        /// Open the desktop's file chooser; answers with `picked`.
        #[qinvokable]
        #[cxx_name = "pickFiles"]
        fn pick_files(self: Pin<&mut Transfer>, title: &QString);

        /// Choose an image and read a QR code from it; answers with
        /// `codeFound` or `codeNotFound`.
        #[qinvokable]
        #[cxx_name = "readQrImage"]
        fn read_qr_image(self: Pin<&mut Transfer>, title: &QString);

        /// Save the rendezvous server. False if it is not a wss:// URL.
        #[qinvokable]
        #[cxx_name = "saveServer"]
        fn save_server(self: Pin<&mut Transfer>, url: &QString) -> bool;

        /// Turn privacy mode on or off and keep the choice. False, and
        /// unchanged, if the choice could not be saved.
        #[qinvokable]
        #[cxx_name = "savePrivacyMode"]
        fn save_privacy_mode(self: Pin<&mut Transfer>, on: bool) -> bool;

        /// Back to the default server.
        #[qinvokable]
        #[cxx_name = "resetServer"]
        fn reset_server(self: Pin<&mut Transfer>);

        /// A desktop notification, for when the window is in the background.
        #[qinvokable]
        fn notify(self: &Transfer, title: &QString, body: &QString);

        /// Files chosen in the file chooser.
        #[qsignal]
        fn picked(self: Pin<&mut Transfer>, paths: QStringList);

        /// A code was read from an image.
        #[qsignal]
        #[cxx_name = "codeFound"]
        fn code_found(self: Pin<&mut Transfer>, code: QString);

        /// The image held no TotalDrop code.
        #[qsignal]
        #[cxx_name = "codeNotFound"]
        fn code_not_found(self: Pin<&mut Transfer>);

        /// The size of a folder passed to `measure`.
        #[qsignal]
        fn measured(self: Pin<&mut Transfer>, path: QString, bytes: f64);
    }

    impl cxx_qt::Threading for Transfer {}
    impl cxx_qt::Initialize for Transfer {}
}

/// What lives between invokables: the pending offer and the stop of the
/// current operation.
#[derive(Default)]
struct Session {
    offer: Mutex<Option<IncomingOffer>>,
    /// Stops the current transfer at any stage; once the session is up the
    /// other side is told, and partial files are removed.
    stop: Mutex<Stop>,
    /// Bumped by every new operation and by cancel; events of an older
    /// operation are dropped.
    generation: AtomicU64,
}

/// The sessions of every `Transfer` the window made, so quitting can stop
/// them (#433).
static SESSIONS: Mutex<Vec<Weak<Session>>> = Mutex::new(Vec::new());

/// Stop every running transfer and decline every unanswered offer, as
/// Cancel does. The window has closed: called once its event loop ended,
/// before the process waits for [`TRANSFERS`] and exits. What the stopped
/// work queues for the window is never shown.
pub fn stop_all() {
    let sessions: Vec<Arc<Session>> = SESSIONS
        .lock()
        .map(|s| s.iter().filter_map(Weak::upgrade).collect())
        .unwrap_or_default();
    for session in sessions {
        session.generation.fetch_add(1, Ordering::SeqCst);
        if let Ok(s) = session.stop.lock() {
            s.stop();
        }
        let pending = session.offer.lock().ok().and_then(|mut o| o.take());
        if let Some(offer) = pending {
            TRANSFERS.spawn(&RUNTIME, async move {
                let _ = offer.decline().await;
            });
        }
    }
}

/// Backing state of [`qobject::Transfer`].
#[derive(Default)]
pub struct TransferRust {
    /// [`Self::stage`] as QML reads it; set only by `set_stage`.
    state: QString,
    /// [`Self::side`] as QML reads it; set only by `set_side`.
    role: QString,
    stage: Stage,
    side: Role,
    code: QString,
    link: QString,
    qr: QString,
    qr_size: i32,
    words: QStringList,
    done: f64,
    total: f64,
    items: i32,
    size: f64,
    offer_files: i32,
    offer_names: QStringList,
    files_ok: i32,
    files_failed: i32,
    saved_to: QString,
    saved_to_url: QString,
    error_kind: QString,
    error_detail: QString,
    error_cause: QString,
    server: QString,
    default_server: QString,
    privacy_mode: bool,
    destination: QString,
    /// This transfer is a text (#326): sent with `sendText`, or an offer
    /// that accepting shows instead of saving.
    is_text: bool,
    /// The text received, as it came: what Copy copies.
    received_text: QString,
    /// The text received as it is shown: hidden characters named, and at
    /// most [`SHOWN_CHARS`] characters.
    shown_text: QString,
    /// `shown_text` is only the beginning of the text.
    shown_short: bool,
    /// The text holds characters `shown_text` names by their code.
    shown_hidden: bool,
    session: Arc<Session>,
}

/// The most characters of a text shown in a text box (#326). Qt lays out
/// a line of n characters in a time that grows faster than n: 16 Ki take
/// some 40 ms, 1 Mi with no line break nearly a minute, during which the
/// window does not answer. The whole text is still sent and copied.
const SHOWN_CHARS: usize = 16 * 1024;

/// How a received text is shown.
#[derive(Debug, PartialEq, Eq)]
struct Shown {
    /// Hidden characters named ([`reveal_text`], which can make the text
    /// up to eight times longer), then cut to [`SHOWN_CHARS`] characters.
    text: String,
    /// It was cut.
    short: bool,
    /// Some characters are named by their code.
    hidden: bool,
}

fn shown(text: &str) -> Shown {
    let revealed = reveal_text(text);
    // Only named characters make it differ; \r\n is shown as \n.
    let hidden = revealed != text.replace("\r\n", "\n");
    match revealed.char_indices().nth(SHOWN_CHARS) {
        Some((at, _)) => Shown {
            text: revealed[..at].to_owned(),
            short: true,
            hidden,
        },
        None => Shown {
            text: revealed,
            short: false,
            hidden,
        },
    }
}

/// What a send sends.
enum Source {
    Paths(Vec<PathBuf>),
    Text(String),
}

type Thread = cxx_qt::CxxQtThread<qobject::Transfer>;

fn list(items: impl IntoIterator<Item = impl AsRef<str>>) -> QStringList {
    items
        .into_iter()
        .map(|s| QString::from(s.as_ref()))
        .collect()
}

fn strings(l: &QStringList) -> Vec<String> {
    QList::<QString>::from(l)
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn path_of(s: &str) -> PathBuf {
    match s.strip_prefix("file://") {
        Some(rest) => PathBuf::from(desktop::percent_decode(rest)),
        None => PathBuf::from(s),
    }
}

/// The error kinds QML knows (Strings: `error` + kind).
fn kind(e: &totaldrop_core::Error) -> &'static str {
    use totaldrop_core::Error as E;
    match e {
        E::WrongCode => "WrongCode",
        E::PeerLeft => "PeerLeft",
        E::Rejected(_) => "Declined",
        E::Cancelled(_) => "Cancelled",
        E::Stopped => "Stopped",
        E::Code(_) => "InvalidCode",
        E::Rendezvous(_) if e.is_bad_server_address() => "BadServer",
        E::Rendezvous(_) => "Server",
        E::Connect(_) => "Connect",
        E::InvalidOffer(_) => "InvalidOffer",
        E::Io(_) => "Storage",
        _ => "Other",
    }
}

/// [`kind`] for an error while the receiver connects with the code. Only
/// there does "no sender uses this code" mean the code: a sender whose
/// code expired unused hears the same from the server.
fn connect_kind(e: &totaldrop_core::Error) -> &'static str {
    if e.is_code_not_in_use() {
        "CodeNotInUse"
    } else {
        kind(e)
    }
}

/// [`kind`] for an error while the sender waits and sends. What the server
/// says when no sender uses a code reaches the sender only from the waiting
/// step, when its own code expired with nobody coming.
fn send_kind(e: &totaldrop_core::Error) -> &'static str {
    if e.is_code_not_in_use() {
        "CodeExpired"
    } else {
        kind(e)
    }
}

fn to_f64(v: u64) -> f64 {
    // Byte counts stay far below 2^53.
    v as f64
}

fn count(v: usize) -> i32 {
    i32::try_from(v).unwrap_or(i32::MAX)
}

impl qobject::Transfer {
    /// Move to `stage`. The only place `state` is set (#343).
    fn set_stage(mut self: Pin<&mut Self>, stage: Stage) {
        let mut rust = self.as_mut().rust_mut();
        rust.stage = stage;
        rust.state = QString::from(stage.name());
        crate::sleep::awake().set(stage.keeps_awake());
        self.state_changed();
    }

    fn stage(&self) -> Stage {
        self.rust().stage
    }

    /// The only place `role` is set.
    fn set_side(mut self: Pin<&mut Self>, side: Role) {
        let mut rust = self.as_mut().rust_mut();
        rust.side = side;
        rust.role = QString::from(side.name());
        self.role_changed();
    }

    fn side(&self) -> Role {
        self.rust().side
    }

    /// Start a new operation; returns its generation.
    fn begin(mut self: Pin<&mut Self>, side: Role, stage: Stage) -> u64 {
        let g = self
            .rust()
            .session
            .generation
            .fetch_add(1, Ordering::SeqCst)
            + 1;
        // Whatever ran before stops, and an offer it left is declined.
        if let Ok(mut s) = self.rust().session.stop.lock() {
            std::mem::replace(&mut *s, Stop::new()).stop();
        }
        let left = self
            .rust()
            .session
            .offer
            .lock()
            .ok()
            .and_then(|mut o| o.take());
        if let Some(offer) = left {
            TRANSFERS.spawn(&RUNTIME, async move {
                let _ = offer.decline().await;
            });
        }
        self.as_mut().set_side(side);
        self.as_mut().set_code(QString::default());
        self.as_mut().set_link(QString::default());
        self.as_mut().set_qr(QString::default());
        self.as_mut().set_qr_size(0);
        self.as_mut().set_words(QStringList::default());
        self.as_mut().set_done(0.0);
        self.as_mut().set_total(0.0);
        self.as_mut().set_items(0);
        self.as_mut().set_size(0.0);
        self.as_mut().set_offer_files(0);
        self.as_mut().set_offer_names(QStringList::default());
        self.as_mut().set_files_ok(0);
        self.as_mut().set_files_failed(0);
        self.as_mut().set_saved_to(QString::default());
        self.as_mut().set_saved_to_url(QString::default());
        self.as_mut().set_error_kind(QString::default());
        self.as_mut().set_error_detail(QString::default());
        self.as_mut().set_error_cause(QString::default());
        self.as_mut().set_is_text(false);
        self.as_mut().set_received_text(QString::default());
        self.as_mut().set_shown_text(QString::default());
        self.as_mut().set_shown_short(false);
        self.as_mut().set_shown_hidden(false);
        self.as_mut().set_stage(stage);
        g
    }

    fn current(&self, g: u64) -> bool {
        self.rust().session.generation.load(Ordering::SeqCst) == g
    }

    /// `cause` names the explanation QML shows in the person's language
    /// (`cause` + it in Strings), next to the English `detail` (#212).
    fn fail(mut self: Pin<&mut Self>, kind: &str, detail: &str, cause: &str) {
        self.as_mut().set_error_kind(QString::from(kind));
        self.as_mut().set_error_detail(QString::from(detail));
        self.as_mut().set_error_cause(QString::from(cause));
        let stage = if kind == "Declined" && self.side() == Role::Send {
            Stage::Declined
        } else {
            Stage::Failed
        };
        self.as_mut().set_stage(stage);
    }

    /// Core events for operation `g`, queued onto the Qt thread.
    fn events(thread: Thread, g: u64) -> totaldrop_core::Events {
        let thread = Mutex::new(thread);
        Arc::new(move |event| {
            let Ok(t) = thread.lock() else { return };
            let _ = t.queue(move |mut obj| {
                if !obj.current(g) {
                    return;
                }
                match event {
                    Event::Waiting => obj.as_mut().set_stage(Stage::Waiting),
                    Event::Connected { words } => {
                        obj.as_mut().set_words(list(&words));
                        if obj.side() == Role::Send {
                            obj.as_mut().set_stage(Stage::Connected);
                        }
                    }
                    Event::Progress { done, total } => {
                        obj.as_mut().set_done(to_f64(done));
                        obj.as_mut().set_total(to_f64(total));
                        if obj.stage() != Stage::Transferring {
                            obj.as_mut().set_stage(Stage::Transferring);
                        }
                    }
                }
            });
        })
    }

    /// The stop of the current operation.
    fn stop_handle(&self) -> Stop {
        self.rust()
            .session
            .stop
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Run `fut` on the runtime, counted so quitting waits for it (#433).
    fn spawn(&self, fut: impl std::future::Future<Output = ()> + Send + 'static) {
        TRANSFERS.spawn(&RUNTIME, fut);
    }

    fn opts_server(&self) -> String {
        self.server().to_string()
    }

    /// The paths a transfer may take: the relay only in privacy mode.
    fn route(&self) -> PathOptions {
        PathOptions {
            relay_only: *self.privacy_mode(),
            ..PathOptions::default()
        }
    }

    fn save_privacy_mode(mut self: Pin<&mut Self>, on: bool) -> bool {
        let mut s = Settings::load();
        s.privacy_mode = on;
        // A choice that would quietly be gone after a restart is not made.
        if !s.save() {
            return false;
        }
        self.as_mut().set_privacy_mode(on);
        true
    }

    fn send(self: Pin<&mut Self>, paths: &QStringList, strong: bool) {
        let paths: Vec<PathBuf> = strings(paths).iter().map(|p| path_of(p)).collect();
        if paths.is_empty() {
            return;
        }
        self.start_send(Source::Paths(paths), strong);
    }

    fn send_text(self: Pin<&mut Self>, text: &QString, strong: bool) {
        let text = text.to_string();
        // QML does not offer an empty or too long text; the core refuses
        // one too.
        if text.is_empty() {
            return;
        }
        self.start_send(Source::Text(text), strong);
    }

    fn text_bytes(&self, text: &QString) -> f64 {
        to_f64(u64::try_from(text.to_string().len()).unwrap_or(u64::MAX))
    }

    fn max_text(&self) -> f64 {
        to_f64(totaldrop_core::MAX_TEXT)
    }

    fn shown_chars(&self) -> i32 {
        count(SHOWN_CHARS)
    }

    fn clipboard_text(&self) -> QString {
        crate::bridge::clipboard::clipboard_text()
    }

    fn copy_text(&self, text: &QString) {
        crate::bridge::clipboard::set_clipboard_text(text);
    }

    fn start_send(mut self: Pin<&mut Self>, source: Source, strong: bool) {
        let g = self.as_mut().begin(Role::Send, Stage::Preparing);
        self.as_mut().set_is_text(matches!(source, Source::Text(_)));
        let thread = self.qt_thread();
        let opts = SendOptions {
            server: self.opts_server(),
            words: if strong { 3 } else { 2 },
            paths: self.route(),
        };
        let stop = self.stop_handle();
        self.spawn(async move {
            // Dropping the preparation releases the reserved code at once.
            let prepared = tokio::select! {
                biased;
                () = stop.stopped() => Err(totaldrop_core::Error::Stopped),
                r = async {
                    match &source {
                        Source::Paths(paths) => PreparedSend::new(&opts, paths.clone()).await,
                        Source::Text(text) => {
                            PreparedSend::text(&opts, text, totaldrop_core::TEXT_FILE).await
                        }
                    }
                } => r,
            };
            let prepared = match prepared {
                Ok(p) => p,
                Err(e) => {
                    let (k, d, c) = (kind(&e), e.to_string(), e.cause().as_str());
                    let _ = thread.queue(move |obj| {
                        if obj.current(g) {
                            obj.fail(k, &d, c);
                        }
                    });
                    return;
                }
            };
            let code = prepared.code().to_string();
            let (items, bytes) = prepared.totals();
            let link = desktop::link_for(&code);
            let (qr, qr_size) = desktop::qr_modules(&link);
            let _ = thread.queue(move |mut obj| {
                if !obj.current(g) {
                    return;
                }
                obj.as_mut().set_code(QString::from(&code));
                obj.as_mut().set_link(QString::from(&link));
                obj.as_mut().set_qr(QString::from(&qr));
                obj.as_mut().set_qr_size(qr_size);
                obj.as_mut().set_items(count(items));
                obj.as_mut().set_size(to_f64(bytes));
                obj.as_mut().set_stage(Stage::Waiting);
            });
            let result = prepared
                .run_with(Self::events(thread.clone(), g), stop)
                .await;
            let _ = thread.queue(move |mut obj| {
                if !obj.current(g) {
                    return;
                }
                match result {
                    Ok(r) => {
                        obj.as_mut().set_files_ok(count(r.files_ok));
                        obj.as_mut().set_files_failed(count(r.files_failed));
                        obj.as_mut().set_stage(Stage::Done);
                    }
                    Err(e) => obj.fail(send_kind(&e), &e.to_string(), e.cause().as_str()),
                }
            });
        });
    }

    fn receive(mut self: Pin<&mut Self>, code: &QString) {
        let code = code.to_string();
        let g = self.as_mut().begin(Role::Receive, Stage::Connecting);
        let thread = self.qt_thread();
        let opts = ReceiveOptions {
            server: self.opts_server(),
            paths: self.route(),
        };
        let session = self.rust().session.clone();
        let stop = self.stop_handle();
        self.spawn(async move {
            match IncomingOffer::connect_with(&opts, &code, &stop).await {
                Ok(offer) => {
                    let s = offer.summary().clone();
                    let words = offer.words().to_vec();
                    // Cancel bumps the generation before it takes the offer,
                    // so an offer that comes back after a cancel is told to
                    // stop here instead of being kept.
                    let stale = match session.offer.lock() {
                        Ok(mut o) if session.generation.load(Ordering::SeqCst) == g => {
                            *o = Some(offer);
                            None
                        }
                        _ => Some(offer),
                    };
                    if let Some(offer) = stale {
                        let _ = offer.stop().await;
                        return;
                    }
                    let _ = thread.queue(move |mut obj| {
                        if !obj.current(g) {
                            return;
                        }
                        obj.as_mut().set_code(QString::from(&code));
                        obj.as_mut().set_offer_files(count(s.files));
                        obj.as_mut().set_items(count(s.files + s.dirs));
                        obj.as_mut().set_size(to_f64(s.total_size));
                        obj.as_mut().set_offer_names(list(&s.top_level));
                        obj.as_mut().set_words(list(&words));
                        obj.as_mut().set_is_text(s.text);
                        obj.as_mut().set_stage(Stage::Offer);
                    });
                }
                Err(e) => {
                    let (k, d, c) = (connect_kind(&e), e.to_string(), e.cause().as_str());
                    let _ = thread.queue(move |obj| {
                        if obj.current(g) {
                            obj.fail(k, &d, c);
                        }
                    });
                }
            }
        });
    }

    fn accept(mut self: Pin<&mut Self>) {
        // Only an offer waiting for an answer, as on Android.
        if self.stage() != Stage::Offer {
            return;
        }
        let Some(offer) = self
            .rust()
            .session
            .offer
            .lock()
            .ok()
            .and_then(|mut o| o.take())
        else {
            return;
        };
        let g = self.rust().session.generation.load(Ordering::SeqCst);
        self.as_mut().set_stage(Stage::Transferring);
        if offer.summary().text {
            self.accept_text(offer, g);
            return;
        }
        let thread = self.qt_thread();
        let dest = PathBuf::from(self.destination().to_string());
        let stop = self.stop_handle();
        self.spawn(async move {
            let result = match std::fs::create_dir_all(&dest) {
                Ok(()) => {
                    offer
                        .accept_with(&dest, Self::events(thread.clone(), g), &stop)
                        .await
                }
                Err(e) => Err(totaldrop_core::Error::Io(e)),
            };
            let saved = dest.display().to_string();
            let saved_url = desktop::file_url(&dest);
            let _ = thread.queue(move |mut obj| {
                if !obj.current(g) {
                    return;
                }
                match result {
                    Ok(r) => {
                        obj.as_mut().set_files_ok(count(r.files_ok));
                        obj.as_mut().set_files_failed(count(r.files_failed));
                        obj.as_mut().set_saved_to(QString::from(&saved));
                        obj.as_mut().set_saved_to_url(QString::from(&saved_url));
                        obj.as_mut().set_stage(Stage::Done);
                    }
                    Err(e) => obj.fail(kind(&e), &e.to_string(), e.cause().as_str()),
                }
            });
        });
    }

    /// Accept a text offer: kept in memory and shown, nothing written.
    fn accept_text(self: Pin<&mut Self>, offer: IncomingOffer, g: u64) {
        let thread = self.qt_thread();
        let stop = self.stop_handle();
        self.spawn(async move {
            let result = offer.accept_text_with(&stop).await;
            let _ = thread.queue(move |mut obj| {
                if !obj.current(g) {
                    return;
                }
                match result {
                    Ok(r) => match r.text {
                        Some(text) => {
                            obj.as_mut().set_files_ok(1);
                            let seen = shown(&text);
                            obj.as_mut()
                                .set_shown_text(QString::from(seen.text.as_str()));
                            obj.as_mut().set_shown_short(seen.short);
                            obj.as_mut().set_shown_hidden(seen.hidden);
                            obj.as_mut().set_received_text(QString::from(text.as_str()));
                            obj.as_mut().set_stage(Stage::Done);
                        }
                        // It failed its hash or is not UTF-8: not shown.
                        None => obj.fail("TextDamaged", "the text failed its check", ""),
                    },
                    Err(e) => obj.fail(kind(&e), &e.to_string(), e.cause().as_str()),
                }
            });
        });
    }

    fn decline(mut self: Pin<&mut Self>) {
        if self.stage() != Stage::Offer {
            return;
        }
        let Some(offer) = self
            .rust()
            .session
            .offer
            .lock()
            .ok()
            .and_then(|mut o| o.take())
        else {
            return;
        };
        self.as_mut().set_stage(Stage::Declined);
        TRANSFERS.spawn(&RUNTIME, async move {
            let _ = offer.decline().await;
        });
    }

    fn cancel(mut self: Pin<&mut Self>) {
        self.rust()
            .session
            .generation
            .fetch_add(1, Ordering::SeqCst);
        // The running operation tells the other side and cleans up.
        self.stop_handle().stop();
        // An unanswered offer is declined, so the sender hears why.
        let pending = self
            .rust()
            .session
            .offer
            .lock()
            .ok()
            .and_then(|mut o| o.take());
        if let Some(offer) = pending {
            TRANSFERS.spawn(&RUNTIME, async move {
                let _ = offer.decline().await;
            });
        }
        // A received text is kept no longer than it is on screen.
        self.as_mut().set_received_text(QString::default());
        self.as_mut().set_shown_text(QString::default());
        self.as_mut().set_shown_short(false);
        self.as_mut().set_shown_hidden(false);
        self.as_mut().set_is_text(false);
        self.as_mut().set_stage(Stage::Idle);
    }

    fn reset(mut self: Pin<&mut Self>) {
        self.as_mut().cancel();
    }

    fn normalize_code(&self, input: &QString) -> QString {
        QString::from(
            desktop::normalize_code(&input.to_string())
                .unwrap_or_default()
                .as_str(),
        )
    }

    fn completions(&self, prefix: &QString, limit: i32) -> QStringList {
        let limit = usize::try_from(limit).unwrap_or(0);
        list(desktop::completions(&prefix.to_string(), limit))
    }

    fn code_from_link(&self, text: &QString) -> QString {
        QString::from(
            desktop::code_from_link(&text.to_string())
                .unwrap_or_default()
                .as_str(),
        )
    }

    fn describe(&self, path: &QString) -> QString {
        let p = path_of(&path.to_string());
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        // A folder's size can take minutes to add up; it comes later, by
        // `measure`, and is -1 until then.
        let (bytes, dir) = match std::fs::metadata(&p) {
            Ok(m) if m.is_dir() => (-1, true),
            Ok(m) => (i64::try_from(m.len()).unwrap_or(i64::MAX), false),
            Err(_) => (0, false),
        };
        QString::from(format!("{name}\t{bytes}\t{}", u8::from(dir)).as_str())
    }

    fn measure(self: Pin<&mut Self>, path: &QString) {
        let thread = self.qt_thread();
        let path = path.to_string();
        RUNTIME.spawn(async move {
            let Ok(_turn) = MEASURING.acquire().await else {
                return;
            };
            let dir = path_of(&path);
            let bytes = tokio::task::spawn_blocking(move || desktop::folder_size(&dir))
                .await
                .unwrap_or(0);
            let _ = thread.queue(move |obj| obj.measured(QString::from(&path), to_f64(bytes)));
        });
    }

    fn local_paths(&self, urls: &QList<QUrl>) -> QStringList {
        list(
            urls.iter()
                .map(|u| u.to_string())
                .filter(|u| u.starts_with("file://"))
                .map(|u| path_of(&u).display().to_string()),
        )
    }

    fn pick_files(self: Pin<&mut Self>, title: &QString) {
        let thread = self.qt_thread();
        let title = title.to_string();
        RUNTIME.spawn(async move {
            let paths = desktop::pick_files(&title).await;
            if !paths.is_empty() {
                let _ = thread.queue(move |obj| obj.picked(list(&paths)));
            }
        });
    }

    fn read_qr_image(self: Pin<&mut Self>, title: &QString) {
        let thread = self.qt_thread();
        let title = title.to_string();
        RUNTIME.spawn(async move {
            let Some(image) = desktop::pick_image(&title).await else {
                return;
            };
            let code = tokio::task::spawn_blocking(move || desktop::read_qr(&image))
                .await
                .ok()
                .flatten()
                .and_then(|text| desktop::code_from_link(&text));
            let _ = thread.queue(move |obj| match code {
                Some(c) => obj.code_found(QString::from(&c)),
                None => obj.code_not_found(),
            });
        });
    }

    fn save_server(mut self: Pin<&mut Self>, url: &QString) -> bool {
        let url = url.to_string();
        if !Settings::valid_server(&url) {
            return false;
        }
        let mut s = Settings::load();
        s.server = Some(url.clone());
        let _ = s.save();
        self.as_mut().set_server(QString::from(&url));
        true
    }

    fn reset_server(mut self: Pin<&mut Self>) {
        let mut s = Settings::load();
        s.server = None;
        let _ = s.save();
        self.as_mut()
            .set_server(QString::from(totaldrop_core::rendezvous::DEFAULT_SERVER));
    }

    fn notify(&self, title: &QString, body: &QString) {
        let (t, b) = (title.to_string(), body.to_string());
        RUNTIME.spawn(desktop::notify(t, b));
    }
}

impl cxx_qt::Initialize for qobject::Transfer {
    fn initialize(mut self: Pin<&mut Self>) {
        if let Ok(mut sessions) = SESSIONS.lock() {
            sessions.retain(|s| s.strong_count() > 0);
            sessions.push(Arc::downgrade(&self.rust().session));
        }
        let s = Settings::load();
        let default = totaldrop_core::rendezvous::DEFAULT_SERVER;
        self.as_mut().set_default_server(QString::from(default));
        self.as_mut()
            .set_server(QString::from(s.server.as_deref().unwrap_or(default)));
        self.as_mut().set_privacy_mode(s.privacy_mode);
        self.as_mut().set_destination(QString::from(
            desktop::downloads().display().to_string().as_str(),
        ));
        self.as_mut().set_stage(Stage::Idle);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use super::{SESSIONS, SHOWN_CHARS, Session, connect_kind, kind, send_kind, shown, stop_all};
    use totaldrop_core::Error as E;

    /// Quitting stops a live transfer as Cancel does: its stop is set,
    /// and its generation moves on, so what it still queues for the
    /// window is dropped (#433). No Qt object is needed: the window's
    /// `Transfer` only registers its session.
    #[test]
    fn quitting_stops_every_live_transfer() {
        let session = Arc::new(Session::default());
        if let Ok(mut sessions) = SESSIONS.lock() {
            sessions.push(Arc::downgrade(&session));
        }
        let stop = session.stop.lock().map(|s| s.clone()).unwrap_or_default();
        let before = session.generation.load(Ordering::SeqCst);
        stop_all();
        assert!(stop.is_stopped(), "the transfer was stopped");
        assert_ne!(
            session.generation.load(Ordering::SeqCst),
            before,
            "its later events are dropped"
        );
    }

    #[test]
    fn a_code_not_in_use_is_not_a_server_problem() {
        // What the rendezvous server says when no sender uses the number.
        for reason in ["unknown nameplate", "nameplate full", "mailbox expired"] {
            assert_eq!(
                connect_kind(&E::Rendezvous(reason.into())),
                "CodeNotInUse",
                "{reason}"
            );
        }
        assert_eq!(
            connect_kind(&E::Rendezvous("connection refused".into())),
            "Server"
        );
    }

    #[test]
    fn a_sender_is_told_its_code_expired() {
        // A sender's mailbox expires when nobody comes (#164).
        assert_eq!(
            send_kind(&E::Rendezvous("mailbox expired".into())),
            "CodeExpired"
        );
        assert_eq!(
            send_kind(&E::Rendezvous("connection refused".into())),
            "Server"
        );
    }

    /// 1 MiB of one line, or of control characters each shown as eight,
    /// froze the window for minutes (#326 review): only the beginning is
    /// shown, cut between characters.
    #[test]
    fn a_long_text_is_shown_in_part() {
        let plain = shown("kısa\r\nmetin\t\n");
        assert_eq!(plain.text, "kısa\nmetin\t\n");
        assert!(!plain.short && !plain.hidden);
        let full = shown(&"ğ".repeat(SHOWN_CHARS));
        assert!(!full.short);
        assert_eq!(full.text.chars().count(), SHOWN_CHARS);
        let cut = shown(&"\u{1}".repeat(1 << 20));
        assert!(cut.short && cut.hidden);
        assert_eq!(cut.text.chars().count(), SHOWN_CHARS);
        assert!(cut.text.starts_with("<U+0001><U+0001>"));
        assert!(shown("a\u{202E}b").hidden);
    }

    #[test]
    fn a_bad_server_address_is_named() {
        let e = E::Rendezvous("the server address must start with wss://".into());
        assert_eq!(kind(&e), "BadServer");
    }
}
