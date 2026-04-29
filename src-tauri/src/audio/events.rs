//! Event payloads emitted to the frontend.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStateEvent {
    pub item_id: String,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub is_playing: bool,
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
pub struct OutputDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub channels: Option<u16>,
    pub sample_rate: Option<u32>,
}
