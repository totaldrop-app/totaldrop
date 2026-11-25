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

use ashpd::desktop::global_shortcuts::{
    BindShortcutsOptions, ConfigureShortcutsOptions, GlobalShortcuts, ListShortcutsOptions,
    NewShortcut, Shortcut,
};
use ashpd::desktop::{CreateSessionOptions, Session};
use ashpd::{ActivationToken, WindowIdentifier};
use futures_util::StreamExt as _;

use crate::instance::Package;

/// Portal shortcut identifier for the "Send clipboard" action.
pub const SEND_CLIPBOARD_ID: &str = "send-clipboard";

/// Maximum characters kept from a portal `trigger_description` label.
pub const MAX_TRIGGER_CHARS: usize = 128;

/// Startup timeout for reaching `org.freedesktop.portal.GlobalShortcuts`.
const SETUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Bounded wait when closing a half-initialized portal session after timeout.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(2);

/// Maximum time a Wayland xdg-foreign parent handle stays exported after a
/// non-blocking `ConfigureShortcuts` call if `ShortcutsChanged` never arrives
/// (for example when the user closes the portal dialog without saving) (#631).
const CONFIGURE_PARENT_TTL: Duration = Duration::from_secs(120);

/// Initial delay before recreating a `GlobalShortcuts` session after `Closed`.
const RECONNECT_INITIAL_BACKOFF: Duration = Duration::from_secs(1);

/// Maximum delay between `GlobalShortcuts` session reconnect attempts.
const RECONNECT_MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Consecutive failed reconnect attempts after `Session::Closed` before falling
/// back to `(false, "")` so a brief portal restart never flashes the manual
/// `--screen=clipboard` instruction in Settings (#638).
const RECONNECT_FALLBACK_ATTEMPTS: usize = 3;

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

/// Extracts the D-Bus object path of an `ashpd::desktop::Session`.
///
/// `ashpd 0.14.0` keeps `Session::path()` `pub(crate)` while its `Serialize`
/// impl serializes `ObjectPath` as a string (required by D-Bus signature `o`).
/// Guarded by `session_object_path_matches_real_dbus_session_handle` so an
/// `ashpd` update changing the serialized form fails CI instead of silently
/// dropping `Activated` events.
fn session_object_path(session: &Session<GlobalShortcuts>) -> Option<String> {
    match serde_json::to_value(session).ok()? {
        serde_json::Value::String(path) if !path.is_empty() => Some(path),
        _ => None,
    }
}

fn session_matches(expected: Option<&str>, got: &str) -> bool {
    expected.is_some_and(|e| e == got)
}

/// Parent window reference passed to `BindShortcuts` / `ConfigureShortcuts`
/// so the desktop's shortcut dialog opens above TotalDrop (#54, #631).
pub enum ParentWindow {
    /// X11 window ID (`x11:<XID>`).
    X11(u64),
    /// Wayland `zxdg_exporter_v2` exported handle (`wayland:<handle>`).
    ///
    /// `unexport` runs on [`Drop`] so the compositor handle stays exported for
    /// the duration of the portal request and is always cleaned up afterwards.
    Wayland {
        /// Validated handle string returned by `zxdg_exported_v2.handle`.
        handle: String,
        /// Cleanup callback that destroys the `zxdg_exported_v2` object on the
        /// Qt GUI thread once the handle is no longer needed.
        unexport: Option<Box<dyn FnOnce() + Send>>,
    },
}

impl Drop for ParentWindow {
    fn drop(&mut self) {
        if let Self::Wayland { unexport, .. } = self
            && let Some(cleanup) = unexport.take()
        {
            cleanup();
        }
    }
}

fn parent_window_identifier(parent: Option<&ParentWindow>) -> Option<WindowIdentifier> {
    match parent? {
        ParentWindow::X11(xid) => (*xid != 0)
            .then_some(*xid)
            .and_then(|id| std::os::raw::c_ulong::try_from(id).ok())
            .map(WindowIdentifier::from_xid),
        ParentWindow::Wayland { handle, .. } => {
            valid_token(handle).map(WindowIdentifier::from_xdg_foreign_exported)
        }
    }
}

/// Decides whether Settings should show the interactive portal button
/// (`true`) or fall back to the manual `--screen=clipboard` command (`false`).
///
/// Portal v2+ (`ConfigureShortcuts`) and any non-empty bound trigger always
/// show the portal UI. Portal v1 on Hyprland/Omarchy has no interactive picker
/// when unbound (`false`). Other v1 backends keep `prev` (`true` at startup or
/// after a cancelled dialog; `false` once a bind attempt returned empty or
/// failed).
fn supported(version: u32, hyprland: bool, trigger: &str, prev: bool) -> bool {
    if version >= 2 || !trigger.is_empty() {
        true
    } else if hyprland {
        false
    } else {
        prev
    }
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
    /// Optional parent window handle (`x11:<XID>` or Wayland `xdg-foreign-v2`
    /// handle) for stacking portal dialogs above TotalDrop (#54, #631).
    pub parent: Option<ParentWindow>,
}

struct HandleInner {
    tx: tokio::sync::mpsc::Sender<Configure>,
    configuring: Arc<AtomicBool>,
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
    /// no portal task is running or a configuration dialog is already open; any
    /// exported Wayland parent handle is dropped and unexported immediately.
    pub fn configure(
        &self,
        description: String,
        token: Option<String>,
        parent: Option<ParentWindow>,
    ) {
        let Some(inner) = &self.inner else {
            return;
        };
        if inner
            .configuring
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            && inner
                .tx
                .try_send(Configure {
                    description,
                    token,
                    parent,
                })
                .is_err()
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
    let configuring = Arc::new(AtomicBool::new(false));
    let task_configuring = Arc::clone(&configuring);
    crate::runtime::RUNTIME.spawn(async move {
        let Ok(conn) = zbus::Connection::session().await else {
            (callbacks.state)(false, String::new());
            return;
        };
        let register_host = !crate::theme::host::sandboxed();
        let hyprland = crate::theme::hyprland::running()
            || desktop_mentions_hyprland(std::env::var_os("XDG_CURRENT_DESKTOP").as_deref());
        run_on(
            conn,
            register_host,
            hyprland,
            callbacks,
            rx,
            task_configuring,
        )
        .await;
    });
    Handle {
        inner: Some(Arc::new(HandleInner { tx, configuring })),
    }
}

struct PortalSetup {
    portal: GlobalShortcuts,
    session: Session<GlobalShortcuts>,
    session_path: String,
    session_closed: zbus::proxy::SignalStream<'static>,
    bound: bool,
    trigger: String,
}

async fn watch_session_closed(
    conn: &zbus::Connection,
    session_path: &str,
) -> Option<zbus::proxy::SignalStream<'static>> {
    let path = zbus::wire::OwnedObjectPath::try_from(session_path.to_owned()).ok()?;
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

async fn close_session_bounded(session: Session<GlobalShortcuts>) {
    let _ = tokio::time::timeout(CLOSE_TIMEOUT, session.close()).await;
}

fn emit_state(
    callbacks: &Callbacks,
    last_state: &mut Option<(bool, String)>,
    supported: bool,
    trigger: String,
) {
    if last_state
        .as_ref()
        .is_some_and(|(prev_sup, prev_trig)| *prev_sup == supported && *prev_trig == trigger)
    {
        return;
    }
    *last_state = Some((supported, trigger.clone()));
    (callbacks.state)(supported, trigger);
}

async fn setup_portal(
    conn: &zbus::Connection,
    deadline: tokio::time::Instant,
) -> Option<PortalSetup> {
    let portal = tokio::time::timeout_at(deadline, GlobalShortcuts::with_connection(conn.clone()))
        .await
        .ok()?
        .ok()?;
    let session = tokio::time::timeout_at(
        deadline,
        portal.create_session(CreateSessionOptions::default()),
    )
    .await
    .ok()?
    .ok()?;
    let Some(session_path) = session_object_path(&session) else {
        close_session_bounded(session).await;
        return None;
    };
    let Ok(Some(session_closed)) =
        tokio::time::timeout_at(deadline, watch_session_closed(conn, &session_path)).await
    else {
        close_session_bounded(session).await;
        return None;
    };
    let Ok(Ok(listed)) = tokio::time::timeout_at(
        deadline,
        portal.list_shortcuts(&session, ListShortcutsOptions::default()),
    )
    .await
    else {
        close_session_bounded(session).await;
        return None;
    };
    let Ok(response) = listed.response() else {
        close_session_bounded(session).await;
        return None;
    };
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
    parent: Option<&WindowIdentifier>,
    token: Option<&str>,
) -> bool {
    let mut opts = ConfigureShortcutsOptions::default();
    if let Some(tok) = token.and_then(valid_token) {
        opts = opts.set_activation_token(ActivationToken::from(tok));
    }
    portal
        .configure_shortcuts(session, parent, opts)
        .await
        .is_ok()
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

enum SessionEnd {
    /// Window dropped its handle or the session bus went away: exit task.
    Stop,
    /// Portal closed our session (e.g. portal restart/update): reconnect with
    /// backoff.
    Closed,
}

#[allow(clippy::too_many_arguments)]
async fn run_session(
    conn: &zbus::Connection,
    hyprland: bool,
    callbacks: &Callbacks,
    last_state: &mut Option<(bool, String)>,
    rx: &mut tokio::sync::mpsc::Receiver<Configure>,
    configuring: &AtomicBool,
    configure_parent_ttl: Duration,
    setup: PortalSetup,
) -> SessionEnd {
    let portal = setup.portal;
    let Ok(activated_raw) = portal.receive_activated().await else {
        close_session_bounded(setup.session).await;
        emit_state(callbacks, last_state, false, String::new());
        return SessionEnd::Stop;
    };
    let Ok(changed_raw) = portal.receive_shortcuts_changed().await else {
        close_session_bounded(setup.session).await;
        emit_state(callbacks, last_state, false, String::new());
        return SessionEnd::Stop;
    };
    let mut activated = Box::pin(activated_raw);
    let mut changed = Box::pin(changed_raw);
    let mut session = setup.session;
    let mut session_path = setup.session_path;
    let mut session_closed = setup.session_closed;
    let mut bound = setup.bound;
    let mut bind_attempted = false;
    let mut trigger = setup.trigger;
    let mut is_supported = supported(portal.version(), hyprland, &trigger, true);
    // `ConfigureShortcuts` is a non-blocking D-Bus call that returns before the
    // desktop's shortcut dialog finishes importing the Wayland xdg-foreign
    // handle; retain the active handle until `ShortcutsChanged`, a newer
    // configure call, `configure_parent_ttl`, or session end (#631).
    let mut active_configure_parent: Option<ParentWindow> = None;
    let configure_parent_expiry = tokio::time::sleep(Duration::ZERO);
    tokio::pin!(configure_parent_expiry);

    emit_state(callbacks, last_state, is_supported, trigger.clone());

    loop {
        tokio::select! {
            () = &mut configure_parent_expiry, if active_configure_parent.is_some() => {
                let _expired = active_configure_parent.take();
            }
            ev = activated.next() => {
                let Some(ev) = ev else {
                    close_session_bounded(session).await;
                    return SessionEnd::Stop;
                };
                if session_matches(Some(&session_path), ev.session_handle().as_str())
                    && ev.shortcut_id() == SEND_CLIPBOARD_ID
                {
                    (callbacks.activated)(activation_token(ev.options()));
                }
            }
            ev = changed.next() => {
                let Some(ev) = ev else {
                    close_session_bounded(session).await;
                    return SessionEnd::Stop;
                };
                if !session_matches(Some(&session_path), ev.session_handle().as_str()) {
                    continue;
                }
                let _released = active_configure_parent.take();
                let found = trigger_from_portal(ev.shortcuts());
                bound = found.is_some();
                let next_trigger = found.unwrap_or_default();
                let next_supported = supported(portal.version(), hyprland, &next_trigger, is_supported);
                if next_supported != is_supported || next_trigger != trigger {
                    is_supported = next_supported;
                    trigger = next_trigger;
                    emit_state(callbacks, last_state, is_supported, trigger.clone());
                }
            }
            closed = session_closed.next() => {
                if closed.is_some() {
                    // Keep the last known UI state during transient portal
                    // restarts so Settings does not flash the manual fallback;
                    // the reconnect loop falls back only after repeated failures (#638).
                    return SessionEnd::Closed;
                }
                return SessionEnd::Stop;
            }
            cmd = rx.recv() => {
                let Some(mut req) = cmd else {
                    close_session_bounded(session).await;
                    return SessionEnd::Stop;
                };
                let _guard = ResetConfiguring(configuring);
                let parent = parent_window_identifier(req.parent.as_ref());
                if !bound && !(portal.version() < 2 && bind_attempted) {
                    bind_attempted = true;
                    let item = NewShortcut::new(SEND_CLIPBOARD_ID, req.description);
                    let Ok(request) = portal
                        .bind_shortcuts(
                            &session,
                            &[item],
                            parent.as_ref(),
                            BindShortcutsOptions::default(),
                        )
                        .await
                    else {
                        is_supported = false;
                        trigger.clear();
                        emit_state(callbacks, last_state, false, String::new());
                        continue;
                    };
                    let resp = match request.response() {
                        Ok(resp) => resp,
                        Err(ashpd::Error::Response(ashpd::desktop::ResponseError::Cancelled)) => {
                            continue;
                        }
                        Err(_) => {
                            is_supported = false;
                            trigger.clear();
                            emit_state(callbacks, last_state, false, String::new());
                            continue;
                        }
                    };
                    match trigger_from_portal(resp.shortcuts()) {
                        Some(next_trigger) => {
                            bound = true;
                            if next_trigger.is_empty() && portal.version() >= 2 {
                                // Portal v2 (e.g. KDE Plasma) binds silently and configures
                                // keys in ConfigureShortcuts.
                                if open_configure_dialog(
                                    &portal,
                                    &session,
                                    parent.as_ref(),
                                    req.token.as_deref(),
                                )
                                .await
                                {
                                    let _previous = match (parent.is_some(), req.parent.take()) {
                                        (true, Some(kept)) => {
                                            configure_parent_expiry.as_mut().reset(
                                                tokio::time::Instant::now() + configure_parent_ttl,
                                            );
                                            active_configure_parent.replace(kept)
                                        }
                                        _ => active_configure_parent.take(),
                                    };
                                }
                                if let Some(Some(refreshed)) = refresh_from_list(&portal, &session).await {
                                    trigger = refreshed;
                                }
                                is_supported = true;
                                emit_state(callbacks, last_state, is_supported, trigger.clone());
                            } else {
                                is_supported = supported(portal.version(), hyprland, &next_trigger, false);
                                trigger = if is_supported { next_trigger } else { String::new() };
                                emit_state(callbacks, last_state, is_supported, trigger.clone());
                            }
                        }
                        None if portal.version() < 2 => {
                            is_supported = false;
                            trigger.clear();
                            emit_state(callbacks, last_state, false, String::new());
                        }
                        None => {}
                    }
                } else if portal.version() >= 2 {
                    if open_configure_dialog(&portal, &session, parent.as_ref(), req.token.as_deref())
                        .await
                    {
                        let _previous = match (parent.is_some(), req.parent.take()) {
                            (true, Some(kept)) => {
                                configure_parent_expiry.as_mut().reset(
                                    tokio::time::Instant::now() + configure_parent_ttl,
                                );
                                active_configure_parent.replace(kept)
                            }
                            _ => active_configure_parent.take(),
                        };
                    }
                    if let Some(found) = refresh_from_list(&portal, &session).await {
                        bound = found.is_some();
                        let refreshed = found.unwrap_or_default();
                        if !is_supported || refreshed != trigger {
                            is_supported = true;
                            trigger = refreshed;
                            emit_state(callbacks, last_state, true, trigger.clone());
                        }
                    }
                } else {
                    // Portal v1 allows BindShortcuts once per session; recreate the
                    // session so a v1 dialog backend can rebind.
                    close_session_bounded(session).await;
                    let Ok(new_session) = portal.create_session(CreateSessionOptions::default()).await else {
                        emit_state(callbacks, last_state, false, String::new());
                        return SessionEnd::Stop;
                    };
                    let Some(next_path) = session_object_path(&new_session) else {
                        close_session_bounded(new_session).await;
                        emit_state(callbacks, last_state, false, String::new());
                        return SessionEnd::Stop;
                    };
                    let Some(new_closed) = watch_session_closed(conn, &next_path).await else {
                        close_session_bounded(new_session).await;
                        emit_state(callbacks, last_state, false, String::new());
                        return SessionEnd::Stop;
                    };
                    session_path = next_path;
                    session_closed = new_closed;
                    session = new_session;
                    bind_attempted = true;
                    let item = NewShortcut::new(SEND_CLIPBOARD_ID, req.description);
                    let rebound = match portal
                        .bind_shortcuts(
                            &session,
                            &[item],
                            parent.as_ref(),
                            BindShortcutsOptions::default(),
                        )
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
                            let next_supported = supported(portal.version(), hyprland, &next_trigger, false);
                            if next_supported != is_supported || next_trigger != trigger {
                                is_supported = next_supported;
                                trigger = next_trigger;
                                emit_state(callbacks, last_state, is_supported, trigger.clone());
                            }
                        }
                        Ok(None) | Err(_) => {
                            let cancelled = matches!(rebound, Err(true));
                            if let Some(found) = refresh_from_list(&portal, &session).await {
                                bound = found.is_some();
                                let next_trigger = found.unwrap_or_default();
                                let next_supported = supported(
                                    portal.version(),
                                    hyprland,
                                    &next_trigger,
                                    cancelled && is_supported,
                                );
                                if next_supported != is_supported || next_trigger != trigger {
                                    is_supported = next_supported;
                                    trigger = next_trigger;
                                    emit_state(callbacks, last_state, is_supported, trigger.clone());
                                }
                            } else {
                                bound = false;
                                if is_supported || !trigger.is_empty() {
                                    is_supported = false;
                                    trigger.clear();
                                    emit_state(callbacks, last_state, false, String::new());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Runs the `GlobalShortcuts` session loop on `conn` (testable on a `PrivateBus`).
async fn run_on(
    conn: zbus::Connection,
    register_host: bool,
    hyprland: bool,
    callbacks: Callbacks,
    rx: tokio::sync::mpsc::Receiver<Configure>,
    configuring: Arc<AtomicBool>,
) {
    run_on_with_backoff(
        conn,
        register_host,
        hyprland,
        callbacks,
        rx,
        configuring,
        RECONNECT_INITIAL_BACKOFF,
        RECONNECT_MAX_BACKOFF,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn run_on_with_backoff(
    conn: zbus::Connection,
    register_host: bool,
    hyprland: bool,
    callbacks: Callbacks,
    rx: tokio::sync::mpsc::Receiver<Configure>,
    configuring: Arc<AtomicBool>,
    initial_backoff: Duration,
    max_backoff: Duration,
) {
    run_on_with_timeouts(
        conn,
        register_host,
        hyprland,
        callbacks,
        rx,
        configuring,
        SETUP_TIMEOUT,
        initial_backoff,
        max_backoff,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn run_on_with_timeouts(
    conn: zbus::Connection,
    register_host: bool,
    hyprland: bool,
    callbacks: Callbacks,
    rx: tokio::sync::mpsc::Receiver<Configure>,
    configuring: Arc<AtomicBool>,
    setup_timeout: Duration,
    initial_backoff: Duration,
    max_backoff: Duration,
) {
    run_on_with_parent_ttl(
        conn,
        register_host,
        hyprland,
        callbacks,
        rx,
        configuring,
        setup_timeout,
        initial_backoff,
        max_backoff,
        CONFIGURE_PARENT_TTL,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn run_on_with_parent_ttl(
    conn: zbus::Connection,
    register_host: bool,
    hyprland: bool,
    callbacks: Callbacks,
    mut rx: tokio::sync::mpsc::Receiver<Configure>,
    configuring: Arc<AtomicBool>,
    setup_timeout: Duration,
    initial_backoff: Duration,
    max_backoff: Duration,
    configure_parent_ttl: Duration,
) {
    let mut last_state: Option<(bool, String)> = None;
    let initial_deadline = tokio::time::Instant::now() + setup_timeout;
    if register_host && let Ok(app_id) = ashpd::AppID::try_from(crate::APP_ID) {
        // Register once per D-Bus connection rather than on every session
        // reconnect (#638); best-effort for older portals without Registry.
        let _ = tokio::time::timeout_at(
            initial_deadline,
            ashpd::register_host_app_with_connection(conn.clone(), app_id),
        )
        .await;
    }

    let Some(mut setup) = setup_portal(&conn, initial_deadline).await else {
        emit_state(&callbacks, &mut last_state, false, String::new());
        return;
    };

    loop {
        match run_session(
            &conn,
            hyprland,
            &callbacks,
            &mut last_state,
            &mut rx,
            &configuring,
            configure_parent_ttl,
            setup,
        )
        .await
        {
            SessionEnd::Stop => return,
            SessionEnd::Closed => {}
        }

        // Portal closed the session (e.g. portal restart or update): recreate
        // with bounded exponential backoff while the window remains open.
        let mut backoff = initial_backoff;
        let mut failed_attempts: usize = 0;
        loop {
            tokio::select! {
                cmd = rx.recv() => {
                    if cmd.is_none() {
                        return;
                    }
                    let _guard = ResetConfiguring(&configuring);
                }
                () = tokio::time::sleep(backoff) => {
                    let deadline = tokio::time::Instant::now() + setup_timeout;
                    if let Some(reconnected) = setup_portal(&conn, deadline).await {
                        setup = reconnected;
                        break;
                    }
                    failed_attempts = failed_attempts.saturating_add(1);
                    if failed_attempts >= RECONNECT_FALLBACK_ATTEMPTS {
                        emit_state(&callbacks, &mut last_state, false, String::new());
                    }
                    backoff = backoff.saturating_mul(2).min(max_backoff);
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
        register_calls: usize,
        create_session_calls: usize,
        close_calls: usize,
        bind_calls: usize,
        configure_calls: usize,
        last_bind_parent: String,
        last_configure_parent: String,
        last_configure_token: Option<String>,
        fail_list_shortcuts: bool,
        list_shortcuts_delay: Option<Duration>,
        bind_outcome: MockBindOutcome,
        bound_trigger_on_configure: String,
        suppress_changed_on_configure: bool,
        clear_shortcuts_on_create_session: bool,
    }

    struct MockRegistry {
        state: Arc<Mutex<MockState>>,
    }

    #[zbus::interface(name = "org.freedesktop.host.portal.Registry")]
    impl MockRegistry {
        fn register(&self, _app_id: String, _options: HashMap<String, zbus::OwnedVariant>) {
            self.state.lock().unwrap().register_calls += 1;
        }
    }

    struct MockSession {
        state: Arc<Mutex<MockState>>,
    }

    #[zbus::interface(name = "org.freedesktop.portal.Session")]
    impl MockSession {
        fn close(&self) {
            self.state.lock().unwrap().close_calls += 1;
        }
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
            let _ = self
                .conn
                .object_server()
                .at(
                    &sess,
                    MockSession {
                        state: Arc::clone(&self.state),
                    },
                )
                .await;
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
            let (current, fail, delay) = {
                let st = self.state.lock().unwrap();
                (
                    st.shortcuts.clone(),
                    st.fail_list_shortcuts,
                    st.list_shortcuts_delay.unwrap_or(Duration::from_millis(15)),
                )
            };
            let conn = self.conn.clone();
            let req_emit = req.clone();
            tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                if fail {
                    let res: HashMap<&str, Variant<'_>> = HashMap::new();
                    let _ = conn
                        .emit_signal(
                            Some(sender.as_str()),
                            &req_emit,
                            "org.freedesktop.portal.Request",
                            "Response",
                            &(2u32, res),
                        )
                        .await;
                    return;
                }
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
            parent_window: String,
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
                st.last_bind_parent = parent_window;
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
            parent_window: String,
            options: HashMap<String, zbus::OwnedVariant>,
        ) {
            let sender = hdr.sender().unwrap().as_str().to_owned();
            let (updated, suppress_changed) = {
                let mut st = self.state.lock().unwrap();
                st.configure_calls += 1;
                st.last_configure_parent = parent_window;
                st.last_configure_token = options
                    .get("activation_token")
                    .and_then(|v| v.downcast_ref::<&str>().ok())
                    .map(ToOwned::to_owned);
                let trig = st.bound_trigger_on_configure.clone();
                for (_, t) in &mut st.shortcuts {
                    *t = trig.clone();
                }
                (st.shortcuts.clone(), st.suppress_changed_on_configure)
            };
            if suppress_changed {
                return;
            }
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
            configuring: Arc::new(AtomicBool::new(false)),
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
            Arc::clone(&inner.configuring),
        )
        .await;
        // Notifies the window once so `shortcutReady` flips to `true` and
        // Settings reveals the manual command fallback.
        assert_eq!(*seen.lock().unwrap(), vec![(false, String::new())]);
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
            configuring: Arc::new(AtomicBool::new(false)),
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
            Arc::clone(&inner.configuring),
        ));

        // 1. Startup lists shortcuts without calling BindShortcuts.
        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, String::new()));
        assert_eq!(state.lock().unwrap().bind_calls, 0);

        // 2. Clicking Set shortcut binds `send-clipboard` (passing X11 parent
        //    window ID when available) and opens ConfigureShortcuts on v2.
        handle.configure(
            "Send clipboard".to_owned(),
            Some("tok_cfg".to_owned()),
            Some(ParentWindow::X11(0x2a)),
        );
        let mut last = initial;
        while last.1.is_empty() {
            last = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
                .await
                .unwrap()
                .unwrap();
        }
        assert_eq!(last, (true, "Meta+Shift+V<U+202E>".to_owned()));
        {
            let st = state.lock().unwrap();
            assert_eq!(st.bind_calls, 1);
            assert_eq!(st.configure_calls, 1);
            assert_eq!(st.last_bind_parent, "x11:2a");
            assert_eq!(st.last_configure_parent, "x11:2a");
            assert_eq!(st.last_configure_token.as_deref(), Some("tok_cfg"));
        }

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
            configuring: Arc::new(AtomicBool::new(false)),
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
            Arc::clone(&inner.configuring),
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, String::new()));

        // v1 backend returns Ok with empty trigger -> switches to manual fallback.
        handle.configure("Send clipboard".to_owned(), None, None);
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
            configuring: Arc::new(AtomicBool::new(false)),
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
            Arc::clone(&inner.configuring),
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, "Alt+Shift+V".to_owned()));
        assert_eq!(state.lock().unwrap().bind_calls, 0);

        // Reconfiguring an already-bound shortcut calls ConfigureShortcuts directly.
        handle.configure("Send clipboard".to_owned(), None, None);
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
            configuring: Arc::new(AtomicBool::new(false)),
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
            Arc::clone(&inner.configuring),
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
            configuring: Arc::new(AtomicBool::new(false)),
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
            Arc::clone(&inner.configuring),
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, String::new()));

        // 1. Response(1, {}) (`Cancelled`) preserves `(true, "")` silently.
        handle.configure("Send clipboard".to_owned(), None, None);
        wait_until_idle(&inner).await;
        assert_eq!(state.lock().unwrap().bind_calls, 1);
        assert!(state_rx.try_recv().is_err());

        // 2. Returning `shortcuts = []` on v1 falls back to `(false, "")`.
        state.lock().unwrap().bind_outcome = MockBindOutcome::EmptyList;
        handle.configure("Send clipboard".to_owned(), None, None);
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
            configuring: Arc::new(AtomicBool::new(false)),
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
            Arc::clone(&inner2.configuring),
        ));
        let initial2 = tokio::time::timeout(Duration::from_secs(3), state_rx2.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial2, (true, String::new()));

        handle2.configure("Send clipboard".to_owned(), None, None);
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
            configuring: Arc::new(AtomicBool::new(false)),
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
            Arc::clone(&inner.configuring),
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, "Alt+Shift+V".to_owned()));
        assert_eq!(state.lock().unwrap().create_session_calls, 1);

        // 1. When the v1 backend keeps `"Alt+Shift+V"` across sessions and the user
        //    cancels re-bind, `refresh_from_list` preserves `bound = true` and trigger.
        handle.configure("Send clipboard".to_owned(), None, None);
        wait_until_idle(&inner).await;
        assert_eq!(state.lock().unwrap().create_session_calls, 2);
        assert!(state_rx.try_recv().is_err());

        // 2. When recreating the v1 session drops the old binding and the user cancels,
        //    `refresh_from_list` clears the stale trigger so UI matches `new_session`.
        state.lock().unwrap().clear_shortcuts_on_create_session = true;
        handle.configure("Send clipboard".to_owned(), None, None);
        let synced = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(synced, (true, String::new()));
        assert_eq!(state.lock().unwrap().create_session_calls, 3);

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }

    #[test]
    fn supported_rules_table() {
        // (version, hyprland, trigger, prev) -> expected
        let cases: &[(u32, bool, &str, bool, bool)] = &[
            // Portal v2+ is always supported regardless of hyprland/trigger/prev.
            (2, false, "", false, true),
            (2, false, "", true, true),
            (2, false, "Meta+Shift+V", false, true),
            (2, false, "Meta+Shift+V", true, true),
            (2, true, "", false, true),
            (2, true, "", true, true),
            (2, true, "Meta+Shift+V", false, true),
            (2, true, "Meta+Shift+V", true, true),
            // Portal v1 with a non-empty bound trigger is always supported.
            (1, false, "Meta+Shift+V", false, true),
            (1, false, "Meta+Shift+V", true, true),
            (1, true, "Meta+Shift+V", false, true),
            (1, true, "Meta+Shift+V", true, true),
            // Portal v1 on Hyprland/Omarchy with empty trigger falls back to manual.
            (1, true, "", false, false),
            (1, true, "", true, false),
            // Other v1 backends with empty trigger preserve `prev`.
            (1, false, "", false, false),
            (1, false, "", true, true),
        ];
        for &(version, hyprland, trigger, prev, expect) in cases {
            assert_eq!(
                supported(version, hyprland, trigger, prev),
                expect,
                "version={version} hyprland={hyprland} trigger={trigger:?} prev={prev}"
            );
        }
        assert!(parent_window_identifier(None).is_none());
        assert!(parent_window_identifier(Some(&ParentWindow::X11(0))).is_none());
        assert_eq!(
            parent_window_identifier(Some(&ParentWindow::X11(0x2a))).map(|w| w.to_string()),
            Some("x11:2a".to_owned())
        );
    }

    /// Fails immediately if a future `ashpd` update changes `Session`'s
    /// `Serialize` representation away from its D-Bus object path string.
    #[tokio::test]
    async fn session_object_path_matches_real_dbus_session_handle() {
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
        let portal = GlobalShortcuts::with_connection(client).await.unwrap();
        let session = portal
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();
        let expected = state.lock().unwrap().session_path.clone();
        assert!(!expected.is_empty());
        assert_eq!(
            session_object_path(&session).as_deref(),
            Some(expected.as_str())
        );
        let _ = session.close().await;
    }

    #[tokio::test]
    async fn setup_failure_after_create_session_closes_session_and_notifies_window() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            fail_list_shortcuts: true,
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
        let seen = Arc::new(Mutex::new(Vec::<(bool, String)>::new()));
        let seen_cb = Arc::clone(&seen);
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx,
            configuring: Arc::new(AtomicBool::new(false)),
        });
        run_on(
            client,
            false,
            false,
            Callbacks {
                state: Box::new(move |sup, trig| seen_cb.lock().unwrap().push((sup, trig))),
                activated: Box::new(|_| {}),
            },
            rx,
            Arc::clone(&inner.configuring),
        )
        .await;
        let st = state.lock().unwrap();
        assert_eq!(st.create_session_calls, 1);
        assert_eq!(st.close_calls, 1);
        assert_eq!(*seen.lock().unwrap(), vec![(false, String::new())]);
    }

    #[tokio::test]
    async fn portal_session_closed_reconnects_without_fallback_flash_and_registers_host_once() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            shortcuts: vec![(SEND_CLIPBOARD_ID.to_owned(), "Alt+Shift+V".to_owned())],
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
            .object_server()
            .at(
                "/org/freedesktop/portal/desktop",
                MockRegistry {
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
            configuring: Arc::new(AtomicBool::new(false)),
        });
        let handle = Handle {
            inner: Some(Arc::clone(&inner)),
        };

        let task = tokio::spawn(run_on_with_backoff(
            client,
            true,
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
            Arc::clone(&inner.configuring),
            Duration::from_millis(10),
            Duration::from_millis(40),
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, "Alt+Shift+V".to_owned()));
        let first_session = state.lock().unwrap().session_path.clone();
        let expected_registers = usize::from(!ashpd::is_sandboxed());
        assert_eq!(state.lock().unwrap().register_calls, expected_registers);

        // Update the trigger returned after portal restart so the new session emits
        // a distinct state callback directly without any intermediate `(false, "")` flash.
        state.lock().unwrap().shortcuts =
            vec![(SEND_CLIPBOARD_ID.to_owned(), "Super+Shift+V".to_owned())];

        // Simulate a portal restart closing the active session.
        let closed_details: HashMap<&str, Variant<'_>> = HashMap::new();
        server
            .emit_signal(
                Some(client_name.as_str()),
                ObjectPath::try_from(first_session.as_str()).unwrap(),
                "org.freedesktop.portal.Session",
                "Closed",
                &(closed_details,),
            )
            .await
            .unwrap();

        let recovered = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(recovered, (true, "Super+Shift+V".to_owned()));
        let second_session = state.lock().unwrap().session_path.clone();
        assert_eq!(state.lock().unwrap().create_session_calls, 2);
        assert_eq!(state.lock().unwrap().register_calls, expected_registers);
        assert_ne!(second_session, first_session);

        // Activated on the recreated session reaches the window.
        let mut opts: HashMap<&str, Variant<'_>> = HashMap::new();
        opts.insert("activation_token", Variant::from("tok_after_reconnect"));
        server
            .emit_signal(
                Some(client_name.as_str()),
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.GlobalShortcuts",
                "Activated",
                &(
                    ObjectPath::try_from(second_session.as_str()).unwrap(),
                    SEND_CLIPBOARD_ID,
                    55u64,
                    opts,
                ),
            )
            .await
            .unwrap();
        let got_tok = tokio::time::timeout(Duration::from_secs(3), act_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got_tok.as_deref(), Some("tok_after_reconnect"));

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }

    #[tokio::test]
    async fn portal_timeout_closes_half_initialized_session() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            shortcuts: vec![(SEND_CLIPBOARD_ID.to_owned(), "Alt+Shift+V".to_owned())],
            list_shortcuts_delay: Some(Duration::from_secs(5)),
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
        let seen = Arc::new(Mutex::new(Vec::<(bool, String)>::new()));
        let seen_cb = Arc::clone(&seen);
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx,
            configuring: Arc::new(AtomicBool::new(false)),
        });
        run_on_with_timeouts(
            client,
            false,
            false,
            Callbacks {
                state: Box::new(move |sup, trig| seen_cb.lock().unwrap().push((sup, trig))),
                activated: Box::new(|_| {}),
            },
            rx,
            Arc::clone(&inner.configuring),
            Duration::from_millis(120),
            Duration::from_millis(10),
            Duration::from_millis(40),
        )
        .await;
        let st = state.lock().unwrap();
        assert_eq!(st.create_session_calls, 1);
        assert_eq!(st.close_calls, 1);
        assert_eq!(*seen.lock().unwrap(), vec![(false, String::new())]);
    }

    #[tokio::test]
    async fn portal_prolonged_disconnect_emits_fallback_then_recovers() {
        let Some(bus) = PrivateBus::start() else {
            return skip_without_dbus("dbus-daemon is not installed");
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            shortcuts: vec![(SEND_CLIPBOARD_ID.to_owned(), "Alt+Shift+V".to_owned())],
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
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
        let inner = Arc::new(HandleInner {
            tx: cmd_tx,
            configuring: Arc::new(AtomicBool::new(false)),
        });
        let handle = Handle {
            inner: Some(Arc::clone(&inner)),
        };

        let task = tokio::spawn(run_on_with_backoff(
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
            Arc::clone(&inner.configuring),
            Duration::from_millis(10),
            Duration::from_millis(20),
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, "Alt+Shift+V".to_owned()));
        let first_session = state.lock().unwrap().session_path.clone();

        // Keep portal failing across RECONNECT_FALLBACK_ATTEMPTS so fallback triggers.
        state.lock().unwrap().fail_list_shortcuts = true;
        let closed_details: HashMap<&str, Variant<'_>> = HashMap::new();
        server
            .emit_signal(
                Some(client_name.as_str()),
                ObjectPath::try_from(first_session.as_str()).unwrap(),
                "org.freedesktop.portal.Session",
                "Closed",
                &(closed_details,),
            )
            .await
            .unwrap();

        let fallback = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fallback, (false, String::new()));
        {
            let st = state.lock().unwrap();
            assert!(st.create_session_calls > RECONNECT_FALLBACK_ATTEMPTS);
            assert!(st.close_calls >= RECONNECT_FALLBACK_ATTEMPTS);
        }

        // Restoring the portal brings the shortcut state back cleanly.
        state.lock().unwrap().fail_list_shortcuts = false;
        let recovered = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(recovered, (true, "Alt+Shift+V".to_owned()));

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }

    #[test]
    fn parent_window_identifier_formats_x11_and_validates_wayland_handles() {
        assert!(parent_window_identifier(None).is_none());
        assert!(parent_window_identifier(Some(&ParentWindow::X11(0))).is_none());
        assert_eq!(
            parent_window_identifier(Some(&ParentWindow::X11(0x2a)))
                .unwrap()
                .to_string(),
            "x11:2a"
        );

        let wl_ok = ParentWindow::Wayland {
            handle: "xdg-foreign-abc_123".to_owned(),
            unexport: None,
        };
        assert_eq!(
            parent_window_identifier(Some(&wl_ok)).unwrap().to_string(),
            "wayland:xdg-foreign-abc_123"
        );

        for bad in [
            "",
            "has space",
            "line\nbreak",
            "ctrl\u{001B}[31m",
            "bidi\u{202E}rtl",
            &"a".repeat(1025),
        ] {
            let wl_bad = ParentWindow::Wayland {
                handle: bad.to_owned(),
                unexport: None,
            };
            assert!(
                parent_window_identifier(Some(&wl_bad)).is_none(),
                "{bad:?} should be rejected"
            );
        }
    }

    #[tokio::test]
    async fn portal_v2_wayland_parent_and_fresh_token_retain_until_replaced_or_closed() {
        let Some(bus) = PrivateBus::start() else {
            skip_without_dbus("dbus-daemon unavailable");
            return;
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            bound_trigger_on_configure: "Meta+Shift+V".to_owned(),
            suppress_changed_on_configure: true,
            ..Default::default()
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
        let configuring = Arc::new(AtomicBool::new(false));
        let handle = Handle {
            inner: Some(Arc::new(HandleInner {
                tx: cmd_tx,
                configuring: Arc::clone(&configuring),
            })),
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
            configuring,
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, String::new()));

        // First configure: BindShortcuts + ConfigureShortcuts receive the
        // exported Wayland handle and fresh activation token; handle stays
        // exported after non-blocking ConfigureShortcuts returns.
        let unexported_1 = Arc::new(AtomicBool::new(false));
        let flag_1 = Arc::clone(&unexported_1);
        handle.configure(
            "Send clipboard".to_owned(),
            Some("fresh-xdg-act-1".to_owned()),
            Some(ParentWindow::Wayland {
                handle: "wl-handle-001".to_owned(),
                unexport: Some(Box::new(move || {
                    flag_1.store(true, Ordering::Release);
                })),
            }),
        );

        let mut last = initial;
        while last.1.is_empty() {
            last = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
                .await
                .unwrap()
                .unwrap();
        }
        assert_eq!(last, (true, "Meta+Shift+V".to_owned()));
        {
            let st = state.lock().unwrap();
            assert_eq!(st.bind_calls, 1);
            assert_eq!(st.configure_calls, 1);
            assert_eq!(st.last_bind_parent, "wayland:wl-handle-001");
            assert_eq!(st.last_configure_parent, "wayland:wl-handle-001");
            assert_eq!(st.last_configure_token.as_deref(), Some("fresh-xdg-act-1"));
        }
        assert!(
            !unexported_1.load(Ordering::Acquire),
            "handle must remain exported while non-blocking ConfigureShortcuts dialog is open"
        );

        // Second configure: replaces first exported handle (unexporting it)
        // and passes the new Wayland handle + fresh activation token.
        state.lock().unwrap().bound_trigger_on_configure = "Ctrl+Alt+V".to_owned();
        let unexported_2 = Arc::new(AtomicBool::new(false));
        let flag_2 = Arc::clone(&unexported_2);
        handle.configure(
            "Send clipboard".to_owned(),
            Some("fresh-xdg-act-2".to_owned()),
            Some(ParentWindow::Wayland {
                handle: "wl-handle-002".to_owned(),
                unexport: Some(Box::new(move || {
                    flag_2.store(true, Ordering::Release);
                })),
            }),
        );
        let updated = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated, (true, "Ctrl+Alt+V".to_owned()));
        {
            let st = state.lock().unwrap();
            assert_eq!(st.configure_calls, 2);
            assert_eq!(st.last_configure_parent, "wayland:wl-handle-002");
            assert_eq!(st.last_configure_token.as_deref(), Some("fresh-xdg-act-2"));
        }
        assert!(
            unexported_1.load(Ordering::Acquire),
            "first Wayland handle must be unexported when replaced by a newer ConfigureShortcuts"
        );
        assert!(
            !unexported_2.load(Ordering::Acquire),
            "second Wayland handle must stay exported while active"
        );

        // Dropping the window handle stops the session and unexports the active handle.
        drop(handle);
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .expect("portal task must exit when Handle drops")
            .unwrap();
        assert!(
            unexported_2.load(Ordering::Acquire),
            "active Wayland handle must be unexported when the session ends"
        );
    }

    #[tokio::test]
    async fn portal_v1_wayland_parent_unexports_after_bind_and_when_dropped() {
        let Some(bus) = PrivateBus::start() else {
            skip_without_dbus("dbus-daemon unavailable");
            return;
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            bind_outcome: MockBindOutcome::BindTrigger("Super+Shift+V".to_owned()),
            ..Default::default()
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
        let configuring = Arc::new(AtomicBool::new(false));
        let handle = Handle {
            inner: Some(Arc::new(HandleInner {
                tx: cmd_tx,
                configuring: Arc::clone(&configuring),
            })),
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
            configuring,
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, String::new()));

        let unexported = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&unexported);
        handle.configure(
            "Send clipboard".to_owned(),
            None,
            Some(ParentWindow::Wayland {
                handle: "wl-v1-handle".to_owned(),
                unexport: Some(Box::new(move || {
                    flag.store(true, Ordering::Release);
                })),
            }),
        );
        let bound = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(bound, (true, "Super+Shift+V".to_owned()));
        assert_eq!(
            state.lock().unwrap().last_bind_parent,
            "wayland:wl-v1-handle"
        );
        assert!(
            unexported.load(Ordering::Acquire),
            "BindShortcuts waits for Request::Response, so handle unexports right after bind"
        );

        // An inactive Handle (no portal task) drops and unexports immediately.
        let idle = Handle::default();
        let dropped_unexported = Arc::new(AtomicBool::new(false));
        let dropped_flag = Arc::clone(&dropped_unexported);
        idle.configure(
            "Send clipboard".to_owned(),
            None,
            Some(ParentWindow::Wayland {
                handle: "wl-dropped".to_owned(),
                unexport: Some(Box::new(move || {
                    dropped_flag.store(true, Ordering::Release);
                })),
            }),
        );
        assert!(dropped_unexported.load(Ordering::Acquire));

        drop(handle);
        let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
    }

    #[tokio::test]
    async fn portal_v2_strips_malformed_wayland_handle_and_activation_token() {
        let Some(bus) = PrivateBus::start() else {
            skip_without_dbus("dbus-daemon unavailable");
            return;
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            shortcuts: vec![(SEND_CLIPBOARD_ID.to_owned(), "Alt+V".to_owned())],
            bound_trigger_on_configure: "Alt+Shift+V".to_owned(),
            ..Default::default()
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
        let configuring = Arc::new(AtomicBool::new(false));
        let handle = Handle {
            inner: Some(Arc::new(HandleInner {
                tx: cmd_tx,
                configuring: Arc::clone(&configuring),
            })),
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
            configuring,
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, "Alt+V".to_owned()));

        let unexported = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&unexported);
        handle.configure(
            "Send clipboard".to_owned(),
            Some("bad token\nwith newline".to_owned()),
            Some(ParentWindow::Wayland {
                handle: "bad handle\u{202E}".to_owned(),
                unexport: Some(Box::new(move || {
                    flag.store(true, Ordering::Release);
                })),
            }),
        );
        let updated = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated, (true, "Alt+Shift+V".to_owned()));
        {
            let st = state.lock().unwrap();
            assert_eq!(st.configure_calls, 1);
            assert_eq!(st.last_configure_parent, "");
            assert_eq!(st.last_configure_token, None);
        }

        assert!(
            unexported.load(Ordering::Acquire),
            "malformed Wayland handle must be unexported immediately rather than retained"
        );

        drop(handle);
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .expect("portal task must exit when Handle drops")
            .unwrap();
    }

    #[tokio::test]
    async fn portal_v2_wayland_parent_unexports_on_shortcuts_changed_and_ttl_expiry() {
        let Some(bus) = PrivateBus::start() else {
            skip_without_dbus("dbus-daemon unavailable");
            return;
        };
        let server = bus.connect().await;
        let state = Arc::new(Mutex::new(MockState {
            shortcuts: vec![(SEND_CLIPBOARD_ID.to_owned(), "Alt+V".to_owned())],
            bound_trigger_on_configure: "Alt+V".to_owned(),
            suppress_changed_on_configure: true,
            ..Default::default()
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
        let client_name = client.unique_name().unwrap().as_str().to_owned();
        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(1);
        let configuring = Arc::new(AtomicBool::new(false));
        let inner = Arc::new(HandleInner {
            tx: cmd_tx,
            configuring: Arc::clone(&configuring),
        });
        let handle = Handle {
            inner: Some(Arc::clone(&inner)),
        };

        let task = tokio::spawn(run_on_with_parent_ttl(
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
            configuring,
            SETUP_TIMEOUT,
            RECONNECT_INITIAL_BACKOFF,
            RECONNECT_MAX_BACKOFF,
            Duration::from_secs(60),
        ));

        let initial = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial, (true, "Alt+V".to_owned()));

        // 1. ShortcutsChanged releases the retained Wayland parent export immediately.
        let (unexport_tx, mut unexport_rx) = tokio::sync::oneshot::channel();
        let unexport_slot = Mutex::new(Some(unexport_tx));
        handle.configure(
            "Send clipboard".to_owned(),
            Some("tok-changed".to_owned()),
            Some(ParentWindow::Wayland {
                handle: "wl-handle-changed".to_owned(),
                unexport: Some(Box::new(move || {
                    if let Some(tx) = unexport_slot.lock().unwrap().take() {
                        let _ = tx.send(());
                    }
                })),
            }),
        );
        wait_until_idle(&inner).await;
        assert!(
            unexport_rx.try_recv().is_err(),
            "handle must stay exported until ShortcutsChanged arrives"
        );

        let sess_path = ObjectPath::try_from(state.lock().unwrap().session_path.clone()).unwrap();
        let mut meta = HashMap::new();
        meta.insert("description", Variant::from("Send clipboard"));
        meta.insert("trigger_description", Variant::from("Super+V"));
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

        tokio::time::timeout(Duration::from_secs(3), unexport_rx)
            .await
            .expect("ShortcutsChanged must release the retained Wayland parent export")
            .unwrap();
        let updated = tokio::time::timeout(Duration::from_secs(3), state_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated, (true, "Super+V".to_owned()));

        drop(inner);
        drop(handle);
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .expect("portal task must exit when Handle drops")
            .unwrap();

        // 2. When ShortcutsChanged never arrives (e.g. cancelled dialog), TTL expiry
        //    automatically releases the Wayland parent export.
        let client2 = bus.connect().await;
        let (state_tx2, mut state_rx2) = tokio::sync::mpsc::unbounded_channel();
        let (cmd_tx2, cmd_rx2) = tokio::sync::mpsc::channel(1);
        let configuring2 = Arc::new(AtomicBool::new(false));
        let handle2 = Handle {
            inner: Some(Arc::new(HandleInner {
                tx: cmd_tx2,
                configuring: Arc::clone(&configuring2),
            })),
        };
        let task2 = tokio::spawn(run_on_with_parent_ttl(
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
            configuring2,
            SETUP_TIMEOUT,
            RECONNECT_INITIAL_BACKOFF,
            RECONNECT_MAX_BACKOFF,
            Duration::from_millis(50),
        ));

        let initial2 = tokio::time::timeout(Duration::from_secs(3), state_rx2.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(initial2, (true, "Alt+V".to_owned()));

        let (ttl_tx, ttl_rx) = tokio::sync::oneshot::channel();
        let ttl_slot = Mutex::new(Some(ttl_tx));
        handle2.configure(
            "Send clipboard".to_owned(),
            Some("tok-ttl".to_owned()),
            Some(ParentWindow::Wayland {
                handle: "wl-handle-ttl".to_owned(),
                unexport: Some(Box::new(move || {
                    if let Some(tx) = ttl_slot.lock().unwrap().take() {
                        let _ = tx.send(());
                    }
                })),
            }),
        );

        tokio::time::timeout(Duration::from_secs(3), ttl_rx)
            .await
            .expect("TTL expiry must unexport retained Wayland parent handle")
            .unwrap();

        drop(handle2);
        tokio::time::timeout(Duration::from_secs(2), task2)
            .await
            .expect("portal task must exit when Handle drops")
            .unwrap();
    }
}
