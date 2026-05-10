//! Audio thread — orchestrates download, queue, session persistence, and Tauri
//! event emission. Delegates all media rendering to a [`PlaybackEngine`].

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::db;
use crate::models::PlaybackSessionRecord;
use crate::queue::{QueueState, QueuedItem};

use super::cache::{
    cleanup_failed_playback_start, get_audio_cache_path, get_audio_cache_size_limit_bytes,
    hash_item_id, is_cache_complete,
};
use super::commands::AudioCommand;
use super::download::download_to_file;
use super::engine::{PlayConfig, PlaybackEngine};
use super::events::{PlaybackEndedEvent, PlaybackErrorEvent, PlaybackStateEvent};
use super::rodio_engine::RodioEngine;
use super::streaming_file::StreamingFile;
use rodio::Source;

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
    /// Media rendering engine. Swapping this field is sufficient to support
    /// a new media type (e.g. video) without touching any orchestration logic.
    engine: Box<dyn PlaybackEngine>,
    current_item_id: Option<String>,
    current_item_title: String,
    current_feed_title: String,
    stored_position_seconds: f64,
    duration_seconds: f64,
    download_meta: Option<Arc<super::download::DownloadMeta>>,
    temp_path: Option<PathBuf>,
    volume: f32,
    speed: f32,
    queue: QueueState,
    app: AppHandle,
    prefetch: Option<PrefetchState>,
    duration_probe_done: bool,
    #[cfg(target_os = "macos")]
    power_assertion: Option<super::macos::power::PowerAssertion>,
}

impl AudioThread {
    pub fn new(app: AppHandle) -> Self {
        Self {
            engine: Box::new(RodioEngine::new()),
            current_item_id: None,
            current_item_title: String::new(),
            current_feed_title: String::new(),
            stored_position_seconds: 0.0,
            duration_seconds: 0.0,
            download_meta: None,
            temp_path: None,
            volume: 1.0,
            speed: 1.0,
            queue: QueueState::default(),
            app,
            prefetch: None,
            duration_probe_done: false,
            #[cfg(target_os = "macos")]
            power_assertion: None,
        }
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

        self.current_item_id = Some(item_id.clone());
        self.hydrate_metadata(&item_id);
        self.duration_seconds = duration_hint_seconds.max(0.0);
        self.duration_probe_done = false;

        // Clamp start position to valid bounds [0, duration - epsilon]
        // The epsilon prevents seeking exactly to the end which can cause immediate playback end
        // This handles cases where RSS metadata duration doesn't match actual audio
        const END_EPSILON: f64 = 0.5; // Leave 0.5s buffer at the end
        let clamped_start_position = if self.duration_seconds > END_EPSILON {
            start_position_seconds.clamp(0.0, self.duration_seconds - END_EPSILON)
        } else {
            start_position_seconds.max(0.0)
        };

        if (clamped_start_position - start_position_seconds).abs() > 1.0 {
            log::debug!(
                "Start position clamped: requested={}s, duration={}s, clamped={}s",
                start_position_seconds,
                self.duration_seconds,
                clamped_start_position
            );
        }

        self.stored_position_seconds = clamped_start_position;

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

        // Byte-length hint for accurate VBR MP3 duration calculation.
        // Only pass the hint when the cache is complete — using a partial file
        // size causes the decoder to calculate a truncated duration and stop
        // decoding early while the download is still in progress.
        let byte_len_hint = if meta.complete.load(Ordering::Acquire) {
            std::fs::metadata(&cache_path).map(|m| m.len()).ok()
        } else {
            None
        };

        let config = PlayConfig {
            start_position_seconds: self.stored_position_seconds,
            duration_hint_seconds: self.duration_seconds,
            volume: self.volume,
            speed: self.speed,
            byte_len_hint,
        };

        let decode_start = Instant::now();
        let actual_duration = self.engine.play_stream(streaming, config).map_err(|e| {
            cleanup_failed_playback_start(&meta, &cache_path);
            e.to_string()
        })?;
        log::debug!("Decoder + seek took {:?}", decode_start.elapsed());

        // Use actual audio duration from decoder if available (more accurate than RSS metadata)
        if let Some(actual_seconds) = actual_duration {
            if actual_seconds > 0.0 && actual_seconds != self.duration_seconds {
                log::info!(
                    "Using actual audio duration: {}s (RSS hint was: {}s)",
                    actual_seconds,
                    self.duration_seconds
                );
                self.duration_seconds = actual_seconds;

                // Update queue current item duration so frontend gets correct value
                if let Some(ref mut current) = self.queue.current {
                    current.duration_seconds = actual_seconds;
                }

                // Persist detected duration to the database for future reference
                if let Some(ref item_id) = self.current_item_id {
                    let db_state = self.app.state::<db::DatabaseState>();
                    let db_path = db_state.db_path();
                    let duration_i64 = actual_seconds.floor() as i64;
                    if let Err(error) = db::update_item_duration(&db_path, item_id, duration_i64) {
                        log::warn!("Failed to persist detected duration: {error}");
                    }
                }
            }
        }

        self.download_meta = Some(meta);
        self.temp_path = Some(cache_path);

        #[cfg(target_os = "macos")]
        {
            self.power_assertion = Some(super::macos::power::PowerAssertion::new("JRSS audio playback"));
        }

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
        self.engine.stop();
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
        self.current_item_title.clear();
        self.current_feed_title.clear();
        self.stored_position_seconds = 0.0;
        self.duration_seconds = 0.0;
        #[cfg(target_os = "macos")]
        {
            self.power_assertion = None;
        }
    }

    fn hydrate_metadata(&mut self, item_id: &str) {
        let db_state = self.app.state::<db::DatabaseState>();
        let db_path = db_state.db_path();

        if let Some(item) = db::get_item_by_id(&db_path, item_id).ok().flatten() {
            self.current_item_title = item.title;
            if let Some(feed) = db::get_feed_by_id(&db_path, &item.feed_id).ok().flatten() {
                self.current_feed_title = feed.title;
            } else {
                self.current_feed_title.clear();
            }
        } else {
            self.current_item_title.clear();
            self.current_feed_title.clear();
        }
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

        let position_seconds = if self.engine.has_active_playback() {
            self.engine.position_seconds()
        } else {
            self.stored_position_seconds
        };

        let is_playing = self.engine.has_active_playback()
            && !self.engine.is_paused()
            && !self.engine.is_finished();

        // Dynamically extend duration if position exceeds it (handles dynamic ad injection)
        // Use the greater of: stored duration, current position, or stored position
        let effective_duration = self
            .duration_seconds
            .max(position_seconds)
            .max(self.stored_position_seconds);

        Some(PlaybackStateEvent {
            item_id: item_id.clone(),
            title: self.current_item_title.clone(),
            artist: self.current_feed_title.clone(),
            position_seconds,
            duration_seconds: effective_duration,
            is_playing,
            volume: self.volume as f64,
            speed: self.speed as f64,
        })
    }

    fn try_probe_complete_file_duration(&mut self) -> Option<f64> {
        if self.duration_probe_done {
            return None;
        }

        let meta = self.download_meta.as_ref()?;
        if !meta.complete.load(Ordering::Acquire) {
            return None;
        }

        self.duration_probe_done = true;

        let path = self.temp_path.as_ref()?;
        let file = std::fs::File::open(path).ok()?;
        let byte_len = std::fs::metadata(path).map(|m| m.len()).ok()?;

        let decoder = rodio::Decoder::builder()
            .with_data(file)
            .with_byte_len(byte_len)
            .build()
            .ok()?;

        let duration = decoder.total_duration().map(|d| d.as_secs_f64());
        log::debug!(
            "Probed duration for {}: {:?} (file_size={} bytes)",
            path.display(),
            duration,
            byte_len
        );
        duration
    }

    fn sync_cached_position(&mut self) {
        if self.engine.has_active_playback() {
            self.stored_position_seconds = self.engine.position_seconds();
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

        // Update stored duration to effective duration if position has exceeded it
        // This handles dynamic ad injection where actual audio is longer than RSS metadata
        let effective_duration = self
            .duration_seconds
            .max(self.stored_position_seconds);
        if effective_duration > self.duration_seconds {
            log::debug!(
                "Updating duration from {}s to {}s based on playback position",
                self.duration_seconds,
                effective_duration
            );
            self.duration_seconds = effective_duration;
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
        let should_pause_after_restore =
            !self.engine.has_active_playback() || self.engine.is_paused();

        self.teardown_output_only();
        self.engine
            .set_output_device(device_id)
            .map_err(|e| e.to_string())?;

        if had_current_item {
            resume_current_item(self)?;
            if should_pause_after_restore {
                self.engine.pause();
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
    #[cfg(target_os = "macos")]
    super::macos::set_audio_thread_qos();

    let mut state = AudioThread::new(app.clone());
    restore_persisted_session(&mut state, &app);

    state.engine.initialize();

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
                        item_id.clone(),
                        url,
                        start_position_seconds,
                        duration_hint_seconds,
                    ) {
                        log::error!("Play failed: {error}");
                        let _ = app.emit(
                            "playback-error",
                            PlaybackErrorEvent {
                                item_id: item_id.clone(),
                                error: error.clone(),
                            },
                        );
                    }

                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::Pause => {
                    state.engine.pause();
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::Resume => {
                    if state.engine.has_active_playback() {
                        state.engine.resume();
                    } else if let Err(error) = resume_current_item(&mut state) {
                        log::info!("Resume ignored: {error}");
                    }
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::TogglePlayback => {
                    if state.engine.has_active_playback() {
                        if state.engine.is_paused() {
                            state.engine.resume();
                        } else {
                            state.engine.pause();
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

                    const END_EPSILON: f64 = 0.5;
                    let clamped_position = if state.duration_seconds > END_EPSILON {
                        position_seconds.clamp(0.0, state.duration_seconds - END_EPSILON)
                    } else {
                        position_seconds.max(0.0)
                    };

                    if (clamped_position - position_seconds).abs() > 1.0 {
                        log::debug!(
                            "Seek position clamped: requested={}s, actual_duration={}s, clamped={}s",
                            position_seconds,
                            state.duration_seconds,
                            clamped_position
                        );
                    }

                    if state.engine.has_active_playback() {
                        if !state.engine.seek(clamped_position) {
                            log::warn!(
                                "Seek failed in decoder (MP3 without seek table?), position unchanged"
                            );
                        }
                        state.stored_position_seconds = clamped_position;
                    }

                    log::debug!("Seek command took {:?}", seek_start.elapsed());
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::SkipForward { delta_seconds } | AudioCommand::SkipBackward { delta_seconds } => {
                    let current_position = if state.engine.has_active_playback() {
                        state.engine.position_seconds()
                    } else {
                        state.stored_position_seconds
                    };

                    let direction = if matches!(cmd, AudioCommand::SkipForward { .. }) {
                        1.0
                    } else {
                        -1.0
                    };

                    let target = current_position + direction * delta_seconds;

                    const END_EPSILON: f64 = 0.5;
                    let clamped_position = if state.duration_seconds > END_EPSILON {
                        target.clamp(0.0, state.duration_seconds - END_EPSILON)
                    } else {
                        target.max(0.0)
                    };

                    if state.engine.has_active_playback() {
                        if !state.engine.seek(clamped_position) {
                            log::warn!("Skip seek failed in decoder, position unchanged");
                        }
                        state.stored_position_seconds = clamped_position;
                    }

                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::SetVolume { volume } => {
                    state.volume = volume;
                    state.engine.set_volume(volume);
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::SetSpeed { speed } => {
                    state.speed = speed;
                    state.engine.set_speed(speed);
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
                        let current_item_id = current.item_id.clone();
                        if let Err(error) = state.handle_play(
                            current.item_id,
                            current.url,
                            start_position_seconds,
                            current.duration_seconds,
                        ) {
                            log::error!("PlayWithQueue failed: {error}");
                            let _ = app.emit(
                                "playback-error",
                                PlaybackErrorEvent {
                                    item_id: current_item_id,
                                    error: error.clone(),
                                },
                            );
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
                AudioCommand::QueueNext => {
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
                    if let Some(next_item) = state.queue.shift_next() {
                        let next_item_id = next_item.item_id.clone();
                        match state.handle_play(
                            next_item.item_id,
                            next_item.url,
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
                                log::error!("QueueNext failed: {error}");
                                let _ = app.emit(
                                    "playback-error",
                                    PlaybackErrorEvent {
                                        item_id: next_item_id,
                                        error: error.clone(),
                                    },
                                );
                            }
                        }
                    } else {
                        state.queue.clear_current();
                        let _ = app.emit("playback-stopped", ());
                    }
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueuePrev => {
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
                    if let Some(prev_item) = state.queue.shift_prev() {
                        let prev_item_id = prev_item.item_id.clone();
                        match state.handle_play(
                            prev_item.item_id,
                            prev_item.url,
                            0.0,
                            prev_item.duration_seconds,
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
                                log::error!("QueuePrev failed: {error}");
                                let _ = app.emit(
                                    "playback-error",
                                    PlaybackErrorEvent {
                                        item_id: prev_item_id,
                                        error: error.clone(),
                                    },
                                );
                            }
                        }
                    } else {
                        let _ = app.emit("playback-stopped", ());
                    }
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
                    let _ = reply.send(state.engine.list_output_devices());
                }
                AudioCommand::GetSelectedOutputDevice { reply } => {
                    let _ = reply
                        .send(state.engine.selected_output_device_id().map(|s| s.to_string()));
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
                        || (last.duration_seconds as i64) != (snapshot.duration_seconds as i64)
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

            if let Some(actual) = state.try_probe_complete_file_duration() {
                if actual > 0.0 && actual != state.duration_seconds {
                    log::info!(
                        "Detected duration from complete file: {}s (previous was: {}s)",
                        actual,
                        state.duration_seconds
                    );
                    state.duration_seconds = actual;

                    if let Some(ref mut current) = state.queue.current {
                        current.duration_seconds = actual;
                    }

                    if let Some(ref item_id) = state.current_item_id {
                        let db_state = state.app.state::<db::DatabaseState>();
                        let db_path = db_state.db_path();
                        let duration_i64 = actual.floor() as i64;
                        if let Err(error) = db::update_item_duration(&db_path, item_id, duration_i64) {
                            log::warn!("Failed to persist detected duration: {error}");
                        }
                    }

                    if let Some(snapshot) = state.snapshot() {
                        last_emitted_state = Some(snapshot.clone());
                        let _ = app.emit("playback-state-changed", &snapshot);
                    }
                }
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

            // Auto-advance: fire when the stream was playing but has now ended naturally.
            if was_playing && !snapshot.is_playing && state.engine.is_finished() {
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
                    let next_item_id = next_item.item_id.clone();
                    match state.handle_play(
                        next_item.item_id,
                        next_item.url,
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
                            let _ = app.emit(
                                "playback-error",
                                PlaybackErrorEvent {
                                    item_id: next_item_id,
                                    error: error.clone(),
                                },
                            );
                        }
                    }
                } else {
                    state.queue.clear_current();
                    let _ = app.emit("playback-stopped", ());
                }

                state.persist_session();
                let _ = app.emit("queue-changed", state.queue.to_event());
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
