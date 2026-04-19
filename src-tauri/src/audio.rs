//! Backend audio engine — owns playback lifecycle, survives UI destruction.
//!
//! ## Architecture
//!
//! A dedicated audio thread owns the rodio device sink handle, player, device
//! selection state, and all playback/queue state. The rest of the app
//! communicates with it via [`AudioCommand`] messages.
//!
//! Audio data is streamed from an HTTP URL into a temporary file on disk.
//! A [`StreamingFile`] wrapper presents this growing file as a blocking
//! `Read + Seek` source suitable for rodio's `Decoder`.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use ::rodio::cpal;
use ::rodio::cpal::traits::{DeviceTrait, HostTrait};
use ::rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::db;
use crate::models::PlaybackSessionRecord;
use crate::queue::{QueueState, QueuedItem};

// ---------------------------------------------------------------------------
// Event payloads — emitted to the frontend
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStateEvent {
    pub item_id: String,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub is_playing: bool,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackEndedEvent {
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub channels: Option<u16>,
    pub sample_rate: Option<u32>,
}

// ---------------------------------------------------------------------------
// StreamingFile — a growing file that blocks reads at the download edge
// ---------------------------------------------------------------------------

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

struct StreamingFile {
    file: File,
    cursor: u64,
    meta: Arc<DownloadMeta>,
}

impl StreamingFile {
    fn open(path: &Path, meta: Arc<DownloadMeta>) -> io::Result<Self> {
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
    PlayWithQueue {
        item: QueuedItem,
        manual_queue: Vec<QueuedItem>,
        auto_queue: Vec<QueuedItem>,
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
    GetState {
        reply: mpsc::Sender<Option<PlaybackStateEvent>>,
    },
    QueueEnqueue {
        item: QueuedItem,
    },
    QueuePlayNext {
        item: QueuedItem,
    },
    QueueRemove {
        item_id: String,
    },
    QueueMoveUp {
        item_id: String,
    },
    QueueMoveDown {
        item_id: String,
    },
    QueueClear,
    QueueGetState {
        reply: mpsc::Sender<QueueState>,
    },
    QueueSet {
        items: Vec<QueuedItem>,
    },
    ListOutputDevices {
        reply: mpsc::Sender<Vec<OutputDeviceInfo>>,
    },
    GetSelectedOutputDevice {
        reply: mpsc::Sender<Option<String>>,
    },
    SetOutputDevice {
        device_id: Option<String>,
        reply: mpsc::Sender<Result<(), String>>,
    },
}

// ---------------------------------------------------------------------------
// AudioState — Tauri-managed handle (just a sender)
// ---------------------------------------------------------------------------

pub struct AudioState {
    tx: mpsc::Sender<AudioCommand>,
}

impl AudioState {
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
    manual_queue: Vec<QueuedItem>,
    auto_queue: Vec<QueuedItem>,
    start_position_seconds: f64,
) -> Result<(), String> {
    let state = app.state::<AudioState>();
    let temp_dir = app
        .path()
        .temp_dir()
        .map_err(|e| format!("No temp dir: {e}"))?;

    state.send(AudioCommand::PlayWithQueue {
        item,
        manual_queue,
        auto_queue,
        start_position_seconds,
        temp_dir,
    })
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

pub fn list_output_devices(app: &AppHandle) -> Vec<OutputDeviceInfo> {
    let state = app.state::<AudioState>();
    let (reply_tx, reply_rx) = mpsc::channel();
    let _ = state.send(AudioCommand::ListOutputDevices { reply: reply_tx });
    reply_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap_or_default()
}

pub fn get_selected_output_device(app: &AppHandle) -> Option<String> {
    let state = app.state::<AudioState>();
    let (reply_tx, reply_rx) = mpsc::channel();
    let _ = state.send(AudioCommand::GetSelectedOutputDevice { reply: reply_tx });
    reply_rx.recv_timeout(Duration::from_secs(2)).ok().flatten()
}

pub fn set_output_device(app: &AppHandle, device_id: Option<String>) -> Result<(), String> {
    let state = app.state::<AudioState>();
    let (reply_tx, reply_rx) = mpsc::channel();

    state.send(AudioCommand::SetOutputDevice {
        device_id,
        reply: reply_tx,
    })?;

    reply_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "Timed out waiting for audio device change".to_string())?
}

// ---------------------------------------------------------------------------
// Audio thread — owns MixerDeviceSink, Player, and all playback state
// ---------------------------------------------------------------------------

struct AudioThread {
    sink_handle: Option<MixerDeviceSink>,
    player: Option<Player>,
    selected_output_device_id: Option<String>,
    current_item_id: Option<String>,
    stored_position_seconds: f64,
    duration_seconds: f64,
    download_meta: Option<Arc<DownloadMeta>>,
    temp_path: Option<PathBuf>,
    volume: f32,
    speed: f32,
    queue: QueueState,
    app: AppHandle,
}

impl AudioThread {
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

    fn handle_play(
        &mut self,
        item_id: String,
        url: String,
        start_position_seconds: f64,
        duration_hint_seconds: f64,
        temp_dir: PathBuf,
    ) -> Result<(), String> {
        self.teardown_output_only();
        self.ensure_output_sink()?;

        self.current_item_id = Some(item_id.clone());
        self.stored_position_seconds = start_position_seconds.max(0.0);
        self.duration_seconds = duration_hint_seconds.max(0.0);

        let meta = DownloadMeta::new();
        let temp_path = temp_dir.join(format!("jrss-audio-{}.tmp", uuid::Uuid::new_v4()));

        File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file for audio stream: {e}"))?;

        let dl_meta = Arc::clone(&meta);
        let dl_path = temp_path.clone();
        let dl_url = url.clone();
        std::thread::Builder::new()
            .name("jrss-download".into())
            .spawn(move || {
                if let Err(e) = download_to_file(&dl_url, &dl_path, &dl_meta) {
                    log::error!("Audio download failed: {}", e);
                    dl_meta.complete.store(true, Ordering::Release);
                }
            })
            .map_err(|e| {
                cleanup_failed_playback_start(&meta, &temp_path);
                format!("Failed to spawn audio download thread: {e}")
            })?;

        if let Err(e) = wait_for_minimum_data(&meta, 64 * 1024, Duration::from_secs(10)) {
            cleanup_failed_playback_start(&meta, &temp_path);
            return Err(format!("Audio startup failed: {e}"));
        }

        let streaming = match StreamingFile::open(&temp_path, Arc::clone(&meta)) {
            Ok(s) => s,
            Err(e) => {
                cleanup_failed_playback_start(&meta, &temp_path);
                return Err(format!("Failed to open streaming file: {e}"));
            }
        };

        let decoder = match Decoder::new(streaming) {
            Ok(d) => d,
            Err(e) => {
                cleanup_failed_playback_start(&meta, &temp_path);
                return Err(format!("Failed to decode audio: {e}"));
            }
        };

        let handle = self
            .sink_handle
            .as_ref()
            .ok_or_else(|| "No output sink available".to_string())?;

        let player = Player::connect_new(handle.mixer());
        player.set_volume(self.volume);
        player.set_speed(self.speed);
        player.append(decoder);

        if start_position_seconds > 0.0 {
            let _ = player.try_seek(Duration::from_secs_f64(start_position_seconds));
        }

        self.player = Some(player);
        self.download_meta = Some(meta);
        self.temp_path = Some(temp_path);

        Ok(())
    }

    fn teardown_output_only(&mut self) {
        if let Some(ref meta) = self.download_meta {
            meta.cancelled.store(true, Ordering::Release);
        }
        if let Some(player) = self.player.take() {
            player.stop();
        }
        if let Some(ref path) = self.temp_path {
            let _ = std::fs::remove_file(path);
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

    fn snapshot(&self) -> Option<PlaybackStateEvent> {
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
        })
    }

    fn sync_cached_position(&mut self) {
        if let Some(ref player) = self.player {
            self.stored_position_seconds = player.get_pos().as_secs_f64();
        }
    }

    fn persist_session(&mut self) {
        self.sync_cached_position();

        let db_state = self.app.state::<db::DatabaseState>();
        let db_path = db_state.db_path();

        if self.queue.current.is_none() && self.queue.is_empty() {
            if let Err(error) = db::clear_playback_session(&db_path) {
                log::error!("Failed to clear playback session: {error}");
            }
            return;
        }

        let (manual_queue, auto_queue) = self.queue.to_session_parts();
        let session = PlaybackSessionRecord {
            current_item_id: self.queue.current_item().map(|item| item.item_id.clone()),
            position_seconds: self.stored_position_seconds.floor() as i64,
            duration_seconds: self.duration_seconds.floor() as i64,
            manual_queue,
            auto_queue,
        };

        if let Err(error) = db::save_playback_session(&db_path, &session) {
            log::error!("Failed to save playback session: {error}");
        }
    }

    fn change_output_device(&mut self, device_id: Option<String>) -> Result<(), String> {
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
}

impl Drop for AudioThread {
    fn drop(&mut self) {
        self.stop_current();
    }
}

fn audio_thread_main(rx: mpsc::Receiver<AudioCommand>, app: AppHandle) {
    let mut state = AudioThread {
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
        app: app.clone(),
    };

    restore_persisted_session(&mut state);

    if let Err(error) = state.rebuild_output_sink() {
        log::error!("Failed to open initial audio output: {error}");
    }

    let poll_interval = Duration::from_millis(500);
    let persist_interval = Duration::from_secs(5);
    let mut last_persist = Instant::now();
    let mut was_playing = false;

    loop {
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
                        temp_dir,
                    ) {
                        log::error!("Play failed: {error}");
                    }

                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::Pause => {
                    if let Some(ref player) = state.player {
                        player.pause();
                    }
                    state.persist_session();
                }
                AudioCommand::Resume => {
                    if let Some(ref player) = state.player {
                        player.play();
                    } else if let Err(error) = resume_current_item(&mut state) {
                        log::info!("Resume ignored: {error}");
                    }
                    state.persist_session();
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
                    let _ = app.emit("playback-stopped", ());
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::Seek { position_seconds } => {
                    if let Some(ref player) = state.player {
                        let _ = player.try_seek(Duration::from_secs_f64(position_seconds.max(0.0)));
                    } else if state.current_item_id.is_some() {
                        state.stored_position_seconds = position_seconds.max(0.0);
                    }
                    state.persist_session();
                }
                AudioCommand::SetVolume { volume } => {
                    state.volume = volume;
                    if let Some(ref player) = state.player {
                        player.set_volume(volume);
                    }
                }
                AudioCommand::SetSpeed { speed } => {
                    state.speed = speed;
                    if let Some(ref player) = state.player {
                        player.set_speed(speed);
                    }
                }
                AudioCommand::GetState { reply } => {
                    let _ = reply.send(state.snapshot());
                }
                AudioCommand::PlayWithQueue {
                    item,
                    manual_queue,
                    auto_queue,
                    start_position_seconds,
                    temp_dir,
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
                            temp_dir,
                        ) {
                            log::error!("PlayWithQueue failed: {error}");
                        }
                    }

                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueEnqueue { item } => {
                    state.queue.enqueue(item);
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueuePlayNext { item } => {
                    state.queue.play_next(item);
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueRemove { item_id } => {
                    state.queue.remove(&item_id);
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueMoveUp { item_id } => {
                    state.queue.move_up(&item_id);
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueMoveDown { item_id } => {
                    state.queue.move_down(&item_id);
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::QueueClear => {
                    state.queue.clear();
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
                    state.persist_session();
                    let _ = app.emit("queue-changed", state.queue.to_event());
                }
                AudioCommand::ListOutputDevices { reply } => {
                    let _ = reply.send(list_output_devices_internal());
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
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if let Some(snapshot) = state.snapshot() {
            let _ = app.emit("playback-state-changed", &snapshot);

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
                            match app.path().temp_dir() {
                                Ok(temp_dir) => {
                                    if let Err(error) = state.handle_play(
                                        next_item.item_id.clone(),
                                        next_item.url.clone(),
                                        0.0,
                                        next_item.duration_seconds,
                                        temp_dir,
                                    ) {
                                        log::error!("Auto-advance failed: {error}");
                                    }
                                }
                                Err(error) => {
                                    log::error!(
                                        "Failed to resolve temp dir for auto-advance: {error}"
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
                }
            }

            was_playing = snapshot.is_playing;
        }
    }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn list_output_devices_internal() -> Vec<OutputDeviceInfo> {
    let host = cpal::default_host();

    let default_name = host
        .default_output_device()
        .and_then(|device| device.name().ok());

    let Ok(devices) = host.output_devices() else {
        return Vec::new();
    };

    devices
        .filter_map(|device| {
            let name = device.name().ok()?;
            let default_config = device.default_output_config().ok();

            Some(OutputDeviceInfo {
                id: name.clone(),
                name: name.clone(),
                is_default: default_name.as_deref() == Some(name.as_str()),
                channels: default_config.as_ref().map(|cfg| cfg.channels()),
                sample_rate: default_config.as_ref().map(|cfg| cfg.sample_rate()),
            })
        })
        .collect()
}

fn find_output_device(device_id: Option<&str>) -> Result<cpal::Device, String> {
    let host = cpal::default_host();

    match device_id {
        Some(target) => {
            let mut devices = host
                .output_devices()
                .map_err(|e| format!("Failed to enumerate output devices: {e}"))?;

            devices
                .find(|device| device.name().ok().as_deref() == Some(target))
                .ok_or_else(|| format!("Output device not found: {target}"))
        }
        None => host
            .default_output_device()
            .ok_or_else(|| "No default output device".to_string()),
    }
}

fn open_output_sink(selected_device_id: Option<&str>) -> Result<MixerDeviceSink, String> {
    let device = find_output_device(selected_device_id)?;

    let default_config = device
        .default_output_config()
        .map_err(|e| format!("Failed to query default output config: {e}"))?;

    let channels = 2u16
        .try_into()
        .map_err(|_| "Invalid channel count for output sink".to_string())?;

    let sample_rate = default_config
        .sample_rate()
        .try_into()
        .map_err(|_| "Invalid sample rate for output sink".to_string())?;

    let builder = DeviceSinkBuilder::from_device(device)
        .map_err(|e| format!("Failed to create output sink builder: {e}"))?
        .with_channels(channels)
        .with_sample_rate(sample_rate);

    builder
        .open_sink_or_fallback()
        .map_err(|e| format!("Failed to open output sink: {e}"))
}

fn cleanup_failed_playback_start(meta: &DownloadMeta, temp_path: &Path) {
    meta.cancelled.store(true, Ordering::Release);
    let _ = std::fs::remove_file(temp_path);
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

fn load_queued_item(db_path: &Path, item_id: String) -> Option<QueuedItem> {
    db::get_item_by_id(db_path, &item_id)
        .ok()
        .flatten()
        .and_then(feed_item_to_queued_item)
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
        return;
    };

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

    state.queue.replace(current.clone(), manual, auto);
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

    if state.queue.current.is_none() && !state.queue.is_empty() {
        state.persist_session();
    }
}

fn resume_current_item(state: &mut AudioThread) -> Result<(), String> {
    let Some(current) = state.queue.current_item().cloned() else {
        return Err("No current item to resume".to_string());
    };

    let temp_dir = state
        .app
        .path()
        .temp_dir()
        .map_err(|error| format!("No temp dir: {error}"))?;

    state.handle_play(
        current.item_id,
        current.url,
        state.stored_position_seconds,
        if state.duration_seconds > 0.0 {
            state.duration_seconds
        } else {
            current.duration_seconds
        },
        temp_dir,
    )
}

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

fn download_to_file(url: &str, path: &Path, meta: &DownloadMeta) -> Result<(), String> {
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
