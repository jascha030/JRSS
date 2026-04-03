use crate::db::{self, DatabaseState};
use crate::feed_ingest;
use crate::models::{FeedItemRecord, FeedRecord};
use crate::reader_extract;
use tauri::State;

#[tauri::command]
pub async fn list_feeds(state: State<'_, DatabaseState>) -> Result<Vec<FeedRecord>, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::list_feeds(&db_path))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn add_feed(url: String, state: State<'_, DatabaseState>) -> Result<FeedRecord, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || {
        let normalized_url = feed_ingest::normalize_feed_url(&url)?;
        let parsed_feed = feed_ingest::fetch_and_parse_feed(&normalized_url)?;

        db::upsert_feed_snapshot(&db_path, &normalized_url, parsed_feed)
    })
    .await
    .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn refresh_feed(
    id: String,
    state: State<'_, DatabaseState>,
) -> Result<FeedRecord, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || {
        let existing_feed =
            db::get_feed_by_id(&db_path, &id)?.ok_or_else(|| "Feed not found.".to_string())?;
        let parsed_feed = feed_ingest::fetch_and_parse_feed(&existing_feed.url)?;

        db::upsert_feed_snapshot(&db_path, &existing_feed.url, parsed_feed)
    })
    .await
    .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn remove_feed(id: String, state: State<'_, DatabaseState>) -> Result<(), String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::remove_feed(&db_path, &id))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn list_items(
    feed_id: Option<String>,
    state: State<'_, DatabaseState>,
) -> Result<Vec<FeedItemRecord>, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::list_items(&db_path, feed_id.as_deref()))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn mark_read(
    item_id: String,
    read: bool,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::mark_read(&db_path, &item_id, read))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn save_playback(
    item_id: String,
    position_seconds: i64,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || {
        db::save_playback(&db_path, &item_id, position_seconds)
    })
    .await
    .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn load_reader_content(
    item_id: String,
    state: State<'_, DatabaseState>,
) -> Result<FeedItemRecord, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || {
        let item =
            db::get_item_by_id(&db_path, &item_id)?.ok_or_else(|| "Item not found.".to_string())?;

        if item.media_enclosure.is_some() || item.reader_status == "ready" {
            return Ok(item);
        }

        match reader_extract::fetch_reader_content(&item.url, &item.title) {
            Ok(reader_content) => db::save_reader_content(&db_path, &item_id, &reader_content)?,
            Err(error) => {
                log::warn!("Reader extraction failed for {}: {}", item.url, error);
                db::save_reader_failure(&db_path, &item_id)?;
            }
        }

        db::get_item_by_id(&db_path, &item_id)?
            .ok_or_else(|| "Item not found after reader update.".to_string())
    })
    .await
    .map_err(|error| format!("Native task failed: {error}"))?
}
