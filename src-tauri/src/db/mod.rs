//! Database module — SQLite persistence layer.
//!
//! Organized by domain:
//! - `connection.rs` — Database connection management
//! - `schema.rs` — Schema initialization and migrations
//! - `rows.rs` — Row mapping functions
//! - `feeds.rs` — Feed repository operations
//! - `items.rs` — Item repository operations
//! - `stations.rs` — Station repository operations
//! - `playback.rs` — Playback state, session, and context
//! - `settings.rs` — App settings

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub mod connection;
pub mod feeds;
pub mod items;
pub mod playback;
pub mod rows;
pub mod schema;
pub mod settings;
pub mod stations;

pub use feeds::{
    get_feed_by_id, list_feeds, remove_feed, set_feed_sort_order, upsert_feed_snapshot,
};
pub use items::{
    get_item_by_id, get_items_by_ids, mark_read, query_items, query_items_page, save_playback,
    save_reader_content, save_reader_failure,
};
pub use playback::{
    clear_playback_session, load_playback_context, load_playback_session, save_playback_context,
    save_playback_session,
};
pub use schema::initialize_database;
pub use settings::{DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES, load_app_settings, save_app_settings};
pub use stations::{
    create_station, delete_station, list_stations, query_station_episodes, update_station,
};

pub type AppResult<T> = Result<T, String>;

pub struct DatabaseState {
    db_path: PathBuf,
}

impl DatabaseState {
    pub fn new(app: &AppHandle) -> AppResult<Self> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| format!("Failed to resolve app data directory: {error}"))?;

        std::fs::create_dir_all(&app_data_dir)
            .map_err(|error| format!("Failed to create app data directory: {error}"))?;

        let state = Self {
            db_path: app_data_dir.join("jrss.sqlite3"),
        };

        initialize_database(&state.db_path)?;

        Ok(state)
    }

    pub fn db_path(&self) -> PathBuf {
        self.db_path.clone()
    }
}
