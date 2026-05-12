//! Send-safe proxy for the main-thread AV actor.
//!
//! [`AvProxyEngine`] implements [`PlaybackEngine`] by forwarding each command
//! to [`AvMainThreadActor`] via a one-shot `dispatch_async_f` call on the main
//! queue.  No timer polling; no shared channels between commands.

use std::time::Duration;

use super::av_main_thread_actor::{dispatch_cmd, start_on_main_queue, ActorHandle, AvCmd, AvResp};
use super::engine::{EngineError, PlayConfig, PlaybackEngine, PlaybackSnapshot};
use super::streaming_file::StreamingFile;

const CMD_TIMEOUT: Duration = Duration::from_secs(5);

pub struct AvProxyEngine {
    handle: ActorHandle,
    cached_duration: f64,
    has_active: bool,
}

impl AvProxyEngine {
    pub fn new() -> Self {
        Self {
            handle: start_on_main_queue(),
            cached_duration: 0.0,
            has_active: false,
        }
    }

    fn send(&self, cmd: AvCmd) -> Result<AvResp, EngineError> {
        // dispatch_cmd blocks on an mpsc::SyncSender; apply a wall-clock guard
        // with a thread so we don't block the audio thread indefinitely.
        let handle = self.handle.clone();
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let resp = dispatch_cmd(&handle, cmd);
            let _ = tx.send(resp);
        });
        rx.recv_timeout(CMD_TIMEOUT)
            .map_err(|_| EngineError::Internal("AV command timed out".into()))?
            .ok_or_else(|| EngineError::Internal("AV actor closed".into()))
    }
}

impl PlaybackEngine for AvProxyEngine {
    fn initialize(&mut self) {}

    fn play_stream(
        &mut self,
        _stream: StreamingFile,
        config: PlayConfig,
    ) -> Result<Option<f64>, EngineError> {
        let Some(ref path) = config.file_path else {
            return Err(EngineError::Internal(
                "AvProxyEngine requires a file path".into(),
            ));
        };

        let cmd = AvCmd::Play {
            file_path: path.to_string_lossy().into_owned(),
            start_position: config.start_position_seconds,
            volume: config.volume,
            speed: config.speed,
            title: config.title.clone(),
            artist: config.artist.clone(),
        };

        match self.send(cmd)? {
            AvResp::Ok => {
                self.has_active = true;
                // Fetch duration via snapshot immediately after play succeeds.
                let snap = match self.send(AvCmd::GetSnapshot)? {
                    AvResp::Snapshot(s) => s,
                    _ => return Ok(None),
                };
                if snap.duration > 0.0 {
                    self.cached_duration = snap.duration;
                    Ok(Some(snap.duration))
                } else {
                    Ok(None)
                }
            }
            AvResp::Err(msg) => Err(EngineError::DecodeFailed(msg)),
            _ => Err(EngineError::Internal(
                "Unexpected AV response for Play".into(),
            )),
        }
    }

    fn stop(&mut self) {
        let _ = self.send(AvCmd::Stop);
        self.has_active = false;
        self.cached_duration = 0.0;
    }

    fn pause(&mut self) {
        let _ = self.send(AvCmd::Pause);
    }

    fn resume(&mut self) {
        let _ = self.send(AvCmd::Resume);
    }

    fn seek(&mut self, position_seconds: f64) -> bool {
        matches!(
            self.send(AvCmd::Seek {
                position: position_seconds
            }),
            Ok(AvResp::Ok)
        )
    }

    fn set_volume(&mut self, volume: f32) {
        let _ = self.send(AvCmd::SetVolume { volume });
    }

    fn set_speed(&mut self, speed: f32) {
        let _ = self.send(AvCmd::SetSpeed { speed });
    }

    fn position_seconds(&self) -> f64 {
        match self.send(AvCmd::GetSnapshot) {
            Ok(AvResp::Snapshot(s)) => s.position,
            _ => 0.0,
        }
    }

    fn has_active_playback(&self) -> bool {
        self.has_active
    }

    fn is_paused(&self) -> bool {
        match self.send(AvCmd::GetSnapshot) {
            Ok(AvResp::Snapshot(s)) => s.is_paused,
            _ => false,
        }
    }

    fn is_finished(&self) -> bool {
        match self.send(AvCmd::GetSnapshot) {
            Ok(AvResp::Snapshot(s)) => s.is_finished,
            _ => false,
        }
    }

    fn has_error(&self) -> bool {
        match self.send(AvCmd::GetSnapshot) {
            Ok(AvResp::Snapshot(s)) => s.has_error,
            _ => false,
        }
    }

    fn playback_snapshot(&self) -> PlaybackSnapshot {
        match self.send(AvCmd::GetSnapshot) {
            Ok(AvResp::Snapshot(s)) => PlaybackSnapshot {
                position: s.position,
                duration: s.duration,
                is_paused: s.is_paused,
                is_finished: s.is_finished,
                has_error: s.has_error,
                has_active: s.has_active,
            },
            _ => PlaybackSnapshot::default(),
        }
    }

    fn selected_output_device_id(&self) -> Option<&str> {
        None
    }
}
