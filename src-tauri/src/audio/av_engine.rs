//! AVPlayer-backed playback engine for macOS.
//!
//! [`AvEngine`] implements [`PlaybackEngine`] by dispatching commands to a
//! dedicated serial GCD queue (`com.jrss.avplayer`) where a persistent
//! [`AVPlayer`] lives. `AVPlayer` is `MainThreadOnly` in `objc2`, so
//! `AvActor` is `!Send`; the `unsafe impl Send` is sound because the actor is
//! only ever touched on that single queue.
//!
//! `MPNowPlayingInfoCenter` updates are dispatched separately to the main
//! queue since that API is documented as main-thread-only.

use std::ffi::{c_char, c_void};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly, class, msg_send};
use objc2_av_foundation::{AVPlayer, AVPlayerItem};
use objc2_core_media::CMTime;
use objc2_foundation::{NSString, NSURL};

use super::engine::{EngineError, PlayConfig, PlaybackEngine, PlaybackSnapshot};
use super::streaming_file::StreamingFile;

const CMD_TIMEOUT: Duration = Duration::from_secs(5);

// ---------------------------------------------------------------------------
// MPNowPlayingInfoCenter — Now Playing publishing
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
#[link(name = "MediaPlayer", kind = "framework")]
unsafe extern "C" {
	static MPMediaItemPropertyTitle: *mut AnyObject;
	static MPMediaItemPropertyArtist: *mut AnyObject;
	static MPMediaItemPropertyPlaybackDuration: *mut AnyObject;
	static MPNowPlayingInfoPropertyPlaybackRate: *mut AnyObject;
	static MPNowPlayingInfoPropertyElapsedPlaybackTime: *mut AnyObject;
}

const MP_STATE_PLAYING: u64 = 1;
const MP_STATE_PAUSED: u64 = 2;
const MP_STATE_STOPPED: u64 = 3;

// ---------------------------------------------------------------------------
// Public command / response types
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum AvCmd {
	Play {
		file_path: String,
		stream_url: Option<String>,
		start_position: f64,
		volume: f32,
		speed: f32,
		title: String,
		artist: String,
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
	GetSnapshot,
}

struct PlayParams {
	file_path: String,
	stream_url: Option<String>,
	start_position: f64,
	volume: f32,
	speed: f32,
	title: String,
	artist: String,
}

/// Collapsed state snapshot — one round-trip replaces four separate queries.
#[derive(Debug, Default)]
struct AvSnapshot {
	position: f64,
	duration: f64,
	is_paused: bool,
	is_finished: bool,
	has_active: bool,
}

#[derive(Debug)]
enum AvResp {
	Ok,
	Snapshot(AvSnapshot),
}

// ---------------------------------------------------------------------------
// AvActor
// ---------------------------------------------------------------------------

/// Owns a persistent `AVPlayer` that is reused across tracks via
/// `replaceCurrentItemWithPlayerItem:`. Must run exclusively on the
/// dedicated serial queue; the `!Send` bound is enforced by `PhantomData<Rc<()>>`.
struct AvActor {
	player: Option<Retained<AVPlayer>>,
	current_item: Option<Retained<AVPlayerItem>>,
	last_duration: f64,
	desired_speed: f32,
	current_title: String,
	current_artist: String,
	_not_send: PhantomData<std::rc::Rc<()>>,
}

// SAFETY: every access is routed through `dispatch_async_f` targeting a
// private serial queue, so the actor is only ever touched on that queue.
unsafe impl Send for AvActor {}

impl AvActor {
	fn new() -> Self {
		Self {
			player: None,
			current_item: None,
			last_duration: 0.0,
			desired_speed: 1.0,
			current_title: String::new(),
			current_artist: String::new(),
			_not_send: PhantomData,
		}
	}

    fn handle_cmd(&mut self, cmd: AvCmd) -> AvResp {
        match cmd {
            AvCmd::Play {
                file_path,
                stream_url,
                start_position,
                volume,
                speed,
                title,
                artist,
            } => self.play(PlayParams {
				file_path,
				stream_url,
				start_position,
				volume,
				speed,
				title,
				artist,
			}),
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
			AvCmd::GetSnapshot => AvResp::Snapshot(self.snapshot()),
		}
	}

	fn play(&mut self, params: PlayParams) -> AvResp {
		// SAFETY: called exclusively on the dedicated serial queue via dispatch
		// trampoline. The queue is the only thread touching this actor.
		let mtm = unsafe { MainThreadMarker::new_unchecked() };

		self.current_title = params.title;
		self.current_artist = params.artist;
		self.desired_speed = params.speed;

		let url: Option<Retained<NSURL>> = if let Some(ref url_str) = params.stream_url {
			let url_ns = NSString::from_str(url_str);
			NSURL::URLWithString(&url_ns)
		} else {
			let path_ns = NSString::from_str(&params.file_path);
			Some(NSURL::fileURLWithPath(&path_ns))
		};
		let Some(ref url) = url else {
			return AvResp::Ok;
		};
		let item = unsafe { AVPlayerItem::playerItemWithURL(url, mtm) };

		match self.player.as_deref() {
			Some(player) => {
				unsafe {
				player.replaceCurrentItemWithPlayerItem(Some(&item));
				player.setVolume(params.volume);
				if params.start_position > 0.0 {
					let time = CMTime::with_seconds(params.start_position, 1_000);
					player.seekToTime(time);
				}
				if (params.speed - 1.0).abs() < 0.01 {
					player.play();
				} else {
					player.setRate(params.speed);
				}
			}
			self.last_duration = item_duration(&item).unwrap_or(0.0);
			self.current_item = Some(item);
		}
		None => {
			let player =
				unsafe { AVPlayer::initWithPlayerItem(AVPlayer::alloc(mtm), Some(&item)) };
			unsafe {
				player.setVolume(params.volume);
				if params.start_position > 0.0 {
					let time = CMTime::with_seconds(params.start_position, 1_000);
					player.seekToTime(time);
				}
				if (params.speed - 1.0).abs() < 0.01 {
					player.play();
				} else {
					player.setRate(params.speed);
				}
			}
				self.last_duration = item_duration(&item).unwrap_or(0.0);
				self.current_item = Some(item);
				self.player = Some(player);
			}
		}

		self.publish_now_playing_snapshot(MP_STATE_PLAYING);

		AvResp::Ok
	}

	fn stop(&mut self) {
		if let Some(player) = self.player.as_deref() {
			unsafe {
				player.pause();
				player.replaceCurrentItemWithPlayerItem(None);
			}
		}
		self.current_item = None;
		self.last_duration = 0.0;
		self.current_title.clear();
		self.current_artist.clear();
		self.erase_now_playing();
	}

	fn pause(&mut self) {
		if let Some(player) = self.player.as_deref() {
			unsafe { player.pause() };
		}
		self.publish_now_playing_snapshot(MP_STATE_PAUSED);
	}

	fn resume(&mut self) {
		let Some(player) = self.player.as_deref() else {
			return;
		};
		let target = if self.desired_speed > 0.0 {
			self.desired_speed
		} else {
			1.0
		};
		unsafe {
			if (target - 1.0).abs() < 0.01 {
				player.play();
			} else {
				player.setRate(target);
			}
		}
		self.publish_now_playing_snapshot(MP_STATE_PLAYING);
	}

	fn seek(&mut self, position_seconds: f64) {
		if let Some(player) = self.player.as_deref() {
			unsafe {
				let time = CMTime::with_seconds(position_seconds, 1_000);
				player.seekToTime(time);
			}
		}
		let state = match self.player.as_deref() {
			Some(p) if unsafe { p.rate() } != 0.0 => MP_STATE_PLAYING,
			_ => MP_STATE_PAUSED,
		};
		self.publish_now_playing_snapshot(state);
	}

	fn set_volume(&mut self, volume: f32) {
		if let Some(player) = self.player.as_deref() {
			unsafe { player.setVolume(volume) };
		}
	}

	fn set_speed(&mut self, speed: f32) {
		self.desired_speed = speed;
		if let Some(player) = self.player.as_deref() {
			unsafe { player.setRate(speed) };
		}
		let state = if speed == 0.0 {
			MP_STATE_PAUSED
		} else {
			MP_STATE_PLAYING
		};
		self.publish_now_playing_snapshot(state);
	}

	fn position_seconds(&self) -> f64 {
		let Some(player) = self.player.as_deref() else {
			return 0.0;
		};
		let time = unsafe { player.currentTime() };
		unsafe {
			if time.timescale > 0 {
				let s = CMTime::seconds(time);
				if s.is_finite() && s >= 0.0 { s } else { 0.0 }
			} else {
				0.0
			}
		}
	}

	fn item_duration(&self) -> f64 {
		self.current_item
			.as_deref()
			.and_then(item_duration)
			.unwrap_or(self.last_duration)
	}

	fn snapshot(&self) -> AvSnapshot {
		let Some(player) = self.player.as_deref() else {
			return AvSnapshot::default();
		};

		let rate = unsafe { player.rate() };
		let pos = self.position_seconds();
		let dur = self.item_duration();

		const END_EPSILON: f64 = 3.0;
		let is_finished = dur > 0.0 && pos >= dur - END_EPSILON && rate == 0.0;

		AvSnapshot {
			position: pos,
			duration: dur,
			is_paused: rate == 0.0,
			is_finished,
			has_active: self.current_item.is_some(),
		}
	}

	fn publish_now_playing_snapshot(&self, playback_state: u64) {
		let Some(player) = self.player.as_deref() else {
			return;
		};

		let elapsed = self.position_seconds();
		let duration = self.item_duration();
		let rate = unsafe { player.rate() } as f64;

		let payload = Box::new(NowPlayingPayload {
			title: self.current_title.clone(),
			artist: self.current_artist.clone(),
			duration,
			elapsed,
			rate,
			playback_state,
		});

		unsafe {
			dispatch_async_f(
				main_queue(),
				Box::into_raw(payload) as *mut c_void,
				now_playing_trampoline,
			);
		}
	}

	fn erase_now_playing(&self) {
		let payload = Box::new(NowPlayingPayload {
			title: String::new(),
			artist: String::new(),
			duration: 0.0,
			elapsed: 0.0,
			rate: 0.0,
			playback_state: MP_STATE_STOPPED,
		});

		unsafe {
			dispatch_async_f(
				main_queue(),
				Box::into_raw(payload) as *mut c_void,
				now_playing_trampoline,
			);
		}
	}
}

impl Drop for AvActor {
	fn drop(&mut self) {
		self.stop();
	}
}

// ---------------------------------------------------------------------------
// Now Playing payload and main-queue trampoline
// ---------------------------------------------------------------------------

struct NowPlayingPayload {
	title: String,
	artist: String,
	duration: f64,
	elapsed: f64,
	rate: f64,
	playback_state: u64,
}

unsafe extern "C" fn now_playing_trampoline(ctx: *mut c_void) {
	let payload = unsafe { Box::from_raw(ctx as *mut NowPlayingPayload) };

	unsafe {
		let center: *mut AnyObject = msg_send![class!(MPNowPlayingInfoCenter), defaultCenter];
		let dict: *mut AnyObject = msg_send![class!(NSMutableDictionary), dictionary];

		if !payload.title.is_empty() {
			let title = NSString::from_str(&payload.title);
			let _: () = msg_send![dict, setObject: &*title, forKey: MPMediaItemPropertyTitle];
		}

		if !payload.artist.is_empty() {
			let artist = NSString::from_str(&payload.artist);
			let _: () = msg_send![dict, setObject: &*artist, forKey: MPMediaItemPropertyArtist];
		}

		if payload.duration.is_finite() && payload.duration > 0.0 {
			let dur_num: *mut AnyObject =
				msg_send![class!(NSNumber), numberWithDouble: payload.duration];
			let _: () = msg_send![dict, setObject: dur_num,
				forKey: MPMediaItemPropertyPlaybackDuration];
		}

		let elapsed_num: *mut AnyObject =
			msg_send![class!(NSNumber), numberWithDouble: payload.elapsed.max(0.0)];
		let _: () = msg_send![dict, setObject: elapsed_num,
			forKey: MPNowPlayingInfoPropertyElapsedPlaybackTime];

		let rate_num: *mut AnyObject =
			msg_send![class!(NSNumber), numberWithDouble: payload.rate.max(0.0)];
		let _: () = msg_send![dict, setObject: rate_num,
			forKey: MPNowPlayingInfoPropertyPlaybackRate];

		let _: () = msg_send![center, setNowPlayingInfo: dict];
		let _: () = msg_send![center, setPlaybackState: payload.playback_state];
	}
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn item_duration(item: &AVPlayerItem) -> Option<f64> {
	let dur = unsafe { item.duration() };
	if dur.timescale > 0 {
		let secs = unsafe { CMTime::seconds(dur) };
		if secs.is_finite() && secs > 0.0 {
			Some(secs)
		} else {
			None
		}
	} else {
		None
	}
}

// ---------------------------------------------------------------------------
// Dedicated serial queue handle
// ---------------------------------------------------------------------------

struct AvEngineInner {
	actor: Mutex<Option<AvActor>>,
	queue: *mut c_void,
}

// SAFETY: the queue pointer is immutable after creation and only used for
// dispatch_async_f. The actor mutex provides exclusive access.
unsafe impl Send for AvEngineInner {}
unsafe impl Sync for AvEngineInner {}

#[link(name = "System", kind = "framework")]
unsafe extern "C" {
	static _dispatch_main_q: c_void;
	fn dispatch_async_f(
		queue: *const c_void,
		context: *mut c_void,
		work: unsafe extern "C" fn(*mut c_void),
	);
	fn dispatch_queue_create(label: *const c_char, attr: *const c_void) -> *mut c_void;
}

fn main_queue() -> *const c_void {
	unsafe { &_dispatch_main_q as *const _ }
}

struct CmdPayload {
	inner: Arc<AvEngineInner>,
	cmd: AvCmd,
	resp: std::sync::mpsc::SyncSender<AvResp>,
}

unsafe extern "C" fn cmd_trampoline(ctx: *mut c_void) {
	let payload = unsafe { Box::from_raw(ctx as *mut CmdPayload) };
	let mut guard = payload.inner.actor.lock().unwrap();
	let actor = guard.get_or_insert_with(AvActor::new);
	let resp = actor.handle_cmd(payload.cmd);
	let _ = payload.resp.send(resp);
}

// ---------------------------------------------------------------------------
// AvEngine — PlaybackEngine implementation
// ---------------------------------------------------------------------------

pub struct AvEngine {
	inner: Arc<AvEngineInner>,
	has_active: bool,
}

impl AvEngine {
	pub fn new() -> Self {
		let label = std::ffi::CString::new("com.jrss.avplayer").unwrap();
		let queue = unsafe { dispatch_queue_create(label.as_ptr(), std::ptr::null()) };
		assert!(
			!queue.is_null(),
			"failed to create AVPlayer serial dispatch queue"
		);

		Self {
			inner: Arc::new(AvEngineInner {
				actor: Mutex::new(None),
				queue,
			}),
			has_active: false,
		}
	}

    fn send(&self, cmd: AvCmd) -> Result<AvResp, EngineError> {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let payload = Box::new(CmdPayload {
            inner: Arc::clone(&self.inner),
            cmd,
            resp: tx,
        });
        unsafe {
            dispatch_async_f(
                self.inner.queue,
                Box::into_raw(payload) as *mut c_void,
                cmd_trampoline,
            );
        }
        rx.recv_timeout(CMD_TIMEOUT)
            .map_err(|e| match e {
                std::sync::mpsc::RecvTimeoutError::Timeout => {
                    EngineError::Internal("AV command timed out".into())
                }
                std::sync::mpsc::RecvTimeoutError::Disconnected => {
                    EngineError::Internal("AV actor closed".into())
                }
            })
    }
}

impl PlaybackEngine for AvEngine {
	fn initialize(&mut self) {}

	fn play_stream(
		&mut self,
		_stream: StreamingFile,
		config: PlayConfig,
	) -> Result<Option<f64>, EngineError> {
		let Some(ref path) = config.file_path else {
			return Err(EngineError::Internal(
				"AvEngine requires a file path".into(),
			));
		};

		let cmd = AvCmd::Play {
			file_path: path.to_string_lossy().into_owned(),
			stream_url: config.stream_url,
			start_position: config.start_position_seconds,
			volume: config.volume,
			speed: config.speed,
			title: config.title,
			artist: config.artist,
		};

		match self.send(cmd)? {
			AvResp::Ok => {
				self.has_active = true;
				let snap = match self.send(AvCmd::GetSnapshot)? {
					AvResp::Snapshot(s) => s,
					_ => return Ok(None),
				};
				if snap.duration > 0.0 {
					Ok(Some(snap.duration))
				} else {
					Ok(None)
				}
			}
			_ => Err(EngineError::Internal(
				"Unexpected AV response for Play".into(),
			)),
		}
	}

	fn stop(&mut self) {
		let _ = self.send(AvCmd::Stop);
		self.has_active = false;
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

	fn playback_snapshot(&self) -> PlaybackSnapshot {
		match self.send(AvCmd::GetSnapshot) {
			Ok(AvResp::Snapshot(s)) => PlaybackSnapshot {
				position: s.position,
				is_paused: s.is_paused,
				is_finished: s.is_finished,
				has_active: s.has_active,
			},
			_ => PlaybackSnapshot::default(),
		}
	}

	fn selected_output_device_id(&self) -> Option<&str> {
		None
	}
}
