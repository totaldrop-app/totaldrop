// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! Finishing the window's work before the process exits (#433).
//!
//! `main` ends with `std::process::exit`, and the runtime is a static that
//! is never dropped. A transfer still running when the window closes, or a
//! UPnP delete started just before, would die with the process and leave a
//! router port open until its lease ends. So on quit the window stops what
//! runs and waits, briefly, for the work to end and the ports to close.

use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::Notify;

/// The longest the process waits after the window has closed: stopping a
/// transfer (the other side is told) and the UPnP delete that follows
/// (`totaldrop_core::portmap::LIMIT`, 1.6 s). Nothing is on screen by then.
pub const LIMIT: Duration = Duration::from_secs(3);

/// Transfers and declines, which may hold a router port.
pub static TRANSFERS: Tasks = Tasks::new();

/// Counts the tasks that run, so quitting can wait for them.
#[derive(Debug)]
pub struct Tasks {
    running: AtomicUsize,
    idle: Notify,
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
        }
    }

    /// `work`, counted from this call until it ends or is dropped, polled
    /// or not, also when it panics.
    pub fn track<F: Future>(&'static self, work: F) -> impl Future<Output = F::Output> + use<F> {
        let running = Running::new(self);
        async move {
            let _running = running;
            work.await
        }
    }

    /// Resolves once no tracked task runs; at once if none does.
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
}

/// One tracked task, counted while it lives.
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

/// Wait, at most `limit`, until `tasks` are idle and then until `then`
/// (the port releases) is done. Whether it all ended in time.
pub async fn finish(tasks: &Tasks, then: impl Future<Output = ()>, limit: Duration) -> bool {
    tokio::time::timeout(limit, async {
        tasks.idle().await;
        then.await;
    })
    .await
    .is_ok()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use super::{Tasks, finish};

    /// The window closed during a transfer: quitting waits until the
    /// stopped transfer has ended and dropped what it held, and then for
    /// the release that drop started.
    #[tokio::test]
    async fn quitting_waits_for_a_stopped_transfer_and_its_release() {
        static TASKS: Tasks = Tasks::new();
        let stop = totaldrop_core::Stop::new();
        let (dropped, released) = (
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
        );
        let (d, r, s) = (Arc::clone(&dropped), Arc::clone(&released), stop.clone());
        tokio::spawn(TASKS.track(async move {
            s.stopped().await;
            // Telling the other side takes a moment.
            tokio::time::sleep(Duration::from_millis(30)).await;
            d.store(true, Ordering::SeqCst);
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(30)).await;
                r.store(true, Ordering::SeqCst);
            });
        }));
        stop.stop();
        let r = Arc::clone(&released);
        let done = finish(
            &TASKS,
            async move {
                while !r.load(Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            },
            Duration::from_secs(2),
        )
        .await;
        assert!(done);
        assert!(dropped.load(Ordering::SeqCst), "the transfer ended first");
        assert!(released.load(Ordering::SeqCst), "and its release finished");
    }

    /// A task that never ends cannot keep the process alive.
    #[tokio::test]
    async fn quitting_is_bounded() {
        static TASKS: Tasks = Tasks::new();
        tokio::spawn(TASKS.track(std::future::pending::<()>()));
        let started = tokio::time::Instant::now();
        assert!(!finish(&TASKS, async {}, Duration::from_millis(100)).await);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    /// Nothing running: quitting does not wait.
    #[tokio::test]
    async fn nothing_running_ends_at_once() {
        static TASKS: Tasks = Tasks::new();
        assert!(finish(&TASKS, async {}, Duration::from_millis(1)).await);
    }

    /// A task is counted from the moment it is handed over, before it
    /// first runs, and also when it is dropped without running.
    #[tokio::test]
    async fn a_task_counts_before_it_runs_and_until_it_is_dropped() {
        static TASKS: Tasks = Tasks::new();
        let work = TASKS.track(async {});
        assert!(!finish(&TASKS, async {}, Duration::from_millis(20)).await);
        drop(work);
        assert!(finish(&TASKS, async {}, Duration::from_millis(20)).await);
    }
}
