use crate::audio::{self, PlaybackStateEvent};
use crate::db::{self, DatabaseState};
use crate::feed_ingest;
use crate::models::{
    CreateStationInput, FeedItemRecord, FeedListItemRecord, FeedRecord, ItemPageQueryRecord,
    ItemPageRecord, PlaybackSessionRecord, StationWithFeedsRecord, UpdateStationInput,
};
use crate::queue::{QueuedItem, QueueState};
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
        let resolved_input = feed_ingest::resolve_feed_input(&url)?;
        let parsed_feed = feed_ingest::fetch_and_parse_feed(&resolved_input.feed_url)
            .map_err(|error| resolved_input.map_fetch_error(error))?;

        db::upsert_feed_snapshot(&db_path, &resolved_input.feed_url, parsed_feed)
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
pub async fn query_items_page(
    query: ItemPageQueryRecord,
    state: State<'_, DatabaseState>,
) -> Result<ItemPageRecord, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::query_items_page(&db_path, &query))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn get_item_details(
    item_id: String,
    state: State<'_, DatabaseState>,
) -> Result<FeedItemRecord, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || {
        db::get_item_by_id(&db_path, &item_id)?.ok_or_else(|| "Item not found.".to_string())
    })
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
            log::debug!(
                "Reader Mode: using cached reader content for item {}",
                item_id
            );
            return Ok(item);
        }

        log::info!(
            "Reader Mode: loading fresh content for item {} (URL: {})",
            item_id,
            item.url
        );

        match reader_extract::fetch_reader_content(&item.url, &item.title) {
            Ok(reader_content) => {
                log::info!(
                    "Reader Mode: successfully extracted content for item {} (title: {})",
                    item_id,
                    reader_content.title
                );
                db::save_reader_content(&db_path, &item_id, &reader_content)?;
            }
            Err(error) => {
                log::warn!(
                    "Reader Mode: extraction failed for item {} ({}): {}",
                    item_id,
                    item.url,
                    error
                );
                db::save_reader_failure(&db_path, &item_id)?;
            }
        }

        db::get_item_by_id(&db_path, &item_id)?
            .ok_or_else(|| "Item not found after reader update.".to_string())
    })
    .await
    .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn get_items_by_ids(
    item_ids: Vec<String>,
    state: State<'_, DatabaseState>,
) -> Result<Vec<FeedListItemRecord>, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::get_items_by_ids(&db_path, &item_ids))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn save_playback_session(
    session: PlaybackSessionRecord,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || {
        db::save_playback_session(&db_path, &session)
    })
    .await
    .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn load_playback_session(
    state: State<'_, DatabaseState>,
) -> Result<Option<PlaybackSessionRecord>, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::load_playback_session(&db_path))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn set_feed_sort_order(
    feed_id: String,
    sort_order: Option<String>,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || {
        db::set_feed_sort_order(&db_path, &feed_id, sort_order.as_deref())
    })
    .await
    .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn clear_playback_session(
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::clear_playback_session(&db_path))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

// ---------------------------------------------------------------------------
// Stations
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_stations(
    state: State<'_, DatabaseState>,
) -> Result<Vec<StationWithFeedsRecord>, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::list_stations(&db_path))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn create_station(
    input: CreateStationInput,
    state: State<'_, DatabaseState>,
) -> Result<StationWithFeedsRecord, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::create_station(&db_path, &input))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn update_station(
    input: UpdateStationInput,
    state: State<'_, DatabaseState>,
) -> Result<StationWithFeedsRecord, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::update_station(&db_path, &input))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn delete_station(
    id: String,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || db::delete_station(&db_path, &id))
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn query_station_episodes(
    station_id: String,
    offset: i64,
    limit: i64,
    state: State<'_, DatabaseState>,
) -> Result<ItemPageRecord, String> {
    let db_path = state.db_path();

    tauri::async_runtime::spawn_blocking(move || {
        db::query_station_episodes(&db_path, &station_id, offset, limit)
    })
    .await
    .map_err(|error| format!("Native task failed: {error}"))?
}

// ---------------------------------------------------------------------------
// Audio playback — backend-owned via rodio
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn audio_play(
    app: tauri::AppHandle,
    item_id: String,
    url: String,
    start_position_seconds: f64,
    duration_hint_seconds: f64,
) -> Result<(), String> {
    audio::play_url(&app, item_id, url, start_position_seconds, duration_hint_seconds)
}

#[tauri::command]
pub fn audio_pause(app: tauri::AppHandle) -> Result<(), String> {
    audio::pause(&app)
}

#[tauri::command]
pub fn audio_resume(app: tauri::AppHandle) -> Result<(), String> {
    audio::resume(&app)
}

#[tauri::command]
pub fn audio_toggle(app: tauri::AppHandle) -> Result<(), String> {
    audio::toggle_playback(&app)
}

#[tauri::command]
pub fn audio_stop(app: tauri::AppHandle) -> Result<(), String> {
    audio::stop(&app)
}

#[tauri::command]
pub fn audio_seek(app: tauri::AppHandle, position_seconds: f64) -> Result<(), String> {
    audio::seek(&app, position_seconds)
}

#[tauri::command]
pub fn audio_set_volume(app: tauri::AppHandle, volume: f64) -> Result<(), String> {
    audio::set_volume(&app, volume)
}

#[tauri::command]
pub fn audio_set_speed(app: tauri::AppHandle, speed: f64) -> Result<(), String> {
    audio::set_speed(&app, speed)
}

#[tauri::command]
pub fn audio_get_state(app: tauri::AppHandle) -> Option<PlaybackStateEvent> {
    audio::get_playback_state(&app)
}

// ---------------------------------------------------------------------------
// Queue management — backend-owned for headless playback
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn audio_play_with_queue(
    app: tauri::AppHandle,
    item: QueuedItem,
    queue: Vec<QueuedItem>,
    start_position_seconds: f64,
) -> Result<(), String> {
    log::info!("audio_play_with_queue: item={}, queue_len={}", item.item_id, queue.len());
    let result = audio::play_with_queue(&app, item, queue, start_position_seconds);
    log::info!("audio_play_with_queue: result={:?}", result);
    result
}

#[tauri::command]
pub fn audio_queue_enqueue(app: tauri::AppHandle, item: QueuedItem) -> Result<(), String> {
    audio::queue_enqueue(&app, item)
}

#[tauri::command]
pub fn audio_queue_play_next(app: tauri::AppHandle, item: QueuedItem) -> Result<(), String> {
    audio::queue_play_next(&app, item)
}

#[tauri::command]
pub fn audio_queue_remove(app: tauri::AppHandle, item_id: String) -> Result<(), String> {
    audio::queue_remove(&app, item_id)
}

#[tauri::command]
pub fn audio_queue_move_up(app: tauri::AppHandle, item_id: String) -> Result<(), String> {
    audio::queue_move_up(&app, item_id)
}

#[tauri::command]
pub fn audio_queue_move_down(app: tauri::AppHandle, item_id: String) -> Result<(), String> {
    audio::queue_move_down(&app, item_id)
}

#[tauri::command]
pub fn audio_queue_clear(app: tauri::AppHandle) -> Result<(), String> {
    audio::queue_clear(&app)
}

#[tauri::command]
pub fn audio_queue_get_state(app: tauri::AppHandle) -> QueueState {
    audio::get_queue_state(&app)
}

#[tauri::command]
pub fn audio_queue_set(app: tauri::AppHandle, items: Vec<QueuedItem>) -> Result<(), String> {
    audio::queue_set(&app, items)
}
