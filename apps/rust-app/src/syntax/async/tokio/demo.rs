//! Minimal Tokio async/await demo.

use std::time::Duration;

pub async fn run() {
    println!("tokio demo: sleeping 1000ms…");
    tokio::time::sleep(Duration::from_millis(1000)).await;
    println!("tokio demo: done");
}
