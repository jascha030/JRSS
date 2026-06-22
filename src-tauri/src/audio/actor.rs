//! Audio thread — orchestrates download, queue, session persistence, and Tauri
//! event emission. Delegates all media rendering to a [`PlaybackEngine`].

use std::path::{Path, PathBuf};
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
use super::engine::{PlayConfig, PlaybackEngine, PlaybackSnapshot};
use super::engine_factory;
use super::events::{PlaybackEndedEvent, PlaybackErrorEvent, PlaybackStateEvent};
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
    engine: Box<dyn PlaybackEngine + Send>,
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
    stalled_at_download_edge: bool,
    /// Set to true when the user explicitly pauses. Used to distinguish
    /// manual pause from an unexpected system pause (e.g. audio route change).
    manual_pause: bool,
    /// Number of consecutive poll ticks where the engine reports finished.
    /// Used to de-bounce false finished states (e.g. decoder buffer underrun).
    finished_consecutive_count: u32,
    /// Whether `duration_seconds` came from the actual decoder rather than
    /// an RSS hint. Prevents clamping resume positions to incorrect durations.
    duration_from_decoder: bool,
    /// Whether the current playback is using HTTP streaming (macOS AVPlayer)
    /// rather than the local cache file. When true, the engine handles its
    /// own buffering and seeking past the download edge.
    is_streaming: bool,
    /// Cached engine snapshot from the last poll iteration. Used to serve
    /// GetState commands without blocking on the audio thread (AVPlayer).
    cached_engine_snapshot: PlaybackSnapshot,
}

impl AudioThread {
    fn new(app: AppHandle) -> Self {
        Self {
            engine: engine_factory::create_engine(),
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
            stalled_at_download_edge: false,
            manual_pause: false,
            finished_consecutive_count: 0,
            duration_from_decoder: false,
            is_streaming: false,
            cached_engine_snapshot: PlaybackSnapshot::default(),
        }
    }

    fn handle_play(
        &mut self,
        item_id: String,
        url: String,
        start_position_seconds: f64,
        duration_hint_seconds: f64,
    ) -> Result<(), String> {
        self.manual_pause = false;
        self.stalled_at_download_edge = false;
        self.finished_consecutive_count = 0;

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

        let same_item = self.current_item_id.as_ref() == Some(&item_id);
        self.current_item_id = Some(item_id.clone());
        self.hydrate_metadata(&item_id);

        if same_item && self.duration_from_decoder && self.duration_seconds > 0.0 {
            // Preserve verified duration when resuming the same item
            // (prevents clamping to a potentially wrong RSS hint)
        } else {
            self.duration_seconds = duration_hint_seconds.max(0.0);
            self.duration_from_decoder = false;
        }
        self.duration_probe_done = false;

        let clamped_start_position = if self.duration_from_decoder {
            clamp_to_duration(start_position_seconds, self.duration_seconds)
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
                            &Vec::new(),
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
        let byte_len_hint = if is_cache_complete(&cache_path) {
            std::fs::metadata(&cache_path).map(|m| m.len()).ok()
        } else {
            None
        };

        let cache_complete = is_cache_complete(&cache_path);

        #[cfg(target_os = "macos")]
        let stream_url = if !cache_complete {
            self.is_streaming = true;
            Some(url.clone())
        } else {
            self.is_streaming = false;
            None
        };

        let config = PlayConfig {
            start_position_seconds: self.stored_position_seconds,
            duration_hint_seconds: self.duration_seconds,
            volume: self.volume,
            speed: self.speed,
            byte_len_hint,
            file_path: Some(cache_path.clone()),
            #[cfg(target_os = "macos")]
            stream_url,
            #[cfg(target_os = "macos")]
            title: self.current_item_title.clone(),
            #[cfg(target_os = "macos")]
            artist: self.current_feed_title.clone(),
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
                self.duration_from_decoder = true;

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
        self.stalled_at_download_edge = false;
        self.manual_pause = false;
        self.finished_consecutive_count = 0;
        self.duration_from_decoder = false;
        self.is_streaming = false;
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

    fn snapshot(&self) -> Option<PlaybackStateEvent> {
        self.snapshot_full().0
    }

    /// Build snapshot from cached engine state — non-blocking, safe for GetState
    /// commands when main thread may be busy (e.g., during window creation).
    fn snapshot_from_cache(&self) -> Option<PlaybackStateEvent> {
        let eng = &self.cached_engine_snapshot;
        let download_complete = self.current_download_is_complete();
        self.current_item_id.as_ref().map(|item_id| {
            let position_seconds = if eng.has_active {
                eng.position
            } else {
                self.stored_position_seconds
            };
            let is_playing = eng.has_active && !eng.is_paused && !eng.is_finished;
            let effective_duration = self
                .duration_seconds
                .max(position_seconds)
                .max(self.stored_position_seconds);
            PlaybackStateEvent {
                item_id: item_id.clone(),
                title: self.current_item_title.clone(),
                artist: self.current_feed_title.clone(),
                position_seconds,
                duration_seconds: effective_duration,
                file_duration_seconds: download_complete.then_some(self.duration_seconds).filter(|d| *d > 0.0),
                is_playing,
                is_buffering: self.stalled_at_download_edge,
                is_fully_downloaded: download_complete,
                volume: self.volume as f64,
                speed: self.speed as f64,
            }
        })
    }

    /// One engine round-trip that yields both the Tauri event and the raw
    /// engine fields needed by the poll loop. Avoids repeated IPC for engines
    /// whose state queries cross a dispatch boundary.
    fn snapshot_full(&self) -> (Option<PlaybackStateEvent>, PlaybackSnapshot) {
        let eng = if self.engine.has_active_playback() {
            self.engine.playback_snapshot()
        } else {
            PlaybackSnapshot::default()
        };
        let download_complete = self.current_download_is_complete();
        let event = self.current_item_id.as_ref().map(|item_id| {
            let position_seconds = if eng.has_active { eng.position } else { self.stored_position_seconds };
            let is_playing = eng.has_active && !eng.is_paused && !eng.is_finished;
            log::trace!(
                "snapshot: item_id={} has_active={} is_paused={} is_finished={} position={} is_playing={} manual_pause={}",
                item_id, eng.has_active, eng.is_paused, eng.is_finished, position_seconds, is_playing, self.manual_pause
            );
            let effective_duration = self
                .duration_seconds
                .max(position_seconds)
                .max(self.stored_position_seconds);
            PlaybackStateEvent {
                item_id: item_id.clone(),
                title: self.current_item_title.clone(),
                artist: self.current_feed_title.clone(),
                position_seconds,
                duration_seconds: effective_duration,
                file_duration_seconds: download_complete.then_some(self.duration_seconds).filter(|d| *d > 0.0),
                is_playing,
                is_buffering: self.stalled_at_download_edge,
                is_fully_downloaded: download_complete,
                volume: self.volume as f64,
                speed: self.speed as f64,
            }
        });
        (event, eng)
    }

    fn try_probe_complete_file_duration(&mut self) -> Option<f64> {
        if self.duration_probe_done {
            return None;
        }

        if !self.current_download_is_complete() {
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

    fn current_download_is_complete(&self) -> bool {
        self.temp_path
            .as_ref()
            .is_some_and(|path| is_cache_complete(path))
    }

    fn sync_cached_position(&mut self) {
        if self.engine.has_active_playback() {
            self.stored_position_seconds = self.engine.position_seconds();
        }
    }

    fn persist_session(&mut self) {
        self.sync_cached_position();

        let db_state = self.app.state::<db::DatabaseState>();
        let db_path = db_state.db_path();

        if self.queue.current.is_none() && self.queue.is_empty() {
            log::debug!("Clearing playback session (no current item and empty queue)");
            if let Err(error) = db::clear_playback_session(&db_path) {
                log::error!("Failed to clear playback session: {error}");
            }
            return;
        }

        // Update stored duration to effective duration if position has exceeded it
        // This handles dynamic ad injection where actual audio is longer than RSS metadata
        let effective_duration = self.duration_seconds.max(self.stored_position_seconds);
        if effective_duration > self.duration_seconds {
            log::debug!(
                "Updating duration from {}s to {}s based on playback position",
                self.duration_seconds,
                effective_duration
            );
            self.duration_seconds = effective_duration;
        }

        let (history_queue, manual_queue, auto_queue) = self.queue.to_session_parts();

        log::debug!(
            "Saving playback session: current={:?}, history={}, manual={}, auto={}",
            self.queue.current_item().map(|i| &i.item_id),
            history_queue.len(),
            manual_queue.len(),
            auto_queue.len()
        );

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

    fn change_output_device(&mut self, device_id: Option<String>) -> Result<(), String> {
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
        let protected_paths = self.temp_path.iter().cloned().collect::<Vec<_>>();
        let handle = std::thread::Builder::new()
            .name("jrss-prefetch".into())
            .spawn(move || {
                let cache_dir = cache_dir_for_prefetch.as_deref().unwrap_or(Path::new(""));
                if let Err(e) = download_to_file(
                    &dl_url,
                    &dl_path,
                    &dl_meta,
                    cache_dir,
                    &protected_paths,
                    cache_limit_bytes,
                ) {
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
    // Temporarily disabled for spatial audio crash debugging
    // #[cfg(target_os = "macos")]
    // super::macos::set_audio_thread_qos();

    let mut state = AudioThread::new(app.clone());
    restore_persisted_session(&mut state);

    state.engine.initialize();

    let poll_interval = Duration::from_millis(500);
    let persist_interval = Duration::from_secs(5);
    let mut last_persist = Instant::now();
    let mut was_playing = false;

    // Throttling for playback-state-changed events
    // Using longer intervals for AVPlayer on macOS to reduce dispatch-queue
    // contention during window creation (miniplayer freeze issue).
    let min_emit_interval_while_playing = Duration::from_millis(1000);
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
                    state.manual_pause = false;
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
                    log::trace!("AudioCommand::Pause received (media controls)");
                    // Do NOT set manual_pause here — this command comes from system
                    // media controls (keyboard, AirPods gestures, Control Center), not
                    // the in-app UI. macOS sends spurious Pause events during audio
                    // route changes (e.g. spatial audio reconfiguration). Setting
                    // manual_pause would prevent auto-recovery.
                    state.engine.pause();
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::Resume => {
                    log::trace!("AudioCommand::Resume received");
                    state.manual_pause = false;
                    if state.engine.has_active_playback() {
                        state.engine.resume();
                    } else if let Err(error) = resume_current_item(&mut state) {
                        log::info!("Resume ignored: {error}");
                    }
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::TogglePlayback => {
                    log::trace!("AudioCommand::TogglePlayback received");
                    if state.engine.has_active_playback() {
                        if state.engine.is_paused() {
                            state.manual_pause = false;
                            state.engine.resume();
                        } else {
                            state.manual_pause = true;
                            state.engine.pause();
                        }
                    } else {
                        state.manual_pause = false;
                        if let Err(error) = resume_current_item(&mut state) {
                            log::info!("TogglePlayback ignored: {error}");
                        }
                    }
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::Stop => {
                    log::trace!("AudioCommand::Stop received");
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

                    let mut clamped_position = clamp_to_duration(position_seconds, state.duration_seconds);

                    // For incomplete downloads, clamp seeks to what is safely available.
                    // Seeking past the downloaded edge causes the decoder to read garbage
                    // (invalid MPEG headers, junk scanning) because the byte-level seek
                    // lands in undownloaded territory or mid-frame.
                    // Skip this clamping when streaming — AVPlayer handles range requests.
                    if !state.is_streaming {
                        if let Some(ref meta) = state.download_meta {
                            let bytes_written = meta.bytes_written.load(Ordering::Acquire);
                            let total_size = meta.total_size.load(Ordering::Acquire);
                            let complete = meta.complete.load(Ordering::Acquire);
                            if !complete && total_size > 0 && state.duration_seconds > 0.0 {
                                let downloaded_ratio = bytes_written as f64 / total_size as f64;
                                let seek_ratio = clamped_position / state.duration_seconds;
                                // Keep a 5 % safety margin so we don't land exactly at the edge
                                let safe_ratio = downloaded_ratio * 0.95;
                                if seek_ratio > safe_ratio {
                                    let safe_position = safe_ratio * state.duration_seconds;
                                    log::warn!(
                                        "Seek to {:.1}s clamped to {:.1}s (only {:.1}% downloaded)",
                                        clamped_position,
                                        safe_position,
                                        downloaded_ratio * 100.0
                                    );
                                    clamped_position = safe_position.max(0.0);
                                }
                            }
                        }
                    }

                    if (clamped_position - position_seconds).abs() > 1.0 {
                        log::debug!(
                            "Seek position clamped: requested={}s, actual_duration={}s, clamped={}s",
                            position_seconds,
                            state.duration_seconds,
                            clamped_position
                        );
                    }

                    if state.engine.has_active_playback() {
                        if state.engine.seek(clamped_position) {
                            state.stored_position_seconds = clamped_position;
                        } else {
                            log::warn!(
                                "Seek failed in decoder (MP3 without seek table?), position unchanged"
                            );
                        }
                    }

                    log::debug!("Seek command took {:?}", seek_start.elapsed());
                    state.persist_session();
                    emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
                }
                AudioCommand::SkipForward { delta_seconds }
                | AudioCommand::SkipBackward { delta_seconds } => {
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

                    let mut clamped_position = clamp_to_duration(target, state.duration_seconds);

                    // Same download-edge protection as AudioCommand::Seek
                    // Skip when streaming — AVPlayer handles range requests.
                    if !state.is_streaming {
                        if let Some(ref meta) = state.download_meta {
                            let bytes_written = meta.bytes_written.load(Ordering::Acquire);
                            let total_size = meta.total_size.load(Ordering::Acquire);
                            let complete = meta.complete.load(Ordering::Acquire);
                            if !complete && total_size > 0 && state.duration_seconds > 0.0 {
                                let downloaded_ratio = bytes_written as f64 / total_size as f64;
                                let seek_ratio = clamped_position / state.duration_seconds;
                                let safe_ratio = downloaded_ratio * 0.95;
                                if seek_ratio > safe_ratio {
                                    let safe_position = safe_ratio * state.duration_seconds;
                                    log::warn!(
                                        "Skip seek to {:.1}s clamped to {:.1}s (only {:.1}% downloaded)",
                                        clamped_position,
                                        safe_position,
                                        downloaded_ratio * 100.0
                                    );
                                    clamped_position = safe_position.max(0.0);
                                }
                            }
                        }
                    }

                    if state.engine.has_active_playback() {
                        if state.engine.seek(clamped_position) {
                            state.stored_position_seconds = clamped_position;
                        } else {
                            log::warn!("Skip seek failed in decoder, position unchanged");
                        }
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
                    // Use cached snapshot to avoid blocking on main thread
                    // (critical for AVPlayer during window creation)
                    let event = state.snapshot_from_cache();
                    let _ = reply.send(event);
                }
                AudioCommand::PlayWithQueue {
                    item,
                    manual_queue,
                    auto_queue,
                    start_position_seconds,
                } => {
                    was_playing = false;
                    state.manual_pause = false;
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
                    save_position_and_stop(&mut state, &app);
                    if let Some(next_item) = state.queue.shift_next() {
                        try_handle_play(
                            &mut state,
                            &app,
                            next_item,
                            "QueueNext",
                            &mut last_emit,
                            &mut last_emitted_state,
                        );
                    } else {
                        state.queue.clear_current();
                        let _ = app.emit("playback-stopped", ());
                    }
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueuePrev => {
                    save_position_and_stop(&mut state, &app);
                    if let Some(prev_item) = state.queue.shift_prev() {
                        try_handle_play(
                            &mut state,
                            &app,
                            prev_item,
                            "QueuePrev",
                            &mut last_emit,
                            &mut last_emitted_state,
                        );
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
                    let _ = reply.send(
                        state
                            .engine
                            .selected_output_device_id()
                            .map(|s| s.to_string()),
                    );
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

            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        let (snapshot_opt, eng_snap) = state.snapshot_full();
        // Cache engine snapshot for non-blocking GetState responses
        state.cached_engine_snapshot = eng_snap.clone();

        // De-bounce engine finished state: a single tick of is_finished can be
        // a decoder buffer underrun rather than a true natural end.
        if eng_snap.is_finished {
            state.finished_consecutive_count = state.finished_consecutive_count.saturating_add(1);
        } else {
            state.finished_consecutive_count = 0;
        }

        if let Some(snapshot) = snapshot_opt {
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
                        || last.is_buffering != snapshot.is_buffering
                        || last.is_fully_downloaded != snapshot.is_fully_downloaded
                        || (last.position_seconds as i64) != (snapshot.position_seconds as i64)
                        || (last.duration_seconds as i64) != (snapshot.duration_seconds as i64)
                        || (last.file_duration_seconds.unwrap_or(0.0) as i64)
                            != (snapshot.file_duration_seconds.unwrap_or(0.0) as i64)
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

            let download_complete = state.current_download_is_complete();

            if let Some(actual) = state.try_probe_complete_file_duration() {
                if actual > 0.0 && actual != state.duration_seconds {
                    log::info!(
                        "Detected duration from complete file: {}s (previous was: {}s)",
                        actual,
                        state.duration_seconds
                    );
                    state.duration_seconds = actual;
                    state.duration_from_decoder = true;

                    if let Some(ref mut current) = state.queue.current {
                        current.duration_seconds = actual;
                    }

                    if let Some(ref item_id) = state.current_item_id {
                        let db_state = state.app.state::<db::DatabaseState>();
                        let db_path = db_state.db_path();
                        let duration_i64 = actual.floor() as i64;
                        if let Err(error) =
                            db::update_item_duration(&db_path, item_id, duration_i64)
                        {
                            log::warn!("Failed to persist detected duration: {error}");
                        }
                    }

                    if let Some(snapshot) = state.snapshot() {
                        last_emitted_state = Some(snapshot.clone());
                        let _ = app.emit("playback-state-changed", &snapshot);
                    }
                }
            }

            if state.stalled_at_download_edge && download_complete && !state.is_streaming {
                log::info!("Download completed after stall; resuming current item");
                match resume_current_item(&mut state) {
                    Ok(()) => {
                        emit_playback_snapshot(
                            &app,
                            &state,
                            &mut last_emit,
                            &mut last_emitted_state,
                        );
                    }
                    Err(error) => {
                        log::error!("Auto-resume after stall failed: {error}");
                    }
                }
                was_playing = state.snapshot().is_some_and(|snapshot| snapshot.is_playing);
                continue;
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

            // Auto-advance only after a true natural end. When streaming,
            // AVPlayer handles its own buffering so auto-advance does not
            // require the cache to be complete. Otherwise, require a fully
            // cached file to de-bounce decoder stalls at the download edge.
            let auto_advance_ready = if state.is_streaming {
                state.finished_consecutive_count >= 2
            } else {
                state.finished_consecutive_count >= 2 && download_complete
            };

            if was_playing && !snapshot.is_playing && auto_advance_ready {
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
                    try_handle_play(
                        &mut state,
                        &app,
                        next_item,
                        "Auto-advance",
                        &mut last_emit,
                        &mut last_emitted_state,
                    );
                } else {
                    state.queue.clear_current();
                    let _ = app.emit("playback-stopped", ());
                }

                state.persist_session();
                let _ = app.emit("queue-changed", state.queue.to_event());
            } else if was_playing
                && !snapshot.is_playing
                && state.finished_consecutive_count >= 1
                && !download_complete
                && !state.is_streaming
            {
                state.stalled_at_download_edge = true;
                state.stored_position_seconds = snapshot.position_seconds;
                state.engine.stop();
                log::info!(
                    "Playback reached the downloaded edge before cache completion; waiting for more data"
                );
                emit_playback_snapshot(&app, &state, &mut last_emit, &mut last_emitted_state);
            }

            // Log unexpected stops (route changes, errors) so they show up in
            // crash reports. Auto-recovery is disabled; AVPlayer resumes reliably
            // via manual play/pause and auto-resume fought AirPods route changes.
            if was_playing && !snapshot.is_playing && !state.manual_pause && !eng_snap.is_finished {
                log::debug!(
                    "Playback stopped unexpectedly (possibly route change); click play to resume."
                );
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
    let item = match crate::db::get_item_by_id(db_path, &item_id) {
        Ok(Some(item)) => item,
        Ok(None) => {
            log::debug!(
                "Item {} not found in database during session restore",
                item_id
            );
            return None;
        }
        Err(error) => {
            log::debug!("Failed to load item {} from database: {}", item_id, error);
            return None;
        }
    };

    match feed_item_to_queued_item(item) {
        Some(queued) => Some(queued),
        None => {
            log::debug!(
                "Item {} skipped during session restore: no media enclosure",
                item_id
            );
            None
        }
    }
}

fn restore_persisted_session(state: &mut AudioThread) {
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
        log::debug!("No playback session found to restore");
        return;
    };

    log::debug!(
        "Restoring session: current={:?}, history={}, manual={}, auto={}",
        session.current_item_id,
        session.history_queue.len(),
        session.manual_queue.len(),
        session.auto_queue.len()
    );

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

    log::debug!(
        "Session restored: current={:?}, history={}, manual={}, auto={}",
        current.as_ref().map(|i| &i.item_id),
        history.len(),
        manual.len(),
        auto.len()
    );

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

fn try_handle_play(
    state: &mut AudioThread,
    app: &AppHandle,
    item: QueuedItem,
    label: &str,
    last_emit: &mut Instant,
    last_emitted_state: &mut Option<PlaybackStateEvent>,
) {
    let item_id = item.item_id.clone();
    match state.handle_play(item.item_id, item.url, 0.0, item.duration_seconds) {
        Ok(()) => {
            emit_playback_snapshot(app, state, last_emit, last_emitted_state);
        }
        Err(error) => {
            log::error!("{label} failed: {error}");
            let _ = app.emit("playback-error", PlaybackErrorEvent { item_id, error });
        }
    }
}

fn clamp_to_duration(position: f64, duration: f64) -> f64 {
    if duration > 0.5 {
        position.clamp(0.0, duration - 0.5)
    } else {
        position.max(0.0)
    }
}

fn save_position_and_stop(state: &mut AudioThread, app: &AppHandle) {
    state.sync_cached_position();
    if let Some(ref item_id) = state.current_item_id {
        let db = app.state::<crate::db::DatabaseState>();
        let _ = crate::db::save_playback(&db.db_path(), item_id, state.stored_position_seconds as i64);
    }
    state.stop_current();
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


