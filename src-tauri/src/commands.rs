use crate::audio::{self, OutputDeviceInfo, PlaybackStateEvent};
use crate::auto_refresh::AutoRefreshState;
use crate::cover_art;
use crate::db::{self, DatabaseState};
use crate::feed_ingest;
use crate::image_cache::ImageCache;
use crate::models::{
    AppSettingsRecord, CreateStationInput, FeedItemRecord, FeedListItemRecord, FeedRecord,
    ItemPageRecord, PlaybackContextRecord, PlaybackSessionRecord, PodcastSearchResultRecord,
    StationWithFeedsRecord, UpdateStationInput,
};
use crate::queue::{QueueState, QueuedItem};
use crate::reader_extract;
use crate::theme::{cmd_discover_themes, cmd_load_theme, ThemeInfo};
use base64::Engine;
use tauri::{Manager, State};

// CGSize ABI on 64-bit macOS: {CGFloat CGFloat} = {double double}.
#[cfg(target_os = "macos")]
#[repr(C)]
struct CGSize {
    width: f64,
    height: f64,
}

// SAFETY: CGSize is `{CGFloat CGFloat}` in Objective-C on 64-bit, matching this repr(C) layout.
#[cfg(target_os = "macos")]
unsafe impl objc2::Encode for CGSize {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

#[cfg(target_os = "macos")]
fn set_macos_window_content_aspect_ratio(
    window: &tauri::WebviewWindow,
    width: f64,
    height: f64,
) -> Result<(), String> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let ns_window = window.ns_window().map_err(|error| error.to_string())? as *mut AnyObject;
    let size = CGSize { width, height };
    unsafe { msg_send![ns_window, setContentAspectRatio: size] }

    Ok(())
}

#[cfg(target_os = "macos")]
fn clear_macos_window_content_aspect_ratio(window: &tauri::WebviewWindow) -> Result<(), String> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let ns_window = window.ns_window().map_err(|error| error.to_string())? as *mut AnyObject;
    let increments = CGSize {
        width: 1.0,
        height: 1.0,
    };
    // SAFETY: Calling setResizeIncrements: with {1,1} restores freeform resizing, which
    // is the standard way to remove an aspect-ratio lock on AppKit.
    unsafe { msg_send![ns_window, setResizeIncrements: increments] }

    Ok(())
}

#[cfg(target_os = "macos")]
fn is_macos_window_in_live_resize(window: &tauri::WebviewWindow) -> Result<bool, String> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let ns_window = window.ns_window().map_err(|error| error.to_string())? as *mut AnyObject;
    // SAFETY: inLiveResize is a standard BOOL property on NSWindow.
    let in_live: bool = unsafe { msg_send![ns_window, inLiveResize] };

    Ok(in_live)
}

#[cfg(not(target_os = "macos"))]
fn set_macos_window_content_aspect_ratio(
    _window: &tauri::WebviewWindow,
    _width: f64,
    _height: f64,
) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn clear_macos_window_content_aspect_ratio(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn is_macos_window_in_live_resize(_window: &tauri::WebviewWindow) -> Result<bool, String> {
    Ok(false)
}

/// Helper to run blocking tasks on a thread pool and convert errors.
async fn blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| format!("Native task failed: {error}"))?
}

#[tauri::command]
pub async fn list_feeds(state: State<'_, DatabaseState>) -> Result<Vec<FeedRecord>, String> {
    let db_path = state.db_path();
    blocking(move || db::list_feeds(&db_path)).await
}

#[tauri::command]
pub async fn add_feed(url: String, state: State<'_, DatabaseState>) -> Result<FeedRecord, String> {
    let db_path = state.db_path();

    blocking(move || {
        let resolved_input = feed_ingest::resolve_feed_input(&url)?;
        let parsed_feed = feed_ingest::fetch_and_parse_feed(&resolved_input.feed_url)
            .map_err(|error| resolved_input.map_fetch_error(error))?;

        db::upsert_feed_snapshot(&db_path, &resolved_input.feed_url, parsed_feed)
    })
    .await
}

#[tauri::command]
pub async fn refresh_feed(
    id: String,
    state: State<'_, DatabaseState>,
) -> Result<FeedRecord, String> {
    let db_path = state.db_path();

    blocking(move || {
        let existing_feed =
            db::get_feed_by_id(&db_path, &id)?.ok_or_else(|| "Feed not found.".to_string())?;
        let parsed_feed = feed_ingest::fetch_and_parse_feed(&existing_feed.url)?;

        db::upsert_feed_snapshot(&db_path, &existing_feed.url, parsed_feed)
    })
    .await
}

#[tauri::command]
pub async fn remove_feed(id: String, state: State<'_, DatabaseState>) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::remove_feed(&db_path, &id)).await
}

#[tauri::command]
pub async fn search_podcasts(term: String) -> Result<Vec<PodcastSearchResultRecord>, String> {
    blocking(move || feed_ingest::search_podcasts(&term)).await
}

#[tauri::command]
pub async fn fetch_feed_raw(
    feed_id: String,
    state: State<'_, DatabaseState>,
) -> Result<String, String> {
    let db_path = state.db_path();

    blocking(move || {
        let feed =
            db::get_feed_by_id(&db_path, &feed_id)?.ok_or_else(|| "Feed not found.".to_string())?;
        feed_ingest::fetch_raw_feed_xml(&feed.url)
    })
    .await
}

#[tauri::command]
pub async fn query_items(
    query: crate::models::ItemsQueryRecord,
    state: State<'_, DatabaseState>,
) -> Result<ItemPageRecord, String> {
    let db_path = state.db_path();
    blocking(move || db::query_items(&db_path, &query)).await
}

#[tauri::command]
pub async fn get_feeds_unread_counts(
    state: State<'_, DatabaseState>,
) -> Result<std::collections::HashMap<String, i64>, String> {
    let db_path = state.db_path();
    blocking(move || db::get_unread_counts_by_feed(&db_path)).await
}

#[tauri::command]
pub async fn get_item_details(
    item_id: String,
    state: State<'_, DatabaseState>,
) -> Result<FeedItemRecord, String> {
    let db_path = state.db_path();

    blocking(move || {
        db::get_item_by_id(&db_path, &item_id)?.ok_or_else(|| "Item not found.".to_string())
    })
    .await
}

#[tauri::command]
pub async fn mark_read(
    item_id: String,
    read: bool,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::mark_read(&db_path, &item_id, read)).await
}

#[tauri::command]
pub async fn mark_read_batch(
    item_ids: Vec<String>,
    read: bool,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::mark_read_batch(&db_path, &item_ids, read)).await
}

#[tauri::command]
pub async fn mark_favorite(
    item_id: String,
    favorite: bool,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::mark_favorite(&db_path, &item_id, favorite)).await
}

#[tauri::command]
pub async fn mark_favorite_batch(
    item_ids: Vec<String>,
    favorite: bool,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::mark_favorite_batch(&db_path, &item_ids, favorite)).await
}

#[tauri::command]
pub async fn save_playback(
    item_id: String,
    position_seconds: i64,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::save_playback(&db_path, &item_id, position_seconds)).await
}

#[tauri::command]
pub async fn load_reader_content(
    item_id: String,
    state: State<'_, DatabaseState>,
) -> Result<FeedItemRecord, String> {
    let db_path = state.db_path();

    blocking(move || {
        let item =
            db::get_item_by_id(&db_path, &item_id)?.ok_or_else(|| "Item not found.".to_string())?;

        // Return cached content if available (either media enclosure or reader ready)
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
}

#[tauri::command]
pub async fn get_items_by_ids(
    item_ids: Vec<String>,
    state: State<'_, DatabaseState>,
) -> Result<Vec<FeedListItemRecord>, String> {
    let db_path = state.db_path();
    blocking(move || db::get_items_by_ids(&db_path, &item_ids)).await
}

#[tauri::command]
pub async fn save_playback_session(
    session: PlaybackSessionRecord,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::save_playback_session(&db_path, &session)).await
}

#[tauri::command]
pub async fn load_playback_session(
    state: State<'_, DatabaseState>,
) -> Result<Option<PlaybackSessionRecord>, String> {
    let db_path = state.db_path();
    blocking(move || db::load_playback_session(&db_path)).await
}

#[tauri::command]
pub async fn set_feed_sort_order(
    feed_id: String,
    sort_order: Option<String>,
    state: State<'_, DatabaseState>,
) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::set_feed_sort_order(&db_path, &feed_id, sort_order.as_deref())).await
}

#[tauri::command]
pub async fn clear_playback_session(state: State<'_, DatabaseState>) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::clear_playback_session(&db_path)).await
}

#[tauri::command]
pub async fn load_app_settings(
    state: State<'_, DatabaseState>,
) -> Result<AppSettingsRecord, String> {
    let db_path = state.db_path();
    blocking(move || db::load_app_settings(&db_path)).await
}

#[tauri::command]
pub async fn save_app_settings(
    settings: AppSettingsRecord,
    state: State<'_, DatabaseState>,
    auto_refresh_state: State<'_, AutoRefreshState>,
) -> Result<AppSettingsRecord, String> {
    let db_path = state.db_path();
    let saved = blocking(move || db::save_app_settings(&db_path, &settings)).await?;
    auto_refresh_state.set_interval(saved.auto_refresh_interval_minutes);
    Ok(saved)
}

#[tauri::command]
pub async fn discover_themes(app: tauri::AppHandle) -> Result<Vec<ThemeInfo>, String> {
    cmd_discover_themes(app).await
}

#[tauri::command]
pub async fn load_theme(filename: String, app: tauri::AppHandle) -> Result<String, String> {
    cmd_load_theme(filename, app).await
}

// ---------------------------------------------------------------------------
// Stations
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_stations(
    state: State<'_, DatabaseState>,
) -> Result<Vec<StationWithFeedsRecord>, String> {
    let db_path = state.db_path();
    blocking(move || db::list_stations(&db_path)).await
}

#[tauri::command]
pub async fn create_station(
    input: CreateStationInput,
    state: State<'_, DatabaseState>,
) -> Result<StationWithFeedsRecord, String> {
    let db_path = state.db_path();
    blocking(move || db::create_station(&db_path, &input)).await
}

#[tauri::command]
pub async fn update_station(
    input: UpdateStationInput,
    state: State<'_, DatabaseState>,
) -> Result<StationWithFeedsRecord, String> {
    let db_path = state.db_path();
    blocking(move || db::update_station(&db_path, &input)).await
}

#[tauri::command]
pub async fn delete_station(id: String, state: State<'_, DatabaseState>) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::delete_station(&db_path, &id)).await
}

#[tauri::command]
pub async fn query_station_episodes(
    station_id: String,
    offset: i64,
    limit: i64,
    search: Option<String>,
    state: State<'_, DatabaseState>,
) -> Result<ItemPageRecord, String> {
    let db_path = state.db_path();
    blocking(move || {
        db::query_station_episodes(&db_path, &station_id, offset, limit, search.as_deref())
    })
    .await
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
    audio::play_url(
        &app,
        item_id,
        url,
        start_position_seconds,
        duration_hint_seconds,
    )
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

#[tauri::command]
pub fn audio_list_output_devices(app: tauri::AppHandle) -> Vec<OutputDeviceInfo> {
    audio::list_output_devices(&app)
}

#[tauri::command]
pub fn audio_get_output_device(app: tauri::AppHandle) -> Option<String> {
    audio::get_selected_output_device(&app)
}

#[tauri::command]
pub fn audio_set_output_device(
    app: tauri::AppHandle,
    device_id: Option<String>,
) -> Result<(), String> {
    audio::set_output_device(&app, device_id)
}

// ---------------------------------------------------------------------------
// Queue management — backend-owned for headless playback
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn audio_play_with_queue(
    app: tauri::AppHandle,
    item: QueuedItem,
    manual_queue: Vec<QueuedItem>,
    auto_queue: Vec<QueuedItem>,
    start_position_seconds: f64,
) -> Result<(), String> {
    audio::play_with_queue(&app, item, manual_queue, auto_queue, start_position_seconds)
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
pub fn audio_queue_next(app: tauri::AppHandle) -> Result<(), String> {
    audio::queue_next(&app)
}

#[tauri::command]
pub fn audio_queue_prev(app: tauri::AppHandle) -> Result<(), String> {
    audio::queue_prev(&app)
}

#[tauri::command]
pub fn audio_queue_clear(app: tauri::AppHandle) -> Result<(), String> {
    audio::queue_clear(&app)
}

#[tauri::command]
pub fn audio_queue_clear_history(app: tauri::AppHandle) -> Result<(), String> {
    audio::queue_clear_history(&app)
}

#[tauri::command]
pub fn audio_queue_get_state(app: tauri::AppHandle) -> QueueState {
    audio::get_queue_state(&app)
}

#[tauri::command]
pub fn audio_queue_set(app: tauri::AppHandle, items: Vec<QueuedItem>) -> Result<(), String> {
    audio::queue_set(&app, items)
}

#[tauri::command]
pub fn clear_audio_cache(app: tauri::AppHandle) -> Result<(), String> {
    audio::cache::clear_audio_cache(&app)
}

// ---------------------------------------------------------------------------
// Playback context — frontend-managed persistence for feed/station context
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn save_playback_context(
    state: tauri::State<'_, db::DatabaseState>,
    context: Option<PlaybackContextRecord>,
) -> Result<(), String> {
    let db_path = state.db_path();
    blocking(move || db::save_playback_context(&db_path, context.as_ref())).await
}

#[tauri::command]
pub async fn load_playback_context(
    state: tauri::State<'_, db::DatabaseState>,
) -> Result<Option<PlaybackContextRecord>, String> {
    let db_path = state.db_path();
    blocking(move || db::load_playback_context(&db_path)).await
}

#[tauri::command]
pub async fn extract_cover_palette(
    image_url: String,
    image_cache: tauri::State<'_, ImageCache>,
) -> Result<Vec<String>, String> {
    let cache_dir = image_cache.cache_dir().to_path_buf();
    blocking(move || {
        let cache = crate::image_cache::ImageCache::new_from_path(&cache_dir)?;
        cover_art::extract_cover_palette(&image_url, &cache)
    })
    .await
}

#[tauri::command]
pub async fn get_cached_image_path(
    image_url: String,
    size: Option<u32>,
    image_cache: tauri::State<'_, ImageCache>,
    db: tauri::State<'_, DatabaseState>,
) -> Result<String, String> {
    let cache_dir = image_cache.cache_dir().to_path_buf();
    let cache_dir2 = cache_dir.clone();
    let db_path = db.db_path();

    let path = blocking(move || {
        let cache = crate::image_cache::ImageCache::new_from_path(&cache_dir)?;

        if let Some(s) = size {
            cache.get_or_create_thumbnail(&image_url, s)
        } else {
            cache.ensure_cached(&image_url)
        }
    })
    .await?;

    let bytes =
        std::fs::read(&path).map_err(|e| format!("Failed to read cached image: {e}"))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");
    let mime_type = match ext {
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/jpeg",
    };
    let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:{mime_type};base64,{base64}");

    let cache = crate::image_cache::ImageCache::new_from_path(&cache_dir2)?;
    let max_size = match db::load_app_settings(&db_path) {
        Ok(s) => u64::try_from(s.max_image_cache_size_bytes)
            .unwrap_or(crate::image_cache::DEFAULT_MAX_IMAGE_CACHE_SIZE_BYTES),
        Err(_) => crate::image_cache::DEFAULT_MAX_IMAGE_CACHE_SIZE_BYTES,
    };
    let _ = cache.enforce_size_limit(max_size);

    Ok(data_url)
}

#[tauri::command]
pub async fn get_cached_image_dimensions(
    image_url: String,
    image_cache: tauri::State<'_, ImageCache>,
) -> Result<Option<(u32, u32)>, String> {
    let cache_dir = image_cache.cache_dir().to_path_buf();
    blocking(move || {
        let cache = crate::image_cache::ImageCache::new_from_path(&cache_dir)?;
        cache.ensure_cached(&image_url)?;
        cache.get_cached_dimensions(&image_url)
    })
    .await
}

#[tauri::command]
pub fn set_window_content_aspect_ratio(
    app: tauri::AppHandle,
    label: String,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Window '{label}' not found."))?;

    set_macos_window_content_aspect_ratio(&window, width, height)?;

    Ok(())
}

#[tauri::command]
pub fn resize_mini_player(
    app: tauri::AppHandle,
    label: String,
    compact: bool,
) -> Result<(), String> {
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Window '{label}' not found."))?;

    // Skip the mutation if the user is actively dragging the resize handle.
    // AppKit internal state is unstable during live resize and multiple
    // constraint/frame changes can cause a re-entrant crash.
    if is_macos_window_in_live_resize(&window)? {
        return Ok(());
    }

    let current_size = window.inner_size().map_err(|e| e.to_string())?;
    let scale_factor = window.scale_factor().map_err(|e| e.to_string())?;
    let logical_size = current_size.to_logical::<f64>(scale_factor);
    if compact {
        // Remove the 1:1 aspect lock using a valid AppKit pattern.
        // setContentAspectRatio:0x0 is not documented as a "clear" value.
        clear_macos_window_content_aspect_ratio(&window)?;

        // Relax minimum size so the compact height is allowed.
        window
            .set_min_size(Some(tauri::LogicalSize::new(340.0, 160.0)))
            .map_err(|e| e.to_string())?;

        // Shrink to compact bar height, preserving current width but clamping to min.
        window
            .set_size(tauri::LogicalSize::new(
                logical_size.width.max(340.0),
                160.0,
            ))
            .map_err(|e| e.to_string())?;

        // Pin the height so the user cannot resize vertically.
        window
            .set_max_size(Some(tauri::LogicalSize::new(800.0, 160.0)))
            .map_err(|e| e.to_string())?;
    } else {
        // Relax max height so the window can grow back to square.
        window
            .set_max_size(Some(tauri::LogicalSize::new(800.0, 800.0)))
            .map_err(|e| e.to_string())?;

        // Make sure constraints allow the transition first.
        window
            .set_min_size(Some(tauri::LogicalSize::new(340.0, 80.0)))
            .map_err(|e| e.to_string())?;

        // Restore square size before locking the aspect ratio.
        window
            .set_size(tauri::LogicalSize::new(
                logical_size.width.max(340.0),
                logical_size.width.max(340.0),
            ))
            .map_err(|e| e.to_string())?;

        // Apply the aspect lock only after the frame is already square.
        set_macos_window_content_aspect_ratio(&window, 1.0, 1.0)?;

        // Restore original minimum size now the window is at a valid square frame.
        window
            .set_min_size(Some(tauri::LogicalSize::new(340.0, 340.0)))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
