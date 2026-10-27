//! The one background runtime for network and desktop-integration work.

use std::sync::LazyLock;

/// Shared multi-thread runtime.
pub static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("totaldrop")
        .build()
        .unwrap_or_else(|_| std::process::abort())
});
