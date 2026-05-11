//! Send-safe proxy for the main-thread AV actor.
//!
//! [`AvProxyEngine`] implements [`PlaybackEngine`] by forwarding all commands
//! to an [`AvMainThreadActor`] running on the main dispatch queue via mpsc
//! channels. This allows the orchestration layer to own the engine on a
//! background thread while AVPlayer (which is main-thread-only) lives on the
//! main queue.

use std::sync::mpsc;
use std::time::Duration;

use super::av_main_thread_actor::{start_on_main_queue, AvCmd, AvResp};
use super::engine::{EngineError, PlayConfig, PlaybackEngine};
use super::streaming_file::StreamingFile;

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

/// Send-safe proxy engine that delegates all playback operations to an
/// [`AvMainThreadActor`] on the main thread.
pub struct AvProxyEngine {
	cmd_tx: mpsc::Sender<AvCmd>,
	resp_rx: mpsc::Receiver<AvResp>,
	cached_duration: f64,
	has_active: bool,
}

impl AvProxyEngine {
	/// Spawn the [`AvMainThreadActor`] on the main dispatch queue and return
	/// the proxy.
	pub fn new() -> Self {
		let (cmd_tx, cmd_rx) = mpsc::channel();
		let (resp_tx, resp_rx) = mpsc::channel();

		start_on_main_queue(cmd_rx, resp_tx);

		Self {
			cmd_tx,
			resp_rx,
			cached_duration: 0.0,
			has_active: false,
		}
	}

	/// Return the duration reported by the actor, falling back to the cached
	/// hint if the query fails.
	pub fn duration_seconds(&self) -> f64 {
		match self.send_cmd(AvCmd::GetDuration) {
			Ok(AvResp::Duration(d)) => d,
			_ => self.cached_duration,
		}
	}

	fn send_cmd(&self, cmd: AvCmd) -> Result<AvResp, EngineError> {
		self.cmd_tx
			.send(cmd)
			.map_err(|_| EngineError::Internal("AV command channel closed".into()))?;

		self.resp_rx
			.recv_timeout(RESPONSE_TIMEOUT)
			.map_err(|_| EngineError::Internal("AV response timed out".into()))
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
				"AVProxyEngine requires a file path".into(),
			));
		};

		let file_path = path.to_string_lossy().into_owned();

		let cmd = AvCmd::Play {
			file_path,
			start_position: config.start_position_seconds,
			volume: config.volume,
			speed: config.speed,
		};

		match self.send_cmd(cmd)? {
			AvResp::Ok => {
				self.has_active = true;
				let actual = match self.send_cmd(AvCmd::GetDuration)? {
					AvResp::Duration(d) if d > 0.0 => {
						self.cached_duration = d;
						Some(d)
					}
					_ => None,
				};
				Ok(actual)
			}
			AvResp::Err(msg) => Err(EngineError::DecodeFailed(msg)),
			_ => Err(EngineError::Internal(
				"Unexpected AV response for Play".into(),
			)),
		}
	}

	fn stop(&mut self) {
		let _ = self.send_cmd(AvCmd::Stop);
		self.has_active = false;
		self.cached_duration = 0.0;
	}

	fn pause(&mut self) {
		let _ = self.send_cmd(AvCmd::Pause);
	}

	fn resume(&mut self) {
		let _ = self.send_cmd(AvCmd::Resume);
	}

	fn seek(&mut self, position_seconds: f64) -> bool {
		matches!(
			self.send_cmd(AvCmd::Seek {
				position: position_seconds
			}),
			Ok(AvResp::Ok)
		)
	}

	fn set_volume(&mut self, volume: f32) {
		let _ = self.send_cmd(AvCmd::SetVolume { volume });
	}

	fn set_speed(&mut self, speed: f32) {
		let _ = self.send_cmd(AvCmd::SetSpeed { speed });
	}

	fn position_seconds(&self) -> f64 {
		match self.send_cmd(AvCmd::GetPosition) {
			Ok(AvResp::Position(p)) => p,
			_ => 0.0,
		}
	}

	fn has_active_playback(&self) -> bool {
		self.has_active
	}

	fn is_paused(&self) -> bool {
		match self.send_cmd(AvCmd::IsPaused) {
			Ok(AvResp::Paused(p)) => p,
			_ => false,
		}
	}

	fn is_finished(&self) -> bool {
		match self.send_cmd(AvCmd::IsFinished) {
			Ok(AvResp::Finished(f)) => f,
			_ => false,
		}
	}

	fn has_error(&self) -> bool {
		match self.send_cmd(AvCmd::HasError) {
			Ok(AvResp::HasError(e)) => e,
			_ => false,
		}
	}

	fn selected_output_device_id(&self) -> Option<&str> {
		None
	}
}
