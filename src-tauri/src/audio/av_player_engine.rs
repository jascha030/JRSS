#![allow(deprecated)]

//! macOS-native audio playback engine using `AVPlayer`.
//!
//! [`AvPlayerEngine`] implements [`PlaybackEngine`] via Objective-C messaging
//! to `AVPlayer`, `AVPlayerItem`, and `AVURLAsset`. It replaces
//! [`RodioEngine`](super::rodio_engine::RodioEngine) on macOS and provides:
//!
//! - Hardware-accelerated AAC/MP3 decoding
//! - Native AirPlay support (automatic — follows the system audio route)
//! - System audio ducking and interruption handling
//! - Lower first-play latency
//!
//! Output device selection follows the system default, which is the idiomatic
//! macOS behaviour. The in-app device picker is disabled on macOS; Linux
//! continues to use the rodio backend with full CPAL device enumeration.

#![allow(unexpected_cfgs)]

use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::{class, msg_send, sel, sel_impl};

use super::engine::{EngineError, PlayConfig, PlaybackEngine};
use super::events::OutputDeviceInfo;
use super::streaming_file::StreamingFile;

// ---------------------------------------------------------------------------
// CoreMedia CMTime bridging
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
struct CMTime {
	value: i64,
	timescale: i32,
	flags: u32,
	epoch: i64,
}

#[link(name = "CoreMedia", kind = "framework")]
unsafe extern "C" {
	fn CMTimeMakeWithSeconds(seconds: f64, preferredTimescale: i32) -> CMTime;
	fn CMTimeGetSeconds(time: CMTime) -> f64;
}

// ---------------------------------------------------------------------------
// AvPlayerEngine
// ---------------------------------------------------------------------------

/// `AVPlayer`-backed playback engine for macOS.
pub struct AvPlayerEngine {
	player: Option<id>,
	current_item: Option<id>,
	last_duration: f64,
	desired_speed: f32,
}

impl AvPlayerEngine {
	pub fn new() -> Self {
		Self {
			player: None,
			current_item: None,
			last_duration: 0.0,
			desired_speed: 1.0,
		}
	}

	fn stop_and_release(&mut self) {
		log::trace!("AvPlayerEngine::stop_and_release()");
		if let Some(player) = self.player.take() {
			unsafe {
				let _: () = msg_send![player, pause];
				let _: () = msg_send![player, release];
			}
		}
		if let Some(item) = self.current_item.take() {
			unsafe {
				let _: () = msg_send![item, release];
			}
		}
		self.last_duration = 0.0;
	}

	fn item_duration(&self) -> f64 {
		let Some(item) = self.current_item else {
			return 0.0;
		};
		unsafe {
			let duration: CMTime = msg_send![item, duration];
			if duration.timescale > 0 {
				let secs = CMTimeGetSeconds(duration);
				if secs.is_finite() && secs > 0.0 {
					secs
				} else {
					0.0
				}
			} else {
				0.0
			}
		}
	}
}

impl PlaybackEngine for AvPlayerEngine {
	fn play_stream(
		&mut self,
		_stream: StreamingFile,
		config: PlayConfig,
	) -> Result<Option<f64>, EngineError> {
		log::trace!(
			"AvPlayerEngine::play_stream() path={:?} start={} volume={} speed={}",
			config.file_path, config.start_position_seconds, config.volume, config.speed
		);
		self.stop_and_release();

		let Some(ref path) = config.file_path else {
			return Err(EngineError::Internal(
				"No file path provided for AVPlayer".to_string(),
			));
		};

		let path_str = path.to_str().ok_or_else(|| {
			EngineError::Internal("Invalid file path for AVPlayer".to_string())
		})?;

		unsafe {
			// Wrap all ObjC creation in autorelease pool to manage temporary object lifetimes
			let (player, item, duration) = objc::rc::autoreleasepool(|| {
				let path_ns = NSString::alloc(nil).init_str(path_str);
				let url: id = msg_send![class!(NSURL), fileURLWithPath:path_ns];
				let _: () = msg_send![path_ns, release];

				let item: id = msg_send![class!(AVPlayerItem), playerItemWithURL:url];
				if item == nil {
					return Err(EngineError::DecodeFailed(
						"Failed to create AVPlayerItem".to_string(),
					));
				}
				// Retain the item since playerItemWithURL: returns autoreleased
				let _: () = msg_send![item, retain];

				let player: id = msg_send![class!(AVPlayer), alloc];
				let player: id = msg_send![player, initWithPlayerItem:item];
				if player == nil {
					let _: () = msg_send![item, release];
					return Err(EngineError::Internal(
						"Failed to create AVPlayer".to_string(),
					));
				}

				let _: () = msg_send![player, setVolume:config.volume];

				if config.start_position_seconds > 0.0 {
					let time = CMTimeMakeWithSeconds(config.start_position_seconds, 1000);
					let _: () = msg_send![player, seekToTime:time];
				}

				self.desired_speed = config.speed;
				let _: () = msg_send![player, setRate:config.speed];

				let duration = self.item_duration_from_item(item);

				Ok((player, item, duration))
			})?;

			self.player = Some(player);
			self.current_item = Some(item);
			self.last_duration = duration.unwrap_or(config.duration_hint_seconds);

			Ok(duration)
		}
	}

	fn stop(&mut self) {
		self.stop_and_release();
	}

	fn pause(&mut self) {
		log::trace!(
			"AvPlayerEngine::pause() player_exists={} rate={}",
			self.player.is_some(),
			self.player.map_or(0.0f64, |p| unsafe { msg_send![p, rate] })
		);
		if let Some(player) = self.player {
			unsafe {
				let _: () = msg_send![player, pause];
				let rate_after: f64 = msg_send![player, rate];
				log::trace!("AvPlayerEngine::pause() rate_after={}", rate_after);
			}
		}
	}

	fn resume(&mut self) {
		log::trace!(
			"AvPlayerEngine::resume() player_exists={} desired_speed={}",
			self.player.is_some(),
			self.desired_speed
		);
		if let Some(player) = self.player {
			unsafe {
				let rate_before: f64 = msg_send![player, rate];
				let item_before = self.current_item;
				let item_status_before: i32 = item_before.map_or(-1, |i| msg_send![i, status]);
				let player_status_before: i32 = msg_send![player, status];
				log::trace!(
					"AvPlayerEngine::resume() before: rate={} player_status={} item_status={}",
					rate_before, player_status_before, item_status_before
				);
				
				// Use play() for normal speed (gentler on audio system), setRate only for non-1x
				let target_rate = if self.desired_speed > 0.0 {
					self.desired_speed
				} else {
					1.0
				};
				
				if (target_rate - 1.0).abs() < 0.01 {
					// Normal speed - use play() which is gentler on the audio system
					let _: () = msg_send![player, play];
					log::trace!("AvPlayerEngine::resume() using play() for normal speed");
				} else {
					// Non-standard speed - use setRate:
					let _: () = msg_send![player, setRate:target_rate];
					log::trace!("AvPlayerEngine::resume() using setRate:{} for speed change", target_rate);
				}
				
				let rate_after: f64 = msg_send![player, rate];
				log::trace!("AvPlayerEngine::resume() after: rate_after={}", rate_after);
			}
		} else {
			log::warn!("AvPlayerEngine::resume() called with no player");
		}
	}

	fn seek(&mut self, position_seconds: f64) -> bool {
		let Some(player) = self.player else {
			return false;
		};
		unsafe {
			let time = CMTimeMakeWithSeconds(position_seconds, 1000);
			let _: () = msg_send![player, seekToTime:time];
		}
		true
	}

	fn set_volume(&mut self, volume: f32) {
		if let Some(player) = self.player {
			unsafe {
				let _: () = msg_send![player, setVolume:volume];
			}
		}
	}

	fn set_speed(&mut self, speed: f32) {
		self.desired_speed = speed;
		if let Some(player) = self.player {
			unsafe {
				let _: () = msg_send![player, setRate:speed];
			}
		}
	}

	fn position_seconds(&self) -> f64 {
		let Some(player) = self.player else {
			return 0.0;
		};
		unsafe {
			let time: CMTime = msg_send![player, currentTime];
			let secs = if time.timescale > 0 {
				let s = CMTimeGetSeconds(time);
				if s.is_finite() && s >= 0.0 { s } else { 0.0 }
			} else {
				0.0
			};
			log::trace!("AvPlayerEngine::position_seconds() -> {}", secs);
			secs
		}
	}

	fn has_active_playback(&self) -> bool {
		self.player.is_some()
	}

	fn is_paused(&self) -> bool {
		if !self.has_active_playback() {
			log::trace!("AvPlayerEngine::is_paused() -> false (no player)");
			return false;
		}
		let rate: f64 = unsafe { msg_send![self.player.unwrap(), rate] };
		let paused = rate == 0.0;
		log::trace!("AvPlayerEngine::is_paused() rate={} -> {}", rate, paused);
		paused
	}

	fn is_finished(&self) -> bool {
		if !self.has_active_playback() {
			log::trace!("AvPlayerEngine::is_finished() -> false (no player)");
			return false;
		}
		let rate: f64 = unsafe { msg_send![self.player.unwrap(), rate] };
		let pos = self.position_seconds();
		let dur = self.item_duration();
		const END_EPSILON: f64 = 0.2;
		let finished = dur > 0.0 && pos >= dur - END_EPSILON && rate == 0.0;
		log::trace!(
			"AvPlayerEngine::is_finished() rate={} pos={} dur={} epsilon={} -> {}",
			rate, pos, dur, END_EPSILON, finished
		);
		finished
	}

	fn has_error(&self) -> bool {
		let Some(player) = self.player else {
			return false;
		};
		unsafe {
			let player_status: i32 = msg_send![player, status];
			let item_status: i32 = self.current_item.map_or(-1, |i| msg_send![i, status]);
			let error = player_status == 2 || item_status == 2;
			log::trace!(
				"AvPlayerEngine::has_error() player_status={} item_status={} -> {}",
				player_status, item_status, error
			);
			if error {
				return true;
			}
		}
		false
	}

	fn list_output_devices(&self) -> Vec<OutputDeviceInfo> {
		Vec::new()
	}

	fn selected_output_device_id(&self) -> Option<&str> {
		None
	}

	fn set_output_device(&mut self, _device_id: Option<String>) -> Result<(), EngineError> {
		Ok(())
	}
}

impl Drop for AvPlayerEngine {
	fn drop(&mut self) {
		self.stop_and_release();
	}
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl AvPlayerEngine {
	fn item_duration_from_item(&self, item: id) -> Option<f64> {
		if item == nil {
			return None;
		}
		unsafe {
			let duration: CMTime = msg_send![item, duration];
			if duration.timescale > 0 {
				let secs = CMTimeGetSeconds(duration);
				if secs.is_finite() && secs > 0.0 {
					Some(secs)
				} else {
					None
				}
			} else {
				None
			}
		}
	}
}
