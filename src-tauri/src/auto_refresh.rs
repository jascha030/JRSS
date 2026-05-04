//! Background feed auto-refresh task.
//!
//! Spawns a single Tokio task on app start. The task sleeps for the configured
//! interval, then refreshes every feed sequentially and emits the
//! `auto-refresh-complete` event to the frontend.
//!
//! Sending a new interval via [`AutoRefreshState::set_interval`] cancels the
//! current sleep immediately via a `tokio::sync::watch` channel, so changes
//! take effect without waiting out the old timer.
//!
//! An interval of `0` disables auto-refresh; the task blocks on the channel
//! until the user re-enables it.

use crate::db::{self, DatabaseState};
use crate::feed_ingest;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::watch;
use tokio::time::Duration;

// ---------------------------------------------------------------------------
// Public state handle
// ---------------------------------------------------------------------------

/// Managed state that lets commands update the auto-refresh interval at
/// runtime without restarting the background task.
pub struct AutoRefreshState {
    sender: watch::Sender<i64>,
}

impl AutoRefreshState {
    /// Update the interval (minutes). `0` disables auto-refresh.
    /// The running background task picks up the change immediately.
    pub fn set_interval(&self, minutes: i64) {
        // Errors only occur if the receiver was dropped (task panic/abort).
        let _ = self.sender.send(minutes);
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Refresh a single feed given its RSS URL. Errors are returned as strings so
/// the caller can log them without aborting the full sweep.
fn refresh_feed_by_url(db_path: &std::path::Path, url: &str) -> Result<(), String> {
    let parsed = feed_ingest::fetch_and_parse_feed(url)?;
    db::upsert_feed_snapshot(db_path, url, parsed).map(|_| ())
}

/// Fetch all feed URLs from SQLite, refresh each one in turn, then emit
/// `auto-refresh-complete` to the frontend so it can reload state.
async fn run_refresh_sweep(db_path: PathBuf, app_handle: AppHandle) {
    // Load the feed list on a thread-pool thread (rusqlite is blocking).
    let feeds = tauri::async_runtime::spawn_blocking({
        let db_path = db_path.clone();
        move || db::list_feeds(&db_path)
    })
    .await;

    let feeds = match feeds {
        Ok(Ok(feeds)) => feeds,
        Ok(Err(error)) => {
            log::error!("Auto-refresh: failed to list feeds: {error}");
            return;
        }
        Err(error) => {
            log::error!("Auto-refresh: list_feeds task panicked: {error}");
            return;
        }
    };

    log::info!("Auto-refresh: refreshing {} feed(s)", feeds.len());

    for feed in feeds {
        let url = feed.url.clone();
        let title = feed.title.clone();
        let db_path = db_path.clone();

        let result =
            tauri::async_runtime::spawn_blocking(move || refresh_feed_by_url(&db_path, &url))
                .await;

        match result {
            Ok(Ok(())) => log::info!("Auto-refresh: refreshed '{title}'"),
            Ok(Err(error)) => log::warn!("Auto-refresh: failed to refresh '{title}': {error}"),
            Err(error) => {
                log::warn!("Auto-refresh: refresh task for '{title}' panicked: {error}")
            }
        }
    }

    if let Err(error) = app_handle.emit("auto-refresh-complete", ()) {
        log::warn!("Auto-refresh: failed to emit completion event: {error}");
    }
}

// ---------------------------------------------------------------------------
// Task spawn
// ---------------------------------------------------------------------------

/// Spawn the background auto-refresh loop and return its control handle.
///
/// `initial_interval_minutes == 0` means disabled on start; the task blocks
/// until the interval is updated via [`AutoRefreshState::set_interval`].
pub fn spawn(app_handle: AppHandle, initial_interval_minutes: i64) -> AutoRefreshState {
    let (tx, mut rx) = watch::channel(initial_interval_minutes);

    tauri::async_runtime::spawn(async move {
        // Mark the initial value as seen so `rx.changed()` won't fire spuriously.
        let mut current = *rx.borrow_and_update();

        loop {
            if current <= 0 {
                // Disabled — block until the user enables auto-refresh.
                if rx.changed().await.is_err() {
                    break; // sender dropped; task should terminate
                }
                current = *rx.borrow_and_update();
                continue;
            }

            let delay = Duration::from_secs(current as u64 * 60);

            tokio::select! {
                // Timer fired — run the sweep.
                () = tokio::time::sleep(delay) => {
                    let db_path = app_handle.state::<DatabaseState>().db_path();
                    run_refresh_sweep(db_path, app_handle.clone()).await;
                    // Re-read interval in case it changed during the sweep.
                    current = *rx.borrow();
                }
                // Interval changed — restart the loop with the new value so
                // the stale timer is cancelled immediately.
                result = rx.changed() => {
                    if result.is_err() {
                        break; // sender dropped
                    }
                    current = *rx.borrow_and_update();
                }
            }
        }
    });

    AutoRefreshState { sender: tx }
}
