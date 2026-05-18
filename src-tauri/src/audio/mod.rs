//! Backend media playback module — owns the playback lifecycle and survives UI
//! destruction.
//!
//! ## Architecture
//!
//! A dedicated audio thread ([`actor`]) owns all playback and queue state and
//! communicates with the rest of the app via [`AudioCommand`] messages.
//!
//! The thread delegates all media rendering to a [`PlaybackEngine`]
//! ([`engine`] trait, [`rodio_engine`] concrete implementation). Swapping the
//! engine is the only change needed to support a new media type (e.g. video).
//!
//! Audio data is streamed from an HTTP URL into a cache file on disk.
//! A [`StreamingFile`] wrapper presents this growing file as a blocking
//! `Read + Seek` source suitable for the decoder.

use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::queue::{QueueState, QueuedItem};

pub mod actor;
#[cfg(target_os = "macos")]
pub mod av_main_thread_actor;
#[cfg(target_os = "macos")]
pub mod av_proxy_engine;
pub mod cache;
pub mod commands;
pub mod devices;
pub mod download;
pub mod engine;
pub mod engine_factory;
pub mod events;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod rodio_engine;
pub mod streaming_file;

use commands::AudioCommand;
pub use events::{OutputDeviceInfo, PlaybackStateEvent};

/// Tauri-managed handle to the audio thread (just a sender).
pub struct AudioState {
    tx: mpsc::Sender<AudioCommand>,
}

impl AudioState {
    pub fn new(app: AppHandle) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel::<AudioCommand>();

        std::thread::Builder::new()
            .name("jrss-audio".into())
            .spawn(move || actor::audio_thread_main(rx, app))
            .map_err(|e| format!("Failed to spawn audio thread: {e}"))?;

        Ok(Self { tx })
    }

    pub fn send(&self, cmd: AudioCommand) -> Result<(), String> {
        self.tx
            .send(cmd)
            .map_err(|_| "Audio thread is not running".to_string())
    }

    pub fn sender(&self) -> mpsc::Sender<AudioCommand> {
        self.tx.clone()
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

    state.send(AudioCommand::Play {
        item_id,
        url,
        start_position_seconds,
        duration_hint_seconds,
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
    // Longer timeout (15s) to accommodate latency when multiple windows are
    // open and the audio thread is busy emitting events to all of them.
    match reply_rx.recv_timeout(Duration::from_secs(15)) {
        Ok(state) => state,
        Err(error) => {
            log::warn!("Timed out waiting for audio playback state: {error}");
            None
        }
    }
}

pub fn play_with_queue(
    app: &AppHandle,
    item: QueuedItem,
    manual_queue: Vec<QueuedItem>,
    auto_queue: Vec<QueuedItem>,
    start_position_seconds: f64,
) -> Result<(), String> {
    let state = app.state::<AudioState>();

    state.send(AudioCommand::PlayWithQueue {
        item,
        manual_queue,
        auto_queue,
        start_position_seconds,
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

pub fn queue_next(app: &AppHandle) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::QueueNext)
}

pub fn queue_prev(app: &AppHandle) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::QueuePrev)
}

pub fn queue_clear(app: &AppHandle) -> Result<(), String> {
    app.state::<AudioState>().send(AudioCommand::QueueClear)
}

pub fn queue_clear_history(app: &AppHandle) -> Result<(), String> {
    app.state::<AudioState>()
        .send(AudioCommand::QueueClearHistory)
}

pub fn get_queue_state(app: &AppHandle) -> QueueState {
    let state = app.state::<AudioState>();
    let (reply_tx, reply_rx) = mpsc::channel();
    let _ = state.send(AudioCommand::QueueGetState { reply: reply_tx });
    // Longer timeout (15s) to accommodate latency when multiple windows are
    // open and the audio thread is busy emitting events to all of them.
    match reply_rx.recv_timeout(Duration::from_secs(15)) {
        Ok(queue) => queue,
        Err(error) => {
            log::warn!("Timed out waiting for audio queue state: {error}");
            QueueState::default()
        }
    }
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

#[allow(dead_code)]
pub fn prefetch_item(app: &AppHandle, item_id: String, url: String) -> Result<(), String> {
    app.state::<AudioState>()
        .send(AudioCommand::Prefetch { item_id, url })
}
