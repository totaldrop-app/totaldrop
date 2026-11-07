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
use std::os::unix::ffi::OsStrExt as _;
use std::path::Path;
use std::sync::{Mutex, PoisonError};
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

/// What a launch asks the window to show.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Request {
    /// Absolute paths of local files, and anything else that may be a
    /// TotalDrop link or code, in command-line order.
    pub items: Vec<String>,
    /// `send`, `receive` or `settings`.
    pub screen: Option<String>,
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

    /// The list QML reads: the items, then `--screen=…`.
    pub fn args(&self) -> Vec<String> {
        let mut out = self.items.clone();
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
                    file_uri(i)
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

/// A `file://` URL for an absolute path, with every byte outside the
/// unreserved set and `/` percent-encoded.
fn file_uri(path: &str) -> String {
    let mut out = String::from("file://");
    for &b in OsStr::new(path).as_bytes() {
        if b.is_ascii_alphanumeric() || b"/-._~".contains(&b) {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// A request the window has to show, with the launcher's activation token
/// (Wayland `xdg-activation`), which lets the window come to the front.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    pub request: Request,
    pub token: Option<String>,
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

/// `org.freedesktop.Application`, as the Desktop Entry Specification
/// defines it.
struct Application;

#[zbus::interface(name = "org.freedesktop.Application")]
impl Application {
    fn activate(&self, platform_data: HashMap<String, OwnedVariant>) {
        push(Opened {
            request: Request::default(),
            token: token(&platform_data),
        });
    }

    fn open(&self, uris: Vec<String>, platform_data: HashMap<String, OwnedVariant>) {
        let screen = text(&platform_data, SCREEN_KEY);
        push(Opened {
            request: Request::from_open(&uris, screen.as_deref()),
            token: token(&platform_data),
        });
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
        push(Opened {
            request: Request {
                items: Vec::new(),
                screen: Some(action_name),
            },
            token: token(&platform_data),
        });
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
        .serve_at(OBJECT_PATH, Application)
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
/// token.
pub async fn forward(conn: &zbus::Connection, request: &Request) -> zbus::Result<()> {
    let mut platform_data: HashMap<&str, zbus::Variant<'_>> = HashMap::new();
    let token = std::env::var("XDG_ACTIVATION_TOKEN").ok();
    if let Some(t) = &token {
        platform_data.insert("activation-token", zbus::Variant::from(t.as_str()));
    }
    let startup = std::env::var("DESKTOP_STARTUP_ID").ok();
    if let Some(s) = &startup {
        platform_data.insert("desktop-startup-id", zbus::Variant::from(s.as_str()));
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
        };
        let claim = try_claim_on(bus.builder(), &second, Package::System).await;
        // Same package: nothing to explain.
        assert!(
            matches!(claim, Ok(Claim::Forwarded { other: None })),
            "second launch"
        );
        let o = rx.recv_timeout(TIMEOUT).unwrap();
        assert_eq!(o.request, second);

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
            .serve_at(OBJECT_PATH, Application)
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
                .serve_at(OBJECT_PATH, Application)
                .build(),
            Builder::authenticated_socket(b, guid).p2p().build(),
        )
        .unwrap();

        let call = |method: &'static str, uris: Vec<String>, screen: Option<&'static str>| {
            let client = client.clone();
            async move {
                let mut pd: HashMap<&str, zbus::Variant<'_>> = HashMap::new();
                pd.insert("activation-token", zbus::Variant::from("tok"));
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
        )
        .await
        .unwrap();
        let o = rx.recv().unwrap();
        assert_eq!(o.request.items, ["/tmp/a b", "totaldrop:7-a-b"]);
        assert_eq!(o.request.screen.as_deref(), Some("send"));
        assert_eq!(o.token.as_deref(), Some("tok"));

        call("ActivateAction", vec![], Some("receive"))
            .await
            .unwrap();
        assert_eq!(rx.recv().unwrap().request.args(), ["--screen=receive"]);
        assert!(
            call("ActivateAction", vec![], Some("format-disk"))
                .await
                .is_err()
        );

        call("Activate", vec![], None).await.unwrap();
        assert_eq!(rx.recv().unwrap().request, Request::default());
    }
}
