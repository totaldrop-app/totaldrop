// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Finishing the window's work before the process exits (#433).
//!
//! `main` ends with `std::process::exit`, and the runtime is a static that
//! is never dropped. A transfer still running when the window closes, or a
//! UPnP delete started just before, would die with the process and leave a
//! router port open until its lease ends. So on quit the window stops what
//! runs, waits briefly for it to end, drops what has not, and then waits
//! for the ports to close.

use std::future::Future;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::Notify;
use tokio::task::AbortHandle;

/// How long a stopped transfer may take to tell the other side and end.
/// One still running then is dropped: its port mapping goes with it, and
/// the release starts. Telling a silent peer can take longer (QUIC waits
/// up to 5 s for an idle endpoint); the port matters more.
pub const STOPPING: Duration = Duration::from_millis(1500);

/// How long dropped tasks get to unwind on the runtime's threads.
const ABORTED: Duration = Duration::from_millis(200);

/// Transfers and declines, which may hold a router port.
pub static TRANSFERS: Tasks = Tasks::new();

/// The tasks that run, so quitting can wait for them and drop the rest.
#[derive(Debug)]
pub struct Tasks {
    running: AtomicUsize,
    idle: Notify,
    handles: Mutex<Vec<AbortHandle>>,
}

impl Default for Tasks {
    fn default() -> Self {
        Self::new()
    }
}

impl Tasks {
    /// No task running; `const`, so it can be a static.
    pub const fn new() -> Self {
        Self {
            running: AtomicUsize::new(0),
            idle: Notify::const_new(),
            handles: Mutex::new(Vec::new()),
        }
    }

    /// Run `work` on `runtime`, counted from this call until it ends or is
    /// dropped, also when it panics, and droppable by [`finish`].
    pub fn spawn(
        &'static self,
        runtime: &tokio::runtime::Runtime,
        work: impl Future<Output = ()> + Send + 'static,
    ) {
        let running = Running::new(self);
        let task = runtime.spawn(async move {
            let _running = running;
            work.await;
        });
        if let Ok(mut handles) = self.handles.lock() {
            handles.retain(|h| !h.is_finished());
            handles.push(task.abort_handle());
        }
    }

    /// Resolves once no counted task runs; at once if none does.
    pub async fn idle(&self) {
        loop {
            // Made before the check, so an end in between is not missed:
            // `notify_waiters` reaches a `Notified` from its creation on.
            let ended = self.idle.notified();
            if self.running.load(Ordering::Acquire) == 0 {
                return;
            }
            ended.await;
        }
    }

    /// Drop every task still running.
    fn abort_all(&self) {
        if let Ok(handles) = self.handles.lock() {
            for h in handles.iter() {
                h.abort();
            }
        }
    }
}

/// One counted task, counted while it lives.
struct Running(&'static Tasks);

impl Running {
    fn new(tasks: &'static Tasks) -> Self {
        tasks.running.fetch_add(1, Ordering::AcqRel);
        Self(tasks)
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        self.0.running.fetch_sub(1, Ordering::AcqRel);
        self.0.idle.notify_waiters();
    }
}

/// Wait, at most `stopping`, until `tasks` have ended; drop the ones that
/// have not, so what they hold is released; then wait for `release`, which
/// bounds itself (`totaldrop_core::portmap::released`, at most 1.6 s).
/// Whether every task ended by itself.
pub async fn finish(tasks: &Tasks, release: impl Future<Output = ()>, stopping: Duration) -> bool {
    let ended = tokio::time::timeout(stopping, tasks.idle()).await.is_ok();
    if !ended {
        tasks.abort_all();
        let _ = tokio::time::timeout(ABORTED, tasks.idle()).await;
    }
    release.await;
    ended
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use super::{Tasks, finish};

    fn runtime() -> tokio::runtime::Runtime {
        #[allow(clippy::unwrap_used)]
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
    }

    /// Sets its flag when dropped, as a port mapping sends its release.
    struct Held(Arc<AtomicBool>);

    impl Drop for Held {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    fn flag() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }

    /// Wait until `f` is set, at most `limit`.
    async fn set(f: Arc<AtomicBool>, limit: Duration) {
        let _ = tokio::time::timeout(limit, async {
            while !f.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await;
    }

    /// The window closed during a transfer: quitting waits until the
    /// stopped transfer has ended and dropped what it held, and only then
    /// for the release that drop started. The release records whether the
    /// drop came first, so a `finish` that waits for both at once fails.
    #[test]
    fn quitting_waits_for_a_stopped_transfer_and_its_release() {
        static TASKS: Tasks = Tasks::new();
        let rt = runtime();
        let stop = totaldrop_core::Stop::new();
        let (dropped, released) = (flag(), flag());
        let (held, r, s) = (
            Held(Arc::clone(&dropped)),
            Arc::clone(&released),
            stop.clone(),
        );
        TASKS.spawn(&rt, async move {
            let _held = held;
            s.stopped().await;
            // Telling the other side takes a moment.
            tokio::time::sleep(Duration::from_millis(30)).await;
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(30)).await;
                r.store(true, Ordering::SeqCst);
            });
        });
        stop.stop();
        let (r, d, seen) = (Arc::clone(&released), Arc::clone(&dropped), flag());
        let s = Arc::clone(&seen);
        let ended = rt.block_on(finish(
            &TASKS,
            async move {
                s.store(d.load(Ordering::SeqCst), Ordering::SeqCst);
                set(r, Duration::from_secs(2)).await;
            },
            Duration::from_secs(2),
        ));
        assert!(ended, "the transfer ended by itself");
        assert!(
            seen.load(Ordering::SeqCst),
            "it ended before the release wait began"
        );
        assert!(released.load(Ordering::SeqCst), "and its release finished");
    }

    /// A transfer that does not end in time is dropped, so what it holds
    /// is still released before the process exits (review of #433: a
    /// silent peer can keep a stopped send waiting for seconds). The drop
    /// comes before the release wait begins, or that wait would miss it.
    #[test]
    fn a_transfer_that_does_not_end_in_time_is_dropped() {
        static TASKS: Tasks = Tasks::new();
        let rt = runtime();
        let dropped = flag();
        let held = Held(Arc::clone(&dropped));
        TASKS.spawn(&rt, async move {
            let _held = held;
            std::future::pending::<()>().await;
        });
        let started = std::time::Instant::now();
        let (d, seen) = (Arc::clone(&dropped), flag());
        let s = Arc::clone(&seen);
        let ended = rt.block_on(finish(
            &TASKS,
            async move { s.store(d.load(Ordering::SeqCst), Ordering::SeqCst) },
            Duration::from_millis(50),
        ));
        assert!(!ended);
        assert!(
            seen.load(Ordering::SeqCst),
            "dropped before the release wait began"
        );
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    /// Nothing running: quitting does not wait.
    #[test]
    fn nothing_running_ends_at_once() {
        static TASKS: Tasks = Tasks::new();
        let rt = runtime();
        assert!(rt.block_on(finish(&TASKS, async {}, Duration::from_millis(1))));
    }

    /// A task that waits is counted, and once dropped is not; a later
    /// task is counted again.
    #[test]
    fn a_waiting_task_is_counted_until_dropped() {
        static TASKS: Tasks = Tasks::new();
        let rt = runtime();
        let go = Arc::new(tokio::sync::Notify::new());
        let wait = Arc::clone(&go);
        TASKS.spawn(&rt, async move { wait.notified().await });
        assert!(!rt.block_on(finish(&TASKS, async {}, Duration::from_millis(20))));
        // It was dropped by then; a new task counts again.
        TASKS.spawn(&rt, async {});
        assert!(rt.block_on(finish(&TASKS, async {}, Duration::from_secs(1))));
        go.notify_waiters();
    }
}
