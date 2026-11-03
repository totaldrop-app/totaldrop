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

/// The screens `--screen=` opens.
const SCREENS: [&str; 3] = ["send", "receive", "settings"];

/// Qt's own options that take the next argument as their value; that value
/// is not a file to send.
const QT_VALUE_OPTIONS: [&str; 14] = [
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
];

/// Limits on what another process may hand the window.
const MAX_ARGS: usize = 4096;
const MAX_ARG_BYTES: usize = 16 * 1024;
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
        let items = uris
            .iter()
            .filter(|u| !u.is_empty() && u.len() <= MAX_ARG_BYTES)
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
        // The desktop entry's `%U` hands files over as `file://` URLs.
        (_, Some(s)) if s.starts_with("file:") => local_path(s),
        // A link (`totaldrop:…`, `https://…`) stays as it is.
        (_, Some(s)) if has_scheme(s) => Some(s.to_owned()),
        (Some(p), _) => Some(p.to_string_lossy().into_owned()),
        (None, Some(s)) => Some(s.to_owned()),
        (None, None) => None,
    }
}

/// The path of a local `file://` URL. One with a host, or without `//`,
/// names nothing here.
fn local_path(url: &str) -> Option<String> {
    url.strip_prefix("file://")
        .filter(|p| p.starts_with('/'))
        .map(crate::desktop::percent_decode)
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

/// How this launch goes on.
pub enum Claim {
    /// This is the window. Keep the connection for as long as it runs.
    Owner(zbus::Connection),
    /// The running window has the request; exit.
    Forwarded,
    /// No session bus, or no answer from it: open a window of our own.
    Alone,
}

/// How long a launch waits for the bus and the running window.
const TIMEOUT: Duration = Duration::from_secs(5);

/// Take the name, or hand `request` to whoever has it.
pub async fn claim(request: &Request) -> Claim {
    match tokio::time::timeout(TIMEOUT, try_claim(request)).await {
        Ok(Ok(claim)) => claim,
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

async fn try_claim(request: &Request) -> zbus::Result<Claim> {
    // The interface is served before the name is taken, so no call can
    // arrive at a name without it.
    let conn = zbus::connection::Builder::session()
        .serve_at(OBJECT_PATH, Application)
        .method_timeout(TIMEOUT)
        .build()
        .await?;
    match conn
        .request_name_with_flags(crate::APP_ID, RequestNameFlags::DO_NOT_QUEUE)
        .await?
    {
        RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner => Ok(Claim::Owner(conn)),
        RequestNameReply::Exists | RequestNameReply::InQueue => {
            forward(&conn, request).await?;
            Ok(Claim::Forwarded)
        }
    }
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
    use std::path::PathBuf;

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
        // After `--`, a file may look like an option.
        let r = req(&["--", "--screen=send", "-platform"], Path::new("/"));
        assert_eq!(r.items, ["--screen=send", "-platform"]);
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
        ];
        uris.extend((0..MAX_ARGS + 10).map(|i| format!("/f{i}")));
        let r = Request::from_open(&uris, Some("../../etc"));
        assert_eq!(r.items[0], "/ok");
        assert_eq!(r.items.len(), MAX_ARGS - 2);
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
