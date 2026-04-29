//! Audio thread — owns MixerDeviceSink, Player, and all playback state.

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use rodio::{Decoder, Player};
use tauri::{AppHandle, Emitter, Manager};

use crate::db;
use crate::models::PlaybackSessionRecord;
use crate::queue::{QueueState, QueuedItem};

use super::cache::{
    cleanup_failed_playback_start, get_audio_cache_path, get_audio_cache_size_limit_bytes,
    hash_item_id, is_cache_complete,
};
use super::commands::AudioCommand;
use super::devices::open_output_sink;
use super::download::download_to_file;
use super::events::{PlaybackEndedEvent, PlaybackStateEvent};
use super::streaming_file::StreamingFile;
use rodio::MixerDeviceSink;

/// Background download that can be adopted by playback.
struct PrefetchState {
    item_id: String,
    _url: String,
    cache_path: PathBuf,
    meta: Arc<super::download::DownloadMeta>,
    download_thread: Option<std::thread::JoinHandle<()>>,
}

impl PrefetchState {
    fn cancel(&mut self) {
        self.meta.cancelled.store(true, Ordering::Release);
        let _ = self.download_thread.take();
    }
}

pub struct AudioThread {
    sink_handle: Option<MixerDeviceSink>,
    player: Option<Player>,
    selected_output_device_id: Option<String>,
    current_item_id: Option<String>,
    stored_position_seconds: f64,
    duration_seconds: f64,
    download_meta: Option<Arc<super::download::DownloadMeta>>,
    temp_path: Option<PathBuf>,
    volume: f32,
    speed: f32,
    queue: QueueState,
    app: AppHandle,
    prefetch: Option<PrefetchState>,
}

impl AudioThread {
    pub fn new(app: AppHandle) -> Self {
        Self {
            sink_handle: None,
            player: None,
            selected_output_device_id: None,
            current_item_id: None,
            stored_position_seconds: 0.0,
            duration_seconds: 0.0,
            download_meta: None,
            temp_path: None,
            volume: 1.0,
            speed: 1.0,
            queue: QueueState::default(),
            app,
            prefetch: None,
        }
    }

    fn ensure_output_sink(&mut self) -> Result<(), String> {
        if self.sink_handle.is_none() {
            self.rebuild_output_sink()?;
        }
        Ok(())
    }

    fn rebuild_output_sink(&mut self) -> Result<(), String> {
        let handle = open_output_sink(self.selected_output_device_id.as_deref())?;
        self.sink_handle = Some(handle);
        Ok(())
    }

    pub fn handle_play(
        &mut self,
        item_id: String,
        url: String,
        start_position_seconds: f64,
        duration_hint_seconds: f64,
    ) -> Result<(), String> {
        let play_start = Instant::now();

        // Check for matching prefetch BEFORE tearing down
        let prefetch_match = self
            .prefetch
            .as_ref()
            .is_some_and(|p| p.item_id == item_id && !p.meta.cancelled.load(Ordering::Acquire));

        if prefetch_match {
            log::info!("Prefetch hit for item_id={}", item_id);
        } else {
            // Cancel any existing prefetch before teardown
            if let Some(mut prefetch) = self.prefetch.take() {
                log::debug!("Cancelling prefetch for item_id={}", prefetch.item_id);
                prefetch.cancel();
            }
        }

        let teardown_start = Instant::now();
        self.teardown_output_only();
        log::debug!("Teardown took {:?}", teardown_start.elapsed());

        self.ensure_output_sink()?;

        self.current_item_id = Some(item_id.clone());
        self.stored_position_seconds = start_position_seconds.max(0.0);
        self.duration_seconds = duration_hint_seconds.max(0.0);

        let (meta, cache_path, is_adopted_prefetch) = if prefetch_match {
            let prefetch = self.prefetch.take().unwrap();
            let meta = Arc::clone(&prefetch.meta);
            let path = prefetch.cache_path.clone();

            (meta, path, true)
        } else {
            let cache_dir = get_audio_cache_path(&self.app)?;
            let cache_path = cache_dir.join(format!("{}.mp3", hash_item_id(&item_id)));
            log::debug!(
                "Checking cache for playback: item_id={}, path={:?}",
                item_id,
                cache_path
            );
            let meta = super::download::DownloadMeta::new();

            // Check for complete cache
            if is_cache_complete(&cache_path) {
                log::info!("Cache hit (complete) for item_id={}", item_id);
                meta.complete.store(true, Ordering::Release);
                let size = std::fs::metadata(&cache_path).map(|m| m.len()).unwrap_or(0);
                meta.bytes_written.store(size, Ordering::Release);
                meta.total_size.store(size, Ordering::Release);
            } else {
                // Start download in background
                let dl_meta = Arc::clone(&meta);
                let dl_path = cache_path.clone();
                let dl_url = url.clone();
                let dl_cache_dir = cache_dir.clone();
                let dl_cache_limit_bytes = get_audio_cache_size_limit_bytes(&self.app);
                std::thread::Builder::new()
                    .name("jrss-download".into())
                    .spawn(move || {
                        if let Err(e) = download_to_file(
                            &dl_url,
                            &dl_path,
                            &dl_meta,
                            &dl_cache_dir,
                            dl_cache_limit_bytes,
                        ) {
                            log::error!("Audio download failed: {}", e);
                            dl_meta.complete.store(true, Ordering::Release);
                        }
                    })
                    .map_err(|e| format!("Failed to spawn audio download thread: {e}"))?;
            }

            (meta, cache_path, false)
        };

        let wait_start = Instant::now();
        if !is_adopted_prefetch {
            if let Err(e) = wait_for_minimum_data(&meta, 64 * 1024, Duration::from_secs(10)) {
                cleanup_failed_playback_start(&meta, &cache_path);
                return Err(format!("Audio startup failed: {e}"));
            }
        }
        log::debug!("Wait for data took {:?}", wait_start.elapsed());

        let open_start = Instant::now();
        let streaming = match StreamingFile::open(&cache_path, Arc::clone(&meta)) {
            Ok(s) => s,
            Err(e) => {
                cleanup_failed_playback_start(&meta, &cache_path);
                return Err(format!("Failed to open streaming file: {e}"));
            }
        };
        log::debug!("Open streaming file took {:?}", open_start.elapsed());

        let decode_start = Instant::now();
        let decoder = match Decoder::new(streaming) {
            Ok(d) => d,
            Err(e) => {
                cleanup_failed_playback_start(&meta, &cache_path);
                return Err(format!("Failed to decode audio: {e}"));
            }
        };
        log::debug!("Decoder creation took {:?}", decode_start.elapsed());

        let handle = self
            .sink_handle
            .as_ref()
            .ok_or_else(|| "No output sink available".to_string())?;

        let player = Player::connect_new(handle.mixer());
        player.set_volume(self.volume);
        player.set_speed(self.speed);
        player.append(decoder);

        let seek_start = Instant::now();
        if start_position_seconds > 0.0 {
            let seek_duration = Duration::from_secs_f64(start_position_seconds);
            if player.try_seek(seek_duration).is_err() {
                log::warn!("try_seek failed (MP3 without seek table?), restarting at position");
                // MP3 doesn't support seeking, restart at position by dropping and recreating
                drop(player);

                let streaming = StreamingFile::open(&cache_path, Arc::clone(&meta))
                    .map_err(|e| format!("Failed to reopen streaming file: {e}"))?;
                let decoder = Decoder::new(streaming)
                    .map_err(|e| format!("Failed to recreate decoder: {e}"))?;
                let player = Player::connect_new(handle.mixer());
                player.set_volume(self.volume);
                player.set_speed(self.speed);
                player.append(decoder);

                self.player = Some(player);
            } else {
                self.player = Some(player);
            }
        } else {
            self.player = Some(player);
        }
        log::debug!("Seek operation took {:?}", seek_start.elapsed());

        self.download_meta = Some(meta);
        self.temp_path = Some(cache_path);

        // Prefetch next item in queue for seamless transition
        self.prefetch_next_in_queue();

        log::info!("Total handle_play took {:?}", play_start.elapsed());

        Ok(())
    }

    fn teardown_output_only(&mut self) {
        if let Some(ref meta) = self.download_meta {
            meta.cancelled.store(true, Ordering::Release);
            meta.complete.store(true, Ordering::Release);
        }
        if let Some(player) = self.player.take() {
            player.stop();
        }
        if let Some(ref path) = self.temp_path {
            // Only remove if not in cache (cache files persist)
            let cache_dir = get_audio_cache_path(&self.app).ok();
            let is_cache_file = cache_dir.as_ref().is_some_and(|dir| path.starts_with(dir));
            if !is_cache_file {
                let _ = std::fs::remove_file(path);
            }
        }
        self.download_meta = None;
        self.temp_path = None;
    }

    fn stop_current(&mut self) {
        self.teardown_output_only();
        self.current_item_id = None;
        self.stored_position_seconds = 0.0;
        self.duration_seconds = 0.0;
    }

    /// Prefetch the next item in queue so it's ready when current finishes.
    fn prefetch_next_in_queue(&mut self) {
        if let Some(next_item) = self.queue.peek_next() {
            let item_id = next_item.item_id.clone();
            let url = next_item.url.clone();
            log::debug!("Prefetching next item in queue: item_id={}", item_id);
            self.handle_prefetch(item_id, url);
        } else {
            log::debug!("No next item in queue to prefetch");
        }
    }

    pub fn snapshot(&self) -> Option<PlaybackStateEvent> {
        let item_id = self.current_item_id.as_ref()?;
        let position_seconds = self
            .player
            .as_ref()
            .map(|player| player.get_pos().as_secs_f64())
            .unwrap_or(self.stored_position_seconds);
        let is_playing = self
            .player
            .as_ref()
            .map(|player| !player.is_paused() && !player.empty())
            .unwrap_or(false);

        Some(PlaybackStateEvent {
            item_id: item_id.clone(),
            position_seconds,
            duration_seconds: self.duration_seconds,
            is_playing,
            volume: self.volume as f64,
            speed: self.speed as f64,
        })
    }

    fn sync_cached_position(&mut self) {
        if let Some(ref player) = self.player {
            self.stored_position_seconds = player.get_pos().as_secs_f64();
        }
    }

    pub fn persist_session(&mut self) {
        self.sync_cached_position();

        let db_state = self.app.state::<db::DatabaseState>();
        let db_path = db_state.db_path();

        if self.queue.current.is_none() && self.queue.is_empty() {
            if let Err(error) = db::clear_playback_session(&db_path) {
                log::error!("Failed to clear playback session: {error}");
            }
            return;
        }

        let (history_queue, manual_queue, auto_queue) = self.queue.to_session_parts();
        let session = PlaybackSessionRecord {
            current_item_id: self.queue.current_item().map(|item| item.item_id.clone()),
            position_seconds: self.stored_position_seconds.floor() as i64,
            duration_seconds: self.duration_seconds.floor() as i64,
            history_queue,
            manual_queue,
            auto_queue,
            playback_context: None, // Frontend-managed, not stored in backend session
        };

        if let Err(error) = db::save_playback_session(&db_path, &session) {
            log::error!("Failed to save playback session: {error}");
        }
    }

    pub fn change_output_device(&mut self, device_id: Option<String>) -> Result<(), String> {
        self.sync_cached_position();

        let had_current_item = self.queue.current_item().is_some();
        let should_pause_after_restore = self
            .player
            .as_ref()
            .map(|player| player.is_paused())
            .unwrap_or(true);

        self.teardown_output_only();
        self.sink_handle = None;
        self.selected_output_device_id = device_id;

        self.rebuild_output_sink()?;

        if had_current_item {
            resume_current_item(self)?;
            if should_pause_after_restore {
                if let Some(ref player) = self.player {
                    player.pause();
                }
            }
        }

        Ok(())
    }

    fn handle_prefetch(&mut self, item_id: String, url: String) {
        // Cancel any existing prefetch
        if let Some(mut prefetch) = self.prefetch.take() {
            if prefetch.item_id != item_id {
                log::debug!("Cancelling prefetch for item_id={}", prefetch.item_id);
                prefetch.cancel();
            } else {
                self.prefetch = Some(prefetch);
                return;
            }
        }

        // Check if already cached
        let cache_path = match get_audio_cache_path(&self.app) {
            Ok(dir) => dir.join(format!("{}.mp3", hash_item_id(&item_id))),
            Err(e) => {
                log::error!("Failed to get cache path for prefetch: {e}");
                return;
            }
        };

        log::debug!(
            "Checking cache for prefetch: item_id={}, path={:?}",
            item_id,
            cache_path
        );

        if is_cache_complete(&cache_path) {
            log::info!("Prefetch skipped - already cached: item_id={}", item_id);
            return;
        }

        // Start prefetch download
        let meta = super::download::DownloadMeta::new();
        let dl_meta = Arc::clone(&meta);
        let dl_path = cache_path.clone();
        let dl_url = url.clone();
        let cache_limit_bytes = get_audio_cache_size_limit_bytes(&self.app);

        let cache_dir_for_prefetch = cache_path.parent().map(|p| p.to_path_buf());
        let handle = std::thread::Builder::new()
            .name("jrss-prefetch".into())
            .spawn(move || {
                let cache_dir = cache_dir_for_prefetch.as_deref().unwrap_or(Path::new(""));
                if let Err(e) =
                    download_to_file(&dl_url, &dl_path, &dl_meta, cache_dir, cache_limit_bytes)
                {
                    log::error!("Prefetch download failed: {}", e);
                    dl_meta.complete.store(true, Ordering::Release);
                }
            });

        match handle {
            Ok(thread) => {
                log::info!("Started prefetch for item_id={}", item_id);
                self.prefetch = Some(PrefetchState {
                    item_id,
                    _url: url,
                    cache_path,
                    meta,
                    download_thread: Some(thread),
                });
            }
            Err(e) => {
                log::error!("Failed to spawn prefetch thread: {e}");
            }
        }
    }
}

impl Drop for AudioThread {
    fn drop(&mut self) {
        self.stop_current();
        if let Some(mut prefetch) = self.prefetch.take() {
            prefetch.cancel();
        }
    }
}

fn emit_playback_snapshot(
    app: &AppHandle,
    state: &AudioThread,
    last_emit: &mut Instant,
    last_emitted_state: &mut Option<PlaybackStateEvent>,
) {
    if let Some(snapshot) = state.snapshot() {
        *last_emit = Instant::now();
        *last_emitted_state = Some(snapshot.clone());
        let _ = app.emit("playback-state-changed", snapshot);
    }
}

pub fn audio_thread_main(rx: mpsc::Receiver<AudioCommand>, app: AppHandle) {
    use super::devices::list_output_devices;

    let mut state = AudioThread::new(app.clone());
    restore_persisted_session(&mut state, &app);

    if let Err(error) = state.rebuild_output_sink() {
        log::error!("Failed to open initial audio output: {error}");
    }

    let poll_interval = Duration::from_millis(500);
    let persist_interval = Duration::from_secs(5);
    let mut last_persist = Instant::now();
    let mut was_playing = false;

    // Throttling for playback-state-changed events
    let min_emit_interval_while_playing = Duration::from_millis(250);
    let min_emit_interval_while_paused = Duration::from_secs(2);
    let mut last_emit = Instant::now();
    let mut last_emitted_state: Option<PlaybackStateEvent> = None;

    loop {
        match rx.recv_timeout(poll_interval) {
            Ok(cmd) => match cmd {
                AudioCommand::Play {
                    item_id,
                    url,
                    start_position_seconds,
                    duration_hint_seconds,
                } => {
                    was_playing = false;
                    state.queue.current = Some(QueuedItem {
                        item_id: item_id.clone(),
                        url: url.clone(),
                        title: String::new(),
                        duration_seconds: duration_hint_seconds,
                    });

                    if let Err(error) = state.handle_play(
                        item_id,
                        url,
                        start_position_seconds,
                        duration_hint_seconds,
                    ) {
                        log::error!("Play failed: {error}");
                    }

                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::Pause => {
                    if let Some(ref player) = state.player {
                        player.pause();
                    }
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::Resume => {
                    if let Some(ref player) = state.player {
                        player.play();
                    } else if let Err(error) = resume_current_item(&mut state) {
                        log::info!("Resume ignored: {error}");
                    }
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::TogglePlayback => {
                    if let Some(ref player) = state.player {
                        if player.is_paused() {
                            player.play();
                        } else {
                            player.pause();
                        }
                    } else if let Err(error) = resume_current_item(&mut state) {
                        log::info!("TogglePlayback ignored: {error}");
                    }
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::Stop => {
                    state.sync_cached_position();
                    if let Some(ref item_id) = state.current_item_id {
                        let db = app.state::<crate::db::DatabaseState>();
                        let _ = crate::db::save_playback(
                            &db.db_path(),
                            item_id,
                            state.stored_position_seconds as i64,
                        );
                    }
                    state.stop_current();
                    state.queue.clear_current();
                    state.persist_session();
                    last_emitted_state = None;
                    let _ = app.emit("playback-stopped", ());
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::Seek { position_seconds } => {
                    let seek_start = Instant::now();
                    let seek_result = state.player.as_ref().map(|player| {
                        let target = Duration::from_secs_f64(position_seconds.max(0.0));
                        player.try_seek(target)
                    });

                    match seek_result {
                        Some(Ok(())) => {
                            // Seek succeeded, nothing more to do
                        }
                        _ => {
                            // Seek failed or no player - restart at position
                            if seek_result.is_some() {
                                log::warn!(
                                    "Seek failed in decoder, will restart playback at position"
                                );
                            }
                            state.stored_position_seconds = position_seconds.max(0.0);
                            let current = state.queue.current_item().cloned();
                            if let Some(item) = current {
                                // Take ownership of player before teardown
                                let _ = state.player.take();
                                state.teardown_output_only();
                                if let Err(error) = state.handle_play(
                                    item.item_id,
                                    item.url,
                                    state.stored_position_seconds,
                                    if state.duration_seconds > 0.0 {
                                        state.duration_seconds
                                    } else {
                                        item.duration_seconds
                                    },
                                ) {
                                    log::error!("Seek-restart failed: {error}");
                                }
                            } else if state.current_item_id.is_some() {
                                state.stored_position_seconds = position_seconds.max(0.0);
                            }
                        }
                    }
                    log::debug!("Seek command took {:?}", seek_start.elapsed());
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::SetVolume { volume } => {
                    state.volume = volume;
                    if let Some(ref player) = state.player {
                        player.set_volume(volume);
                    }
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::SetSpeed { speed } => {
                    state.speed = speed;
                    if let Some(ref player) = state.player {
                        player.set_speed(speed);
                    }
                    // Emit state update so frontend sees speed change
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::GetState { reply } => {
                    let _ = reply.send(state.snapshot());
                }
                AudioCommand::PlayWithQueue {
                    item,
                    manual_queue,
                    auto_queue,
                    start_position_seconds,
                } => {
                    was_playing = false;
                    state
                        .queue
                        .replace(Some(item.clone()), manual_queue, auto_queue);

                    if let Some(current) = state.queue.current_item().cloned() {
                        if let Err(error) = state.handle_play(
                            current.item_id,
                            current.url,
                            start_position_seconds,
                            current.duration_seconds,
                        ) {
                            log::error!("PlayWithQueue failed: {error}");
                        }
                    }

                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::QueueEnqueue { item } => {
                    state.queue.enqueue(item);
                    state.prefetch_next_in_queue();
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueuePlayNext { item } => {
                    state.queue.play_next(item);
                    state.prefetch_next_in_queue();
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueRemove { item_id } => {
                    state.queue.remove(&item_id);
                    state.prefetch_next_in_queue();
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueMoveUp { item_id } => {
                    state.queue.move_up(&item_id);
                    state.prefetch_next_in_queue();
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueMoveDown { item_id } => {
                    state.queue.move_down(&item_id);
                    state.prefetch_next_in_queue();
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueClear => {
                    state.queue.clear();
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueClearHistory => {
                    state.queue.clear_history();
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueGetState { reply } => {
                    let _ = reply.send(state.queue.clone());
                }
                AudioCommand::QueueSet { items } => {
                    state
                        .queue
                        .replace(state.queue.current.clone(), items, Vec::new());
                    state.prefetch_next_in_queue();
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::ListOutputDevices { reply } => {
                    let _ = reply.send(list_output_devices());
                }
                AudioCommand::GetSelectedOutputDevice { reply } => {
                    let _ = reply.send(state.selected_output_device_id.clone());
                }
                AudioCommand::SetOutputDevice { device_id, reply } => {
                    let result = state.change_output_device(device_id);
                    if result.is_ok() {
                        state.persist_session();
                        if let Some(snapshot) = state.snapshot() {
                            let _ = app.emit("playback-state-changed", snapshot);
                        }
                    }
                    let _ = reply.send(result);
                }
                AudioCommand::Prefetch { item_id, url } => {
                    state.handle_prefetch(item_id, url);
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if let Some(snapshot) = state.snapshot() {
            // Throttle playback-state-changed emissions
            let min_interval = if snapshot.is_playing {
                min_emit_interval_while_playing
            } else {
                min_emit_interval_while_paused
            };

            let should_emit = last_emit.elapsed() >= min_interval && {
                if let Some(ref last) = last_emitted_state {
                    // Only emit if something meaningful changed
                    last.item_id != snapshot.item_id
                        || last.is_playing != snapshot.is_playing
                        || (last.position_seconds as i64) != (snapshot.position_seconds as i64)
                        || (last.speed as i64) != (snapshot.speed as i64)
                } else {
                    true // Always emit if we haven't emitted before
                }
            };

            if should_emit {
                last_emit = Instant::now();
                last_emitted_state = Some(snapshot.clone());
                let _ = app.emit("playback-state-changed", &snapshot);
            }

            if snapshot.is_playing && last_persist.elapsed() >= persist_interval {
                last_persist = Instant::now();
                let db = app.state::<crate::db::DatabaseState>();
                let _ = crate::db::save_playback(
                    &db.db_path(),
                    &snapshot.item_id,
                    snapshot.position_seconds as i64,
                );
                state.persist_session();
            }

            if was_playing && !snapshot.is_playing {
                if let Some(ref player) = state.player {
                    if player.empty() {
                        let finished_item_id = snapshot.item_id.clone();
                        let db = app.state::<crate::db::DatabaseState>();
                        let _ = crate::db::save_playback(&db.db_path(), &finished_item_id, 0);

                        state.stop_current();
                        let _ = app.emit(
                            "playback-ended",
                            PlaybackEndedEvent {
                                item_id: finished_item_id.clone(),
                            },
                        );

                        if let Some(next_item) = state.queue.shift_next() {
                            match state.handle_play(
                                next_item.item_id.clone(),
                                next_item.url.clone(),
                                0.0,
                                next_item.duration_seconds,
                            ) {
                                Ok(()) => {
                                    emit_playback_snapshot(
                                        &app,
                                        &state,
                                        &mut last_emit,
                                        &mut last_emitted_state,
                                    );
                                }
                                Err(error) => {
                                    log::error!("Auto-advance failed: {error}");
                                }
                            }
                        } else {
                            state.queue.clear_current();
                            let _ = app.emit("playback-stopped", ());
                        }

                        state.persist_session();
                        let _ = app.emit("queue-changed", state.queue.to_event());
                    }
                }
            }

            was_playing = snapshot.is_playing;
        }
    }
}

fn wait_for_minimum_data(
    meta: &super::download::DownloadMeta,
    min_bytes: u64,
    timeout: Duration,
) -> Result<(), String> {
    let start = Instant::now();
    loop {
        let written = meta.bytes_written.load(Ordering::Acquire);
        let complete = meta.complete.load(Ordering::Acquire);

        if written >= min_bytes || complete {
            return Ok(());
        }

        if start.elapsed() > timeout {
            return Err("Timed out waiting for audio data".to_string());
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}

fn feed_item_to_queued_item(item: crate::models::FeedItemRecord) -> Option<QueuedItem> {
    let enclosure = item.media_enclosure?;

    Some(QueuedItem {
        item_id: item.id,
        url: enclosure.url,
        title: item.title,
        duration_seconds: enclosure.duration_seconds.unwrap_or(0) as f64,
    })
}

fn load_queued_item(db_path: &std::path::Path, item_id: String) -> Option<QueuedItem> {
    crate::db::get_item_by_id(db_path, &item_id)
        .ok()
        .flatten()
        .and_then(feed_item_to_queued_item)
}

fn restore_persisted_session(state: &mut AudioThread, _app: &AppHandle) {
    let db_state = state.app.state::<db::DatabaseState>();
    let db_path = db_state.db_path();

    let session = match db::load_playback_session(&db_path) {
        Ok(session) => session,
        Err(error) => {
            log::error!("Failed to load playback session: {error}");
            return;
        }
    };

    let Some(session) = session else {
        return;
    };

    let history = session
        .history_queue
        .into_iter()
        .filter_map(|item_id| load_queued_item(&db_path, item_id))
        .collect::<Vec<_>>();
    let current = session
        .current_item_id
        .and_then(|item_id| load_queued_item(&db_path, item_id));
    let manual = session
        .manual_queue
        .into_iter()
        .filter_map(|item_id| load_queued_item(&db_path, item_id))
        .collect::<Vec<_>>();
    let auto = session
        .auto_queue
        .into_iter()
        .filter_map(|item_id| load_queued_item(&db_path, item_id))
        .collect::<Vec<_>>();

    state
        .queue
        .replace_with_history(history, current.clone(), manual, auto);
    state.current_item_id = current.as_ref().map(|item| item.item_id.clone());
    state.stored_position_seconds = session.position_seconds.max(0) as f64;
    state.duration_seconds = if session.duration_seconds > 0 {
        session.duration_seconds as f64
    } else {
        current
            .as_ref()
            .map(|item| item.duration_seconds)
            .unwrap_or(0.0)
    };

    // Start prefetch for restored session
    if let Some(ref current_item) = current {
        state.handle_prefetch(current_item.item_id.clone(), current_item.url.clone());
    }

    if state.queue.current.is_none() && !state.queue.is_empty() {
        state.persist_session();
    }
}

fn resume_current_item(state: &mut AudioThread) -> Result<(), String> {
    let Some(current) = state.queue.current_item().cloned() else {
        return Err("No current item to resume".to_string());
    };

    state.handle_play(
        current.item_id,
        current.url,
        state.stored_position_seconds,
        if state.duration_seconds > 0.0 {
            state.duration_seconds
        } else {
            current.duration_seconds
        },
    )
}

use std::path::Path;
