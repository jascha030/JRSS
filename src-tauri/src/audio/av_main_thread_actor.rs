//! Main-thread AV actor — owns a persistent `AVPlayer` and processes commands
//! dispatched one-shot from [`AvProxyEngine`] via `dispatch_async_f`.
//!
//! ## Thread safety contract
//! `AVPlayer` is `MainThreadOnly` in objc2, which makes `AvMainThreadActor`
//! `!Send`. The actor is stored in `Arc<Mutex<Option<…>>>` and accessed
//! exclusively through `dispatch_async_f` targeting `_dispatch_main_q`,
//! guaranteeing every access happens on the main thread.
//!
//! `unsafe impl Send for AvMainThreadActor` is sound under that invariant.

use std::ffi::c_void;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly, class, msg_send};
use objc2_av_foundation::{AVPlayer, AVPlayerItem};
use objc2_core_media::CMTime;
use objc2_foundation::{NSString, NSURL};

// ---------------------------------------------------------------------------
// MPNowPlayingInfoCenter — Now Playing publishing
// ---------------------------------------------------------------------------
//
// Publishing to MPNowPlayingInfoCenter makes the session visible to Control
// Center, AirPods long-press, and external media keys.  It must be called on
// the main thread; the dispatch trampoline in this module guarantees that.
//
// MediaPlayer framework is linked here; MPRemoteCommandCenter (wired in
// macos::remote_commands) lives in the same framework.
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
pub enum AvCmd {
    Play {
        file_path: String,
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

/// Collapsed state snapshot — one round-trip replaces four separate queries.
#[derive(Debug, Default)]
pub struct AvSnapshot {
    pub position: f64,
    pub duration: f64,
    pub is_paused: bool,
    pub is_finished: bool,
    pub has_active: bool,
}

#[derive(Debug)]
pub enum AvResp {
    Ok,
    Snapshot(AvSnapshot),
}

// ---------------------------------------------------------------------------
// Actor
// ---------------------------------------------------------------------------

/// Owns a persistent `AVPlayer` that is reused across tracks via
/// `replaceCurrentItemWithPlayerItem:`.  Must run exclusively on the main
/// thread; the `!Send` bound is enforced by `PhantomData<Rc<()>>`.
pub struct AvMainThreadActor {
    player: Option<Retained<AVPlayer>>,
    current_item: Option<Retained<AVPlayerItem>>,
    last_duration: f64,
    desired_speed: f32,
    current_title: String,
    current_artist: String,
    _not_send: PhantomData<std::rc::Rc<()>>,
}

// SAFETY: every access is routed through `dispatch_async_f` targeting
// `_dispatch_main_q`, so the actor is only ever touched on the main thread.
unsafe impl Send for AvMainThreadActor {}

impl AvMainThreadActor {
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

    pub fn handle_cmd(&mut self, cmd: AvCmd) -> AvResp {
        match cmd {
            AvCmd::Play {
                file_path,
                start_position,
                volume,
                speed,
                title,
                artist,
            } => self.play(&file_path, start_position, volume, speed, &title, &artist),
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

    // -----------------------------------------------------------------------
    // AVPlayer operations
    // -----------------------------------------------------------------------

    fn play(
        &mut self,
        file_path: &str,
        start_position: f64,
        volume: f32,
        speed: f32,
        title: &str,
        artist: &str,
    ) -> AvResp {
        // SAFETY: called exclusively on the main thread via dispatch trampoline.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        self.current_title = title.to_owned();
        self.current_artist = artist.to_owned();
        self.desired_speed = speed;

        let path_ns = NSString::from_str(file_path);
        let url = NSURL::fileURLWithPath(&path_ns);
        let item = unsafe { AVPlayerItem::playerItemWithURL(&url, mtm) };

        match self.player.as_deref() {
            Some(player) => {
                // Reuse the existing player — avoids CoreAudio graph teardown.
                unsafe {
                    player.replaceCurrentItemWithPlayerItem(Some(&item));
                    player.setVolume(volume);
                    if start_position > 0.0 {
                        let time = CMTime::with_seconds(start_position, 1_000);
                        player.seekToTime(time);
                    }
                    if (speed - 1.0).abs() < 0.01 {
                        player.play();
                    } else {
                        player.setRate(speed);
                    }
                }
                self.last_duration = item_duration(&item).unwrap_or(0.0);
                self.current_item = Some(item);
            }
            None => {
                let player =
                    unsafe { AVPlayer::initWithPlayerItem(AVPlayer::alloc(mtm), Some(&item)) };
                unsafe {
                    player.setVolume(volume);
                    if start_position > 0.0 {
                        let time = CMTime::with_seconds(start_position, 1_000);
                        player.seekToTime(time);
                    }
                    if (speed - 1.0).abs() < 0.01 {
                        player.play();
                    } else {
                        player.setRate(speed);
                    }
                }
                self.last_duration = item_duration(&item).unwrap_or(0.0);
                self.current_item = Some(item);
                self.player = Some(player);
            }
        }

        // Publish after the player/item swap is complete and playback has started.
        self.publish_now_playing_snapshot(MP_STATE_PLAYING);

        AvResp::Ok
    }

    fn stop(&mut self) {
        if let Some(player) = self.player.as_deref() {
            unsafe {
                player.pause();
                // Detach item without destroying the player or its audio session.
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

        const END_EPSILON: f64 = 0.2;
        let is_finished = dur > 0.0 && pos >= dur - END_EPSILON && rate == 0.0;

        AvSnapshot {
            position: pos,
            duration: dur,
            is_paused: rate == 0.0,
            is_finished,
            has_active: self.current_item.is_some(),
        }
    }

    // -----------------------------------------------------------------------
    // Now Playing publishing
    // -----------------------------------------------------------------------

    fn publish_now_playing_snapshot(&self, playback_state: u64) {
        let Some(player) = self.player.as_deref() else {
            return;
        };

        let elapsed = self.position_seconds();
        let duration = self.item_duration();
        let rate = unsafe { player.rate() } as f64;

        // SAFETY: called exclusively on the main thread via the dispatch
        // trampoline. The extern statics are stable MediaPlayer.framework
        // symbols. MPNowPlayingInfoCenter is documented as main-thread-only.
        unsafe {
            let center: *mut AnyObject = msg_send![class!(MPNowPlayingInfoCenter), defaultCenter];
            let dict: *mut AnyObject = msg_send![class!(NSMutableDictionary), dictionary];

            if !self.current_title.is_empty() {
                let title = NSString::from_str(&self.current_title);
                let _: () = msg_send![dict, setObject: &*title, forKey: MPMediaItemPropertyTitle];
            }

            if !self.current_artist.is_empty() {
                let artist = NSString::from_str(&self.current_artist);
                let _: () = msg_send![dict, setObject: &*artist, forKey: MPMediaItemPropertyArtist];
            }

            if duration.is_finite() && duration > 0.0 {
                let dur_num: *mut AnyObject =
                    msg_send![class!(NSNumber), numberWithDouble: duration];
                let _: () = msg_send![dict, setObject: dur_num,
                    forKey: MPMediaItemPropertyPlaybackDuration];
            }

            let elapsed_num: *mut AnyObject =
                msg_send![class!(NSNumber), numberWithDouble: elapsed.max(0.0)];
            let _: () = msg_send![dict, setObject: elapsed_num,
                forKey: MPNowPlayingInfoPropertyElapsedPlaybackTime];

            let rate_num: *mut AnyObject =
                msg_send![class!(NSNumber), numberWithDouble: rate.max(0.0)];
            let _: () = msg_send![dict, setObject: rate_num,
                forKey: MPNowPlayingInfoPropertyPlaybackRate];

            let _: () = msg_send![center, setNowPlayingInfo: dict];
            let _: () = msg_send![center, setPlaybackState: playback_state];
        }
    }

    /// Clears Now Playing and sets the stopped state. Called only from `stop`.
    ///
    /// Must be called before actor metadata fields are cleared, and on the
    /// main thread.
    fn erase_now_playing(&self) {
        // SAFETY: called exclusively on the main thread via the dispatch trampoline.
        unsafe {
            let center: *mut AnyObject = msg_send![class!(MPNowPlayingInfoCenter), defaultCenter];
            let nil_info: *mut AnyObject = std::ptr::null_mut();
            let _: () = msg_send![center, setNowPlayingInfo: nil_info];
            let _: () = msg_send![center, setPlaybackState: MP_STATE_STOPPED];
        }
    }
}

impl Drop for AvMainThreadActor {
    fn drop(&mut self) {
        self.stop();
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
// Main-queue handle and per-command dispatch
// ---------------------------------------------------------------------------

/// Thread-safe handle to the main-thread actor.  All field accesses are routed
/// through `dispatch_async_f` on `_dispatch_main_q`.
pub type ActorHandle = Arc<Mutex<Option<AvMainThreadActor>>>;

#[link(name = "System", kind = "framework")]
unsafe extern "C" {
    static _dispatch_main_q: c_void;
    fn dispatch_async_f(
        queue: *const c_void,
        context: *mut c_void,
        work: unsafe extern "C" fn(*mut c_void),
    );
}

fn main_queue() -> *const c_void {
    unsafe { &_dispatch_main_q as *const _ }
}

struct CmdPayload {
    handle: ActorHandle,
    cmd: AvCmd,
    resp: std::sync::mpsc::SyncSender<AvResp>,
}

unsafe extern "C" fn cmd_trampoline(ctx: *mut c_void) {
    // SAFETY: `ctx` is a `Box<CmdPayload>` leaked in `dispatch_cmd`.
    let payload = unsafe { Box::from_raw(ctx as *mut CmdPayload) };
    let mut guard = payload.handle.lock().unwrap();
    let actor = guard.get_or_insert_with(AvMainThreadActor::new);
    let resp = actor.handle_cmd(payload.cmd);
    let _ = payload.resp.send(resp);
}

/// Dispatch a command to the actor on the main queue and block until the
/// response arrives.  Returns `None` if the channel is closed.
pub fn dispatch_cmd(handle: &ActorHandle, cmd: AvCmd) -> Option<AvResp> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let payload = Box::new(CmdPayload {
        handle: Arc::clone(handle),
        cmd,
        resp: tx,
    });
    unsafe {
        dispatch_async_f(
            main_queue(),
            Box::into_raw(payload) as *mut c_void,
            cmd_trampoline,
        );
    }
    rx.recv().ok()
}

/// Create an empty `ActorHandle`.  The actor itself is lazily initialised on
/// the first command dispatched to the main queue.
pub fn start_on_main_queue() -> ActorHandle {
    Arc::new(Mutex::new(None))
}
