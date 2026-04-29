//! Audio commands — sent from any thread to the audio thread.

use std::sync::mpsc;

use crate::queue::{QueueState, QueuedItem};

use super::events::{OutputDeviceInfo, PlaybackStateEvent};

pub enum AudioCommand {
    Play {
        item_id: String,
        url: String,
        start_position_seconds: f64,
        duration_hint_seconds: f64,
    },
    PlayWithQueue {
        item: QueuedItem,
        manual_queue: Vec<QueuedItem>,
        auto_queue: Vec<QueuedItem>,
        start_position_seconds: f64,
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
    QueueClearHistory,
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
    #[allow(dead_code)]
    Prefetch {
        item_id: String,
        url: String,
    },
}
