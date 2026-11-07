// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Keeps the computer from going to sleep while a transfer runs (#340).
//!
//! A computer that suspends on idle in the middle of a transfer cuts it:
//! the other side gives up after QUIC's 30-second idle timeout. While a
//! send or a receive is under way, TotalDrop asks the system not to
//! suspend, and lets go as soon as it ends. The screen may still dim, blank
//! and lock; only sleep is held off.
//!
//! - Outside the Flatpak: systemd-logind's `Inhibit("sleep", …, "block")`
//!   on the system bus. KDE, Hyprland (hypridle) and Omarchy all suspend
//!   through logind, so one lock covers them. It holds for as long as the
//!   file descriptor it returns is open.
//! - In the Flatpak, or where logind does not answer: the desktop portal's
//!   `Inhibit` with the suspend flag. It holds until its request is closed
//!   or the connection ends.
//!
//! When neither answers, nothing is held and the transfer goes on as
//! before.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use zbus::{OwnedFd, OwnedObjectPath, Variant};

const LOGIND: &str = "org.freedesktop.login1";
const LOGIND_PATH: &str = "/org/freedesktop/login1";
const LOGIND_MANAGER: &str = "org.freedesktop.login1.Manager";

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const PORTAL_INHIBIT: &str = "org.freedesktop.portal.Inhibit";
const PORTAL_REQUEST: &str = "org.freedesktop.portal.Request";

/// The portal's flag for "do not suspend".
const SUSPEND: u32 = 4;

/// A lock that holds off sleep until it is released.
pub enum Hold {
    /// logind's: open for as long as the descriptor is.
    Logind(OwnedFd),
    /// The portal's: the request to close, on the connection that made it.
    Portal {
        conn: zbus::Connection,
        request: OwnedObjectPath,
    },
}

impl Hold {
    /// Let the computer sleep again.
    pub async fn release(self) {
        match self {
            Self::Logind(fd) => drop(fd),
            Self::Portal { conn, request } => {
                let _ = conn
                    .call_method(Some(PORTAL), &request, Some(PORTAL_REQUEST), "Close", &())
                    .await;
            }
        }
    }
}

/// logind's sleep lock on `conn` (the system bus).
pub async fn logind_on(conn: &zbus::Connection, reason: &str) -> Option<Hold> {
    let reply = conn
        .call_method(
            Some(LOGIND),
            LOGIND_PATH,
            Some(LOGIND_MANAGER),
            "Inhibit",
            &("sleep", "TotalDrop", reason, "block"),
        )
        .await
        .ok()?;
    let fd: OwnedFd = reply.body().deserialize().ok()?;
    Some(Hold::Logind(fd))
}

/// The portal's suspend lock on `conn` (the session bus).
pub async fn portal_on(conn: zbus::Connection, reason: &str) -> Option<Hold> {
    let mut options: HashMap<&str, Variant<'_>> = HashMap::new();
    options.insert("reason", Variant::from(reason));
    let reply = conn
        .call_method(
            Some(PORTAL),
            PORTAL_PATH,
            Some(PORTAL_INHIBIT),
            "Inhibit",
            // No window to name: the lock is for the transfer, not a window.
            &("", SUSPEND, options),
        )
        .await
        .ok()?;
    let request: OwnedObjectPath = reply.body().deserialize().ok()?;
    Some(Hold::Portal { conn, request })
}

/// The lock this desktop gives: logind's outside the Flatpak, the portal's
/// in it or when logind does not answer.
async fn acquire(reason: String) -> Option<Hold> {
    if !crate::theme::host::sandboxed()
        && let Ok(system) = zbus::Connection::system().await
        && let Some(hold) = logind_on(&system, &reason).await
    {
        return Some(hold);
    }
    let session = zbus::Connection::session().await.ok()?;
    portal_on(session, &reason).await
}

/// The one [`Awake`] of the window, with its reason in the person's
/// language.
pub fn awake() -> &'static Awake {
    static AWAKE: std::sync::LazyLock<Awake> = std::sync::LazyLock::new(|| {
        let reason = crate::bridge::texts::strings().get("keepAwakeReason", &[]);
        Awake::new(&crate::runtime::RUNTIME, reason)
    });
    &AWAKE
}

/// How a lock is taken; the tests use their own.
pub type Acquire =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Option<Hold>> + Send>> + Send + Sync>;

/// Holds off sleep while [`Awake::set`] says so. Taking and letting go of
/// the lock happen on the runtime, in the order they were asked for, so a
/// transfer that ends while the lock is being taken never leaves it held.
pub struct Awake {
    wanted: tokio::sync::watch::Sender<bool>,
}

impl Awake {
    /// Holds off sleep with this desktop's lock; `reason` is what the
    /// desktop may show.
    pub fn new(runtime: &tokio::runtime::Runtime, reason: String) -> Self {
        Self::with(
            runtime.handle(),
            Arc::new(move || Box::pin(acquire(reason.clone()))),
        )
    }

    /// With `acquire` to take the lock.
    pub fn with(runtime: &tokio::runtime::Handle, acquire: Acquire) -> Self {
        let (wanted, mut rx) = tokio::sync::watch::channel(false);
        runtime.spawn(async move {
            let mut hold: Option<Hold> = None;
            loop {
                let want = *rx.borrow_and_update();
                if want && hold.is_none() {
                    // A failure is tried again the next time it is wanted.
                    hold = acquire().await;
                } else if !want && let Some(h) = hold.take() {
                    h.release().await;
                }
                if rx.changed().await.is_err() {
                    break;
                }
            }
            if let Some(h) = hold {
                h.release().await;
            }
        });
        Self { wanted }
    }

    /// Hold off sleep (`true`) or let the computer sleep (`false`).
    pub fn set(&self, on: bool) {
        self.wanted.send_if_modified(|v| {
            let changed = *v != on;
            *v = on;
            changed
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::io::{BufRead as _, Read as _};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::sync::Mutex;
    use std::time::Duration;

    /// A bus of our own, so tests neither need nor touch the desktop's.
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
            let dir = Path::new("/tmp").join(format!("td-sleep-{}-{n}", std::process::id()));
            std::fs::create_dir(&dir).ok()?;
            let Ok(daemon) = Command::new("dbus-daemon")
                .args(["--session", "--nofork", "--print-address=1"])
                .arg(format!("--address=unix:dir={}", dir.display()))
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
            else {
                let _ = std::fs::remove_dir_all(&dir);
                return None;
            };
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

    /// What a stand-in heard.
    #[derive(Default)]
    struct Heard {
        calls: Vec<String>,
    }

    /// logind's `Inhibit`: hands out the write end of a pipe, so the test
    /// sees the lock go when the read end reaches its end.
    struct FakeLogind {
        heard: Arc<Mutex<Heard>>,
        read: Arc<Mutex<Option<std::fs::File>>>,
    }

    #[zbus::interface(name = "org.freedesktop.login1.Manager")]
    impl FakeLogind {
        fn inhibit(&self, what: &str, who: &str, why: &str, mode: &str) -> OwnedFd {
            self.heard
                .lock()
                .unwrap()
                .calls
                .push(format!("{what}|{who}|{why}|{mode}"));
            // The test keeps the read end and watches it for the end.
            let (r, w) = std::io::pipe().unwrap();
            *self.read.lock().unwrap() = Some(std::fs::File::from(std::os::fd::OwnedFd::from(r)));
            OwnedFd::from(std::os::fd::OwnedFd::from(w))
        }
    }

    /// The portal: `Inhibit` answers with a request that `Close` ends.
    struct FakePortal {
        heard: Arc<Mutex<Heard>>,
    }

    #[zbus::interface(name = "org.freedesktop.portal.Inhibit")]
    impl FakePortal {
        async fn inhibit(
            &self,
            #[zbus(object_server)] server: &zbus::ObjectServer,
            window: &str,
            flags: u32,
            options: HashMap<String, zbus::OwnedVariant>,
        ) -> OwnedObjectPath {
            let reason = options
                .get("reason")
                .and_then(|v| String::try_from(v.clone()).ok())
                .unwrap_or_default();
            self.heard
                .lock()
                .unwrap()
                .calls
                .push(format!("inhibit|{window}|{flags}|{reason}"));
            let path =
                OwnedObjectPath::try_from("/org/freedesktop/portal/desktop/request/1/t").unwrap();
            server
                .at(
                    &path,
                    FakeRequest {
                        heard: self.heard.clone(),
                    },
                )
                .await
                .unwrap();
            path
        }
    }

    struct FakeRequest {
        heard: Arc<Mutex<Heard>>,
    }

    #[zbus::interface(name = "org.freedesktop.portal.Request")]
    impl FakeRequest {
        fn close(&self) {
            self.heard.lock().unwrap().calls.push("close".to_owned());
        }
    }

    #[tokio::test]
    async fn logind_holds_sleep_until_released() {
        let Some(bus) = PrivateBus::start() else {
            eprintln!("skipped: no dbus-daemon");
            return;
        };
        let heard = Arc::new(Mutex::new(Heard::default()));
        let read = Arc::new(Mutex::new(None));
        let _logind = zbus::connection::Builder::address(bus.address.as_str())
            .name(LOGIND)
            .serve_at(
                LOGIND_PATH,
                FakeLogind {
                    heard: heard.clone(),
                    read: read.clone(),
                },
            )
            .build()
            .await
            .unwrap();
        let conn = bus.connect().await;
        let hold = logind_on(&conn, "A transfer is running").await.unwrap();
        assert_eq!(
            heard.lock().unwrap().calls,
            ["sleep|TotalDrop|A transfer is running|block"]
        );
        let mut r = read.lock().unwrap().take().unwrap();
        hold.release().await;
        let mut buf = [0u8; 1];
        let n = tokio::task::spawn_blocking(move || r.read(&mut buf).unwrap())
            .await
            .unwrap();
        assert_eq!(n, 0, "the lock is gone once released");
    }

    #[tokio::test]
    async fn the_portal_holds_suspend_until_its_request_is_closed() {
        let Some(bus) = PrivateBus::start() else {
            eprintln!("skipped: no dbus-daemon");
            return;
        };
        let heard = Arc::new(Mutex::new(Heard::default()));
        let _portal = zbus::connection::Builder::address(bus.address.as_str())
            .name(PORTAL)
            .serve_at(
                PORTAL_PATH,
                FakePortal {
                    heard: heard.clone(),
                },
            )
            .build()
            .await
            .unwrap();
        let hold = portal_on(bus.connect().await, "A transfer is running")
            .await
            .unwrap();
        assert_eq!(
            heard.lock().unwrap().calls,
            ["inhibit||4|A transfer is running"]
        );
        hold.release().await;
        assert_eq!(heard.lock().unwrap().calls.last().unwrap(), "close");
    }

    #[tokio::test]
    async fn no_lock_where_nothing_answers() {
        let Some(bus) = PrivateBus::start() else {
            eprintln!("skipped: no dbus-daemon");
            return;
        };
        let conn = bus.connect().await;
        assert!(logind_on(&conn, "x").await.is_none());
        assert!(portal_on(conn, "x").await.is_none());
    }

    /// The controller over a lock that counts what it does.
    fn counted(
        rt: &tokio::runtime::Handle,
    ) -> (
        Awake,
        Arc<std::sync::atomic::AtomicI32>,
        Arc<Mutex<Option<std::fs::File>>>,
    ) {
        let held = Arc::new(std::sync::atomic::AtomicI32::new(0));
        let last = Arc::new(Mutex::new(None));
        let (h, l) = (held.clone(), last.clone());
        let acquire: Acquire = Arc::new(move || {
            let (h, l) = (h.clone(), l.clone());
            Box::pin(async move {
                // Slow, so a change can come while the lock is being taken.
                tokio::time::sleep(Duration::from_millis(50)).await;
                h.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let (r, w) = std::io::pipe().unwrap();
                *l.lock().unwrap() = Some(std::fs::File::from(std::os::fd::OwnedFd::from(r)));
                Some(Hold::Logind(OwnedFd::from(std::os::fd::OwnedFd::from(w))))
            })
        });
        (Awake::with(rt, acquire), held, last)
    }

    fn released(read: &Mutex<Option<std::fs::File>>) -> bool {
        let Some(mut r) = read.lock().unwrap().take() else {
            return false;
        };
        let mut buf = [0u8; 1];
        r.read(&mut buf).unwrap() == 0
    }

    #[tokio::test]
    async fn awake_takes_the_lock_once_and_lets_go_at_the_end() {
        let (awake, held, last) = counted(&tokio::runtime::Handle::current());
        awake.set(true);
        awake.set(true);
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(held.load(std::sync::atomic::Ordering::SeqCst), 1);
        awake.set(false);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(released(&last));
    }

    #[tokio::test]
    async fn a_transfer_that_ends_while_the_lock_is_taken_leaves_none() {
        let (awake, held, last) = counted(&tokio::runtime::Handle::current());
        awake.set(true);
        // Ends while the lock is being taken (it takes 50 ms).
        tokio::time::sleep(Duration::from_millis(10)).await;
        awake.set(false);
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(held.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert!(released(&last), "the lock taken late was let go");
    }

    #[tokio::test]
    async fn dropping_awake_lets_go() {
        let (awake, _held, last) = counted(&tokio::runtime::Handle::current());
        awake.set(true);
        tokio::time::sleep(Duration::from_millis(200)).await;
        drop(awake);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(released(&last));
    }
}
