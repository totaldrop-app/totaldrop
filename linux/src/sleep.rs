// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Keeps the computer from going to sleep while a transfer runs (#340).
//!
//! A computer that suspends on idle in the middle of a transfer cuts it:
//! the other side gives up after QUIC's 30-second idle timeout. While a
//! send or a receive is under way, TotalDrop asks the system not to
//! suspend, and lets go as soon as it ends. The screen may still dim, blank
//! and lock; only sleep is held off. Suspend on idle waits; a suspend the
//! person asks for may be refused or ask for confirmation, as logind
//! decides.
//!
//! - First, systemd-logind's `Inhibit("sleep", …, "block")` on the system
//!   bus, in the Flatpak too (`--system-talk-name=org.freedesktop.login1`).
//!   KDE, Hyprland (hypridle) and Omarchy all suspend through logind, so
//!   one lock covers them. It holds for as long as the file descriptor it
//!   returns is open.
//! - Where logind does not answer: the desktop portal's `Inhibit` with the
//!   suspend flag. It holds until its request is closed or the connection
//!   ends. Not every portal implements it (Hyprland's does not), which is
//!   why it is only the fallback. Its `Response` is not awaited: the lock
//!   is a wish, and a portal that declines it leaves the transfer as it
//!   was without the lock.
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

/// The lock this desktop gives: logind's, or the portal's when logind does
/// not answer.
async fn acquire(reason: String) -> Option<Hold> {
    if let Ok(system) = zbus::Connection::system().await
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
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::time::Duration;

    /// Without `dbus-daemon` a test cannot run. That is fine on a
    /// developer's computer, but CI installs it, so there a skip fails.
    fn skip(why: &str) {
        assert!(std::env::var_os("CI").is_none(), "{why}");
        eprintln!("skipped: {why}");
    }

    /// A lock as a socket pair: the holder gets one end, the test keeps
    /// the other and sees the lock go when its end reads end-of-file.
    fn lock_pair() -> (UnixStream, OwnedFd) {
        let (keep, give) = UnixStream::pair().unwrap();
        (keep, OwnedFd::from(std::os::fd::OwnedFd::from(give)))
    }

    /// Whether the other end of `keep` was closed, waiting at most five
    /// seconds, off the runtime so the lock can be let go meanwhile.
    async fn gone(keep: UnixStream) -> bool {
        tokio::task::spawn_blocking(move || {
            let mut keep = keep;
            keep.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0u8; 1];
            matches!(keep.read(&mut buf), Ok(0))
        })
        .await
        .unwrap()
    }

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

    /// logind's `Inhibit`: hands out one end of a socket pair, so the test
    /// sees the lock go when the other end reads end-of-file.
    struct FakeLogind {
        heard: Arc<Mutex<Heard>>,
        read: Arc<Mutex<Option<UnixStream>>>,
    }

    #[zbus::interface(name = "org.freedesktop.login1.Manager")]
    impl FakeLogind {
        fn inhibit(&self, what: &str, who: &str, why: &str, mode: &str) -> OwnedFd {
            self.heard
                .lock()
                .unwrap()
                .calls
                .push(format!("{what}|{who}|{why}|{mode}"));
            let (keep, give) = lock_pair();
            *self.read.lock().unwrap() = Some(keep);
            give
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
            return skip("no dbus-daemon");
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
        let keep = read.lock().unwrap().take().unwrap();
        hold.release().await;
        assert!(gone(keep).await, "the lock is gone once released");
    }

    #[tokio::test]
    async fn the_portal_holds_suspend_until_its_request_is_closed() {
        let Some(bus) = PrivateBus::start() else {
            return skip("no dbus-daemon");
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
            return skip("no dbus-daemon");
        };
        let conn = bus.connect().await;
        assert!(logind_on(&conn, "x").await.is_none());
        assert!(portal_on(conn, "x").await.is_none());
    }

    /// The controller over a lock that counts what it does. Taking the lock
    /// waits at `gate`, so a test decides when it completes.
    struct Counted {
        awake: Awake,
        taken: Arc<AtomicI32>,
        /// The test's ends of the locks taken, in order.
        ends: Arc<Mutex<Vec<UnixStream>>>,
        /// Told when taking a lock starts, and when it has been taken.
        started: Arc<tokio::sync::Notify>,
        done: Arc<tokio::sync::Notify>,
        gate: Arc<tokio::sync::Semaphore>,
    }

    impl Counted {
        fn new(open: bool) -> Self {
            let taken = Arc::new(AtomicI32::new(0));
            let ends = Arc::new(Mutex::new(Vec::new()));
            let started = Arc::new(tokio::sync::Notify::new());
            let done = Arc::new(tokio::sync::Notify::new());
            let gate = Arc::new(tokio::sync::Semaphore::new(if open {
                tokio::sync::Semaphore::MAX_PERMITS
            } else {
                0
            }));
            let (t, e, s, d, g) = (
                taken.clone(),
                ends.clone(),
                started.clone(),
                done.clone(),
                gate.clone(),
            );
            let acquire: Acquire = Arc::new(move || {
                let (t, e, s, d, g) = (t.clone(), e.clone(), s.clone(), d.clone(), g.clone());
                Box::pin(async move {
                    s.notify_one();
                    g.acquire().await.unwrap().forget();
                    t.fetch_add(1, Ordering::SeqCst);
                    let (keep, give) = lock_pair();
                    e.lock().unwrap().push(keep);
                    d.notify_one();
                    Some(Hold::Logind(give))
                })
            });
            Self {
                awake: Awake::with(&tokio::runtime::Handle::current(), acquire),
                taken,
                ends,
                started,
                done,
                gate,
            }
        }

        fn taken(&self) -> i32 {
            self.taken.load(Ordering::SeqCst)
        }

        /// Whether the `n`th lock taken was let go.
        async fn released(&self, n: usize) -> bool {
            let keep = self
                .ends
                .lock()
                .unwrap()
                .get(n)
                .unwrap()
                .try_clone()
                .unwrap();
            gone(keep).await
        }
    }

    #[tokio::test]
    async fn awake_takes_the_lock_once_and_lets_go_at_the_end() {
        let c = Counted::new(true);
        c.awake.set(true);
        c.awake.set(true);
        c.done.notified().await;
        c.awake.set(false);
        assert!(c.released(0).await);
        assert_eq!(c.taken(), 1, "asked twice, taken once");
        // The next transfer takes it again.
        c.awake.set(true);
        c.done.notified().await;
        assert_eq!(c.taken(), 2);
        c.awake.set(false);
        assert!(c.released(1).await);
    }

    #[tokio::test]
    async fn a_transfer_that_ends_while_the_lock_is_taken_leaves_none() {
        let c = Counted::new(false);
        c.awake.set(true);
        // The lock is being taken, and the transfer ends before it is.
        c.started.notified().await;
        c.awake.set(false);
        c.gate.add_permits(1);
        c.done.notified().await;
        assert!(c.released(0).await, "the lock taken late was let go");
        assert_eq!(c.taken(), 1);
    }

    #[tokio::test]
    async fn dropping_awake_lets_go() {
        let c = Counted::new(true);
        c.awake.set(true);
        c.done.notified().await;
        let Counted { awake, ends, .. } = c;
        drop(awake);
        let keep = ends.lock().unwrap().remove(0);
        assert!(gone(keep).await);
    }
}
