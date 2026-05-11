//! macOS-native helpers for the audio subsystem.

#![allow(unexpected_cfgs)]

// ---------------------------------------------------------------------------
// Power assertion — prevents App Nap and system sleep during playback
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub mod power {
	use cocoa::base::{id, nil};
	use cocoa::foundation::NSString;
	use objc::{class, msg_send, sel, sel_impl};

	const NS_ACTIVITY_IDLE_SYSTEM_SLEEP_DISABLED: usize = 1 << 20;
	const NS_ACTIVITY_IDLE_DISPLAY_SLEEP_DISABLED: usize = 1 << 40;
	const NS_ACTIVITY_USER_INITIATED: usize = 0x00FFFFFF;

	/// Wraps an `NSProcessInfo` activity token that prevents idle sleep.
	/// Dropping the guard ends the activity.
	pub struct PowerAssertion(id);

	impl PowerAssertion {
		pub fn new(reason: &str) -> Self {
			unsafe {
				let process_info: id = msg_send![class!(NSProcessInfo), processInfo];
				let reason_ns = NSString::alloc(nil).init_str(reason);
				let options = NS_ACTIVITY_USER_INITIATED
					| NS_ACTIVITY_IDLE_SYSTEM_SLEEP_DISABLED
					| NS_ACTIVITY_IDLE_DISPLAY_SLEEP_DISABLED;
				let activity: id = msg_send![process_info,
					beginActivityWithOptions:options
					reason:reason_ns
				];
				let _: () = msg_send![reason_ns, release];
				Self(activity)
			}
		}
	}

	impl Drop for PowerAssertion {
		fn drop(&mut self) {
			unsafe {
				let process_info: id = msg_send![class!(NSProcessInfo), processInfo];
				let _: () = msg_send![process_info, endActivity:self.0];
			}
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


