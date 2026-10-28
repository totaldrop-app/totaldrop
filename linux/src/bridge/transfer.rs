//! `Transfer`, the QML object that sends and receives.
//!
//! A thin layer over `totaldrop-core`, like the Android bridge
//! (`app/rust/src/api/transfer.rs`): no protocol logic lives here. Work
//! runs on the background runtime; every event is queued onto the Qt
//! thread and lands in properties QML binds to.

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QList, QString, QStringList, QUrl};
use totaldrop_core::{Event, IncomingOffer, PreparedSend, ReceiveOptions, SendOptions};

use crate::desktop;
use crate::runtime::RUNTIME;
use crate::settings::Settings;

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
        #[qproperty(QString, state)]
        #[qproperty(QString, role)]
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
        #[qproperty(QString, error_kind, cxx_name = "errorKind")]
        #[qproperty(QString, error_detail, cxx_name = "errorDetail")]
        #[qproperty(QString, server)]
        #[qproperty(QString, default_server, cxx_name = "defaultServer")]
        #[qproperty(QString, destination)]
        type Transfer = super::TransferRust;

        /// Scan `paths`, reserve a code, then wait for the receiver and send.
        #[qinvokable]
        #[cxx_name = "send"]
        fn send(self: Pin<&mut Transfer>, paths: &QStringList, strong: bool);

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

        /// `name \t bytes \t is-folder` for a local path or file URL.
        #[qinvokable]
        fn describe(self: &Transfer, path: &QString) -> QString;

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
    }

    impl cxx_qt::Threading for Transfer {}
    impl cxx_qt::Initialize for Transfer {}
}

/// What lives between invokables: the pending send or offer and the handle
/// to stop the running task.
#[derive(Default)]
struct Session {
    offer: Mutex<Option<IncomingOffer>>,
    abort: Mutex<Option<tokio::task::AbortHandle>>,
    /// Bumped by every new operation and by cancel; events of an older
    /// operation are dropped.
    generation: AtomicU64,
}

/// Backing state of [`qobject::Transfer`].
#[derive(Default)]
pub struct TransferRust {
    state: QString,
    role: QString,
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
    error_kind: QString,
    error_detail: QString,
    server: QString,
    default_server: QString,
    destination: QString,
    session: Arc<Session>,
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
        E::Code(_) => "InvalidCode",
        E::Rendezvous(_) => "Server",
        E::Connect(_) => "Connect",
        E::InvalidOffer(_) => "InvalidOffer",
        E::Io(_) => "Storage",
        _ => "Other",
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
    /// Start a new operation; returns its generation.
    fn begin(mut self: Pin<&mut Self>, role: &str, state: &str) -> u64 {
        let g = self
            .rust()
            .session
            .generation
            .fetch_add(1, Ordering::SeqCst)
            + 1;
        self.as_mut().set_role(QString::from(role));
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
        self.as_mut().set_error_kind(QString::default());
        self.as_mut().set_error_detail(QString::default());
        self.as_mut().set_state(QString::from(state));
        g
    }

    fn current(&self, g: u64) -> bool {
        self.rust().session.generation.load(Ordering::SeqCst) == g
    }

    fn fail(mut self: Pin<&mut Self>, kind: &str, detail: &str) {
        self.as_mut().set_error_kind(QString::from(kind));
        self.as_mut().set_error_detail(QString::from(detail));
        let state = if kind == "Declined" && self.role().to_string() == "send" {
            "declined"
        } else {
            "failed"
        };
        self.as_mut().set_state(QString::from(state));
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
                    Event::Waiting => obj.as_mut().set_state(QString::from("waiting")),
                    Event::Connected { words } => {
                        obj.as_mut().set_words(list(&words));
                        if obj.role().to_string() == "send" {
                            obj.as_mut().set_state(QString::from("connected"));
                        }
                    }
                    Event::Progress { done, total } => {
                        obj.as_mut().set_done(to_f64(done));
                        obj.as_mut().set_total(to_f64(total));
                        if obj.state().to_string() != "transferring" {
                            obj.as_mut().set_state(QString::from("transferring"));
                        }
                    }
                }
            });
        })
    }

    /// Run `fut` on the runtime as operation `g`, keeping its abort handle.
    fn spawn(&self, fut: impl std::future::Future<Output = ()> + Send + 'static) {
        let task = RUNTIME.spawn(fut);
        if let Ok(mut a) = self.rust().session.abort.lock() {
            *a = Some(task.abort_handle());
        }
    }

    fn opts_server(&self) -> String {
        self.server().to_string()
    }

    fn send(mut self: Pin<&mut Self>, paths: &QStringList, strong: bool) {
        let paths: Vec<PathBuf> = strings(paths).iter().map(|p| path_of(p)).collect();
        if paths.is_empty() {
            return;
        }
        let g = self.as_mut().begin("send", "preparing");
        let thread = self.qt_thread();
        let opts = SendOptions {
            server: self.opts_server(),
            words: if strong { 3 } else { 2 },
            ..SendOptions::default()
        };
        self.spawn(async move {
            let prepared = match PreparedSend::new(&opts, paths).await {
                Ok(p) => p,
                Err(e) => {
                    let (k, d) = (kind(&e), e.to_string());
                    let _ = thread.queue(move |obj| {
                        if obj.current(g) {
                            obj.fail(k, &d);
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
                obj.as_mut().set_state(QString::from("waiting"));
            });
            let result = prepared.run(Self::events(thread.clone(), g)).await;
            let _ = thread.queue(move |mut obj| {
                if !obj.current(g) {
                    return;
                }
                match result {
                    Ok(r) => {
                        obj.as_mut().set_files_ok(count(r.files_ok));
                        obj.as_mut().set_files_failed(count(r.files_failed));
                        obj.as_mut().set_state(QString::from("done"));
                    }
                    Err(e) => obj.fail(kind(&e), &e.to_string()),
                }
            });
        });
    }

    fn receive(mut self: Pin<&mut Self>, code: &QString) {
        let code = code.to_string();
        let g = self.as_mut().begin("receive", "connecting");
        let thread = self.qt_thread();
        let opts = ReceiveOptions {
            server: self.opts_server(),
            ..ReceiveOptions::default()
        };
        let session = self.rust().session.clone();
        self.spawn(async move {
            match IncomingOffer::connect(&opts, &code).await {
                Ok(offer) => {
                    let s = offer.summary().clone();
                    let words = offer.words().to_vec();
                    if let Ok(mut o) = session.offer.lock() {
                        *o = Some(offer);
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
                        obj.as_mut().set_state(QString::from("offer"));
                    });
                }
                Err(e) => {
                    let (k, d) = (kind(&e), e.to_string());
                    let _ = thread.queue(move |obj| {
                        if obj.current(g) {
                            obj.fail(k, &d);
                        }
                    });
                }
            }
        });
    }

    fn accept(mut self: Pin<&mut Self>) {
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
        self.as_mut().set_state(QString::from("transferring"));
        let thread = self.qt_thread();
        let dest = PathBuf::from(self.destination().to_string());
        self.spawn(async move {
            let result = match std::fs::create_dir_all(&dest) {
                Ok(()) => offer.accept(&dest, Self::events(thread.clone(), g)).await,
                Err(e) => Err(totaldrop_core::Error::Io(e)),
            };
            let saved = dest.display().to_string();
            let _ = thread.queue(move |mut obj| {
                if !obj.current(g) {
                    return;
                }
                match result {
                    Ok(r) => {
                        obj.as_mut().set_files_ok(count(r.files_ok));
                        obj.as_mut().set_files_failed(count(r.files_failed));
                        obj.as_mut().set_saved_to(QString::from(&saved));
                        obj.as_mut().set_state(QString::from("done"));
                    }
                    Err(e) => obj.fail(kind(&e), &e.to_string()),
                }
            });
        });
    }

    fn decline(mut self: Pin<&mut Self>) {
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
        self.as_mut().set_state(QString::from("declined"));
        RUNTIME.spawn(async move {
            let _ = offer.decline().await;
        });
    }

    fn cancel(mut self: Pin<&mut Self>) {
        self.rust()
            .session
            .generation
            .fetch_add(1, Ordering::SeqCst);
        if let Some(h) = self
            .rust()
            .session
            .abort
            .lock()
            .ok()
            .and_then(|mut a| a.take())
        {
            h.abort();
        }
        // Dropping an unanswered offer closes the connection; the sender
        // sees the transfer end.
        if let Ok(mut o) = self.rust().session.offer.lock() {
            o.take();
        }
        self.as_mut().set_state(QString::from("idle"));
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
        let (bytes, dir) = match std::fs::metadata(&p) {
            Ok(m) if m.is_dir() => (desktop::folder_size(&p), true),
            Ok(m) => (m.len(), false),
            Err(_) => (0, false),
        };
        QString::from(format!("{name}\t{bytes}\t{}", u8::from(dir)).as_str())
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
        s.save();
        self.as_mut().set_server(QString::from(&url));
        true
    }

    fn reset_server(mut self: Pin<&mut Self>) {
        let mut s = Settings::load();
        s.server = None;
        s.save();
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
        let s = Settings::load();
        let default = totaldrop_core::rendezvous::DEFAULT_SERVER;
        self.as_mut().set_default_server(QString::from(default));
        self.as_mut()
            .set_server(QString::from(s.server.as_deref().unwrap_or(default)));
        self.as_mut().set_destination(QString::from(
            desktop::downloads().display().to_string().as_str(),
        ));
        self.as_mut().set_state(QString::from("idle"));
    }
}
