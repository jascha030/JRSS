//! Native macOS mini-player window transitions with AppKit.
//! Prevents fullscreen-space ghosting by observing NSWindowDidExitFullScreenNotification.

use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewWindow};

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

// CGSize ABI on 64-bit macOS: {CGFloat CGFloat} = {double double}.
#[cfg(target_os = "macos")]
#[repr(C)]
struct CGSize {
    width: f64,
    height: f64,
}

// SAFETY: CGSize is `{CGFloat CGFloat}` in Objective-C on 64-bit, matching this repr(C) layout.
#[cfg(target_os = "macos")]
unsafe impl objc2::Encode for CGSize {
    const ENCODING: objc2::Encoding =
        objc2::Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

#[cfg(target_os = "macos")]
fn set_macos_window_content_aspect_ratio(
    window: &WebviewWindow,
    width: f64,
    height: f64,
) -> Result<(), String> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let ns_window = window.ns_window().map_err(|error| error.to_string())? as *mut AnyObject;
    let size = CGSize { width, height };
    unsafe { msg_send![ns_window, setContentAspectRatio: size] }

    Ok(())
}

#[cfg(target_os = "macos")]
fn clear_macos_window_content_aspect_ratio(window: &WebviewWindow) -> Result<(), String> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let ns_window = window.ns_window().map_err(|error| error.to_string())? as *mut AnyObject;
    let increments = CGSize {
        width: 1.0,
        height: 1.0,
    };
    // SAFETY: Calling setResizeIncrements: with {1,1} restores freeform resizing, which
    // is the standard way to remove an aspect-ratio lock on AppKit.
    unsafe { msg_send![ns_window, setResizeIncrements: increments] }

    Ok(())
}

#[cfg(target_os = "macos")]
fn is_macos_window_in_live_resize(window: &WebviewWindow) -> Result<bool, String> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let ns_window = window.ns_window().map_err(|error| error.to_string())? as *mut AnyObject;
    // SAFETY: inLiveResize is a standard BOOL property on NSWindow.
    let in_live: bool = unsafe { msg_send![ns_window, inLiveResize] };

    Ok(in_live)
}

#[cfg(not(target_os = "macos"))]
fn set_macos_window_content_aspect_ratio(
    _window: &WebviewWindow,
    _width: f64,
    _height: f64,
) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn clear_macos_window_content_aspect_ratio(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn is_macos_window_in_live_resize(_window: &WebviewWindow) -> Result<bool, String> {
    Ok(false)
}

const MIN_WIDTH: f64 = 340.0;
const COMPACT_HEIGHT: f64 = 170.0;
const EXPANDED_MAX: f64 = 800.0;
const EXPANDED_MIN_HEIGHT: f64 = 340.0;
const TRANSITION_MIN_HEIGHT: f64 = 80.0;

fn current_logical_width(window: &WebviewWindow) -> Result<f64, String> {
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    Ok(size.to_logical::<f64>(scale).width)
}

fn apply_compact_mode(window: &WebviewWindow, width: f64) -> Result<(), String> {
    clear_macos_window_content_aspect_ratio(window)?;

    set_min_size(window, MIN_WIDTH, COMPACT_HEIGHT)?;
    set_size(window, width, COMPACT_HEIGHT)?;
    set_max_size(window, EXPANDED_MAX, COMPACT_HEIGHT)?;

    Ok(())
}

fn apply_expanded_mode(window: &WebviewWindow, width: f64) -> Result<(), String> {
    set_max_size(window, EXPANDED_MAX, EXPANDED_MAX)?;
    set_min_size(window, MIN_WIDTH, TRANSITION_MIN_HEIGHT)?;
    set_size(window, width, width)?;

    set_macos_window_content_aspect_ratio(window, 1.0, 1.0)?;

    set_min_size(window, MIN_WIDTH, EXPANDED_MIN_HEIGHT)?;

    Ok(())
}

fn set_size(window: &WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    window
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())
}

fn set_min_size(window: &WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    window
        .set_min_size(Some(tauri::LogicalSize::new(width, height)))
        .map_err(|e| e.to_string())
}

fn set_max_size(window: &WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    window
        .set_max_size(Some(tauri::LogicalSize::new(width, height)))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_window_content_aspect_ratio(
    app: tauri::AppHandle,
    label: String,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Window '{label}' not found."))?;

    set_macos_window_content_aspect_ratio(&window, width, height)?;

    Ok(())
}

#[tauri::command]
pub fn resize_mini_player(
    app: tauri::AppHandle,
    label: String,
    compact: bool,
) -> Result<(), String> {
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Window '{label}' not found."))?;

    if is_macos_window_in_live_resize(&window)? {
        return Ok(());
    }

    let width = current_logical_width(&window)?.max(MIN_WIDTH);

    if compact {
        apply_compact_mode(&window, width)?;
    } else {
        apply_expanded_mode(&window, width)?;
    }

    Ok(())
}
