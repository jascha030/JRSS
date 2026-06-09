//! Event payloads emitted to the frontend.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStateEvent {
    pub item_id: String,
    pub title: String,
    pub artist: String,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub file_duration_seconds: Option<f64>,
    pub is_playing: bool,
    pub is_buffering: bool,
    pub is_fully_downloaded: bool,
    pub volume: f64,
    pub speed: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackEndedEvent {
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackErrorEvent {
    pub item_id: String,
    pub error: String,
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
