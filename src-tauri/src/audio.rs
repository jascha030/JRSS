//! Backend audio engine — owns playback lifecycle, survives UI destruction.
//!
//! ## Architecture
//!
//! rodio's `OutputStream` is `!Send`, so it cannot live in Tauri managed state
//! directly. Instead, we spawn a dedicated **audio thread** at app startup that
//! owns the `OutputStream`, `Sink`, and all playback state. The rest of the app
//! communicates with it via a channel of [`AudioCommand`] messages.
//!
//! A position-polling loop on the same thread emits Tauri events back to the
//! frontend, and persists playback position to SQLite periodically.
//!
//! ## Streaming
//!
//! Audio data is streamed from an HTTP URL into a temporary file on disk.
//! A [`StreamingFile`] wrapper presents this growing file as a blocking
//! `Read + Seek` source suitable for rodio's `Decoder`. Reads at the download
//! frontier block until data arrives, enabling immediate playback while the
//! download continues in the background.
//!
//! ## Queue integration
//!
//! The queue lives in the audio thread so playback can continue without any
//! frontend window. When a track ends, the audio thread automatically advances
//! to the next queued item.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use rodio::{Decoder, OutputStream, Sink};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::queue::{QueueState, QueuedItem};

// ---------------------------------------------------------------------------
// Event payloads — emitted to the frontend
// ---------------------------------------------------------------------------

/// Playback state emitted to the frontend via Tauri events.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStateEvent {
    pub item_id: String,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub is_playing: bool,
    pub volume: f64,
}

/// Emitted once when a track finishes naturally.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackEndedEvent {
    pub item_id: String,
}

// ---------------------------------------------------------------------------
// StreamingFile — a growing file that blocks reads at the download edge
// ---------------------------------------------------------------------------

/// Shared metadata for the download-in-progress.
struct DownloadMeta {
    bytes_written: AtomicU64,
    complete: AtomicBool,
    cancelled: AtomicBool,
}

impl DownloadMeta {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            bytes_written: AtomicU64::new(0),
            complete: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
        })
    }
}

/// Read handle into a file being written to concurrently.
/// Blocks on read when the cursor is at the download frontier.
struct StreamingFile {
    file: File,
    cursor: u64,
    meta: Arc<DownloadMeta>,
}

impl StreamingFile {
    fn open(path: &std::path::Path, meta: Arc<DownloadMeta>) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            file,
            cursor: 0,
            meta,
        })
    }
}

impl Read for StreamingFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let available = self.meta.bytes_written.load(Ordering::Acquire);
            let is_complete = self.meta.complete.load(Ordering::Acquire);

            if self.cursor < available {
                let readable = (available - self.cursor) as usize;
                let to_read = buf.len().min(readable);
                self.file.seek(SeekFrom::Start(self.cursor))?;
                let n = self.file.read(&mut buf[..to_read])?;
                self.cursor += n as u64;
                return Ok(n);
            }

            if is_complete {
                return Ok(0);
            }

            // At the frontier — wait for more data.
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Seek for StreamingFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let available = self.meta.bytes_written.load(Ordering::Acquire);
        let new_pos = match pos {
            SeekFrom::Start(n) => n,
            SeekFrom::Current(n) => {
                if n >= 0 {
                    self.cursor.saturating_add(n as u64)
                } else {
                    self.cursor.saturating_sub(n.unsigned_abs())
                }
            }
            SeekFrom::End(n) => {
                if n >= 0 {
                    available.saturating_add(n as u64)
                } else {
                    available.saturating_sub(n.unsigned_abs())
                }
            }
        };
        self.cursor = new_pos;
        Ok(new_pos)
    }
}

// ---------------------------------------------------------------------------
// Audio commands — sent from any thread to the audio thread
// ---------------------------------------------------------------------------

enum AudioCommand {
    Play {
        item_id: String,
        url: String,
        start_position_seconds: f64,
        duration_hint_seconds: f64,
        temp_dir: PathBuf,
    },
    /// Play a specific item and set the entire queue (replaces current queue).
    PlayWithQueue {
        item: QueuedItem,
        queue: Vec<QueuedItem>,
        start_position_seconds: f64,
        temp_dir: PathBuf,
    },
    Pause,
    Resume,
    TogglePlayback,
    Stop,
    Seek {
        position_seconds: f64,
    },
    SetVolume {
        volume: f32,
    },
    SetSpeed {
        speed: f32,
    },
    /// Request current state snapshot — response sent via the oneshot.
    GetState {
        reply: mpsc::Sender<Option<PlaybackStateEvent>>,
    },
    /// Enqueue an item to the manual queue.
    QueueEnqueue {
        item: QueuedItem,
    },
    /// Insert an item at the front of the manual queue.
    QueuePlayNext {
        item: QueuedItem,
    },
    /// Remove an item from the queue.
    QueueRemove {
        item_id: String,
    },
    /// Move an item up in its segment.
    QueueMoveUp {
        item_id: String,
    },
    /// Move an item down in its segment.
    QueueMoveDown {
        item_id: String,
    },
    /// Clear the queue.
    QueueClear,
    /// Request current queue state.
    QueueGetState {
        reply: mpsc::Sender<QueueState>,
    },
    /// Replace entire queue (e.g., when loading a playlist).
    QueueSet {
        items: Vec<QueuedItem>,
    },
}

// ---------------------------------------------------------------------------
// AudioState — Tauri-managed handle (just a sender)
// ---------------------------------------------------------------------------

/// Tauri-managed state. Contains only a channel sender to the audio thread.
pub struct AudioState {
    tx: mpsc::Sender<AudioCommand>,
}

impl AudioState {
    /// Spawn the audio thread and return the handle.
    pub fn new(app: AppHandle) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel::<AudioCommand>();

        std::thread::Builder::new()
            .name("jrss-audio".into())
            .spawn(move || audio_thread_main(rx, app))
            .map_err(|e| format!("Failed to spawn audio thread: {e}"))?;

        Ok(Self { tx })
    }

    fn send(&self, cmd: AudioCommand) -> Result<(), String> {
        self.tx
            .send(cmd)
            .map_err(|_| "Audio thread is not running".to_string())
    }
}

// ---------------------------------------------------------------------------
// Public API — called from Tauri commands
// ---------------------------------------------------------------------------

pub fn play_url(
    app: &AppHandle,
    item_id: String,
    url: String,
    start_position_seconds: f64,
    duration_hint_seconds: f64,
) -> Result<(), String> {
    let state = app.state::<AudioState>();
    let temp_dir = app
        .path()
        .temp_dir()
        .map_err(|e| format!("No temp dir: {e}"))?;

    state.send(AudioCommand::Play {
        item_id,
        url,
        start_position_seconds,
        duration_hint_seconds,
        temp_dir,
    })
}

pub fn pause(app: &AppHandle) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::Pause)
}

pub fn resume(app: &AppHandle) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::Resume)
}

pub fn toggle_playback(app: &AppHandle) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::TogglePlayback)
}

pub fn stop(app: &AppHandle) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::Stop)
}

pub fn seek(app: &AppHandle, position_seconds: f64) -> Result<(), String> {
    app.state::<AudioState>()
        .send(AudioCommand::Seek { position_seconds })
}

pub fn set_volume(app: &AppHandle, volume: f64) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::SetVolume {
        volume: volume as f32,
    })
}

pub fn set_speed(app: &AppHandle, speed: f64) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::SetSpeed {
        speed: speed as f32,
    })
}

pub fn get_playback_state(app: &AppHandle) -> Option<PlaybackStateEvent> {
    let state = app.state::<AudioState>();
    let (reply_tx, reply_rx) = mpsc::channel();
    state
        .send(AudioCommand::GetState { reply: reply_tx })
        .ok()?;
    reply_rx.recv_timeout(Duration::from_secs(1)).ok()?
}

pub fn play_with_queue(
    app: &AppHandle,
    item: QueuedItem,
    queue: Vec<QueuedItem>,
    start_position_seconds: f64,
) -> Result<(), String> {
    log::info!("play_with_queue: sending command for item={}", item.item_id);
    let state = app.state::<AudioState>();
    let temp_dir = app
        .path()
        .temp_dir()
        .map_err(|e| format!("No temp dir: {e}"))?;

    let send_result = state.send(AudioCommand::PlayWithQueue {
        item,
        queue,
        start_position_seconds,
        temp_dir,
    });
    log::info!("play_with_queue: send result={:?}", send_result);
    send_result
}

pub fn queue_enqueue(app: &AppHandle, item: QueuedItem) -> Result<(), String> {
    app.state::<AudioState>()
        .send(AudioCommand::QueueEnqueue { item })
}

pub fn queue_play_next(app: &AppHandle, item: QueuedItem) -> Result<(), String> {
    app.state::<AudioState>()
        .send(AudioCommand::QueuePlayNext { item })
}

pub fn queue_remove(app: &AppHandle, item_id: String) -> Result<(), String> {
    app.state::<AudioState>()
        .send(AudioCommand::QueueRemove { item_id })
}

pub fn queue_move_up(app: &AppHandle, item_id: String) -> Result<(), String> {
    app.state::<AudioState>()
        .send(AudioCommand::QueueMoveUp { item_id })
}

pub fn queue_move_down(app: &AppHandle, item_id: String) -> Result<(), String> {
    app.state::<AudioState>()
        .send(AudioCommand::QueueMoveDown { item_id })
}

pub fn queue_clear(app: &AppHandle) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::QueueClear)
}

pub fn get_queue_state(app: &AppHandle) -> QueueState {
    let state = app.state::<AudioState>();
    let (reply_tx, reply_rx) = mpsc::channel();
    let _ = state.send(AudioCommand::QueueGetState { reply: reply_tx });
    reply_rx
        .recv_timeout(Duration::from_secs(1))
        .unwrap_or_default()
}

pub fn queue_set(app: &AppHandle, items: Vec<QueuedItem>) -> Result<(), String> {
    app.state::<AudioState>()
        .send(AudioCommand::QueueSet { items })
}

// ---------------------------------------------------------------------------
// Audio thread — owns OutputStream, Sink, and all playback state
// ---------------------------------------------------------------------------

struct AudioThread {
    _stream: OutputStream,
    sink: Option<Sink>,
    current_item_id: Option<String>,
    duration_seconds: f64,
    download_meta: Option<Arc<DownloadMeta>>,
    temp_path: Option<PathBuf>,
    volume: f32,
    speed: f32,
    queue: QueueState,
    app: AppHandle,
}

impl AudioThread {
    fn handle_play(
        &mut self,
        item_id: String,
        url: String,
        start_position_seconds: f64,
        duration_hint_seconds: f64,
        temp_dir: PathBuf,
        stream_handle: &rodio::OutputStreamHandle,
    ) {
        log::info!("handle_play: item_id={}, url={}", item_id, url);
        // Stop current playback.
        self.stop_current();

        let meta = DownloadMeta::new();
        let temp_path = temp_dir.join(format!("jrss-audio-{}.tmp", uuid::Uuid::new_v4()));

        // Create the temp file.
        if let Err(e) = File::create(&temp_path) {
            log::error!("Failed to create temp file: {}", e);
            return;
        }
        log::info!("handle_play: temp file created at {:?}", temp_path);

        // Spawn HTTP download thread.
        let dl_meta = Arc::clone(&meta);
        let dl_path = temp_path.clone();
        let dl_url = url.clone();
        std::thread::Builder::new()
            .name("jrss-download".into())
            .spawn(move || {
                log::info!("download thread: starting for {}", dl_url);
                if let Err(e) = download_to_file(&dl_url, &dl_path, &dl_meta) {
                    log::error!("Audio download failed: {}", e);
                    dl_meta.complete.store(true, Ordering::Release);
                } else {
                    log::info!("download thread: completed successfully");
                }
            })
            .ok();

        // Wait for minimum data to start decoding.
        log::info!("handle_play: waiting for minimum data (64KB)");
        if let Err(e) = wait_for_minimum_data(&meta, 64 * 1024, Duration::from_secs(10)) {
            log::error!("Audio startup failed: {}", e);
            meta.cancelled.store(true, Ordering::Release);
            let _ = std::fs::remove_file(&temp_path);
            return;
        }
        log::info!("handle_play: got enough data, creating decoder");

        // Open streaming reader and decode.
        let streaming = match StreamingFile::open(&temp_path, Arc::clone(&meta)) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to open streaming file: {}", e);
                meta.cancelled.store(true, Ordering::Release);
                let _ = std::fs::remove_file(&temp_path);
                return;
            }
        };

        let decoder = match Decoder::new(streaming) {
            Ok(d) => d,
            Err(e) => {
                log::error!("Failed to decode audio: {}", e);
                meta.cancelled.store(true, Ordering::Release);
                let _ = std::fs::remove_file(&temp_path);
                return;
            }
        };
        log::info!("handle_play: decoder created successfully");

        let sink = match Sink::try_new(stream_handle) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to create sink: {}", e);
                meta.cancelled.store(true, Ordering::Release);
                let _ = std::fs::remove_file(&temp_path);
                return;
            }
        };
        log::info!("handle_play: sink created, starting playback");

        sink.set_volume(self.volume);
        sink.set_speed(self.speed);
        sink.append(decoder);

        if start_position_seconds > 0.0 {
            let _ = sink.try_seek(Duration::from_secs_f64(start_position_seconds));
        }

        self.sink = Some(sink);
        self.current_item_id = Some(item_id);
        self.duration_seconds = duration_hint_seconds;
        self.download_meta = Some(meta);
        self.temp_path = Some(temp_path);
        log::info!("handle_play: playback started!");
    }

    fn stop_current(&mut self) {
        if let Some(ref meta) = self.download_meta {
            meta.cancelled.store(true, Ordering::Release);
        }
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        if let Some(ref path) = self.temp_path {
            let _ = std::fs::remove_file(path);
        }
        self.current_item_id = None;
        self.duration_seconds = 0.0;
        self.download_meta = None;
        self.temp_path = None;
    }

    fn snapshot(&self) -> Option<PlaybackStateEvent> {
        let sink = self.sink.as_ref()?;
        let item_id = self.current_item_id.as_ref()?;

        Some(PlaybackStateEvent {
            item_id: item_id.clone(),
            position_seconds: sink.get_pos().as_secs_f64(),
            duration_seconds: self.duration_seconds,
            is_playing: !sink.is_paused() && !sink.empty(),
            volume: self.volume as f64,
        })
    }
}

impl Drop for AudioThread {
    fn drop(&mut self) {
        self.stop_current();
    }
}

fn audio_thread_main(rx: mpsc::Receiver<AudioCommand>, app: AppHandle) {
    // Open the audio output on this thread (OutputStream is !Send).
    let (stream, stream_handle) = match OutputStream::try_default() {
        Ok(pair) => pair,
        Err(e) => {
            log::error!("Failed to open audio output: {e}");
            return;
        }
    };

    let mut state = AudioThread {
        _stream: stream,
        sink: None,
        current_item_id: None,
        duration_seconds: 0.0,
        download_meta: None,
        temp_path: None,
        volume: 1.0,
        speed: 1.0,
        queue: QueueState::default(),
        app: app.clone(),
    };

    let poll_interval = Duration::from_millis(500);
    let persist_interval = Duration::from_secs(5);
    let mut last_persist = Instant::now();
    let mut was_playing = false;

    loop {
        // Non-blocking receive with timeout for polling.
        match rx.recv_timeout(poll_interval) {
            Ok(cmd) => match cmd {
                AudioCommand::Play {
                    item_id,
                    url,
                    start_position_seconds,
                    duration_hint_seconds,
                    temp_dir,
                } => {
                    was_playing = false;
                    state.handle_play(
                        item_id,
                        url,
                        start_position_seconds,
                        duration_hint_seconds,
                        temp_dir,
                        &stream_handle,
                    );
                }
                AudioCommand::Pause => {
                    if let Some(ref sink) = state.sink {
                        sink.pause();
                    }
                }
                AudioCommand::Resume => {
                    if let Some(ref sink) = state.sink {
                        sink.play();
                    }
                }
                AudioCommand::TogglePlayback => {
                    log::info!(
                        "TogglePlayback: sink={:?}, is_paused={}",
                        state.sink.as_ref().map(|s| s.is_paused()),
                        state.sink.as_ref().map(|s| s.is_paused()).unwrap_or(false)
                    );
                    if let Some(ref sink) = state.sink {
                        if sink.is_paused() {
                            log::info!("TogglePlayback: playing");
                            sink.play();
                        } else {
                            log::info!("TogglePlayback: pausing");
                            sink.pause();
                        }
                    } else {
                        log::info!("TogglePlayback: no sink!");
                    }
                }
                AudioCommand::Stop => {
                    state.stop_current();
                    let _ = app.emit("playback-stopped", ());
                }
                AudioCommand::Seek { position_seconds } => {
                    if let Some(ref sink) = state.sink {
                        let _ = sink.try_seek(Duration::from_secs_f64(position_seconds));
                    }
                }
                AudioCommand::SetVolume { volume } => {
                    state.volume = volume;
                    if let Some(ref sink) = state.sink {
                        sink.set_volume(volume);
                    }
                }
                AudioCommand::SetSpeed { speed } => {
                    state.speed = speed;
                    if let Some(ref sink) = state.sink {
                        sink.set_speed(speed);
                    }
                }
                AudioCommand::GetState { reply } => {
                    let _ = reply.send(state.snapshot());
                }
                AudioCommand::PlayWithQueue {
                    item,
                    queue,
                    start_position_seconds,
                    temp_dir,
                } => {
                    log::info!(
                        "AudioThread received PlayWithQueue: item={}, queue_len={}",
                        item.item_id,
                        queue.len()
                    );
                    was_playing = false;
                    // Set up the queue: first item is the one to play now, rest go to manual.
                    let mut all_items = vec![item];
                    all_items.extend(queue);
                    state.queue.set_queue(all_items);
                    if let Some(current) = state.queue.current_item() {
                        log::info!("AudioThread calling handle_play for: {}", current.item_id);
                        state.handle_play(
                            current.item_id.clone(),
                            current.url.clone(),
                            start_position_seconds,
                            current.duration_seconds,
                            temp_dir,
                            &stream_handle,
                        );
                    }
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueEnqueue { item } => {
                    state.queue.enqueue(item);
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueuePlayNext { item } => {
                    state.queue.play_next(item);
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueRemove { item_id } => {
                    state.queue.remove(&item_id);
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueMoveUp { item_id } => {
                    state.queue.move_up(&item_id);
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueMoveDown { item_id } => {
                    state.queue.move_down(&item_id);
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueClear => {
                    state.queue.clear();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueGetState { reply } => {
                    let _ = reply.send(state.queue.clone());
                }
                AudioCommand::QueueSet { items } => {
                    state.queue.set_queue(items);
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Fall through to polling logic.
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // App is shutting down.
                break;
            }
        }

        // --- Polling: emit state + detect track end ---
        if let Some(snapshot) = state.snapshot() {
            let _ = app.emit("playback-state-changed", &snapshot);

            // Periodic position persist.
            if snapshot.is_playing && last_persist.elapsed() >= persist_interval {
                last_persist = Instant::now();
                let db = app.state::<crate::db::DatabaseState>();
                let _ = crate::db::save_playback(
                    &db.db_path(),
                    &snapshot.item_id,
                    snapshot.position_seconds as i64,
                );
            }

            // Detect track end.
            if was_playing && !snapshot.is_playing {
                if let Some(ref sink) = state.sink {
                    if sink.empty() {
                        let finished_item_id = snapshot.item_id.clone();
                        state.stop_current();
                        let _ = app.emit(
                            "playback-ended",
                            PlaybackEndedEvent {
                                item_id: finished_item_id.clone(),
                            },
                        );

                        // Auto-advance to next item in queue.
                        if let Some(next_item) = state.queue.shift_next() {
                            let temp_dir = app.path().temp_dir().unwrap_or_default();
                            state.handle_play(
                                next_item.item_id.clone(),
                                next_item.url.clone(),
                                0.0,
                                next_item.duration_seconds,
                                temp_dir,
                                &stream_handle,
                            );
                        }
                    }
                }
            }

            was_playing = snapshot.is_playing;
        }
    }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/// Block until at least `min_bytes` have been downloaded, or `timeout` elapses.
fn wait_for_minimum_data(
    meta: &DownloadMeta,
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

/// Download `url` into `path`, updating `meta` as bytes arrive.
fn download_to_file(url: &str, path: &std::path::Path, meta: &DownloadMeta) -> Result<(), String> {
    let response = reqwest::blocking::Client::new()
        .get(url)
        .send()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format!("Failed to open temp file for writing: {e}"))?;

    let mut reader = response;
    let mut buf = [0u8; 32 * 1024];

    loop {
        if meta.cancelled.load(Ordering::Acquire) {
            return Ok(());
        }

        match reader.read(&mut buf) {
            Ok(0) => {
                meta.complete.store(true, Ordering::Release);
                return Ok(());
            }
            Ok(n) => {
                file.write_all(&buf[..n])
                    .map_err(|e| format!("Write failed: {e}"))?;
                file.flush().map_err(|e| format!("Flush failed: {e}"))?;
                meta.bytes_written.fetch_add(n as u64, Ordering::Release);
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(format!("Read error: {e}")),
        }
    }
}
