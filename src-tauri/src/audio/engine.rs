//! Playback engine trait — decouples the media-rendering layer from the
//! orchestration logic in [`super::actor`].
//!
//! [`PlaybackEngine`] is the only interface [`super::actor::AudioThread`] needs
//! for media playback. Concrete engines (e.g. [`super::rodio_engine::RodioEngine`])
//! own all engine-specific resources and implement this trait.
//!
//! # Adding a new engine
//!
//! 1. Create a new module (e.g. `video_engine.rs`) and implement `PlaybackEngine`.
//! 2. Construct it in `AudioThread::new` (or a separate thread) and box it as
//!    `Box<dyn PlaybackEngine>`.
//! 3. The orchestration layer requires no changes.

use super::events::OutputDeviceInfo;
use super::streaming_file::StreamingFile;

// ---------------------------------------------------------------------------
// PlayConfig
// ---------------------------------------------------------------------------

/// Configuration for a [`PlaybackEngine::play_stream`] call.
pub struct PlayConfig {
    /// Seek to this position immediately after the decoder is ready.
    pub start_position_seconds: f64,
    /// RSS / UI duration hint — used as the initial guess until the decoder
    /// provides a more accurate value via [`PlaybackEngine::play_stream`]'s
    /// return value. Engines that cannot determine duration from the container
    /// header may use this as a fallback.
    #[allow(dead_code)]
    pub duration_hint_seconds: f64,
    /// Playback volume in \[0.0, 1.0\].
    pub volume: f32,
    /// Playback speed multiplier (1.0 = normal speed).
    pub speed: f32,
    /// Known byte length of the media file, if available. Engines may use
    /// this to compute accurate durations for variable-bitrate formats (e.g.
    /// VBR MP3).
    #[allow(dead_code)]
    pub byte_len_hint: Option<u64>,
    /// Path to the local cache file. macOS-native engines that read via
    /// file URL (e.g. AVPlayer) use this instead of the [`StreamingFile`]
    /// stream handle.
    pub file_path: Option<std::path::PathBuf>,
    /// Episode title — used on macOS to populate `MPNowPlayingInfoCenter`
    /// from the main thread, preventing a NULL-client crash in `BTAudioHALPlugin`
    /// during Bluetooth spatial audio property changes.
    #[cfg(target_os = "macos")]
    pub title: String,
    /// Feed/show title, used as the artist field in `MPNowPlayingInfoCenter`.
    #[cfg(target_os = "macos")]
    pub artist: String,
}

// ---------------------------------------------------------------------------
// EngineError
// ---------------------------------------------------------------------------

/// Error type returned by engine operations.
#[derive(Debug)]
pub enum EngineError {
    /// The requested output device could not be found or opened.
    #[allow(dead_code)]
    DeviceNotFound(String),
    /// The media stream could not be decoded.
    #[allow(dead_code)]
    DecodeFailed(String),
    /// An internal engine error.
    Internal(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound(msg) | Self::DecodeFailed(msg) | Self::Internal(msg) => {
                f.write_str(msg)
            }
        }
    }
}

impl std::error::Error for EngineError {}

// ---------------------------------------------------------------------------
// PlaybackSnapshot
// ---------------------------------------------------------------------------

/// Collapsed engine state for callers that need multiple fields at once.
    /// Avoids repeated cross-thread round-trips on engines whose state queries
    /// require a dispatch boundary.
#[derive(Debug, Default, Clone)]
pub struct PlaybackSnapshot {
    pub position: f64,
    pub is_paused: bool,
    pub is_finished: bool,
    pub has_active: bool,
}

// ---------------------------------------------------------------------------
// PlaybackEngine
// ---------------------------------------------------------------------------

/// A media playback engine.
///
/// Implementors own all rendering resources (device handles, decoders, …) and
/// present a uniform interface to the orchestration layer.  The orchestration
/// layer ([`super::actor::AudioThread`]) stays engine-agnostic: it manages
/// download state, the queue, session persistence, and Tauri event emission.
///
/// # Object safety
///
/// The trait is intentionally object-safe so it can be stored as
/// `Box<dyn PlaybackEngine>` when multiple concrete engine types must coexist
/// at runtime (e.g. audio + video).
pub trait PlaybackEngine {
    // ------------------------------------------------------------------
    // Lifecycle
    // ------------------------------------------------------------------

    /// Called once after construction, before any [`play_stream`](Self::play_stream).
    ///
    /// Engines may open their output device eagerly here to avoid first-play
    /// latency. The default implementation is a no-op.
    fn initialize(&mut self) {}

    /// Start playing `stream` according to `config`.
    ///
    /// Returns the actual media duration in seconds if the engine can
    /// determine it from the container header. Returning `None` leaves the
    /// caller's `duration_hint_seconds` in effect.
    ///
    /// Any previous playback session is implicitly stopped before the new
    /// one starts.
    fn play_stream(
        &mut self,
        stream: StreamingFile,
        config: PlayConfig,
    ) -> Result<Option<f64>, EngineError>;

    /// Stop playback and release stream resources.
    ///
    /// The output device (if applicable) remains open so it can be reused
    /// for the next [`play_stream`](Self::play_stream) call without
    /// re-initialisation overhead.
    fn stop(&mut self);

    // ------------------------------------------------------------------
    // Transport
    // ------------------------------------------------------------------

    /// Pause if currently playing. No-op if already paused or stopped.
    fn pause(&mut self);

    /// Resume if currently paused. No-op if already playing or stopped.
    fn resume(&mut self);

    /// Seek to `position_seconds`.
    ///
    /// Returns `true` if the seek succeeded, `false` if the format does not
    /// support seeking (e.g. MP3 without a seek table). Even on failure the
    /// caller may choose to update its stored position for UI purposes.
    fn seek(&mut self, position_seconds: f64) -> bool;

    // ------------------------------------------------------------------
    // Setters
    // ------------------------------------------------------------------

    /// Set volume in \[0.0, 1.0\]. No-op if stopped.
    fn set_volume(&mut self, volume: f32);

    /// Set playback speed multiplier (1.0 = normal). No-op if stopped.
    fn set_speed(&mut self, speed: f32);

    // ------------------------------------------------------------------
    // State queries
    // ------------------------------------------------------------------

    /// Current playback position in seconds. Returns `0.0` if stopped.
    fn position_seconds(&self) -> f64;

    /// Whether the engine has an active (possibly paused) playback session.
    fn has_active_playback(&self) -> bool;

    /// Whether playback is currently paused. Returns `false` if stopped.
    fn is_paused(&self) -> bool;

    /// Whether the media stream has been fully consumed (natural end of
    /// stream). Returns `false` when stopped or not yet started.
    fn is_finished(&self) -> bool;

    /// Collapsed state snapshot — one round-trip for engines whose state
    /// queries are expensive (e.g. cross-thread IPC).
    ///
    /// The default implementation calls each query individually. Override
    /// when batching is cheaper (e.g. a dispatch-queue-backed engine).
    fn playback_snapshot(&self) -> PlaybackSnapshot {
        PlaybackSnapshot {
            position: self.position_seconds(),
            is_paused: self.is_paused(),
            is_finished: self.is_finished(),
            has_active: self.has_active_playback(),
        }
    }

    // ------------------------------------------------------------------
    // Device management (optional — override for engines with selectable
    // output devices)
    // ------------------------------------------------------------------

    /// List available output devices.
    ///
    /// Returns an empty vec for engines that do not support output device
    /// selection (the default).
    fn list_output_devices(&self) -> Vec<OutputDeviceInfo> {
        Vec::new()
    }

    /// The ID of the currently selected output device, if any.
    ///
    /// Returns `None` for engines that do not support device selection (the
    /// default).
    fn selected_output_device_id(&self) -> Option<&str> {
        None
    }

    /// Switch to a different output device.
    ///
    /// The engine must tear down and recreate its internal output resources.
    /// Returns `Ok(())` for engines that do not support device selection (the
    /// default).
    fn set_output_device(&mut self, device_id: Option<String>) -> Result<(), EngineError> {
        let _ = device_id;
        Ok(())
    }
}
