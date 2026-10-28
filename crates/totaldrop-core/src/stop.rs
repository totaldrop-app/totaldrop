//! Stopping a transfer from another task, so the other side hears about it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::Notify;

/// A handle that stops a transfer. Clones share one state; stopping is
/// final. A stopped sender or receiver tells the other side (a `cancel`
/// message, best effort), stops its streams, removes partial files and
/// returns [`crate::Error::Stopped`].
#[derive(Debug, Clone, Default)]
pub struct Stop(Arc<Inner>);

#[derive(Debug, Default)]
struct Inner {
    notify: Notify,
    stopped: AtomicBool,
}

impl Stop {
    /// A handle that has not been stopped.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stop the transfer. Calling it again does nothing.
    pub fn stop(&self) {
        if !self.0.stopped.swap(true, Ordering::SeqCst) {
            self.0.notify.notify_waiters();
        }
    }

    /// Whether [`Stop::stop`] was called.
    pub fn is_stopped(&self) -> bool {
        self.0.stopped.load(Ordering::SeqCst)
    }

    /// Resolves once the transfer is stopped; at once if it already is.
    pub async fn stopped(&self) {
        let notified = self.0.notify.notified();
        tokio::pin!(notified);
        // Register before checking, so a stop in between is not missed.
        notified.as_mut().enable();
        if self.is_stopped() {
            return;
        }
        notified.await;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn stopped_resolves_for_every_waiter_and_after_the_fact() {
        let stop = Stop::new();
        let a = tokio::spawn({
            let s = stop.clone();
            async move { s.stopped().await }
        });
        let b = tokio::spawn({
            let s = stop.clone();
            async move { s.stopped().await }
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        stop.stop();
        tokio::time::timeout(Duration::from_secs(1), a)
            .await
            .unwrap()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(1), b)
            .await
            .unwrap()
            .unwrap();
        // A waiter that comes later returns at once.
        tokio::time::timeout(Duration::from_millis(50), stop.stopped())
            .await
            .unwrap();
        assert!(stop.is_stopped());
    }
}
