#![cfg_attr(target_os = "macos", allow(dead_code))]

//! Rodio-backed audio playback engine.
//!
//! [`RodioEngine`] wraps rodio's [`MixerDeviceSink`] and [`Player`] to
//! implement [`PlaybackEngine`] for all symphonia-supported audio formats
//! (MP3, AAC, WAV, FLAC).
//!
//! The output sink is kept open between tracks to avoid the overhead of
//! re-opening the CPAL device on every play. Only [`set_output_device`] and
//! [`initialize`] (re)create the sink.
//!
//! [`set_output_device`]: PlaybackEngine::set_output_device
//! [`initialize`]: PlaybackEngine::initialize

use std::time::Duration;

use rodio::{Decoder, MixerDeviceSink, Player, Source};

use super::devices::{list_output_devices, open_output_sink};
use super::engine::{EngineError, PlayConfig, PlaybackEngine};
use super::events::OutputDeviceInfo;
use super::streaming_file::StreamingFile;

/// Rodio-based audio engine.
///
/// Owns a CPAL output device sink and the active rodio [`Player`]. The sink
/// persists across track changes; the player is recreated for each stream.
pub struct RodioEngine {
    /// Output device sink — kept open between tracks so the device is not
    /// repeatedly torn down and re-opened.
    sink: Option<MixerDeviceSink>,
    /// Active player for the current track, if any.
    player: Option<Player>,
    /// User-selected output device ID. `None` means the system default.
    selected_device_id: Option<String>,
}

impl RodioEngine {
    pub fn new() -> Self {
        Self {
            sink: None,
            player: None,
            selected_device_id: None,
        }
    }

    /// Ensure the output sink is open, creating it if necessary.
    fn ensure_sink(&mut self) -> Result<(), EngineError> {
        if self.sink.is_none() {
            self.rebuild_sink()?;
        }
        Ok(())
    }

    /// (Re)create the output sink for the currently selected device.
    fn rebuild_sink(&mut self) -> Result<(), EngineError> {
        let handle = open_output_sink(self.selected_device_id.as_deref()).map_err(|e| {
            // If a specific device ID was requested and couldn't be opened it's a
            // device-not-found error; otherwise it's a system/internal error.
            match self.selected_device_id {
                Some(_) => EngineError::DeviceNotFound(e),
                None => EngineError::Internal(e),
            }
        })?;
        self.sink = Some(handle);
        Ok(())
    }

    /// Stop and drop the active player, keeping the sink open.
    fn stop_player(&mut self) {
        if let Some(player) = self.player.take() {
            player.stop();
        }
    }
}

impl PlaybackEngine for RodioEngine {
    fn initialize(&mut self) {
        if let Err(e) = self.rebuild_sink() {
            log::error!("RodioEngine: failed to open initial audio output: {e}");
        }
    }

    fn play_stream(
        &mut self,
        stream: StreamingFile,
        config: PlayConfig,
    ) -> Result<Option<f64>, EngineError> {
        self.stop_player();
        self.ensure_sink()?;

        // Build decoder, supplying byte_len for accurate VBR MP3 duration.
        let decoder = match config.byte_len_hint.filter(|&n| n > 0) {
            Some(byte_len) => Decoder::builder()
                .with_data(stream)
                .with_byte_len(byte_len)
                .build(),
            None => Decoder::new(stream),
        }
        .map_err(|e| EngineError::DecodeFailed(format!("Failed to decode audio: {e}")))?;

        // Prefer actual container duration over the RSS/hint value.
        let actual_duration = decoder.total_duration().map(|d| d.as_secs_f64());

        let sink = self
            .sink
            .as_ref()
            .ok_or_else(|| EngineError::Internal("No output sink available".to_string()))?;

        let player = Player::connect_new(sink.mixer());
        player.set_volume(config.volume);
        player.set_speed(config.speed);
        player.append(decoder);

        if config.start_position_seconds > 0.0 {
            let target = Duration::from_secs_f64(config.start_position_seconds);
            if player.try_seek(target).is_err() {
                return Err(EngineError::DecodeFailed(format!(
                    "Failed to seek to {:.1}s — format may not support seeking",
                    config.start_position_seconds
                )));
            }
        }

        self.player = Some(player);
        Ok(actual_duration)
    }

    fn stop(&mut self) {
        self.stop_player();
    }

    fn pause(&mut self) {
        if let Some(ref player) = self.player {
            player.pause();
        }
    }

    fn resume(&mut self) {
        if let Some(ref player) = self.player {
            player.play();
        }
    }

    fn seek(&mut self, position_seconds: f64) -> bool {
        match self.player.as_ref() {
            Some(player) => {
                let target = Duration::from_secs_f64(position_seconds);
                player.try_seek(target).is_ok()
            }
            None => false,
        }
    }

    fn set_volume(&mut self, volume: f32) {
        if let Some(ref player) = self.player {
            player.set_volume(volume);
        }
    }

    fn set_speed(&mut self, speed: f32) {
        if let Some(ref player) = self.player {
            player.set_speed(speed);
        }
    }

    fn position_seconds(&self) -> f64 {
        self.player
            .as_ref()
            .map(|p| p.get_pos().as_secs_f64())
            .unwrap_or(0.0)
    }

    fn has_active_playback(&self) -> bool {
        self.player.is_some()
    }

    fn is_paused(&self) -> bool {
        self.player.as_ref().map(|p| p.is_paused()).unwrap_or(false)
    }

    fn is_finished(&self) -> bool {
        self.player.as_ref().map(|p| p.empty()).unwrap_or(false)
    }

    fn list_output_devices(&self) -> Vec<OutputDeviceInfo> {
        list_output_devices()
    }

    fn selected_output_device_id(&self) -> Option<&str> {
        self.selected_device_id.as_deref()
    }

    fn set_output_device(&mut self, device_id: Option<String>) -> Result<(), EngineError> {
        self.stop_player();
        self.sink = None;
        self.selected_device_id = device_id;
        self.rebuild_sink()
    }
}
