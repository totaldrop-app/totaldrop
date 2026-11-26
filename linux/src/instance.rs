// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! One window per session (#178).
//!
//! The first `totaldrop-desktop` owns the session-bus name
//! `app.totaldrop.TotalDrop` and serves `org.freedesktop.Application` on
//! `/app/totaldrop/TotalDrop`. A later launch finds the name taken, hands
//! its files, link and `--screen` to the running window over that interface
//! and exits, so "Open with TotalDrop" in a file manager adds the files to
//! the open window instead of starting a second one.
//!
//! The command line is read here for both: the first window gets the same
//! list (`Request::args`) that a later launch sends it.
//!
//! The Flatpak and the distribution package (.pkg, .deb, .rpm) share the
//! app id, so they share this name too, and only one of them runs at a
//! time (#351). The window also says which package it is
//! (`app.totaldrop.TotalDrop.Instance.Package`), so a launch of the other
//! package can say why no window of its own opened.

use std::collections::{HashMap, VecDeque};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::Duration;

use zbus::OwnedVariant;
use zbus::fdo::{RequestNameFlags, RequestNameReply};

/// The object path of the interface, the app id with `/` for `.`.
pub const OBJECT_PATH: &str = "/app/totaldrop/TotalDrop";
const INTERFACE: &str = "org.freedesktop.Application";
/// TotalDrop's own interface next to it: which package the window is.
const INSTANCE_INTERFACE: &str = "app.totaldrop.TotalDrop.Instance";

/// The screens `--screen=` opens; `clipboard` is Send with the
/// clipboard's text filled in (#326).
const SCREENS: [&str; 4] = ["send", "clipboard", "receive", "settings"];

/// Qt's own options that take the next argument as their value; that value
/// is not a file to send.
const QT_VALUE_OPTIONS: [&str; 16] = [
    "platform",
    "platformpluginpath",
    "platformtheme",
    "plugin",
    "qmljsdebugger",
    "style",
    "stylesheet",
    "session",
    "display",
    "geometry",
    "title",
    "name",
    "qwindowgeometry",
    "qwindowtitle",
    "qwindowicon",
    "icon",
];

/// Limits on what another process may hand the window: every item is
/// looked at on the GUI thread.
const MAX_ARGS: usize = 1024;
const MAX_ARG_BYTES: usize = 16 * 1024;
const MAX_CALL_BYTES: usize = 1024 * 1024;
const MAX_TOKEN_BYTES: usize = 1024;
/// Requests kept while the window is not yet listening.
const MAX_PENDING: usize = 16;

/// Platform-data key for the screen, which `Open` has no argument for.
const SCREEN_KEY: &str = "app.totaldrop.screen";

/// Flag [`Request::args`] adds when a Flatpak window drops paths it cannot
/// read (#357). Options from outside never enter `items`, so only `args`
/// produces it.
const FLATPAK_UNREADABLE_ARG: &str = "--flatpak-unreadable";

/// What a launch asks the window to show.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Request {
    /// Absolute paths of local files, and anything else that may be a
    /// TotalDrop link or code, in command-line order.
    pub items: Vec<String>,
    /// `send`, `receive` or `settings`.
    pub screen: Option<String>,
    /// At least one local path could not be read inside the Flatpak
    /// sandbox and was left out (#357).
    pub unreadable_in_flatpak: bool,
}

impl Request {
    /// Read the command line (without the program name). `cwd` resolves
    /// relative paths: a later launch's working directory is not the
    /// window's.
    pub fn from_args(args: impl IntoIterator<Item = OsString>, cwd: &Path) -> Self {
        let mut out = Self::default();
        let mut args = args.into_iter();
        let mut options = true;
        while let Some(a) = args.next() {
            if options {
                if a == "--" {
                    options = false;
                    continue;
                }
                if let Some(s) = a.to_str()
                    && s.len() > 1
                    && s.starts_with('-')
                {
                    if let Some(screen) = s.strip_prefix("--screen=") {
                        if SCREENS.contains(&screen) {
                            out.screen = Some(screen.to_owned());
                        }
                    } else if QT_VALUE_OPTIONS.contains(&s.trim_start_matches('-')) {
                        args.next();
                    }
                    continue;
                }
            }
            if let Some(item) = item(&a, cwd) {
                out.items.push(item);
            }
        }
        out
    }

    /// The list QML reads: the items, `--flatpak-unreadable` when the
    /// sandbox could not read a handed-over path, then `--screen=…`.
    pub fn args(&self) -> Vec<String> {
        let mut out = self.items.clone();
        if self.unreadable_in_flatpak {
            out.push(FLATPAK_UNREADABLE_ARG.to_owned());
        }
        if let Some(s) = &self.screen {
            out.push(format!("--screen={s}"));
        }
        out
    }

    /// The URIs for `Open`: local paths as `file://` URLs.
    fn uris(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|i| {
                if i.starts_with('/') {
                    crate::desktop::file_url(Path::new(i))
                } else {
                    i.clone()
                }
            })
            .collect()
    }

    /// What `Open` received, checked: it may come from any process of
    /// this person's session.
    fn from_open(uris: &[String], screen: Option<&str>) -> Self {
        let mut bytes = 0;
        let items = uris
            .iter()
            .filter(|u| !u.is_empty() && u.len() <= MAX_ARG_BYTES)
            // Options are not items: the screen comes in platform data.
            .filter(|u| !u.starts_with('-'))
            .take_while(|u| {
                bytes += u.len();
                bytes <= MAX_CALL_BYTES
            })
            .take(MAX_ARGS)
            .filter_map(|u| {
                if u.starts_with("file:") {
                    local_path(u)
                } else {
                    Some(u.clone())
                }
            })
            .collect();
        Self {
            items,
            screen: screen.filter(|s| SCREENS.contains(s)).map(str::to_owned),
            unreadable_in_flatpak: false,
        }
    }
}

/// A command-line argument as a list item: an absolute path for a file,
/// otherwise as given (QML reads links and codes from it). Empty and
/// non-UTF-8 arguments that name no file are dropped.
fn item(a: &OsStr, cwd: &Path) -> Option<String> {
    if a.is_empty() {
        return None;
    }
    let path = Path::new(a);
    let local = if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        Some(cwd.join(path)).filter(|p| p.exists())
    };
    match (local, a.to_str()) {
        // A file in this folder, even one called `a:b.txt`.
        (Some(p), _) if !path.is_absolute() => Some(p.to_string_lossy().into_owned()),
        // The desktop entry's `%U` hands files over as `file://` URLs.
        (_, Some(s)) if s.starts_with("file:") => local_path(s),
        // A link (`totaldrop:…`, `https://…`) stays as it is.
        (_, Some(s)) if has_scheme(s) => Some(s.to_owned()),
        (Some(p), _) => Some(p.to_string_lossy().into_owned()),
        // After `--`, something that looks like an option and names no
        // file is not one either.
        (None, Some(s)) if s.starts_with('-') => None,
        (None, Some(s)) => Some(s.to_owned()),
        (None, None) => None,
    }
}

/// The path of a local `file://` URL (an empty host or `localhost`). One
/// with another host, or without `//`, names nothing here.
fn local_path(url: &str) -> Option<String> {
    let rest = url.strip_prefix("file://")?;
    let path = rest.strip_prefix("localhost").unwrap_or(rest);
    path.starts_with('/')
        .then(|| crate::desktop::percent_decode(path))
}

/// `scheme:` as RFC 3986 spells it: a letter, then letters, digits, `+`,
/// `-` or `.`.
fn has_scheme(s: &str) -> bool {
    let Some((scheme, _)) = s.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
}

/// A request the window has to show, with the launcher's activation token
/// (Wayland `xdg-activation`) and X11 startup notification id
/// (`DESKTOP_STARTUP_ID`), which let the window come to the front and end
/// the desktop's busy cursor (#178, #184).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    pub request: Request,
    pub token: Option<String>,
    pub startup_id: Option<String>,
}

type Deliver = Box<dyn Fn(Opened) + Send>;

/// Requests from later launches, until the window listens.
struct Inbox {
    pending: VecDeque<Opened>,
    deliver: Option<Deliver>,
}

static INBOX: Mutex<Inbox> = Mutex::new(Inbox {
    pending: VecDeque::new(),
    deliver: None,
});

fn push(o: Opened) {
    let mut inbox = INBOX.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(deliver) = &inbox.deliver {
        deliver(o);
    } else {
        if inbox.pending.len() == MAX_PENDING {
            inbox.pending.pop_front();
        }
        inbox.pending.push_back(o);
    }
}

/// Hand every request, those already waiting first, to `deliver`.
pub fn listen(deliver: impl Fn(Opened) + Send + 'static) {
    let mut inbox = INBOX.lock().unwrap_or_else(PoisonError::into_inner);
    for o in inbox.pending.drain(..) {
        deliver(o);
    }
    inbox.deliver = Some(Box::new(deliver));
}

/// One item waiting for the hand-off worker, in arrival order (#357).
struct DeliveryJob {
    opened: Opened,
    /// When `Some`, filter local paths for this package before delivering.
    filter: Option<Package>,
}

/// One worker thread behind a bounded queue: keeps the D-Bus and GUI
/// threads off the filesystem while preserving arrival order across all
/// `Application` calls and capping queued work (#357).
struct DeliveryQueue {
    tx: Mutex<std::sync::mpsc::SyncSender<DeliveryJob>>,
}

impl DeliveryQueue {
    fn new(deliver: impl Fn(Opened) + Send + 'static) -> Self {
        Self::with_filter(deliver, |req, package| {
            let mountinfo = mountinfo_for(package, &req);
            filter_for_window(req, package, mountinfo.as_deref())
        })
    }

    fn with_filter(
        deliver: impl Fn(Opened) + Send + 'static,
        filter: impl Fn(Request, Package) -> Request + Send + 'static,
    ) -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel::<DeliveryJob>(MAX_PENDING);
        let worker = move || {
            while let Ok(mut job) = rx.recv() {
                if let Some(package) = job.filter {
                    job.opened.request = filter(job.opened.request, package);
                }
                deliver(job.opened);
            }
        };
        // If the OS refuses a thread, `rx` drops cleanly and `enqueue` returns
        // `false` without panicking.
        let _ = std::thread::Builder::new()
            .name("totaldrop-handoff".into())
            .spawn(worker);
        Self { tx: Mutex::new(tx) }
    }

    /// Enqueue `opened` without blocking the caller. Returns `false` when
    /// the bounded queue is full (for example while the worker is stuck on
    /// a stalled mount) or the worker has exited.
    fn enqueue(&self, opened: Opened, filter: Option<Package>) -> bool {
        let tx = self.tx.lock().unwrap_or_else(PoisonError::into_inner);
        tx.try_send(DeliveryJob { opened, filter }).is_ok()
    }
}

fn delivery_queue() -> &'static DeliveryQueue {
    static QUEUE: OnceLock<DeliveryQueue> = OnceLock::new();
    QUEUE.get_or_init(|| DeliveryQueue::new(push))
}

/// Whether `req` needs Flatpak sandbox path checks (#357).
fn needs_sandbox_filter(package: Package, req: &Request) -> bool {
    package == Package::Flatpak && req.items.iter().any(|i| i.starts_with('/'))
}

/// Split the first launch's command line into arguments ready for the GUI
/// thread and work that must run off-thread (#357).
///
/// Outside Flatpak, or when no local path was passed, returns
/// [`Request::args`] immediately without probing sandbox mounts. Inside Flatpak
/// with local paths, queues sandbox filtering on the hand-off worker so
/// the filtered result arrives through [`listen`] instead of probing a
/// potentially stalled mount on the GUI thread.
pub fn start_args(args: impl IntoIterator<Item = OsString>, cwd: &Path) -> Vec<String> {
    start_args_with(
        Request::from_args(args, cwd),
        Package::current(),
        delivery_queue(),
    )
}

fn start_args_with(req: Request, package: Package, queue: &DeliveryQueue) -> Vec<String> {
    if needs_sandbox_filter(package, &req) {
        let _ = queue.enqueue(
            Opened {
                request: req,
                token: None,
                startup_id: None,
            },
            Some(package),
        );
        return Vec::new();
    }
    req.args()
}

fn text(platform_data: &HashMap<String, OwnedVariant>, key: &str) -> Option<String> {
    platform_data
        .get(key)
        .and_then(|v| <&str>::try_from(v).ok())
        .map(str::to_owned)
}

/// The activation token in `platform_data`, if it is one: printable ASCII
/// of a sane length, since it goes into the environment.
fn token(platform_data: &HashMap<String, OwnedVariant>) -> Option<String> {
    text(platform_data, "activation-token")
        .filter(|t| !t.is_empty() && t.len() <= MAX_TOKEN_BYTES)
        .filter(|t| t.bytes().all(|b| b.is_ascii_graphic()))
}

/// Validate a freedesktop `desktop-startup-id`. The Startup Notification
/// spec sets no length cap, so this intentionally shares `MAX_TOKEN_BYTES`
/// (1024 bytes) with `activation-token` to bound untrusted D-Bus input.
fn valid_startup_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_TOKEN_BYTES && !id.chars().any(char::is_control)
}

/// The X11 startup notification id in `platform_data` (#184): non-empty
/// text of a sane length with no control characters. Unlike
/// `activation-token`, a startup id may hold spaces, quotes or non-ASCII
/// letters from the launcher's or host's name.
fn startup_id(platform_data: &HashMap<String, OwnedVariant>) -> Option<String> {
    text(platform_data, "desktop-startup-id").filter(|s| valid_startup_id(s))
}

/// Extract the X11 server timestamp from a freedesktop `DESKTOP_STARTUP_ID`
/// (`…_TIME<timestamp>`, Startup Notification Protocol, #184).
///
/// Reads the suffix after the last `_TIME`, accepts unsigned 32-bit decimal
/// values as well as signed negative 32-bit decimal values (written by Xlib
/// launchers once server time passes `2^31` ms), and rejects `0`
/// (`XCB_CURRENT_TIME`, which in `_NET_WM_USER_TIME` means "do not focus").
pub fn startup_timestamp(id: &str) -> Option<u32> {
    let (_, tail) = id.rsplit_once("_TIME")?;
    let ts = if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) {
        tail.parse::<u32>().ok()?
    } else if let Some(digits) = tail.strip_prefix('-')
        && !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
    {
        tail.parse::<i32>().ok()?.cast_unsigned()
    } else {
        return None;
    };
    (ts != 0).then_some(ts)
}

/// Format the X11 `_NET_STARTUP_INFO` `remove: ID="…"` client-message
/// payload in NUL-terminated, zero-padded 20-byte chunks (#184).
///
/// Every `\` and `"` inside `id` is escaped with `\` as required by the
/// freedesktop Startup Notification Protocol, so an untrusted `id` cannot
/// close the quote or inject keys. Invalid ids yield an empty vector.
pub fn startup_remove_chunks(id: &str) -> Vec<[u8; 20]> {
    if !valid_startup_id(id) {
        return Vec::new();
    }
    let mut raw = Vec::with_capacity(id.len().saturating_mul(2).saturating_add(14));
    raw.extend_from_slice(b"remove: ID=\"");
    for &b in id.as_bytes() {
        if b == b'\\' || b == b'"' {
            raw.push(b'\\');
        }
        raw.push(b);
    }
    raw.push(b'"');
    raw.push(0);

    raw.chunks(20)
        .map(|part| {
            let mut chunk = [0u8; 20];
            for (dst, &src) in chunk.iter_mut().zip(part) {
                *dst = src;
            }
            chunk
        })
        .collect()
}

/// `org.freedesktop.Application`, as the Desktop Entry Specification
/// defines it.
struct Application {
    package: Package,
}

#[zbus::interface(name = "org.freedesktop.Application")]
impl Application {
    fn activate(&self, platform_data: HashMap<String, OwnedVariant>) {
        let _ = delivery_queue().enqueue(
            Opened {
                request: Request::default(),
                token: token(&platform_data),
                startup_id: startup_id(&platform_data),
            },
            None,
        );
    }

    fn open(&self, uris: Vec<String>, platform_data: HashMap<String, OwnedVariant>) {
        let screen = text(&platform_data, SCREEN_KEY);
        let req = Request::from_open(&uris, screen.as_deref());
        let filter = needs_sandbox_filter(self.package, &req).then_some(self.package);
        // Probing host paths (`metadata` / `read_dir` / `canonicalize`) can
        // block on a stalled FUSE or NFS mount; every `Application` call goes
        // through the single bounded hand-off worker so `Open` returns before
        // the second launch's `TIMEOUT` while preserving arrival order (#357).
        let _ = delivery_queue().enqueue(
            Opened {
                request: req,
                token: token(&platform_data),
                startup_id: startup_id(&platform_data),
            },
            filter,
        );
    }

    fn activate_action(
        &self,
        action_name: String,
        _parameter: Vec<OwnedVariant>,
        platform_data: HashMap<String, OwnedVariant>,
    ) -> zbus::fdo::Result<()> {
        if !SCREENS.contains(&action_name.as_str()) {
            return Err(zbus::fdo::Error::NotSupported(action_name));
        }
        let _ = delivery_queue().enqueue(
            Opened {
                request: Request {
                    items: Vec::new(),
                    screen: Some(action_name),
                    unreadable_in_flatpak: false,
                },
                token: token(&platform_data),
                startup_id: startup_id(&platform_data),
            },
            None,
        );
        Ok(())
    }
}

/// Which package a TotalDrop window comes from (#351).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Package {
    /// The Flatpak.
    Flatpak,
    /// The distribution's package: .pkg, .deb or .rpm.
    System,
}

impl Package {
    /// The package this program is.
    pub fn current() -> Self {
        if crate::theme::host::sandboxed() {
            Self::Flatpak
        } else {
            Self::System
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Flatpak => "flatpak",
            Self::System => "system",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "flatpak" => Some(Self::Flatpak),
            "system" => Some(Self::System),
            _ => None,
        }
    }

    /// The key of the sentence that says this package's window is open
    /// and came forward (l10n/app).
    pub const fn running_key(self) -> &'static str {
        match self {
            Self::Flatpak => "otherPackageFlatpak",
            Self::System => "otherPackageSystem",
        }
    }
}

/// Read `/proc/self/mountinfo` only when a real Flatpak window has local
/// paths to check.
fn mountinfo_for(package: Package, req: &Request) -> Option<String> {
    (needs_sandbox_filter(package, req) && crate::theme::host::sandboxed())
        .then(|| std::fs::read_to_string("/proc/self/mountinfo").ok())
        .flatten()
}

/// Leave out local paths a Flatpak window cannot read (#357), and set
/// [`Request::unreadable_in_flatpak`] when any were dropped.
fn filter_for_window(mut req: Request, package: Package, mountinfo: Option<&str>) -> Request {
    if package != Package::Flatpak {
        return req;
    }
    let before = req.items.len();
    req.items
        .retain(|item| !item.starts_with('/') || readable_in_sandbox(Path::new(item), mountinfo));
    req.unreadable_in_flatpak = req.items.len() < before;
    req
}

/// Whether the Flatpak sandbox can read `path` from the host: it must be a
/// readable regular file or folder, and not one of the memory-only parent
/// folders (`~`, `~/Downloads`) bubblewrap makes on its own tmpfs.
fn readable_in_sandbox(path: &Path, mountinfo: Option<&str>) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    let ok = if meta.is_dir() {
        std::fs::read_dir(path).is_ok()
    } else if meta.is_file() {
        std::fs::File::open(path).is_ok()
    } else {
        false
    };
    if !ok {
        return false;
    }
    if let Some(table) = mountinfo {
        let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if !from_host(table, &resolved) {
            return false;
        }
    }
    true
}

/// Whether `path` (canonical) sits on a mount other than bubblewrap's own
/// tmpfs; same rule as `kept` in `crates/totaldrop-cli/src/sandbox.rs`.
///
/// Keep in sync with `kept` and `unescape` in
/// `crates/totaldrop-cli/src/sandbox.rs`; both crates test the shared
/// fixture `crates/totaldrop-cli/tests/flatpak-mountinfo.txt`.
fn from_host(mountinfo: &str, path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt as _;
    let mut sandbox_memory = None;
    for line in mountinfo.lines() {
        let mut halves = line.splitn(2, " - ");
        let (Some(left), Some(right)) = (halves.next(), halves.next()) else {
            continue;
        };
        let mut fields = left.split(' ');
        let (Some(point), Some(kind)) = (fields.nth(4), right.split(' ').next()) else {
            continue;
        };
        let bytes = unescape_mount(point);
        if !path.starts_with(Path::new(std::ffi::OsStr::from_bytes(&bytes))) {
            continue;
        }
        let shared = fields
            .skip(1)
            .any(|f| f.starts_with("master:") || f.starts_with("shared:"));
        sandbox_memory = Some(matches!(kind, "tmpfs" | "ramfs") && !shared);
    }
    sandbox_memory != Some(true)
}

/// Undo `\040`-style octal escapes in a `mountinfo` mount point.
/// Keep in sync with `unescape` in `crates/totaldrop-cli/src/sandbox.rs`.
fn unescape_mount(field: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(field.len());
    let mut rest = field.as_bytes();
    while let Some((&b, tail)) = rest.split_first() {
        if b == b'\\'
            && let Some((digits, after)) = tail.split_first_chunk::<3>()
            && digits.iter().all(|d| (b'0'..=b'7').contains(d))
            && let Ok(s) = std::str::from_utf8(digits)
            && let Ok(v) = u8::from_str_radix(s, 8)
        {
            out.push(v);
            rest = after;
        } else {
            out.push(b);
            rest = tail;
        }
    }
    out
}

/// `app.totaldrop.TotalDrop.Instance`: what a later launch may ask the
/// window about itself.
struct Instance {
    package: Package,
}

#[zbus::interface(name = "app.totaldrop.TotalDrop.Instance")]
impl Instance {
    /// `flatpak` or `system`.
    fn package(&self) -> String {
        self.package.name().to_owned()
    }
}

/// How this launch goes on.
pub enum Claim {
    /// This is the window. Keep the connection for as long as it runs.
    Owner(zbus::Connection),
    /// The running window has the request; exit. `other` is the window's
    /// package when it is not this one's, so the person can be told why
    /// no window of this package opened.
    Forwarded { other: Option<Package> },
    /// No session bus, or no answer from it: open a window of our own.
    Alone,
}

/// How long a launch waits for the bus and the running window.
const TIMEOUT: Duration = Duration::from_secs(5);

/// How long a launch that handed its request over waits to learn the
/// running window's package. The request is delivered by then, so a slow
/// answer only means nothing is said (#351).
const ASK_PACKAGE: Duration = Duration::from_secs(1);

/// Take the name, or hand `request` to whoever has it.
pub async fn claim(request: &Request) -> Claim {
    let ours = Package::current();
    let bus = zbus::connection::Builder::session();
    match tokio::time::timeout(TIMEOUT, hand_off(bus, request, ours)).await {
        Ok(Ok(step)) => finish(step, ours).await,
        Ok(Err(e)) => {
            eprintln!("totaldrop-desktop: no single-window hand-off: {e}");
            Claim::Alone
        }
        Err(_) => {
            eprintln!(
                "totaldrop-desktop: no single-window hand-off: the session bus did not answer"
            );
            Claim::Alone
        }
    }
}

/// Where the hand-off ended, before the running window is asked its
/// package.
enum Step {
    Owner(zbus::Connection),
    Forwarded(zbus::Connection),
}

async fn hand_off(
    bus: zbus::connection::Builder<'_>,
    request: &Request,
    ours: Package,
) -> zbus::Result<Step> {
    // The interfaces are served before the name is taken, so no call can
    // arrive at a name without them.
    let conn = bus
        .serve_at(OBJECT_PATH, Application { package: ours })
        .serve_at(OBJECT_PATH, Instance { package: ours })
        .method_timeout(TIMEOUT)
        .build()
        .await?;
    if take_name(&conn).await? {
        return Ok(Step::Owner(conn));
    }
    match forward(&conn, request).await {
        Ok(()) => {}
        // The window quit in between: this launch becomes it.
        Err(e) if gone(&e) => {
            if take_name(&conn).await? {
                return Ok(Step::Owner(conn));
            }
            forward(&conn, request).await?;
        }
        Err(e) => return Err(e),
    }
    Ok(Step::Forwarded(conn))
}

/// Outside the hand-off's own timeout: once the request is delivered, a
/// slow answer to the package question must not open a second window.
async fn finish(step: Step, ours: Package) -> Claim {
    match step {
        Step::Owner(conn) => Claim::Owner(conn),
        Step::Forwarded(conn) => {
            let other = tokio::time::timeout(ASK_PACKAGE, running_package(&conn))
                .await
                .ok()
                .flatten()
                .filter(|p| *p != ours);
            Claim::Forwarded { other }
        }
    }
}

#[cfg(test)]
async fn try_claim_on(
    bus: zbus::connection::Builder<'_>,
    request: &Request,
    ours: Package,
) -> zbus::Result<Claim> {
    Ok(finish(hand_off(bus, request, ours).await?, ours).await)
}

/// The running window's package; `None` when it does not say, as a window
/// of 0.3.0-beta.6 or older does not.
async fn running_package(conn: &zbus::Connection) -> Option<Package> {
    let reply = conn
        .call_method(
            Some(crate::APP_ID),
            OBJECT_PATH,
            Some(INSTANCE_INTERFACE),
            "Package",
            &(),
        )
        .await
        .ok()?;
    let name: String = reply.body().deserialize().ok()?;
    Package::from_name(&name)
}

/// Take the app's name: `false` when another window has it. zbus reports
/// a taken name as `Error::NameTaken`, not as `Ok(Exists)`.
async fn take_name(conn: &zbus::Connection) -> zbus::Result<bool> {
    match conn
        .request_name_with_flags(crate::APP_ID, RequestNameFlags::DO_NOT_QUEUE)
        .await
    {
        Ok(RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner) => Ok(true),
        Ok(RequestNameReply::Exists | RequestNameReply::InQueue) | Err(zbus::Error::NameTaken) => {
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

/// Whether a call failed because nobody owns the name any more, or its
/// owner quit before answering (the bus then replies `NoReply`).
fn gone(e: &zbus::Error) -> bool {
    matches!(
        e,
        zbus::Error::MethodError(name, _, _)
            if matches!(
                name.as_str(),
                "org.freedesktop.DBus.Error.ServiceUnknown"
                    | "org.freedesktop.DBus.Error.NameHasNoOwner"
                    | "org.freedesktop.DBus.Error.NoReply"
            )
    )
}

/// Call the running window with `request` and this launch's activation
/// token and X11 startup notification id.
pub async fn forward(conn: &zbus::Connection, request: &Request) -> zbus::Result<()> {
    let token = std::env::var("XDG_ACTIVATION_TOKEN").ok();
    let startup = std::env::var("DESKTOP_STARTUP_ID").ok();
    forward_with(conn, request, token.as_deref(), startup.as_deref()).await
}

async fn forward_with(
    conn: &zbus::Connection,
    request: &Request,
    token: Option<&str>,
    startup: Option<&str>,
) -> zbus::Result<()> {
    let mut platform_data: HashMap<&str, zbus::Variant<'_>> = HashMap::new();
    if let Some(t) = token {
        platform_data.insert("activation-token", zbus::Variant::from(t));
    }
    if let Some(s) = startup {
        platform_data.insert("desktop-startup-id", zbus::Variant::from(s));
    }
    if !request.items.is_empty() {
        if let Some(s) = &request.screen {
            platform_data.insert(SCREEN_KEY, zbus::Variant::from(s.as_str()));
        }
        conn.call_method(
            Some(crate::APP_ID),
            OBJECT_PATH,
            Some(INTERFACE),
            "Open",
            &(request.uris(), platform_data),
        )
        .await?;
    } else if let Some(s) = &request.screen {
        let parameter: Vec<zbus::Variant<'_>> = Vec::new();
        conn.call_method(
            Some(crate::APP_ID),
            OBJECT_PATH,
            Some(INTERFACE),
            "ActivateAction",
            &(s.as_str(), parameter, platform_data),
        )
        .await?;
    } else {
        conn.call_method(
            Some(crate::APP_ID),
            OBJECT_PATH,
            Some(INTERFACE),
            "Activate",
            &(platform_data,),
        )
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead as _;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};

    /// The inbox is one per process: tests that listen take turns.
    static ONE_INBOX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    /// A session bus of our own, so tests neither need nor touch the
    /// desktop's.
    struct PrivateBus {
        daemon: Child,
        address: String,
        dir: PathBuf,
    }

    impl PrivateBus {
        /// `None` where `dbus-daemon` is not installed.
        fn start() -> Option<Self> {
            static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            // /tmp, not $TMPDIR: a socket path must stay under ~108 bytes.
            // create_dir fails on a directory someone else made first.
            let dir = Path::new("/tmp").join(format!("td-bus-{}-{n}", std::process::id()));
            std::fs::create_dir(&dir).ok()?;
            let daemon = Command::new("dbus-daemon")
                .args(["--session", "--nofork", "--print-address=1"])
                .arg(format!("--address=unix:dir={}", dir.display()))
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn();
            let Ok(daemon) = daemon else {
                let _ = std::fs::remove_dir_all(&dir);
                return None;
            };
            // From here `Drop` stops the daemon and removes its socket.
            let mut bus = Self {
                daemon,
                address: String::new(),
                dir,
            };
            let out = bus.daemon.stdout.take()?;
            std::io::BufReader::new(out)
                .read_line(&mut bus.address)
                .ok()?;
            bus.address = bus.address.trim().to_owned();
            (!bus.address.is_empty()).then_some(bus)
        }

        fn builder(&self) -> zbus::connection::Builder<'_> {
            zbus::connection::Builder::address(self.address.as_str())
        }
    }

    impl Drop for PrivateBus {
        fn drop(&mut self) {
            let _ = self.daemon.kill();
            let _ = self.daemon.wait();
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    /// The hand-off as two launches make it, over a real bus: the first
    /// owns the name, the second gives it its request and is done, and
    /// once the first has quit the next launch owns the name again.
    #[tokio::test]
    async fn a_second_launch_hands_over_to_the_first() {
        let _one = ONE_INBOX.lock().await;
        let Some(bus) = PrivateBus::start() else {
            // CI installs it, so the test cannot pass there by skipping.
            assert!(
                std::env::var_os("CI").is_none(),
                "no private session bus: dbus-daemon is missing or failed (see its stderr)"
            );
            eprintln!("skipped: dbus-daemon is not installed");
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        listen(move |o| tx.send(o).unwrap());

        let first = try_claim_on(bus.builder(), &Request::default(), Package::System).await;
        assert!(matches!(first, Ok(Claim::Owner(_))), "first launch");

        let second = Request {
            items: vec!["/tmp/td-a.txt".into()],
            screen: Some("send".into()),
            unreadable_in_flatpak: false,
        };
        let claim = try_claim_on(bus.builder(), &second, Package::System).await;
        // Same package: nothing to explain.
        assert!(
            matches!(claim, Ok(Claim::Forwarded { other: None })),
            "second launch"
        );
        let o = rx.recv_timeout(TIMEOUT).unwrap();
        assert_eq!(o.request, second);

        // Forwarding over the bus carries both Wayland's activation token
        // and X11's DESKTOP_STARTUP_ID (#184).
        let caller = bus.builder().build().await.unwrap();
        forward_with(
            &caller,
            &second,
            Some("wayland-tok"),
            Some("kwin;1_TIME777"),
        )
        .await
        .unwrap();
        let o = rx.recv_timeout(TIMEOUT).unwrap();
        assert_eq!(o.request, second);
        assert_eq!(o.token.as_deref(), Some("wayland-tok"));
        assert_eq!(o.startup_id.as_deref(), Some("kwin;1_TIME777"));

        // The Flatpak started while the distribution package's window is
        // open still hands over, and learns which package has the window
        // (#351).
        let flatpak = try_claim_on(bus.builder(), &Request::default(), Package::Flatpak).await;
        assert!(
            matches!(
                flatpak,
                Ok(Claim::Forwarded {
                    other: Some(Package::System)
                })
            ),
            "the other package"
        );
        let _ = rx.recv_timeout(TIMEOUT).unwrap();

        drop(first);
        let third = try_claim_on(bus.builder(), &Request::default(), Package::Flatpak).await;
        assert!(matches!(third, Ok(Claim::Owner(_))), "after the first quit");
        let system = try_claim_on(bus.builder(), &Request::default(), Package::System).await;
        assert!(
            matches!(
                system,
                Ok(Claim::Forwarded {
                    other: Some(Package::Flatpak)
                })
            ),
            "the Flatpak's window"
        );
        let _ = rx.recv_timeout(TIMEOUT).unwrap();
    }

    /// A window of 0.3.0-beta.6 or older has no Instance interface: the
    /// hand-off works and nothing is said about packages.
    #[tokio::test]
    async fn an_older_window_says_no_package() {
        let _one = ONE_INBOX.lock().await;
        let Some(bus) = PrivateBus::start() else {
            assert!(
                std::env::var_os("CI").is_none(),
                "no private session bus: dbus-daemon is missing or failed (see its stderr)"
            );
            eprintln!("skipped: dbus-daemon is not installed");
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        listen(move |o| tx.send(o).unwrap());
        let old = bus
            .builder()
            .serve_at(
                OBJECT_PATH,
                Application {
                    package: Package::System,
                },
            )
            .build()
            .await
            .unwrap();
        assert!(take_name(&old).await.unwrap());
        let claim = try_claim_on(bus.builder(), &Request::default(), Package::Flatpak).await;
        assert!(matches!(claim, Ok(Claim::Forwarded { other: None })));
        let _ = rx.recv_timeout(TIMEOUT).unwrap();
    }

    #[test]
    fn package_names_go_both_ways() {
        for p in [Package::Flatpak, Package::System] {
            assert_eq!(Package::from_name(p.name()), Some(p));
        }
        assert_eq!(Package::from_name("snap"), None);
        assert_ne!(
            Package::Flatpak.running_key(),
            Package::System.running_key()
        );
    }

    fn args(a: &[&str]) -> Vec<OsString> {
        a.iter().map(OsString::from).collect()
    }

    fn req(a: &[&str], cwd: &Path) -> Request {
        Request::from_args(args(a), cwd)
    }

    fn tmp() -> PathBuf {
        let d = std::env::temp_dir().join(format!("td-instance-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("photo 1.jpg"), b"x").unwrap();
        d
    }

    /// "Send clipboard" in the desktop entry opens Send with the
    /// clipboard's text (#326).
    #[test]
    fn the_clipboard_screen_is_known() {
        let r = req(&["--screen=clipboard"], Path::new("/"));
        assert_eq!(r.screen.as_deref(), Some("clipboard"));
        assert_eq!(r.args(), ["--screen=clipboard"]);
        let from_open = Request::from_open(&[], Some("clipboard"));
        assert_eq!(from_open.screen.as_deref(), Some("clipboard"));
    }

    #[test]
    fn files_links_and_screen_are_read() {
        let r = req(
            &[
                "/tmp/a.jpg",
                "--screen=send",
                "totaldrop:7-a-b",
                "https://totaldrop.app/r#7-a-b",
            ],
            Path::new("/"),
        );
        assert_eq!(
            r.items,
            [
                "/tmp/a.jpg",
                "totaldrop:7-a-b",
                "https://totaldrop.app/r#7-a-b"
            ]
        );
        assert_eq!(r.screen.as_deref(), Some("send"));
        assert_eq!(r.args().last().map(String::as_str), Some("--screen=send"));
    }

    /// A later launch runs in another folder: a relative path is made
    /// absolute against the launch's own.
    #[test]
    fn relative_paths_are_made_absolute() {
        let d = tmp();
        std::fs::write(d.join("a:b.txt"), b"x").unwrap();
        assert_eq!(
            req(&["a:b.txt"], &d).items,
            [d.join("a:b.txt").display().to_string()]
        );
        let r = req(&["photo 1.jpg", "7-abandon-ability"], &d);
        assert_eq!(
            r.items,
            [
                d.join("photo 1.jpg").display().to_string(),
                "7-abandon-ability".into()
            ]
        );
    }

    /// File managers start the app with `file://` URLs (`Exec=… %U`).
    #[test]
    fn file_urls_become_paths() {
        let r = req(
            &["file:///tmp/a%20b.jpg", "file://host/x", "file:rel"],
            Path::new("/"),
        );
        assert_eq!(r.items, ["/tmp/a b.jpg"]);
        let r = req(&["file://localhost/tmp/x"], Path::new("/"));
        assert_eq!(r.items, ["/tmp/x"]);
    }

    /// What the file-manager menus hand over (#303): a name that starts
    /// with `-` or holds quotes, `;` or non-ASCII letters is one file, not
    /// an option or several arguments.
    #[test]
    fn menu_selections_keep_odd_names() {
        let r = req(
            &[
                "file:///home/a/-rf%20%22x%22.txt",
                "file:///home/a/a%3Bb%27c",
                "file:///home/a/%C3%A7i%C3%A7ek",
                // GLib leaves these raw in a file URI.
                "file:///home/a/Bob's$(x)&y.txt",
                "--screen=send",
            ],
            Path::new("/"),
        );
        assert_eq!(
            r.items,
            [
                "/home/a/-rf \"x\".txt",
                "/home/a/a;b'c",
                "/home/a/çiçek",
                "/home/a/Bob's$(x)&y.txt"
            ]
        );
        assert_eq!(r.screen.as_deref(), Some("send"));
    }

    #[test]
    fn qt_options_and_their_values_are_skipped() {
        let r = req(
            &[
                "-platform",
                "wayland",
                "--style",
                "Fusion",
                "-reverse",
                "--screen=nope",
                "/a",
            ],
            Path::new("/"),
        );
        assert_eq!(r.items, ["/a"]);
        assert_eq!(r.screen, None);
        assert_eq!(
            req(&["-qwindowicon", "x.png", "-icon", "y.png"], Path::new("/")).items,
            Vec::<String>::new()
        );
        // After `--`, a file may look like an option; what names no file is
        // dropped, so it cannot pass for `--screen`.
        let d = tmp();
        std::fs::write(d.join("-odd"), b"x").unwrap();
        let r = req(&["--", "--screen=send", "-odd"], &d);
        assert_eq!(r.items, [d.join("-odd").display().to_string()]);
        assert_eq!(r.screen, None);
    }

    #[test]
    fn uris_round_trip() {
        let r = Request {
            items: vec!["/tmp/ğüş #1%.txt".into(), "7-a-b".into()],
            screen: None,
            unreadable_in_flatpak: false,
        };
        let uris = r.uris();
        assert_eq!(uris[0], "file:///tmp/%C4%9F%C3%BC%C5%9F%20%231%25.txt");
        assert_eq!(uris[1], "7-a-b");
        assert_eq!(Request::from_open(&uris, None), r);
    }

    /// `Open` may come from any process in the session: what it sends is
    /// bounded, and only known screens are taken.
    #[test]
    fn what_open_receives_is_checked() {
        let long = format!("/{}", "a".repeat(MAX_ARG_BYTES));
        let mut uris = vec![
            String::new(),
            long,
            "file://host/etc/passwd".into(),
            "file:relative".into(),
            "file:///ok".into(),
            "--screen=send".into(),
        ];
        uris.extend((0..MAX_ARGS + 10).map(|i| format!("/f{i}")));
        let r = Request::from_open(&uris, Some("../../etc"));
        assert_eq!(r.items[0], "/ok");
        assert_eq!(r.items.len(), MAX_ARGS - 2);
        // And bounded in bytes: 1 MiB a call.
        let big: Vec<String> = (0..100)
            .map(|i| format!("/{i:05}{}", "a".repeat(16 * 1024 - 7)))
            .collect();
        assert_eq!(Request::from_open(&big, None).items.len(), 64);
        assert_eq!(r.screen, None);
        assert_eq!(
            Request::from_open(&[], Some("settings")).screen.as_deref(),
            Some("settings")
        );
    }

    #[test]
    fn tokens_are_checked() {
        let pd = |t: &str| {
            HashMap::from([(
                "activation-token".to_owned(),
                zbus::Variant::from(t).try_to_owned().unwrap(),
            )])
        };
        assert_eq!(token(&pd("kwin-42_abc")).as_deref(), Some("kwin-42_abc"));
        assert_eq!(token(&pd("a b")), None);
        assert_eq!(token(&pd("")), None);
        assert_eq!(token(&pd(&"x".repeat(MAX_TOKEN_BYTES + 1))), None);
        assert_eq!(token(&HashMap::new()), None);
    }

    #[test]
    fn startup_ids_are_checked() {
        let pd = |s: &str| {
            HashMap::from([(
                "desktop-startup-id".to_owned(),
                zbus::Variant::from(s).try_to_owned().unwrap(),
            )])
        };
        let kde = "box;1700000000;123456;789-totaldrop_TIME424242";
        assert_eq!(startup_id(&pd(kde)).as_deref(), Some(kde));
        let gnome = "Files/TotalDrop_1234_0-çiçek host_TIME99";
        assert_eq!(startup_id(&pd(gnome)).as_deref(), Some(gnome));
        let quoted = r#"my "launcher"\app_TIME7"#;
        assert_eq!(startup_id(&pd(quoted)).as_deref(), Some(quoted));
        let max = "x".repeat(MAX_TOKEN_BYTES);
        assert_eq!(startup_id(&pd(&max)).as_deref(), Some(max.as_str()));

        assert_eq!(startup_id(&pd("")), None);
        assert_eq!(startup_id(&pd(&"x".repeat(MAX_TOKEN_BYTES + 1))), None);
        for bad in [
            "a\0b",
            "a\nb",
            "a\rb",
            "a\tb",
            "a\x1bb",
            "a\x7fb",
            "a\u{0085}b",
        ] {
            assert_eq!(startup_id(&pd(bad)), None, "{bad:?}");
        }
        let wrong_type = HashMap::from([(
            "desktop-startup-id".to_owned(),
            zbus::Variant::from(42u32).try_to_owned().unwrap(),
        )]);
        assert_eq!(startup_id(&wrong_type), None);
        assert_eq!(startup_id(&HashMap::new()), None);
    }

    #[test]
    fn startup_timestamps_are_parsed() {
        assert_eq!(
            startup_timestamp("box;1700000000;123;456-totaldrop_TIME424242"),
            Some(424242)
        );
        assert_eq!(startup_timestamp("id_TIME1"), Some(1));
        assert_eq!(startup_timestamp("id_TIME4294967295"), Some(u32::MAX));
        // Last `_TIME` wins when the launcher or host name contains `_TIME`.
        assert_eq!(startup_timestamp("host_TIME1_app_TIME98765"), Some(98765));
        // Xlib launchers that format 32-bit Time with `%d` past 2^31 ms.
        assert_eq!(startup_timestamp("id_TIME-1"), Some(u32::MAX));
        assert_eq!(startup_timestamp("id_TIME-2147483648"), Some(0x8000_0000));

        // 0 is XCB_CURRENT_TIME / "do not focus" in _NET_WM_USER_TIME.
        assert_eq!(startup_timestamp("id_TIME0"), None);
        assert_eq!(startup_timestamp("id_TIME000"), None);
        assert_eq!(startup_timestamp("id_TIME-0"), None);

        for missing_or_bad in [
            "plain-startup-id",
            "id_TIME",
            "id_TIME-",
            "id_TIME+42",
            "id_TIME42ms",
            "id_TIME 42",
            "id_TIME4294967296",
            "id_TIME-2147483649",
            "ok_TIME42_bad_TIME",
        ] {
            assert_eq!(
                startup_timestamp(missing_or_bad),
                None,
                "{missing_or_bad:?}"
            );
        }
    }

    #[test]
    fn startup_remove_messages_are_escaped_and_chunked() {
        let decode = |chunks: &[[u8; 20]]| -> Vec<u8> { chunks.as_flattened().to_vec() };

        // Quotes and backslashes inside ID="…" are escaped so an untrusted id
        // cannot close the quote or inject keys; spaces and UTF-8 stay as-is.
        let tricky = r#"a" KEY="evil\çiçek"#;
        let chunks = startup_remove_chunks(tricky);
        let flat = decode(&chunks);
        let expected = b"remove: ID=\"a\\\" KEY=\\\"evil\\\\\xC3\xA7i\xC3\xA7ek\"\0";
        assert_eq!(&flat[..expected.len()], expected);
        assert!(flat[expected.len()..].iter().all(|&b| b == 0));

        // Boundary 19 bytes before NUL (`remove: ID="123456"` = 19 + 1 = 20):
        // fits in a single 20-byte ClientMessage ending with NUL.
        let c19 = startup_remove_chunks("123456");
        assert_eq!(c19.len(), 1);
        assert_eq!(&c19[0], b"remove: ID=\"123456\"\0");

        // Boundary 20 bytes before NUL (`remove: ID="1234567"` = 20 + 1 = 21):
        // needs a second 20-byte ClientMessage carrying the terminating NUL.
        let c20 = startup_remove_chunks("1234567");
        assert_eq!(c20.len(), 2);
        assert_eq!(&c20[0], b"remove: ID=\"1234567\"");
        assert_eq!(c20[1], [0u8; 20]);

        // Boundary 21 bytes before NUL (`remove: ID="12345678"` = 21 + 1 = 22):
        // second chunk carries closing `"` + NUL + 18 padding zeros.
        let c21 = startup_remove_chunks("12345678");
        assert_eq!(c21.len(), 2);
        assert_eq!(&c21[0], b"remove: ID=\"12345678");
        let mut tail = [0u8; 20];
        tail[0] = b'"';
        assert_eq!(c21[1], tail);

        // Escaped quote at / straddling the 20-byte chunk boundary:
        // `"123456\""` puts `\` at 18 and `"` at 19, with closing `"` + NUL in chunk 1;
        // `"1234567\""` splits `\` (byte 19, chunk 0) and `"` (byte 0, chunk 1).
        let eq_end = startup_remove_chunks("123456\"");
        assert_eq!(eq_end.len(), 2);
        assert_eq!(&eq_end[0], b"remove: ID=\"123456\\\"");
        assert_eq!(eq_end[1], tail);

        let eq_split = startup_remove_chunks("1234567\"");
        assert_eq!(eq_split.len(), 2);
        assert_eq!(&eq_split[0], b"remove: ID=\"1234567\\");
        let mut split_tail = [0u8; 20];
        split_tail[0] = b'"';
        split_tail[1] = b'"';
        assert_eq!(eq_split[1], split_tail);

        // Invalid inputs produce no chunks even if called directly.
        assert!(startup_remove_chunks("").is_empty());
        assert!(startup_remove_chunks(&"x".repeat(MAX_TOKEN_BYTES + 1)).is_empty());
        assert!(startup_remove_chunks("bad\nid").is_empty());
        assert!(startup_remove_chunks("bad\0id").is_empty());
    }

    /// The hand-off end to end over a private bus connection: each call
    /// arrives in the inbox as the running window will see it.
    #[tokio::test]
    async fn calls_arrive_in_the_inbox() {
        use zbus::connection::{Builder, socket::Channel};
        let _one = ONE_INBOX.lock().await;
        let (tx, rx) = std::sync::mpsc::channel();
        listen(move |o| tx.send(o).unwrap());

        let guid = zbus::Guid::generate();
        let (a, b) = Channel::pair();
        let (_server, client) = futures_util::try_join!(
            Builder::authenticated_socket(a, guid.clone())
                .p2p()
                .serve_at(
                    OBJECT_PATH,
                    Application {
                        package: Package::System,
                    },
                )
                .build(),
            Builder::authenticated_socket(b, guid).p2p().build(),
        )
        .unwrap();

        let call = |method: &'static str,
                    uris: Vec<String>,
                    screen: Option<&'static str>,
                    startup: &'static str| {
            let client = client.clone();
            async move {
                let mut pd: HashMap<&str, zbus::Variant<'_>> = HashMap::new();
                pd.insert("activation-token", zbus::Variant::from("tok"));
                pd.insert("desktop-startup-id", zbus::Variant::from(startup));
                if let Some(s) = screen {
                    pd.insert(SCREEN_KEY, zbus::Variant::from(s));
                }
                match method {
                    "Open" => {
                        client
                            .call_method(
                                None::<()>,
                                OBJECT_PATH,
                                Some(INTERFACE),
                                "Open",
                                &(uris, pd),
                            )
                            .await
                    }
                    "ActivateAction" => {
                        let p: Vec<zbus::Variant<'_>> = Vec::new();
                        client
                            .call_method(
                                None::<()>,
                                OBJECT_PATH,
                                Some(INTERFACE),
                                "ActivateAction",
                                &(screen.unwrap(), p, pd),
                            )
                            .await
                    }
                    _ => {
                        client
                            .call_method(
                                None::<()>,
                                OBJECT_PATH,
                                Some(INTERFACE),
                                "Activate",
                                &(pd,),
                            )
                            .await
                    }
                }
            }
        };

        call(
            "Open",
            vec!["file:///tmp/a%20b".into(), "totaldrop:7-a-b".into()],
            Some("send"),
            "dolphin;1_TIME101",
        )
        .await
        .unwrap();
        let o = rx.recv().unwrap();
        assert_eq!(o.request.items, ["/tmp/a b", "totaldrop:7-a-b"]);
        assert_eq!(o.request.screen.as_deref(), Some("send"));
        assert_eq!(o.token.as_deref(), Some("tok"));
        assert_eq!(o.startup_id.as_deref(), Some("dolphin;1_TIME101"));

        call("ActivateAction", vec![], Some("receive"), "panel;2_TIME202")
            .await
            .unwrap();
        let o = rx.recv().unwrap();
        assert_eq!(o.request.args(), ["--screen=receive"]);
        assert_eq!(o.startup_id.as_deref(), Some("panel;2_TIME202"));
        assert!(
            call("ActivateAction", vec![], Some("format-disk"), "panel;3")
                .await
                .is_err()
        );

        call("Activate", vec![], None, "launcher;4_TIME404")
            .await
            .unwrap();
        let o = rx.recv().unwrap();
        assert_eq!(o.request, Request::default());
        assert_eq!(o.startup_id.as_deref(), Some("launcher;4_TIME404"));

        // A malformed startup id is dropped while the call itself still lands.
        call("Activate", vec![], None, "bad\nstartup")
            .await
            .unwrap();
        let o = rx.recv().unwrap();
        assert_eq!(o.startup_id, None);
    }

    /// When the Flatpak window is open and the system package's "Send with
    /// TotalDrop" hands over host paths over `org.freedesktop.Application.Open`
    /// (#357), paths the sandbox cannot read are left out and flagged so the
    /// window tells the person why, while readable paths and links still
    /// come through.
    #[tokio::test]
    async fn a_flatpak_window_flags_unreadable_paths_from_open() {
        let _one = ONE_INBOX.lock().await;
        let Some(bus) = PrivateBus::start() else {
            assert!(
                std::env::var_os("CI").is_none(),
                "no private session bus: dbus-daemon is missing or failed (see its stderr)"
            );
            eprintln!("skipped: dbus-daemon is not installed");
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        listen(move |o| tx.send(o).unwrap());

        let d = tmp();
        let readable = d.join("portal-file.txt");
        std::fs::write(&readable, b"ok").unwrap();
        let readable_str = readable.display().to_string();
        let missing = d.join("host-only.pdf").display().to_string();

        let flatpak_win = try_claim_on(bus.builder(), &Request::default(), Package::Flatpak).await;
        assert!(matches!(flatpak_win, Ok(Claim::Owner(_))), "Flatpak window");

        // Only an unreadable host path: dropped, and `--flatpak-unreadable`
        // tells QML to open Send and explain why.
        let from_menu = Request {
            items: vec![missing.clone()],
            screen: None,
            unreadable_in_flatpak: false,
        };
        let claim = try_claim_on(bus.builder(), &from_menu, Package::System).await;
        assert!(matches!(
            claim,
            Ok(Claim::Forwarded {
                other: Some(Package::Flatpak)
            })
        ));
        let got = rx.recv_timeout(TIMEOUT).unwrap();
        assert!(got.request.items.is_empty());
        assert!(got.request.unreadable_in_flatpak);
        assert_eq!(got.request.args(), [FLATPAK_UNREADABLE_ARG]);

        // One readable path (as from ~/Downloads/TotalDrop or the portal)
        // together with one unreadable host path: the readable file is kept
        // and the unreadable one is flagged.
        let mixed = Request {
            items: vec![readable_str.clone(), missing],
            screen: None,
            unreadable_in_flatpak: false,
        };
        try_claim_on(bus.builder(), &mixed, Package::System)
            .await
            .unwrap();
        let got = rx.recv_timeout(TIMEOUT).unwrap();
        assert_eq!(got.request.items, [readable_str.as_str()]);
        assert!(got.request.unreadable_in_flatpak);
        assert_eq!(
            got.request.args(),
            [readable_str.as_str(), FLATPAK_UNREADABLE_ARG]
        );

        // Readable paths and TotalDrop links pass through without the flag.
        let ok = Request {
            items: vec![readable_str.clone(), "totaldrop:7-abandon-ability".into()],
            screen: None,
            unreadable_in_flatpak: false,
        };
        try_claim_on(bus.builder(), &ok, Package::System)
            .await
            .unwrap();
        let got = rx.recv_timeout(TIMEOUT).unwrap();
        assert_eq!(
            got.request.items,
            [readable_str.as_str(), "totaldrop:7-abandon-ability"]
        );
        assert!(!got.request.unreadable_in_flatpak);
        assert_eq!(
            got.request.args(),
            [readable_str.as_str(), "totaldrop:7-abandon-ability"]
        );
    }

    /// Folders bubblewrap synthesizes on its own tmpfs (`~`, `~/Downloads`)
    /// exist inside the sandbox as empty stubs, not the host's folders:
    /// `from_host` rejects them while keeping host-bound and portal mounts.
    /// Uses the shared fixture with `crates/totaldrop-cli/src/sandbox.rs`.
    #[test]
    fn flatpak_tmpfs_folders_are_flagged_as_unreadable() {
        const FLATPAK: &str =
            include_str!("../../crates/totaldrop-cli/tests/flatpak-mountinfo.txt");
        assert!(!from_host(FLATPAK, Path::new("/home/ada")));
        assert!(!from_host(FLATPAK, Path::new("/home/ada/Downloads")));
        assert!(!from_host(
            FLATPAK,
            Path::new("/home/ada/Downloads/TotalDropX")
        ));
        assert!(!from_host(FLATPAK, Path::new("/dev/shm/x")));
        assert!(from_host(
            FLATPAK,
            Path::new("/home/ada/Downloads/TotalDrop/a.pdf")
        ));
        assert!(from_host(
            FLATPAK,
            Path::new("/home/ada/.var/app/app.totaldrop.TotalDrop/data")
        ));
        assert!(from_host(FLATPAK, Path::new("/tmp/x")));
        assert!(from_host(FLATPAK, Path::new("/run/user/1000/x")));
        assert!(from_host(
            FLATPAK,
            Path::new("/run/user/1000/doc/9f8e7d/photo.jpg")
        ));
        assert!(from_host(FLATPAK, Path::new("/home/ada/My Share/file.txt")));

        let d = tmp();
        let canon = std::fs::canonicalize(&d).unwrap();
        let stub_table = format!("1 0 0:1 / {} rw - tmpfs tmpfs rw\n", canon.display());
        let r = Request {
            items: vec![d.display().to_string()],
            screen: None,
            unreadable_in_flatpak: false,
        };
        let filtered = filter_for_window(r, Package::Flatpak, Some(&stub_table));
        assert!(filtered.items.is_empty());
        assert!(filtered.unreadable_in_flatpak);
    }

    /// `readable_in_sandbox` checks `metadata` before opening, so a named
    /// pipe (FIFO) with no writer is dropped at once without blocking
    /// `Open` (and even under `uid 0` in the CI container); also checks
    /// [`Request::args`] order when both `unreadable_in_flatpak` and
    /// `screen` are set.
    #[test]
    fn special_files_do_not_block_and_args_keep_screen() {
        let d = tmp();
        let fifo = d.join("pipe");
        let status = std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap();
        assert!(status.success());

        let r = Request {
            items: vec![fifo.display().to_string()],
            screen: Some("send".into()),
            unreadable_in_flatpak: false,
        };
        // Use a detached OS thread + mpsc deadline rather than a Tokio
        // blocking task so a regression fails with the message instead of
        // hanging runtime drop.
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(filter_for_window(r, Package::Flatpak, None));
        });
        let filtered = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("checking a FIFO does not block");

        assert!(filtered.items.is_empty());
        assert!(filtered.unreadable_in_flatpak);
        assert_eq!(filtered.args(), [FLATPAK_UNREADABLE_ARG, "--screen=send"]);

        let with_item = Request {
            items: vec!["/ok.txt".into()],
            screen: Some("send".into()),
            unreadable_in_flatpak: true,
        };
        assert_eq!(
            with_item.args(),
            ["/ok.txt", FLATPAK_UNREADABLE_ARG, "--screen=send"]
        );
    }

    /// Every `Application` call flows through the single hand-off worker, so
    /// a non-path `Open` or `ActivateAction` arriving while a Flatpak path
    /// probe is still running cannot overtake it (#357).
    #[test]
    fn delivery_queue_preserves_order_across_slow_filter() {
        let (delivered_tx, delivered_rx) = std::sync::mpsc::channel();
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let queue = DeliveryQueue::with_filter(
            move |o| {
                let _ = delivered_tx.send(o);
            },
            move |mut req, _| {
                if req.items.iter().any(|i| i == "/slow") {
                    let _ = entered_tx.send(());
                    let _ = release_rx.recv();
                    req.items.clear();
                    req.unreadable_in_flatpak = true;
                }
                req
            },
        );

        let first = Opened {
            request: Request {
                items: vec!["/slow".into()],
                screen: None,
                unreadable_in_flatpak: false,
            },
            token: Some("t1".into()),
            startup_id: None,
        };
        let second = Opened {
            request: Request {
                items: vec!["totaldrop:7-a-b".into()],
                screen: None,
                unreadable_in_flatpak: false,
            },
            token: Some("t2".into()),
            startup_id: None,
        };
        let third = Opened {
            request: Request {
                items: Vec::new(),
                screen: Some("settings".into()),
                unreadable_in_flatpak: false,
            },
            token: Some("t3".into()),
            startup_id: None,
        };

        assert!(queue.enqueue(first, Some(Package::Flatpak)));
        entered_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("worker enters slow filter");
        assert!(queue.enqueue(second.clone(), None));
        assert!(queue.enqueue(third.clone(), None));
        assert!(
            delivered_rx.try_recv().is_err(),
            "later calls must wait behind the earlier path probe"
        );

        release_tx.send(()).unwrap();
        let got1 = delivered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let got2 = delivered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let got3 = delivered_rx.recv_timeout(Duration::from_secs(2)).unwrap();

        assert!(got1.request.items.is_empty());
        assert!(got1.request.unreadable_in_flatpak);
        assert_eq!(got1.token.as_deref(), Some("t1"));
        assert_eq!(got2, second);
        assert_eq!(got3, third);
    }

    /// When a stalled mount blocks the worker, `enqueue` caps pending work at
    /// `MAX_PENDING` and returns immediately rather than spawning unbounded
    /// threads or blocking the D-Bus handler (#357).
    #[test]
    fn delivery_queue_bounds_pending_work_without_blocking() {
        let (delivered_tx, delivered_rx) = std::sync::mpsc::channel();
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let queue = DeliveryQueue::with_filter(
            move |o| {
                let _ = delivered_tx.send(o);
            },
            move |req, _| {
                let _ = entered_tx.send(());
                let _ = release_rx.recv();
                req
            },
        );

        let blocked = Opened {
            request: Request {
                items: vec!["/stalled".into()],
                screen: None,
                unreadable_in_flatpak: false,
            },
            token: None,
            startup_id: None,
        };
        assert!(queue.enqueue(blocked, Some(Package::Flatpak)));
        entered_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("worker enters stalled filter");

        for i in 0..MAX_PENDING {
            let job = Opened {
                request: Request {
                    items: vec![format!("totaldrop:7-{i}-a")],
                    screen: None,
                    unreadable_in_flatpak: false,
                },
                token: None,
                startup_id: None,
            };
            assert!(queue.enqueue(job, None), "slot {i} fits in bounded queue");
        }
        let overflow = Opened {
            request: Request {
                items: vec!["totaldrop:7-overflow-a".into()],
                screen: None,
                unreadable_in_flatpak: false,
            },
            token: None,
            startup_id: None,
        };
        assert!(
            !queue.enqueue(overflow, None),
            "queue drops overflow without blocking once full"
        );

        release_tx.send(()).unwrap();
        for _ in 0..=MAX_PENDING {
            delivered_rx
                .recv_timeout(Duration::from_secs(2))
                .expect("queued items drain once worker unblocks");
        }
        assert!(delivered_rx.try_recv().is_err());
    }

    /// First-launch Flatpak paths are filtered off the GUI thread and
    /// delivered through the hand-off worker; non-Flatpak launches and
    /// pathless launches return straight away without probing the filesystem
    /// (#357).
    #[test]
    fn start_args_offloads_flatpak_paths_and_returns_other_launches_directly() {
        let (tx, rx) = std::sync::mpsc::channel();
        let queue = DeliveryQueue::new(move |o| {
            let _ = tx.send(o);
        });

        let sys = Request {
            items: vec!["/nonexistent/file.pdf".into()],
            screen: None,
            unreadable_in_flatpak: false,
        };
        assert_eq!(
            start_args_with(sys, Package::System, &queue),
            ["/nonexistent/file.pdf"]
        );
        assert!(rx.try_recv().is_err());

        let settings = Request {
            items: Vec::new(),
            screen: Some("settings".into()),
            unreadable_in_flatpak: false,
        };
        assert_eq!(
            start_args_with(settings, Package::Flatpak, &queue),
            ["--screen=settings"]
        );
        assert!(rx.try_recv().is_err());

        let d = tmp();
        let missing = d.join("not-in-sandbox.pdf").display().to_string();
        let flatpak = Request {
            items: vec![missing],
            screen: Some("send".into()),
            unreadable_in_flatpak: false,
        };
        assert!(start_args_with(flatpak, Package::Flatpak, &queue).is_empty());
        let delivered = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("off-thread first-launch filter delivers result");
        assert_eq!(delivered.token, None);
        assert!(delivered.request.items.is_empty());
        assert!(delivered.request.unreadable_in_flatpak);
        assert_eq!(
            delivered.request.args(),
            [FLATPAK_UNREADABLE_ARG, "--screen=send"]
        );
    }
}
