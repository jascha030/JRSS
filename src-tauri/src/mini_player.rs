//! Native macOS mini-player window transitions with AppKit.
//! Prevents fullscreen-space ghosting by observing NSWindowDidExitFullScreenNotification.

use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
use objc2::ffi::nil;

#[derive(Debug, Clone, Copy, PartialEq)]
enum TransitionPhase {
    Normal,
    OpeningMini,
    Mini,
    RestoringMain,
}

struct InnerState {
    phase: TransitionPhase,
    main_was_fullscreen: bool,
}

/// Guards mini-player transitions so overlapping open/restore calls race safely.
pub struct MiniPlayerTransitionState {
    inner: Mutex<InnerState>,
}

impl MiniPlayerTransitionState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InnerState {
                phase: TransitionPhase::Normal,
                main_was_fullscreen: false,
            }),
        }
    }
}

impl InnerState {
    fn begin_open(&mut self) -> Result<(), String> {
        match self.phase {
            TransitionPhase::Normal => {
                self.phase = TransitionPhase::OpeningMini;
                Ok(())
            }
            TransitionPhase::Mini => Ok(()),
            _ => Err(format!(
                "Mini-player transition already in progress ({:?}).",
                self.phase
            )),
        }
    }

    fn finish_open(&mut self, was_fullscreen: bool) {
        self.phase = TransitionPhase::Mini;
        self.main_was_fullscreen = was_fullscreen;
    }

    fn begin_restore(&mut self) -> Result<bool, String> {
        match self.phase {
            TransitionPhase::Mini => {
                self.phase = TransitionPhase::RestoringMain;
                Ok(self.main_was_fullscreen)
            }
            TransitionPhase::Normal => Ok(false),
            _ => Err(format!(
                "Cannot restore main window from phase {:?}.",
                self.phase
            )),
        }
    }

    fn finish_restore(&mut self) {
        self.phase = TransitionPhase::Normal;
        self.main_was_fullscreen = false;
    }

    fn reset_if_transitioning(&mut self) {
        self.phase = TransitionPhase::Normal;
        self.main_was_fullscreen = false;
    }
}

#[tauri::command]
pub async fn open_mini_player_native(
    app: AppHandle,
    state: tauri::State<'_, MiniPlayerTransitionState>,
    main_label: String,
    mini_label: String,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        {
            let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
            guard.begin_open()?;
        }

        let was_fullscreen = match tauri::async_runtime::spawn_blocking({
            let app = app.clone();
            let main_label = main_label.clone();
            let mini_label = mini_label.clone();
            move || open_mini_player_native_macos(&app, &main_label, &mini_label)
        })
        .await
        {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
                guard.reset_if_transitioning();
                return Err(e);
            }
            Err(e) => {
                let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
                guard.reset_if_transitioning();
                return Err(format!("Task failed: {e}"));
            }
        };

        {
            let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
            guard.finish_open(was_fullscreen);
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = state;
        let mini = app
            .get_webview_window(&mini_label)
            .ok_or_else(|| format!("Window '{mini_label}' not found."))?;
        let main = app
            .get_webview_window(&main_label)
            .ok_or_else(|| format!("Window '{main_label}' not found."))?;
        mini.show().map_err(|e| e.to_string())?;
        main.hide().map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[tauri::command]
pub async fn restore_main_window_native(
    app: AppHandle,
    state: tauri::State<'_, MiniPlayerTransitionState>,
    main_label: String,
    mini_label: String,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let was_fullscreen = {
            let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
            guard.begin_restore()?
        };

        let result = match tauri::async_runtime::spawn_blocking({
            let app = app.clone();
            let main_label = main_label.clone();
            let mini_label = mini_label.clone();
            move || restore_main_window_native_macos(&app, &main_label, &mini_label, was_fullscreen)
        })
        .await
        {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
                guard.reset_if_transitioning();
                Err(e)
            }
            Err(e) => {
                let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
                guard.reset_if_transitioning();
                Err(format!("Task failed: {e}"))
            }
        };

        if result.is_ok() {
            let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
            guard.finish_restore();
        }

        result
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = state;
        let main = app
            .get_webview_window(&main_label)
            .ok_or_else(|| format!("Window '{main_label}' not found."))?;
        let mini = app
            .get_webview_window(&mini_label)
            .ok_or_else(|| format!("Window '{mini_label}' not found."))?;
        main.show().map_err(|e| e.to_string())?;
        let _ = mini.destroy().map_err(|e| e.to_string());
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn open_mini_player_native_macos(
    app: &AppHandle,
    main_label: &str,
    mini_label: &str,
) -> Result<bool, String> {
    use block2::RcBlock;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};

    let main_window = app
        .get_webview_window(main_label)
        .ok_or_else(|| format!("Window '{main_label}' not found."))?;
    let mini_window = app
        .get_webview_window(mini_label)
        .ok_or_else(|| format!("Window '{mini_label}' not found."))?;

    let main_ns = main_window.ns_window().map_err(|e| e.to_string())? as *mut AnyObject;
    let mini_ns = mini_window.ns_window().map_err(|e| e.to_string())? as *mut AnyObject;

    let style_mask: usize = unsafe { msg_send![main_ns, styleMask] };
    let is_fullscreen = (style_mask & (1 << 14)) != 0;

    if is_fullscreen {
        let notification_name: *mut AnyObject = unsafe {
            msg_send![
                class!(NSString),
                stringWithUTF8String: b"NSWindowDidExitFullScreenNotification\0".as_ptr() as *const i8
            ]
        };
        let center: *mut AnyObject =
            unsafe { msg_send![class!(NSNotificationCenter), defaultCenter] };

        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let observer_cell = std::sync::Arc::new(Mutex::<Option<*mut AnyObject>>::new(None));
        let observer_cell_for_block = observer_cell.clone();

        let block = RcBlock::new(move |_notification: *mut AnyObject| {
            if let Ok(guard) = observer_cell_for_block.lock() {
                if let Some(obs) = *guard {
                    let _: () = unsafe { msg_send![center, removeObserver: obs] };
                }
            }

            let main_queue: *mut AnyObject =
                unsafe { msg_send![class!(NSOperationQueue), mainQueue] };
            let inner_tx = tx.clone();
            let inner_block = RcBlock::new(move || {
                let _: () = unsafe { msg_send![main_ns, orderOut: nil] };
                let _: () = unsafe { msg_send![mini_ns, makeKeyAndOrderFront: nil] };
                let ns_app: *mut AnyObject =
                    unsafe { msg_send![class!(NSApplication), sharedApplication] };
                let _: () = unsafe { msg_send![ns_app, activateIgnoringOtherApps: true] };
                let _ = inner_tx.send(());
            });
            let op: *mut AnyObject = unsafe {
                msg_send![class!(NSBlockOperation), blockOperationWithBlock: &*inner_block]
            };
            let _: () = unsafe { msg_send![main_queue, addOperation: op] };
        });

        let observer: *mut AnyObject = unsafe {
            msg_send![
                center,
                addObserverForName: notification_name,
                object: main_ns,
                queue: nil,
                usingBlock: &*block
            ]
        };

        if let Ok(mut guard) = observer_cell.lock() {
            *guard = Some(observer);
        }

        let main_queue: *mut AnyObject = unsafe { msg_send![class!(NSOperationQueue), mainQueue] };
        let toggle_block = RcBlock::new(move || {
            let _: () = unsafe { msg_send![main_ns, toggleFullScreen: nil] };
        });
        let toggle_op: *mut AnyObject =
            unsafe { msg_send![class!(NSBlockOperation), blockOperationWithBlock: &*toggle_block] };
        let _: () = unsafe { msg_send![main_queue, addOperation: toggle_op] };

        rx.recv_timeout(std::time::Duration::from_secs(15))
            .map_err(|e| format!("Timed out waiting for fullscreen exit: {e}"))?;
    } else {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let main_queue: *mut AnyObject = unsafe { msg_send![class!(NSOperationQueue), mainQueue] };
        let block = RcBlock::new(move || {
            let _: () = unsafe { msg_send![main_ns, orderOut: nil] };
            let _: () = unsafe { msg_send![mini_ns, makeKeyAndOrderFront: nil] };
            let ns_app: *mut AnyObject =
                unsafe { msg_send![class!(NSApplication), sharedApplication] };
            let _: () = unsafe { msg_send![ns_app, activateIgnoringOtherApps: true] };
            let _ = tx.send(());
        });
        let op: *mut AnyObject =
            unsafe { msg_send![class!(NSBlockOperation), blockOperationWithBlock: &*block] };
        let _: () = unsafe { msg_send![main_queue, addOperation: op] };

        rx.recv_timeout(std::time::Duration::from_secs(10))
            .map_err(|e| format!("Timed out waiting for window transition: {e}"))?;
    }

    Ok(is_fullscreen)
}

#[cfg(target_os = "macos")]
fn restore_main_window_native_macos(
    app: &AppHandle,
    main_label: &str,
    mini_label: &str,
    was_fullscreen: bool,
) -> Result<(), String> {
    use block2::RcBlock;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};

    let main_window = app
        .get_webview_window(main_label)
        .ok_or_else(|| format!("Window '{main_label}' not found."))?;
    let mini_window = app
        .get_webview_window(mini_label)
        .ok_or_else(|| format!("Window '{mini_label}' not found."))?;

    let main_ns = main_window.ns_window().map_err(|e| e.to_string())? as *mut AnyObject;
    let mini_ns = mini_window.ns_window().map_err(|e| e.to_string())? as *mut AnyObject;

    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let main_queue: *mut AnyObject = unsafe { msg_send![class!(NSOperationQueue), mainQueue] };
    let block = RcBlock::new(move || {
        let _: () = unsafe { msg_send![main_ns, makeKeyAndOrderFront: nil] };
        let ns_app: *mut AnyObject = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let _: () = unsafe { msg_send![ns_app, activateIgnoringOtherApps: true] };

        if was_fullscreen {
            let _: () = unsafe { msg_send![main_ns, toggleFullScreen: nil] };
        }

        let _: () = unsafe { msg_send![mini_ns, orderOut: nil] };
        let _ = tx.send(());
    });
    let op: *mut AnyObject =
        unsafe { msg_send![class!(NSBlockOperation), blockOperationWithBlock: &*block] };
    let _: () = unsafe { msg_send![main_queue, addOperation: op] };

    rx.recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|e| format!("Timed out waiting for window restore: {e}"))?;

    let _ = mini_window.destroy().map_err(|e| e.to_string());

    Ok(())
}
