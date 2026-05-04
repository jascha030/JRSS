use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// Sets up the application menu with keyboard shortcuts.
/// Called during app setup in lib.rs.
pub fn setup_menu(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    {
        use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};

        // App menu items (first menu - shows app name on macOS)
        let settings_item = MenuItemBuilder::with_id("settings", "Settings")
            .accelerator("CmdOrCtrl+,")
            .build(app)?;

        // File menu items
        let add_feed_item = MenuItemBuilder::with_id("add-feed", "Add Feed")
            .accelerator("CmdOrCtrl+N")
            .build(app)?;
        let new_station_item = MenuItemBuilder::with_id("new-station", "New Station")
            .accelerator("CmdOrCtrl+Shift+N")
            .build(app)?;
        let refresh_item = MenuItemBuilder::with_id("refresh-feed", "Refresh Feed")
            .accelerator("CmdOrCtrl+R")
            .build(app)?;

        // Edit menu items
        let search_item = MenuItemBuilder::with_id("search-feed", "Search Feed")
            .accelerator("CmdOrCtrl+F")
            .build(app)?;

        // View menu items
        let go_to_feed_item = MenuItemBuilder::with_id("go-to-feed", "Go to Feed")
            .accelerator("CmdOrCtrl+L")
            .build(app)?;

        // Playback menu items
        // Note: Space as accelerator can interfere with text input.
        // This is handled in the frontend with focus checks.
        let play_pause_item = MenuItemBuilder::with_id("play-pause", "Play/Pause")
            .accelerator("Space")
            .build(app)?;
        let skip_forward_item = MenuItemBuilder::with_id("skip-forward", "Skip Forward")
            .accelerator("CmdOrCtrl+Shift+Right")
            .build(app)?;
        let skip_backward_item = MenuItemBuilder::with_id("skip-backward", "Skip Backward")
            .accelerator("CmdOrCtrl+Shift+Left")
            .build(app)?;
        let volume_up_item = MenuItemBuilder::with_id("volume-up", "Volume Up")
            .accelerator("CmdOrCtrl+Up")
            .build(app)?;
        let volume_down_item = MenuItemBuilder::with_id("volume-down", "Volume Down")
            .accelerator("CmdOrCtrl+Down")
            .build(app)?;

        // Create App submenu (first menu - shows as app name on macOS)
        let app_submenu = SubmenuBuilder::new(app, "JRSS")
            .item(&settings_item)
            .separator()
            .hide()
            .separator()
            .quit()
            .build()?;

        // Create File submenu
        let file_submenu = SubmenuBuilder::new(app, "File")
            .item(&add_feed_item)
            .item(&new_station_item)
            .item(&refresh_item)
            .separator()
            .close_window()
            .build()?;

        // Create Edit submenu
        let edit_submenu = SubmenuBuilder::new(app, "Edit")
            .undo()
            .redo()
            .separator()
            .cut()
            .copy()
            .paste()
            .select_all()
            .separator()
            .item(&search_item)
            .build()?;

        // Create View submenu
        let view_submenu = SubmenuBuilder::new(app, "View")
            .item(&go_to_feed_item)
            .build()?;

        // Create Playback submenu
        let playback_submenu = SubmenuBuilder::new(app, "Playback")
            .item(&play_pause_item)
            .separator()
            .item(&skip_backward_item)
            .item(&skip_forward_item)
            .separator()
            .item(&volume_up_item)
            .item(&volume_down_item)
            .build()?;

        // Create Window submenu with standard window controls
        let window_submenu = SubmenuBuilder::new(app, "Window")
            .minimize()
            .fullscreen()
            .separator()
            .close_window()
            .build()?;

        // Build the full menu
        let menu = MenuBuilder::new(app)
            .item(&app_submenu)
            .item(&file_submenu)
            .item(&edit_submenu)
            .item(&view_submenu)
            .item(&playback_submenu)
            .item(&window_submenu)
            .build()?;

        app.set_menu(menu)?;

        // Handle menu events - forward to frontend (emit globally so all windows receive it)
        app.on_menu_event(move |app_handle: &AppHandle, event| {
            let event_id = event.id().0.as_str();
            let event_name = format!("menu-{event_id}");
            let _ = app_handle.emit(&event_name, ());
        });

        // Register global shortcuts for modifier-based shortcuts (work in mini-player too)
        // Space is handled separately via JS since it has no modifier
        app.global_shortcut().on_shortcut(
            Shortcut::new(
                Some(
                    tauri_plugin_global_shortcut::Modifiers::META
                        | tauri_plugin_global_shortcut::Modifiers::SHIFT,
                ),
                tauri_plugin_global_shortcut::Code::ArrowRight,
            ),
            |_app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    let _ = _app.emit("menu-skip-forward", ());
                }
            },
        )?;

        app.global_shortcut().on_shortcut(
            Shortcut::new(
                Some(
                    tauri_plugin_global_shortcut::Modifiers::META
                        | tauri_plugin_global_shortcut::Modifiers::SHIFT,
                ),
                tauri_plugin_global_shortcut::Code::ArrowLeft,
            ),
            |_app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    let _ = _app.emit("menu-skip-backward", ());
                }
            },
        )?;

        app.global_shortcut().on_shortcut(
            Shortcut::new(
                Some(tauri_plugin_global_shortcut::Modifiers::META),
                tauri_plugin_global_shortcut::Code::ArrowUp,
            ),
            |_app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    let _ = _app.emit("menu-volume-up", ());
                }
            },
        )?;

        app.global_shortcut().on_shortcut(
            Shortcut::new(
                Some(tauri_plugin_global_shortcut::Modifiers::META),
                tauri_plugin_global_shortcut::Code::ArrowDown,
            ),
            |_app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    let _ = _app.emit("menu-volume-down", ());
                }
            },
        )?;

        // Settings shortcut (works in mini-player too)
        app.global_shortcut().on_shortcut(
            Shortcut::new(
                Some(tauri_plugin_global_shortcut::Modifiers::META),
                tauri_plugin_global_shortcut::Code::Comma,
            ),
            |_app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    let _ = _app.emit("menu-settings", ());
                }
            },
        )?;
    }

    Ok(())
}
