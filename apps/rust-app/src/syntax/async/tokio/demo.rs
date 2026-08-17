//! Minimal Tokio demo: spawn an async task from a sync function.

use std::time::Duration;

pub async fn run() {
    let handle = sync_fn();
    handle.await.expect("async_fn task panicked");
}

async fn async_fn() {
    println!("async_fn: sleeping 1000ms…");
    tokio::time::sleep(Duration::from_millis(1000)).await;
    println!("async_fn: done");
}

/// Schedules `async_fn` on the runtime and returns immediately.
fn sync_fn() -> tokio::task::JoinHandle<()> {
    println!("sync_fn: before spawn");
    let handle = tokio::spawn(async_fn());
    println!("sync_fn: after spawn (not blocked)");
    handle
}
