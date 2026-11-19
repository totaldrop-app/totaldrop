// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Desktop-wide keyboard shortcut through `org.freedesktop.portal.GlobalShortcuts` (#54).
//!
//! At window start, TotalDrop opens a `GlobalShortcuts` session on the session
//! bus and checks `ListShortcuts` without opening any system dialog. If the
//! user already bound `send-clipboard` in an earlier run, pressing that
//! shortcut brings TotalDrop forward on the Send screen with the clipboard's
//! text filled in — nothing is sent until the user clicks **Create code**.
//!
//! Clicking **Set shortcut** or **Change shortcut** in Settings binds
//! `send-clipboard` and (on portal version 2+) opens the desktop's shortcut
//! configuration dialog. On desktops without an interactive `GlobalShortcuts`
//! portal (`xdg-desktop-portal-hyprland` on Hyprland and Omarchy, or desktops
//! without the portal such as Sway and Xfce), `shortcutSupported` is `false`
//! and Settings shows the command to bind in the desktop or window manager
//! instead.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ashpd::ActivationToken;
use ashpd::desktop::global_shortcuts::{
    BindShortcutsOptions, ConfigureShortcutsOptions, GlobalShortcuts, ListShortcutsOptions,
    NewShortcut, Shortcut,
};
use ashpd::desktop::{CreateSessionOptions, Session};
use futures_util::StreamExt as _;

use crate::instance::Package;

/// Portal shortcut identifier for the "Send clipboard" action.
pub const SEND_CLIPBOARD_ID: &str = "send-clipboard";

/// Maximum characters kept from a portal `trigger_description` label.
pub const MAX_TRIGGER_CHARS: usize = 128;

/// Startup timeout for reaching `org.freedesktop.portal.GlobalShortcuts`.
const SETUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Command to bind in a desktop or window manager when the `GlobalShortcuts`
/// portal is unavailable (#54).
pub fn command_for(package: Package) -> &'static str {
    match package {
        Package::System => "totaldrop-desktop --screen=clipboard",
        Package::Flatpak => "flatpak run app.totaldrop.TotalDrop --screen=clipboard",
    }
}

/// Sanitizes an untrusted `trigger_description` from the portal backend so
/// control, line-break and bidirectional override characters become visible
/// code points instead of acting on the interface (#54).
pub fn clean_trigger(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let safe = totaldrop_l10n::reveal_controls(trimmed);
    match safe.char_indices().nth(MAX_TRIGGER_CHARS) {
        Some((at, _)) => safe.get(..at).unwrap_or_default().to_owned(),
        None => safe,
    }
}

/// Validates a Wayland/X11 activation token: bounded graphic ASCII (#178, #54).
pub fn valid_token(s: &str) -> Option<String> {
    if s.is_empty() || s.len() > 1024 || !s.bytes().all(|b| b.is_ascii_graphic()) {
        None
    } else {
        Some(s.to_owned())
    }
}

/// Extracts a validated activation token from a portal `options` map.
pub fn activation_token(options: &HashMap<String, zbus::OwnedVariant>) -> Option<String> {
    let v = options
        .get("activation_token")
        .or_else(|| options.get("activation-token"))?;
    let s: &str = v.downcast_ref().ok()?;
    valid_token(s)
}

/// Finds `SEND_CLIPBOARD_ID` in a `(id, trigger_description)` list, returning
/// `Some(sanitized_trigger)` when bound or `None` when not bound.
pub fn extract_trigger<I, K, V>(shortcuts: I) -> Option<String>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    shortcuts
        .into_iter()
        .find(|(id, _)| id.as_ref() == SEND_CLIPBOARD_ID)
        .map(|(_, trigger)| clean_trigger(trigger.as_ref()))
}

fn trigger_from_portal(shortcuts: &[Shortcut]) -> Option<String> {
    extract_trigger(shortcuts.iter().map(|s| (s.id(), s.trigger_description())))
}

fn session_object_path(session: &Session<GlobalShortcuts>) -> Option<String> {
    match serde_json::to_value(session).ok()? {
        serde_json::Value::String(path) if !path.is_empty() => Some(path),
        _ => None,
    }
}

fn session_matches(expected: Option<&str>, got: &str) -> bool {
    expected.is_some_and(|e| e == got)
}

/// Callbacks from the `GlobalShortcuts` portal task to the window.
pub struct Callbacks {
    /// Portal availability (`supported`) and current sanitized trigger (`""` when unset).
    pub state: Box<dyn Fn(bool, String) + Send + Sync>,
    /// User pressed the `send-clipboard` shortcut; carries an optional
    /// validated activation token.
    pub activated: Box<dyn Fn(Option<String>) + Send + Sync>,
}

/// Request from Settings to bind or change the `send-clipboard` shortcut.
pub struct Configure {
    /// Localized description shown by the desktop's shortcut settings.
    pub description: String,
    /// Optional activation token for focusing the desktop's shortcut dialog.
    pub token: Option<String>,
}

struct HandleInner {
    tx: tokio::sync::mpsc::Sender<Configure>,
    configuring: AtomicBool,
}

struct ResetConfiguring<'a>(&'a AtomicBool);

impl Drop for ResetConfiguring<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

/// Non-blocking handle held by `Launcher` to open the desktop's shortcut dialog.
#[derive(Clone, Default)]
pub struct Handle {
    inner: Option<Arc<HandleInner>>,
}

impl Handle {
    /// Asks the portal task to bind or configure `send-clipboard`. Ignored if
    /// no portal task is running or a configuration dialog is already open.
    pub fn configure(&self, description: String, token: Option<String>) {
        let Some(inner) = &self.inner else {
            return;
        };
        if inner
            .configuring
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            && inner.tx.try_send(Configure { description, token }).is_err()
        {
            inner.configuring.store(false, Ordering::Release);
        }
    }
}

fn desktop_mentions_hyprland(xdg_current_desktop: Option<&std::ffi::OsStr>) -> bool {
    xdg_current_desktop.is_some_and(|v| {
        v.to_string_lossy()
            .split(':')
            .any(|d| d.eq_ignore_ascii_case("Hyprland"))
    })
}

/// Starts the background `GlobalShortcuts` portal listener on [`crate::runtime::RUNTIME`].
pub fn start(callbacks: Callbacks) -> Handle {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let inner = Arc::new(HandleInner {
        tx,
        configuring: AtomicBool::new(false),
    });
    let task_inner = Arc::clone(&inner);
    crate::runtime::RUNTIME.spawn(async move {
        let Ok(conn) = zbus::Connection::session().await else {
            return;
        };
        let register_host = !crate::theme::host::sandboxed();
        let hyprland = crate::theme::hyprland::running()
            || desktop_mentions_hyprland(std::env::var_os("XDG_CURRENT_DESKTOP").as_deref());
        run_on(conn, register_host, hyprland, callbacks, rx, task_inner).await;
    });
    Handle { inner: Some(inner) }
}

struct PortalSetup {
    portal: GlobalShortcuts,
    session: Session<GlobalShortcuts>,
    session_path: Option<String>,
    session_closed: zbus::proxy::SignalStream<'static>,
    bound: bool,
    trigger: String,
}

async fn watch_session_closed(
    conn: &zbus::Connection,
    session_path: Option<&str>,
) -> Option<zbus::proxy::SignalStream<'static>> {
    let path = zbus::wire::OwnedObjectPath::try_from(session_path?.to_owned()).ok()?;
    let proxy: zbus::Proxy<'static> = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.portal.Desktop")
        .path(path)
        .interface("org.freedesktop.portal.Session")
        .cache_properties(zbus::proxy::CacheProperties::No)
        .build()
        .await
        .ok()?;
    proxy.receive_signal("Closed").await.ok()
}

async fn setup_portal(conn: &zbus::Connection, register_host: bool) -> Option<PortalSetup> {
    if register_host && let Ok(app_id) = ashpd::AppID::try_from(crate::APP_ID) {
        // Best-effort: older xdg-desktop-portal releases have no host Registry.
        let _ = ashpd::register_host_app_with_connection(conn.clone(), app_id).await;
    }
    let portal = GlobalShortcuts::with_connection(conn.clone()).await.ok()?;
    let session = portal
        .create_session(CreateSessionOptions::default())
        .await
        .ok()?;
    let session_path = session_object_path(&session);
    let session_closed = watch_session_closed(conn, session_path.as_deref()).await?;
    let listed = portal
        .list_shortcuts(&session, ListShortcutsOptions::default())
        .await
        .ok()?;
    let response = listed.response().ok()?;
    let found = trigger_from_portal(response.shortcuts());
    Some(PortalSetup {
        portal,
        session,
        session_path,
        session_closed,
        bound: found.is_some(),
        trigger: found.unwrap_or_default(),
    })
}

async fn open_configure_dialog(
    portal: &GlobalShortcuts,
    session: &Session<GlobalShortcuts>,
    token: Option<&str>,
) {
    let mut opts = ConfigureShortcutsOptions::default();
    if let Some(tok) = token {
        opts = opts.set_activation_token(ActivationToken::from(tok));
    }
    let _ = portal.configure_shortcuts(session, None, opts).await;
}

async fn refresh_from_list(
    portal: &GlobalShortcuts,
    session: &Session<GlobalShortcuts>,
) -> Option<Option<String>> {
    let listed = portal
        .list_shortcuts(session, ListShortcutsOptions::default())
        .await
        .ok()?;
    let resp = listed.response().ok()?;
    Some(trigger_from_portal(resp.shortcuts()))
}

/// Runs the `GlobalShortcuts` session loop on `conn` (testable on a `PrivateBus`).
async fn run_on(
    conn: zbus::Connection,
    register_host: bool,
    hyprland: bool,
    callbacks: Callbacks,
    mut rx: tokio::sync::mpsc::Receiver<Configure>,
    inner: Arc<HandleInner>,
) {
    let Ok(Some(setup)) =
        tokio::time::timeout(SETUP_TIMEOUT, setup_portal(&conn, register_host)).await
    else {
        return;
    };
    let portal = setup.portal;
    let Ok(activated_raw) = portal.receive_activated().await else {
        let _ = setup.session.close().await;
        return;
    };
    let Ok(changed_raw) = portal.receive_shortcuts_changed().await else {
        let _ = setup.session.close().await;
        return;
    };
    let mut activated = Box::pin(activated_raw);
    let mut changed = Box::pin(changed_raw);
    let mut session = setup.session;
    let mut session_path = setup.session_path;
    let mut session_closed = setup.session_closed;
    let mut bound = setup.bound;
    let mut bind_attempted = false;
    let mut trigger = setup.trigger;
    let mut supported = !(portal.version() < 2 && hyprland && trigger.is_empty());

    (callbacks.state)(supported, trigger.clone());

    loop {
        tokio::select! {
            ev = activated.next() => {
                let Some(ev) = ev else { break };
                if session_matches(session_path.as_deref(), ev.session_handle().as_str())
                    && ev.shortcut_id() == SEND_CLIPBOARD_ID
                {
                    (callbacks.activated)(activation_token(ev.options()));
                }
            }
            ev = changed.next() => {
                let Some(ev) = ev else { break };
                if !session_matches(session_path.as_deref(), ev.session_handle().as_str()) {
                    continue;
                }
                let found = trigger_from_portal(ev.shortcuts());
                bound = found.is_some();
                let next_trigger = found.unwrap_or_default();
                let next_supported = if !next_trigger.is_empty() || portal.version() >= 2 {
                    true
                } else if hyprland {
                    false
                } else {
                    supported
                };
                if next_supported != supported || next_trigger != trigger {
                    supported = next_supported;
                    trigger = next_trigger;
                    (callbacks.state)(supported, trigger.clone());
                }
            }
            closed = session_closed.next() => {
                if closed.is_some() && supported {
                    (callbacks.state)(false, String::new());
                }
                break;
            }
            cmd = rx.recv() => {
                let Some(req) = cmd else {
                    let _ = session.close().await;
                    break;
                };
                let _guard = ResetConfiguring(&inner.configuring);
                if !bound && !(portal.version() < 2 && bind_attempted) {
                    bind_attempted = true;
                    let item = NewShortcut::new(SEND_CLIPBOARD_ID, req.description);
                    let Ok(request) = portal
                        .bind_shortcuts(&session, &[item], None, BindShortcutsOptions::default())
                        .await
                    else {
                        supported = false;
                        trigger.clear();
                        (callbacks.state)(false, String::new());
                        continue;
                    };
                    let resp = match request.response() {
                        Ok(resp) => resp,
                        Err(ashpd::Error::Response(ashpd::desktop::ResponseError::Cancelled)) => {
                            continue;
                        }
                        Err(_) => {
                            supported = false;
                            trigger.clear();
                            (callbacks.state)(false, String::new());
                            continue;
                        }
                    };
                    match trigger_from_portal(resp.shortcuts()) {
                        Some(next_trigger) => {
                            bound = true;
                            if next_trigger.is_empty() && portal.version() >= 2 {
                                // Portal v2 (e.g. KDE Plasma) binds silently and configures
                                // keys in ConfigureShortcuts.
                                open_configure_dialog(&portal, &session, req.token.as_deref()).await;
                                if let Some(Some(refreshed)) = refresh_from_list(&portal, &session).await {
                                    trigger = refreshed;
                                }
                                supported = true;
                                (callbacks.state)(supported, trigger.clone());
                            } else if next_trigger.is_empty() {
                                // Portal v1 backend without an interactive picker (such as
                                // xdg-desktop-portal-hyprland): reveal manual command while
                                // keeping the session alive for ShortcutsChanged / Activated.
                                supported = false;
                                trigger.clear();
                                (callbacks.state)(false, String::new());
                            } else {
                                supported = true;
                                trigger = next_trigger;
                                (callbacks.state)(true, trigger.clone());
                            }
                        }
                        None if portal.version() < 2 => {
                            supported = false;
                            trigger.clear();
                            (callbacks.state)(false, String::new());
                        }
                        None => {}
                    }
                } else if portal.version() >= 2 {
                    open_configure_dialog(&portal, &session, req.token.as_deref()).await;
                    if let Some(found) = refresh_from_list(&portal, &session).await {
                        bound = found.is_some();
                        let refreshed = found.unwrap_or_default();
                        if !supported || refreshed != trigger {
                            supported = true;
                            trigger = refreshed;
                            (callbacks.state)(true, trigger.clone());
                        }
                    }
                } else {
                    // Portal v1 allows BindShortcuts once per session; recreate the
                    // session so a v1 dialog backend can rebind.
                    let _ = session.close().await;
                    let Ok(new_session) = portal.create_session(CreateSessionOptions::default()).await else {
                        (callbacks.state)(false, String::new());
                        break;
                    };
                    let next_path = session_object_path(&new_session);
                    let Some(new_closed) = watch_session_closed(&conn, next_path.as_deref()).await else {
                        let _ = new_session.close().await;
                        (callbacks.state)(false, String::new());
                        break;
                    };
                    session_path = next_path;
                    session_closed = new_closed;
                    session = new_session;
                    bind_attempted = true;
                    let item = NewShortcut::new(SEND_CLIPBOARD_ID, req.description);
                    let rebound = match portal
                        .bind_shortcuts(&session, &[item], None, BindShortcutsOptions::default())
                        .await
                    {
                        Ok(request) => match request.response() {
                            Ok(resp) => Ok(trigger_from_portal(resp.shortcuts())),
                            Err(err) => Err(matches!(
                                err,
                                ashpd::Error::Response(ashpd::desktop::ResponseError::Cancelled)
                            )),
                        },
                        Err(_) => Err(false),
                    };
                    match rebound {
                        Ok(Some(next_trigger)) => {
                            bound = true;
                            let next_supported = !next_trigger.is_empty();
                            if next_supported != supported || next_trigger != trigger {
                                supported = next_supported;
                                trigger = next_trigger;
                                (callbacks.state)(supported, trigger.clone());
                            }
                        }
                        Ok(None) | Err(_) => {
                            let cancelled = matches!(rebound, Err(true));
                            if let Some(found) = refresh_from_list(&portal, &session).await {
                                bound = found.is_some();
                                let next_trigger = found.unwrap_or_default();
                                let next_supported = if !next_trigger.is_empty() {
                                    true
                                } else if hyprland || !cancelled {
                                    false
                                } else {
                                    supported
                                };
                                if next_supported != supported || next_trigger != trigger {
                                    supported = next_supported;
                                    trigger = next_trigger;
                                    (callbacks.state)(supported, trigger.clone());
                                }
                            } else {
                                bound = false;
                                if supported || !trigger.is_empty() {
                                    supported = false;
                                    trigger.clear();
                                    (callbacks.state)(false, String::new());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    use super::*;
    use std::io::BufRead as _;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::sync::Mutex;
    use std::time::Duration;
    use zbus::wire::{ObjectPath, OwnedObjectPath, Variant};

    #[test]
    fn command_matches_package_kind() {
        assert_eq!(
            command_for(Package::System),
            "totaldrop-desktop --screen=clipboard"
        );
        assert_eq!(
            command_for(Package::Flatpak),
            "flatpak run app.totaldrop.TotalDrop --screen=clipboard"
        );
    }

    #[test]
    fn desktop_detection_matches_hyprland_and_omarchy() {
        use std::ffi::OsStr;

        assert!(desktop_mentions_hyprland(Some(OsStr::new("Hyprland"))));
        assert!(desktop_mentions_hyprland(Some(OsStr::new(
            "Omarchy:Hyprland"
        ))));
        assert!(desktop_mentions_hyprland(Some(OsStr::new("hyprland"))));
        assert!(!desktop_mentions_hyprland(Some(OsStr::new("KDE"))));
        assert!(!desktop_mentions_hyprland(Some(OsStr::new(""))));
        assert!(!desktop_mentions_hyprland(None));
    }

    #[test]
    fn clean_trigger_sanitizes_controls_bidi_and_bounds_length() {
        assert_eq!(clean_trigger(""), "");
        assert_eq!(clean_trigger("   \t\n  "), "");
        assert_eq!(clean_trigger("  Meta+Shift+V  "), "Meta+Shift+V");
        let spoofed = clean_trigger("Ctrl+\u{202E}V\u{001B}[31m\nX");
        assert!(!spoofed.contains('\u{202E}'));
        assert!(!spoofed.contains('\u{001B}'));
        assert!(!spoofed.contains('\n'));
        assert_eq!(spoofed, "Ctrl+<U+202E>V<U+001B>[31m<U+000A>X");

        let long = "ğ".repeat(MAX_TRIGGER_CHARS + 25);
        let bounded = clean_trigger(&long);
        assert_eq!(bounded.chars().count(), MAX_TRIGGER_CHARS);
    }

    #[test]
    fn extract_trigger_distinguishes_unbound_empty_and_bound() {
        let other = [("other-action", "Ctrl+O")];
        assert_eq!(extract_trigger(other), None);

        let empty_bound = [("other-action", "Ctrl+O"), ("send-clipboard", "   ")];
        assert_eq!(extract_trigger(empty_bound), Some(String::new()));

        let set_bound = [("send-clipboard", "Super+Shift+V\u{202E}")];
        assert_eq!(
            extract_trigger(set_bound),
            Some("Super+Shift+V<U+202E>".to_owned())
        );
    }

    #[test]
    fn activation_token_accepts_only_bounded_printable_ascii() {
        let mut opts = HashMap::new();
        assert_eq!(activation_token(&opts), None);

        opts.insert(
            "activation_token".to_owned(),
            Variant::from("tok_wayland-123").try_to_owned().unwrap(),
        );
        assert_eq!(activation_token(&opts).as_deref(), Some("tok_wayland-123"));

        let mut hyphen_opts = HashMap::new();
        hyphen_opts.insert(
            "activation-token".to_owned(),
            Variant::from("tok_legacy_456").try_to_owned().unwrap(),
        );
        assert_eq!(
            activation_token(&hyphen_opts).as_deref(),
            Some("tok_legacy_456")
        );

        for bad in ["", "has space", "line\nbreak", "non\u{00E9}ascii"] {
            let mut m = HashMap::new();
            m.insert(
                "activation_token".to_owned(),
                Variant::from(bad).try_to_owned().unwrap(),
            );
            assert_eq!(activation_token(&m), None, "{bad:?}");
        }
        let over = "a".repeat(1025);
        let mut m = HashMap::new();
        m.insert(
            "activation_token".to_owned(),
            Variant::from(over.as_str()).try_to_owned().unwrap(),
        );
        assert_eq!(activation_token(&m), None);
    }

    /// Private `dbus-daemon` bus isolated from the host session.
    struct PrivateBus {
        daemon: Child,
        address: String,
        dir: PathBuf,
    }

    impl PrivateBus {
        fn start() -> Option<Self> {
            static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let dir = Path::new("/tmp").join(format!("td-shortcuts-{}-{n}", std::process::id()));
            std::fs::create_dir(&dir).ok()?;
            let Ok(mut daemon) = Command::new("dbus-daemon")
                .args(["--session", "--nofork", "--print-address=1"])
                .arg(format!("--address=unix:dir={}", dir.display()))
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
            else {
                let _ = std::fs::remove_dir_all(&dir);
                return None;
            };
            let stdout = daemon.stdout.take()?;
            let mut address = String::new();
            std::io::BufReader::new(stdout)
                .read_line(&mut address)
                .ok()?;
            let address = address.trim().to_owned();
            if address.is_empty() {
                let _ = daemon.kill();
                let _ = daemon.wait();
                let _ = std::fs::remove_dir_all(&dir);
                return None;
            }
            Some(Self {
                daemon,
                address,
                dir,
            })
        }

        async fn connect(&self) -> zbus::Connection {
            zbus::connection::Builder::address(self.address.as_str())
                .build()
                .await
                .unwrap()
        }
    }

    impl Drop for PrivateBus {
        fn drop(&mut self) {
            let _ = self.daemon.kill();
            let _ = self.daemon.wait();
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn skip_without_dbus(why: &str) {
        assert!(std::env::var_os("CI").is_none(), "{why}");
        eprintln!("skipped: {why}");
    }

    fn portal_object_path(prefix: &str, sender: &str, token: &str) -> OwnedObjectPath {
        let who = sender.trim_start_matches(':').replace('.', "_");
        ObjectPath::try_from(format!("{prefix}/{who}/{token}"))
            .unwrap()
            .into()
    }

    fn token_of(options: &HashMap<String, zbus::OwnedVariant>, key: &str) -> String {
        options
            .get(key)
            .and_then(|v| v.downcast_ref::<&str>().ok())
            .unwrap_or("tok")
            .to_owned()
    }

    #[derive(Clone)]
    enum MockBindOutcome {
        BindTrigger(String),
        EmptyList,
        ResponseCode(u32),
    }

    impl Default for MockBindOutcome {
        fn default() -> Self {
            Self::BindTrigger(String::new())
        }
    }

    #[derive(Default)]
    struct MockState {
        session_path: String,
        shortcuts: Vec<(String, String)>,
        create_session_calls: usize,
        bind_calls: usize,
        configure_calls: usize,
        bind_outcome: MockBindOutcome,
        bound_trigger_on_configure: String,
        clear_shortcuts_on_create_session: bool,
    }

    struct MockSession;

    #[zbus::interface(name = "org.freedesktop.portal.Session")]
    impl MockSession {
        fn close(&self) {}
    }

    struct MockPortal {
        conn: zbus::Connection,
        version: u32,
        state: Arc<Mutex<MockState>>,
    }

    #[zbus::interface(name = "org.freedesktop.portal.GlobalShortcuts")]
    impl MockPortal {
        #[zbus(property(emits_changed_signal = "const"), name = "version")]
        fn version(&self) -> u32 {
            self.version
        }

        async fn create_session(
            &self,
            #[zbus(header)] hdr: zbus::message::Header<'_>,
            options: HashMap<String, zbus::OwnedVariant>,
        ) -> OwnedObjectPath {
            let sender = hdr.sender().unwrap().as_str().to_owned();
            let req = portal_object_path(
                "/org/freedesktop/portal/desktop/request",
                &sender,
                &token_of(&options, "handle_token"),
            );
            let sess = portal_object_path(
                "/org/freedesktop/portal/desktop/session",
                &sender,
                &token_of(&options, "session_handle_token"),
            );
            let _ = self.conn.object_server().at(&sess, MockSession).await;
            {
                let mut st = self.state.lock().unwrap();
                st.create_session_calls += 1;
                if st.clear_shortcuts_on_create_session {
                    st.shortcuts.clear();
                }
                st.session_path = sess.to_string();
            }

            let conn = self.conn.clone();
            let req_emit = req.clone();
            let sess_str = sess.to_string();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(15)).await;
                let mut res: HashMap<&str, Variant<'_>> = HashMap::new();
                res.insert("session_handle", Variant::from(sess_str.as_str()));
                let _ = conn
                    .emit_signal(
                        Some(sender.as_str()),
                        &req_emit,
                        "org.freedesktop.portal.Request",
                        "Response",
                        &(0u32, res),
                    )
                    .await;
            });
            req
        }

        async fn list_shortcuts(
            &self,
            #[zbus(header)] hdr: zbus::message::Header<'_>,
            _session_handle: OwnedObjectPath,
            options: HashMap<String, zbus::OwnedVariant>,
        ) -> OwnedObjectPath {
            let sender = hdr.sender().unwrap().as_str().to_owned();
            let req = portal_object_path(
                "/org/freedesktop/portal/desktop/request",
                &sender,
                &token_of(&options, "handle_token"),
            );
            let current = self.state.lock().unwrap().shortcuts.clone();
            let conn = self.conn.clone();
            let req_emit = req.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(15)).await;
                let encoded: Vec<(String, HashMap<&str, Variant<'_>>)> = current
                    .iter()
                    .map(|(id, trig)| {
                        let mut m = HashMap::new();
                        m.insert("description", Variant::from("Send clipboard"));
                        m.insert("trigger_description", Variant::from(trig.as_str()));
                        (id.clone(), m)
                    })
                    .collect();
                let mut res: HashMap<&str, Variant<'_>> = HashMap::new();
                res.insert("shortcuts", Variant::from(encoded));
                let _ = conn
                    .emit_signal(
                        Some(sender.as_str()),
                        &req_emit,
                        "org.freedesktop.portal.Request",
                        "Response",
                        &(0u32, res),
                    )
                    .await;
            });
            req
        }

        async fn bind_shortcuts(
            &self,
            #[zbus(header)] hdr: zbus::message::Header<'_>,
            _session_handle: OwnedObjectPath,
            shortcuts: Vec<(String, HashMap<String, zbus::OwnedVariant>)>,
            _parent_window: String,
            options: HashMap<String, zbus::OwnedVariant>,
        ) -> OwnedObjectPath {
            let sender = hdr.sender().unwrap().as_str().to_owned();
            let req = portal_object_path(
                "/org/freedesktop/portal/desktop/request",
                &sender,
                &token_of(&options, "handle_token"),
            );
            let ids: Vec<String> = shortcuts.into_iter().map(|(id, _)| id).collect();
            let outcome = {
                let mut st = self.state.lock().unwrap();
                st.bind_calls += 1;
                let out = st.bind_outcome.clone();
                if let MockBindOutcome::BindTrigger(ref trig) = out {
                    st.shortcuts = ids.iter().map(|id| (id.clone(), trig.clone())).collect();
                }
                out
            };
            let conn = self.conn.clone();
            let req_emit = req.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(15)).await;
                match outcome {
                    MockBindOutcome::BindTrigger(trig) => {
                        let encoded: Vec<(String, HashMap<&str, Variant<'_>>)> = ids
                            .iter()
                            .map(|id| {
                                let mut m = HashMap::new();
                                m.insert("description", Variant::from("Send clipboard"));
                                m.insert("trigger_description", Variant::from(trig.as_str()));
                                (id.clone(), m)
                            })
                            .collect();
                        let mut res: HashMap<&str, Variant<'_>> = HashMap::new();
                        res.insert("shortcuts", Variant::from(encoded));
                        let _ = conn
                            .emit_signal(
                                Some(sender.as_str()),
                                &req_emit,
                                "org.freedesktop.portal.Request",
                                "Response",
                                &(0u32, res),
                            )
                            .await;
                    }
                    MockBindOutcome::EmptyList => {
                        let empty: Vec<(String, HashMap<&str, Variant<'_>>)> = Vec::new();
                        let mut res: HashMap<&str, Variant<'_>> = HashMap::new();
                        res.insert("shortcuts", Variant::from(empty));
                        let _ = conn
                            .emit_signal(
                                Some(sender.as_str()),
                                &req_emit,
                                "org.freedesktop.portal.Request",
                                "Response",
                                &(0u32, res),
                            )
                            .await;
                    }
                    MockBindOutcome::ResponseCode(code) => {
                        let res: HashMap<&str, Variant<'_>> = HashMap::new();
                        let _ = conn
                            .emit_signal(
                                Some(sender.as_str()),
                                &req_emit,
                                "org.freedesktop.portal.Request",
                                "Response",
                                &(code, res),
                            )
                            .await;
                    }
                }
            });
            req
        }

        async fn configure_shortcuts(
            &self,
            #[zbus(header)] hdr: zbus::message::Header<'_>,
            session_handle: OwnedObjectPath,
            _parent_window: String,
            _options: HashMap<String, zbus::OwnedVariant>,
        ) {
            let sender = hdr.sender().unwrap().as_str().to_owned();
            let updated = {
                let mut st = self.state.lock().unwrap();
                st.configure_calls += 1;
                let trig = st.bound_trigger_on_configure.clone();
                for (_, t) in &mut st.shortcuts {
                    *t = trig.clone();
                }
                st.shortcuts.clone()
            };
            let encoded: Vec<(String, HashMap<&str, Variant<'_>>)> = updated
                .iter()
                .map(|(id, trig)| {
                    let mut m = HashMap::new();
                    m.insert("description", Variant::from("Send clipboard"));
                    m.insert("trigger_description", Variant::from(trig.as_str()));
                    (id.clone(), m)
                })
                .collect();
            let _ = self
                .conn
                .emit_signal(
                    Some(sender.as_str()),
                    "/org/freedesktop/portal/desktop",
                    "org.freedesktop.portal.GlobalShortcuts",
                    "ShortcutsChanged",
                    &(&session_handle, encoded),
                )
                .await;
        }
    }

    #[tokio::test]
    async fn portal_unavailable_leaves_shortcut_unsupported() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let conn = bus.connect().await;
        let seen = Arc::new(Mutex::new(Vec::<(bool, String)>::new()));
        let seen_cb = Arc::clone(&seen);
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx,
            configuring: AtomicBool::new(false),
        });
        run_on(
            conn,
            false,
            false,
            Callbacks {
                state: Box::new(move |sup, trig| seen_cb.lock().unwrap().push((sup, trig))),
                activated: Box::new(|_| {}),
            },
            rx,
            inner,
        )
        .await;
        assert!(seen.lock().unwrap().is_empty());
    }

    async fn wait_until_idle(inner: &HandleInner) {
        tokio::time::timeout(Duration::from_secs(3), async {
            while inner.configuring.load(Ordering::Acquire) {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn portal_v2_binds_configures_updates_and_activates_clipboard() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            bind_outcome: MockBindOutcome::BindTrigger(String::new()),
            bound_trigger_on_configure: "Meta+Shift+V\u{202E}".to_owned(),
            ..MockState::default()
        }));
        server
            .object_server()
            .at(
                "/org/freedesktop/portal/desktop",
                MockPortal {
                    conn: server.clone(),
                    version: 2,
                    state: Arc::clone(&state),
                },
            )
            .await
            .unwrap();
        server
            .request_name("org.freedesktop.portal.Desktop")
            .await
            .unwrap();

        let client = bus.connect().await;
        let client_name = client.unique_name().unwrap().to_string();
        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();
        let (act_tx, mut act_rx) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx: cmd_tx,
            configuring: AtomicBool::new(false),
        });
        let handle = Handle {
            inner: Some(Arc::clone(&inner)),
        };

        let task = tokio::spawn(run_on(
            client,
            false,
            false,
            Callbacks {
                state: Box::new(move |sup, trig| {
                    let _ = state_tx.send((sup, trig));
                }),
                activated: Box::new(move |tok| {
                    let _ = act_tx.send(tok);
                }),
            },
            cmd_rx,
            inner,
        ));

        // 1. Startup lists shortcuts without calling BindShortcuts.
        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, String::new()));
        assert_eq!(state.lock().unwrap().bind_calls, 0);

        // 2. Clicking Set shortcut binds `send-clipboard` and (on v2 when trigger
        //    is still empty) opens ConfigureShortcuts, sanitizing Bidi chars.
        handle.configure("Send clipboard".to_owned(), Some("tok_cfg".to_owned()));
        let mut last = initial;
        while last.1.is_empty() {
            last = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
                .await
                .unwrap()
                .unwrap();
        }
        assert_eq!(last, (true, "Meta+Shift+V<U+202E>".to_owned()));
        assert_eq!(state.lock().unwrap().bind_calls, 1);
        assert_eq!(state.lock().unwrap().configure_calls, 1);

        // 3. Unrelated shortcut IDs are ignored; `send-clipboard` fires `activated`
        //    with the validated activation token.
        let sess_path = ObjectPath::try_from(state.lock().unwrap().session_path.clone()).unwrap();
        let mut bad_opts: HashMap<&str, Variant<'_>> = HashMap::new();
        bad_opts.insert("activation_token", Variant::from("bad token with spaces"));
        server
            .emit_signal(
                Some(client_name.as_str()),
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.GlobalShortcuts",
                "Activated",
                &(&sess_path, "unrelated-action", 100u64, bad_opts),
            )
            .await
            .unwrap();

        let mut ok_opts: HashMap<&str, Variant<'_>> = HashMap::new();
        ok_opts.insert("activation_token", Variant::from("wayland_tok_99"));
        server
            .emit_signal(
                Some(client_name.as_str()),
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.GlobalShortcuts",
                "Activated",
                &(&sess_path, SEND_CLIPBOARD_ID, 101u64, ok_opts),
            )
            .await
            .unwrap();

        let got_tok = tokio::time::timeout(Duration::from_secs(3), act_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got_tok.as_deref(), Some("wayland_tok_99"));

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }

    #[tokio::test]
    async fn portal_v1_without_dialog_falls_back_to_manual_until_bound() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState::default()));
        server
            .object_server()
            .at(
                "/org/freedesktop/portal/desktop",
                MockPortal {
                    conn: server.clone(),
                    version: 1,
                    state: Arc::clone(&state),
                },
            )
            .await
            .unwrap();
        server
            .request_name("org.freedesktop.portal.Desktop")
            .await
            .unwrap();

        let client = bus.connect().await;
        let client_name = client.unique_name().unwrap().to_string();
        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx: cmd_tx,
            configuring: AtomicBool::new(false),
        });
        let handle = Handle {
            inner: Some(Arc::clone(&inner)),
        };

        let task = tokio::spawn(run_on(
            client,
            false,
            false,
            Callbacks {
                state: Box::new(move |sup, trig| {
                    let _ = state_tx.send((sup, trig));
                }),
                activated: Box::new(|_| {}),
            },
            cmd_rx,
            inner,
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, String::new()));

        // v1 backend returns Ok with empty trigger -> switches to manual fallback.
        handle.configure("Send clipboard".to_owned(), None);
        let fallback = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fallback, (false, String::new()));

        // If compositor later binds a key and emits ShortcutsChanged, UI updates live.
        let sess_path = ObjectPath::try_from(state.lock().unwrap().session_path.clone()).unwrap();
        let mut meta = HashMap::new();
        meta.insert("description", Variant::from("Send clipboard"));
        meta.insert("trigger_description", Variant::from("SUPER+SHIFT+V"));
        server
            .emit_signal(
                Some(client_name.as_str()),
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.GlobalShortcuts",
                "ShortcutsChanged",
                &(&sess_path, vec![(SEND_CLIPBOARD_ID.to_owned(), meta)]),
            )
            .await
            .unwrap();

        let live = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(live, (true, "SUPER+SHIFT+V".to_owned()));

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }

    #[tokio::test]
    async fn portal_restores_saved_shortcut_at_startup_and_reconfigures_without_rebinding() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            shortcuts: vec![(SEND_CLIPBOARD_ID.to_owned(), "Alt+Shift+V".to_owned())],
            bound_trigger_on_configure: "Ctrl+Shift+V".to_owned(),
            ..MockState::default()
        }));
        server
            .object_server()
            .at(
                "/org/freedesktop/portal/desktop",
                MockPortal {
                    conn: server.clone(),
                    version: 2,
                    state: Arc::clone(&state),
                },
            )
            .await
            .unwrap();
        server
            .request_name("org.freedesktop.portal.Desktop")
            .await
            .unwrap();

        let client = bus.connect().await;
        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx: cmd_tx,
            configuring: AtomicBool::new(false),
        });
        let handle = Handle {
            inner: Some(Arc::clone(&inner)),
        };

        let task = tokio::spawn(run_on(
            client,
            true,
            false,
            Callbacks {
                state: Box::new(move |sup, trig| {
                    let _ = state_tx.send((sup, trig));
                }),
                activated: Box::new(|_| {}),
            },
            cmd_rx,
            inner,
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, "Alt+Shift+V".to_owned()));
        assert_eq!(state.lock().unwrap().bind_calls, 0);

        // Reconfiguring an already-bound shortcut calls ConfigureShortcuts directly.
        handle.configure("Send clipboard".to_owned(), None);
        let updated = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated, (true, "Ctrl+Shift+V".to_owned()));
        assert_eq!(state.lock().unwrap().bind_calls, 0);
        assert_eq!(state.lock().unwrap().configure_calls, 1);

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }

    #[tokio::test]
    async fn hyprland_portal_v1_starts_with_manual_command_yet_receives_live_global_shortcut() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState::default()));
        server
            .object_server()
            .at(
                "/org/freedesktop/portal/desktop",
                MockPortal {
                    conn: server.clone(),
                    version: 1,
                    state: Arc::clone(&state),
                },
            )
            .await
            .unwrap();
        server
            .request_name("org.freedesktop.portal.Desktop")
            .await
            .unwrap();

        let client = bus.connect().await;
        let client_name = client.unique_name().unwrap().to_string();
        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();
        let (act_tx, mut act_rx) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx: cmd_tx,
            configuring: AtomicBool::new(false),
        });
        let handle = Handle {
            inner: Some(Arc::clone(&inner)),
        };

        let task = tokio::spawn(run_on(
            client,
            false,
            true,
            Callbacks {
                state: Box::new(move |sup, trig| {
                    let _ = state_tx.send((sup, trig));
                }),
                activated: Box::new(move |tok| {
                    let _ = act_tx.send(tok);
                }),
            },
            cmd_rx,
            inner,
        ));

        // 1. On Hyprland/Omarchy (portal v1, unbound), Settings immediately shows
        //    the copyable `--screen=clipboard` command box (`supported = false`).
        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (false, String::new()));

        // 2. If the user binds `global, app.totaldrop.TotalDrop:send-clipboard` in
        //    hyprland.conf, live ShortcutsChanged promotes state to `(true, trigger)`
        //    and Activated reaches the open window.
        let sess_path = ObjectPath::try_from(state.lock().unwrap().session_path.clone()).unwrap();
        let mut meta = HashMap::new();
        meta.insert("description", Variant::from("Send clipboard"));
        meta.insert("trigger_description", Variant::from("SUPER + SHIFT + V"));
        server
            .emit_signal(
                Some(client_name.as_str()),
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.GlobalShortcuts",
                "ShortcutsChanged",
                &(&sess_path, vec![(SEND_CLIPBOARD_ID.to_owned(), meta)]),
            )
            .await
            .unwrap();

        let live = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(live, (true, "SUPER + SHIFT + V".to_owned()));

        let mut opts: HashMap<&str, Variant<'_>> = HashMap::new();
        opts.insert("activation_token", Variant::from("hypr_tok_7"));
        server
            .emit_signal(
                Some(client_name.as_str()),
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.GlobalShortcuts",
                "Activated",
                &(&sess_path, SEND_CLIPBOARD_ID, 42u64, opts),
            )
            .await
            .unwrap();
        let got_tok = tokio::time::timeout(Duration::from_secs(3), act_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got_tok.as_deref(), Some("hypr_tok_7"));

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }

    #[tokio::test]
    async fn portal_v1_bind_cancelled_preserves_button_while_empty_or_failed_falls_back() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            bind_outcome: MockBindOutcome::ResponseCode(1),
            ..MockState::default()
        }));
        server
            .object_server()
            .at(
                "/org/freedesktop/portal/desktop",
                MockPortal {
                    conn: server.clone(),
                    version: 1,
                    state: Arc::clone(&state),
                },
            )
            .await
            .unwrap();
        server
            .request_name("org.freedesktop.portal.Desktop")
            .await
            .unwrap();

        let client = bus.connect().await;
        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx: cmd_tx,
            configuring: AtomicBool::new(false),
        });
        let handle = Handle {
            inner: Some(Arc::clone(&inner)),
        };

        let task = tokio::spawn(run_on(
            client,
            false,
            false,
            Callbacks {
                state: Box::new(move |sup, trig| {
                    let _ = state_tx.send((sup, trig));
                }),
                activated: Box::new(|_| {}),
            },
            cmd_rx,
            Arc::clone(&inner),
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, String::new()));

        // 1. Response(1, {}) (`Cancelled`) preserves `(true, "")` silently.
        handle.configure("Send clipboard".to_owned(), None);
        wait_until_idle(&inner).await;
        assert_eq!(state.lock().unwrap().bind_calls, 1);
        assert!(state_rx.try_recv().is_err());

        // 2. Returning `shortcuts = []` on v1 falls back to `(false, "")`.
        state.lock().unwrap().bind_outcome = MockBindOutcome::EmptyList;
        handle.configure("Send clipboard".to_owned(), None);
        let empty_fallback = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(empty_fallback, (false, String::new()));

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;

        // 3. Response(2, {}) (`Other`) on v1 also falls back to `(false, "")`.
        let Some(bus2) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server2 = bus2.connect().await;
        let state2 = Arc::new(Mutex::new(MockState {
            bind_outcome: MockBindOutcome::ResponseCode(2),
            ..MockState::default()
        }));
        server2
            .object_server()
            .at(
                "/org/freedesktop/portal/desktop",
                MockPortal {
                    conn: server2.clone(),
                    version: 1,
                    state: Arc::clone(&state2),
                },
            )
            .await
            .unwrap();
        server2
            .request_name("org.freedesktop.portal.Desktop")
            .await
            .unwrap();

        let client2 = bus2.connect().await;
        let (state_tx2, mut state_rx2) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_tx2, cmd_rx2) = tokio::sync::mpsc::channel(1);
        let inner2 = Arc::new(HandleInner {
            tx: cmd_tx2,
            configuring: AtomicBool::new(false),
        });
        let handle2 = Handle {
            inner: Some(Arc::clone(&inner2)),
        };
        let task2 = tokio::spawn(run_on(
            client2,
            false,
            false,
            Callbacks {
                state: Box::new(move |sup, trig| {
                    let _ = state_tx2.send((sup, trig));
                }),
                activated: Box::new(|_| {}),
            },
            cmd_rx2,
            inner2,
        ));
        let initial2 = tokio::time::timeout(Duration::from_secs(3), state_rx2.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial2, (true, String::new()));

        handle2.configure("Send clipboard".to_owned(), None);
        let err_fallback = tokio::time::timeout(Duration::from_secs(3), state_rx2.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(err_fallback, (false, String::new()));

        drop(handle2);
        let _ = tokio::time::timeout(Duration::from_secs(2), task2).await;
    }

    #[tokio::test]
    async fn portal_v1_rebind_cancellation_syncs_state_with_new_session() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            shortcuts: vec![(SEND_CLIPBOARD_ID.to_owned(), "Alt+Shift+V".to_owned())],
            bind_outcome: MockBindOutcome::ResponseCode(1),
            clear_shortcuts_on_create_session: false,
            ..MockState::default()
        }));
        server
            .object_server()
            .at(
                "/org/freedesktop/portal/desktop",
                MockPortal {
                    conn: server.clone(),
                    version: 1,
                    state: Arc::clone(&state),
                },
            )
            .await
            .unwrap();
        server
            .request_name("org.freedesktop.portal.Desktop")
            .await
            .unwrap();

        let client = bus.connect().await;
        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx: cmd_tx,
            configuring: AtomicBool::new(false),
        });
        let handle = Handle {
            inner: Some(Arc::clone(&inner)),
        };

        let task = tokio::spawn(run_on(
            client,
            false,
            false,
            Callbacks {
                state: Box::new(move |sup, trig| {
                    let _ = state_tx.send((sup, trig));
                }),
                activated: Box::new(|_| {}),
            },
            cmd_rx,
            Arc::clone(&inner),
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, "Alt+Shift+V".to_owned()));
        assert_eq!(state.lock().unwrap().create_session_calls, 1);

        // 1. When the v1 backend keeps `"Alt+Shift+V"` across sessions and the user
        //    cancels re-bind, `refresh_from_list` preserves `bound = true` and trigger.
        handle.configure("Send clipboard".to_owned(), None);
        wait_until_idle(&inner).await;
        assert_eq!(state.lock().unwrap().create_session_calls, 2);
        assert!(state_rx.try_recv().is_err());

        // 2. When recreating the v1 session drops the old binding and the user cancels,
        //    `refresh_from_list` clears the stale trigger so UI matches `new_session`.
        state.lock().unwrap().clear_shortcuts_on_create_session = true;
        handle.configure("Send clipboard".to_owned(), None);
        let synced = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(synced, (true, String::new()));
        assert_eq!(state.lock().unwrap().create_session_calls, 3);

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }
}
