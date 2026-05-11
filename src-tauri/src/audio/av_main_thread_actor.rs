//! Main-thread AV actor — owns `AVPlayer` / `AVPlayerItem` and processes commands
//! dispatched from the audio proxy on the main queue.

#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::{class, msg_send, sel, sel_impl};
use std::marker::PhantomData;
use std::sync::mpsc;

// ---------------------------------------------------------------------------
// CoreMedia CMTime bridging
// ---------------------------------------------------------------------------

#[allow(dead_code)]
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
// Command / Response enums
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum AvCmd {
	Play {
		file_path: String,
		start_position: f64,
		volume: f32,
		speed: f32,
	},
	Pause,
	Resume,
	Stop,
	Seek {
		position: f64,
	},
	SetVolume {
		volume: f32,
	},
	SetSpeed {
		speed: f32,
	},
	GetPosition,
	GetDuration,
	IsPaused,
	IsFinished,
	HasError,
}

#[derive(Debug)]
pub enum AvResp {
	Ok,
	Err(String),
	Position(f64),
	Duration(f64),
	Paused(bool),
	Finished(bool),
	HasError(bool),
}

// ---------------------------------------------------------------------------
// Actor
// ---------------------------------------------------------------------------

/// `AVPlayer`-backed actor that must live and run exclusively on the main
/// thread. The `!Send` bound is enforced by a `PhantomData<Rc<()>>` marker
/// because raw pointers are technically `Send` in Rust.
pub struct AvMainThreadActor {
	player: Option<id>,
	current_item: Option<id>,
	cmd_rx: mpsc::Receiver<AvCmd>,
	resp_tx: mpsc::Sender<AvResp>,
	last_duration: f64,
	desired_speed: f32,
	_not_send: PhantomData<std::rc::Rc<()>>,
}

impl AvMainThreadActor {
	pub fn new(cmd_rx: mpsc::Receiver<AvCmd>, resp_tx: mpsc::Sender<AvResp>) -> Self {
		Self {
			player: None,
			current_item: None,
			cmd_rx,
			resp_tx,
			last_duration: 0.0,
			desired_speed: 1.0,
			_not_send: PhantomData,
		}
	}

	/// Blocking command loop. Must be called on the main thread.
	pub fn run(&mut self) {
		while let Ok(cmd) = self.cmd_rx.recv() {
			let resp = self.handle_cmd(cmd);
			if self.resp_tx.send(resp).is_err() {
				break;
			}
		}
	}

	fn handle_cmd(&mut self, cmd: AvCmd) -> AvResp {
		match cmd {
			AvCmd::Play {
				file_path,
				start_position,
				volume,
				speed,
			} => self.play(&file_path, start_position, volume, speed),
			AvCmd::Pause => {
				self.pause();
				AvResp::Ok
			}
			AvCmd::Resume => {
				self.resume();
				AvResp::Ok
			}
			AvCmd::Stop => {
				self.stop();
				AvResp::Ok
			}
			AvCmd::Seek { position } => {
				self.seek(position);
				AvResp::Ok
			}
			AvCmd::SetVolume { volume } => {
				self.set_volume(volume);
				AvResp::Ok
			}
			AvCmd::SetSpeed { speed } => {
				self.set_speed(speed);
				AvResp::Ok
			}
			AvCmd::GetPosition => AvResp::Position(self.position_seconds()),
			AvCmd::GetDuration => AvResp::Duration(self.item_duration()),
			AvCmd::IsPaused => AvResp::Paused(self.is_paused()),
			AvCmd::IsFinished => AvResp::Finished(self.is_finished()),
			AvCmd::HasError => AvResp::HasError(self.has_error()),
		}
	}

	// -----------------------------------------------------------------------
	// AVPlayer operations
	// -----------------------------------------------------------------------

	fn play(
		&mut self,
		file_path: &str,
		start_position: f64,
		volume: f32,
		speed: f32,
	) -> AvResp {
		self.stop_and_release();

		let result: Result<(id, id, Option<f64>), String> = unsafe {
			objc::rc::autoreleasepool(|| {
				let path_ns = NSString::alloc(nil).init_str(file_path);
				let url: id = msg_send![class!(NSURL), fileURLWithPath:path_ns];
				let _: () = msg_send![path_ns, release];

				let item: id = msg_send![class!(AVPlayerItem), playerItemWithURL:url];
				if item == nil {
					return Err("Failed to create AVPlayerItem".to_string());
				}
				// playerItemWithURL: returns autoreleased — retain for ownership.
				let _: () = msg_send![item, retain];

				let player: id = msg_send![class!(AVPlayer), alloc];
				let player: id = msg_send![player, initWithPlayerItem:item];
				if player == nil {
					let _: () = msg_send![item, release];
					return Err("Failed to create AVPlayer".to_string());
				}

				let _: () = msg_send![player, setVolume:volume];

				if start_position > 0.0 {
					let time = CMTimeMakeWithSeconds(start_position, 1000);
					let _: () = msg_send![player, seekToTime:time];
				}

				self.desired_speed = speed;
				let _: () = msg_send![player, setRate:speed];

				let duration = item_duration_from_id(item);

				Ok((player, item, duration))
			})
		};

		match result {
			Ok((player, item, duration)) => {
				self.player = Some(player);
				self.current_item = Some(item);
				self.last_duration = duration.unwrap_or(0.0);
				AvResp::Ok
			}
			Err(e) => AvResp::Err(e),
		}
	}

	fn stop(&mut self) {
		self.stop_and_release();
	}

	fn stop_and_release(&mut self) {
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

	fn pause(&mut self) {
		if let Some(player) = self.player {
			unsafe {
				let _: () = msg_send![player, pause];
			}
		}
	}

	fn resume(&mut self) {
		let Some(player) = self.player else {
			return;
		};
		unsafe {
			let target_rate = if self.desired_speed > 0.0 {
				self.desired_speed
			} else {
				1.0
			};
			if (target_rate - 1.0).abs() < 0.01 {
				let _: () = msg_send![player, play];
			} else {
				let _: () = msg_send![player, setRate:target_rate];
			}
		}
	}

	fn seek(&mut self, position_seconds: f64) {
		if let Some(player) = self.player {
			unsafe {
				let time = CMTimeMakeWithSeconds(position_seconds, 1000);
				let _: () = msg_send![player, seekToTime:time];
			}
		}
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
			if time.timescale > 0 {
				let s = CMTimeGetSeconds(time);
				if s.is_finite() && s >= 0.0 {
					s
				} else {
					0.0
				}
			} else {
				0.0
			}
		}
	}

	fn item_duration(&self) -> f64 {
		self.current_item
			.and_then(|item| item_duration_from_id(item))
			.unwrap_or(self.last_duration)
	}

	fn is_paused(&self) -> bool {
		let Some(player) = self.player else {
			return false;
		};
		unsafe {
			let rate: f64 = msg_send![player, rate];
			rate == 0.0
		}
	}

	fn is_finished(&self) -> bool {
		if self.player.is_none() {
			return false;
		}
		let rate: f64 = unsafe { msg_send![self.player.unwrap(), rate] };
		let pos = self.position_seconds();
		let dur = self.item_duration();
		const END_EPSILON: f64 = 0.2;
		dur > 0.0 && pos >= dur - END_EPSILON && rate == 0.0
	}

	fn has_error(&self) -> bool {
		let Some(player) = self.player else {
			return false;
		};
		unsafe {
			let player_status: i32 = msg_send![player, status];
			let item_status: i32 = self.current_item.map_or(-1, |i| msg_send![i, status]);
			player_status == 2 || item_status == 2
		}
	}
}

impl Drop for AvMainThreadActor {
	fn drop(&mut self) {
		self.stop_and_release();
	}
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn item_duration_from_id(item: id) -> Option<f64> {
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

// ---------------------------------------------------------------------------
// Main-queue scheduling
// ---------------------------------------------------------------------------

#[link(name = "System", kind = "framework")]
unsafe extern "C" {
	static _dispatch_main_q: std::ffi::c_void;
	fn dispatch_async_f(
		queue: *mut std::ffi::c_void,
		context: *mut std::ffi::c_void,
		work: extern "C" fn(*mut std::ffi::c_void),
	);
}

fn main_queue() -> *mut std::ffi::c_void {
	unsafe { &_dispatch_main_q as *const _ as *mut _ }
}

/// Construct the actor on the main dispatch queue and start its command loop.
/// The actor is never moved across threads.
pub fn start_on_main_queue(cmd_rx: mpsc::Receiver<AvCmd>, resp_tx: mpsc::Sender<AvResp>) {
	let channels = Box::new((cmd_rx, resp_tx));
	let ptr = Box::into_raw(channels) as *mut std::ffi::c_void;

	unsafe {
		dispatch_async_f(main_queue(), ptr, run_actor_trampoline);
	}
}

extern "C" fn run_actor_trampoline(ctx: *mut std::ffi::c_void) {
	let (cmd_rx, resp_tx) =
		unsafe { *Box::from_raw(ctx as *mut (mpsc::Receiver<AvCmd>, mpsc::Sender<AvResp>)) };
	let mut actor = AvMainThreadActor::new(cmd_rx, resp_tx);
	actor.run();
}
