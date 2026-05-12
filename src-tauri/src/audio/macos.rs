//! macOS-native helpers for the audio subsystem.

// ---------------------------------------------------------------------------
// Power assertion — prevents App Nap and system sleep during playback
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub mod power {
    use objc2::rc::Retained;
    use objc2::runtime::ProtocolObject;
    use objc2_foundation::{NSActivityOptions, NSObjectProtocol, NSProcessInfo, NSString};

    const NS_ACTIVITY_IDLE_SYSTEM_SLEEP_DISABLED: u64 = 1 << 20;
    const NS_ACTIVITY_IDLE_DISPLAY_SLEEP_DISABLED: u64 = 1 << 40;
    const NS_ACTIVITY_USER_INITIATED: u64 = 0x00FF_FFFF;

    /// Wraps an `NSProcessInfo` activity token that prevents idle sleep.
    /// Dropping the guard ends the activity.
    pub struct PowerAssertion(Retained<ProtocolObject<dyn NSObjectProtocol>>);

    impl PowerAssertion {
        pub fn new(reason: &str) -> Self {
            let info = NSProcessInfo::processInfo();
            let reason_ns = NSString::from_str(reason);
            let options = NSActivityOptions(
                NS_ACTIVITY_USER_INITIATED
                    | NS_ACTIVITY_IDLE_SYSTEM_SLEEP_DISABLED
                    | NS_ACTIVITY_IDLE_DISPLAY_SLEEP_DISABLED,
            );
            let activity =
                info.beginActivityWithOptions_reason(options, &reason_ns);
            Self(activity)
        }
    }

    impl Drop for PowerAssertion {
        fn drop(&mut self) {
            let info = NSProcessInfo::processInfo();
            unsafe { info.endActivity(&self.0) };
        }
    }
}

// ---------------------------------------------------------------------------
// Thread QoS — elevate the audio thread so decoder callbacks aren't delayed
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
    pub fn set_audio_thread_qos() {
    use libc::qos_class_t::QOS_CLASS_USER_INITIATED;
    unsafe {
        libc::pthread_set_qos_class_self_np(QOS_CLASS_USER_INITIATED, 0);
    }
}

// ---------------------------------------------------------------------------
// MPRemoteCommandCenter — hardware/headphone media key handlers
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub mod remote_commands {
    use std::sync::mpsc;
    use std::sync::OnceLock;

    use block2::RcBlock;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};

    use crate::audio::commands::AudioCommand;

    const HANDLER_SUCCESS: isize = 0;
    const SKIP_SECONDS: f64 = 15.0;

    static INSTALLED: OnceLock<()> = OnceLock::new();

    /// Registers `MPRemoteCommandCenter` handlers that forward media key events
    /// to the audio thread.
    ///
    /// Must be called from the main thread — Tauri's `setup()` satisfies this.
    /// Idempotent: subsequent calls are no-ops (logged as warnings).
    pub fn install(tx: mpsc::Sender<AudioCommand>) {
        if INSTALLED.set(()).is_err() {
            log::warn!("[remote_commands] install() called more than once — skipping duplicate registration");
            return;
        }
        // SAFETY: called from the main thread (Tauri setup); all pointers are
        // non-null singletons from class lookups; blocks are intentionally leaked
        // so ObjC retains them for the app lifetime via addTargetWithHandler:.
        unsafe { register_all(tx) };
        log::info!("[remote_commands] MPRemoteCommandCenter handlers registered");
    }

    unsafe fn register_all(tx: mpsc::Sender<AudioCommand>) {
        let center: *mut AnyObject =
            msg_send![class!(MPRemoteCommandCenter), sharedCommandCenter];

        // MPSkipIntervalCommand requires preferredIntervals to be non-empty or
        // the system may not expose the skip commands in Control Center / AirPods.
        let interval_num: *mut AnyObject =
            msg_send![class!(NSNumber), numberWithDouble: SKIP_SECONDS];
        let intervals_arr: *mut AnyObject =
            msg_send![class!(NSArray), arrayWithObject: interval_num];

        {
            let tx = tx.clone();
            let block = RcBlock::new(move |_event: *mut AnyObject| -> isize {
                log::debug!("[remote_commands] play → Resume");
                let _ = tx.send(AudioCommand::Resume);
                HANDLER_SUCCESS
            });
            let cmd: *mut AnyObject = msg_send![center, playCommand];
            let _: () = msg_send![cmd, addTargetWithHandler: &*block];
            std::mem::forget(block);
        }

        {
            let tx = tx.clone();
            let block = RcBlock::new(move |_event: *mut AnyObject| -> isize {
                log::debug!("[remote_commands] pause → Pause");
                let _ = tx.send(AudioCommand::Pause);
                HANDLER_SUCCESS
            });
            let cmd: *mut AnyObject = msg_send![center, pauseCommand];
            let _: () = msg_send![cmd, addTargetWithHandler: &*block];
            std::mem::forget(block);
        }

        {
            let tx = tx.clone();
            let block = RcBlock::new(move |_event: *mut AnyObject| -> isize {
                log::debug!("[remote_commands] togglePlayPause → TogglePlayback");
                let _ = tx.send(AudioCommand::TogglePlayback);
                HANDLER_SUCCESS
            });
            let cmd: *mut AnyObject = msg_send![center, togglePlayPauseCommand];
            let _: () = msg_send![cmd, addTargetWithHandler: &*block];
            std::mem::forget(block);
        }

        {
            let tx = tx.clone();
            let block = RcBlock::new(move |_event: *mut AnyObject| -> isize {
                log::debug!("[remote_commands] stop → Stop");
                let _ = tx.send(AudioCommand::Stop);
                HANDLER_SUCCESS
            });
            let cmd: *mut AnyObject = msg_send![center, stopCommand];
            let _: () = msg_send![cmd, addTargetWithHandler: &*block];
            std::mem::forget(block);
        }

        {
            let tx = tx.clone();
            let block = RcBlock::new(move |_event: *mut AnyObject| -> isize {
                log::debug!("[remote_commands] skipForward → SkipForward({SKIP_SECONDS}s)");
                let _ = tx.send(AudioCommand::SkipForward {
                    delta_seconds: SKIP_SECONDS,
                });
                HANDLER_SUCCESS
            });
            let cmd: *mut AnyObject = msg_send![center, skipForwardCommand];
            let _: () = msg_send![cmd, setPreferredIntervals: intervals_arr];
            let _: () = msg_send![cmd, addTargetWithHandler: &*block];
            std::mem::forget(block);
        }

        {
            let tx = tx.clone();
            let block = RcBlock::new(move |_event: *mut AnyObject| -> isize {
                log::debug!("[remote_commands] skipBackward → SkipBackward({SKIP_SECONDS}s)");
                let _ = tx.send(AudioCommand::SkipBackward {
                    delta_seconds: SKIP_SECONDS,
                });
                HANDLER_SUCCESS
            });
            let cmd: *mut AnyObject = msg_send![center, skipBackwardCommand];
            let _: () = msg_send![cmd, setPreferredIntervals: intervals_arr];
            let _: () = msg_send![cmd, addTargetWithHandler: &*block];
            std::mem::forget(block);
        }

        {
            let tx = tx.clone();
            let block = RcBlock::new(move |event: *mut AnyObject| -> isize {
                let position_seconds: f64 = msg_send![event, positionTime];
                log::debug!("[remote_commands] changePlaybackPosition → Seek({position_seconds:.2}s)");
                let _ = tx.send(AudioCommand::Seek { position_seconds });
                HANDLER_SUCCESS
            });
            let cmd: *mut AnyObject = msg_send![center, changePlaybackPositionCommand];
            let _: () = msg_send![cmd, addTargetWithHandler: &*block];
            std::mem::forget(block);
        }
    }
}
