mod audio;
mod commands;
mod cover_art;
mod db;
mod feed_ingest;
mod models;
mod queue;
mod reader_extract;

use audio::AudioState;
use db::DatabaseState;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Hide window instead of closing on macOS
                #[cfg(target_os = "macos")]
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            let database_state = DatabaseState::new(app.handle())?;
            app.manage(database_state);

            let audio_state = AudioState::new(app.handle().clone())
                .map_err(Box::<dyn std::error::Error>::from)?;
            app.manage(audio_state);

            // Set up menu with keyboard shortcuts
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

                // Build the full menu
                let menu = MenuBuilder::new(app)
                    .item(&app_submenu)
                    .item(&file_submenu)
                    .item(&edit_submenu)
                    .item(&view_submenu)
                    .item(&playback_submenu)
                    .build()?;

                app.set_menu(menu)?;

                app.on_menu_event(move |app_handle: &tauri::AppHandle, event| {
                    let event_id = event.id().0.as_str();
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = match event_id {
                            "search-feed" => window.emit("menu-search-feed", ()),
                            "add-feed" => window.emit("menu-add-feed", ()),
                            "new-station" => window.emit("menu-new-station", ()),
                            "go-to-feed" => window.emit("menu-go-to-feed", ()),
                            "settings" => window.emit("menu-settings", ()),
                            "refresh-feed" => window.emit("menu-refresh-feed", ()),
                            "play-pause" => window.emit("menu-play-pause", ()),
                            "skip-forward" => window.emit("menu-skip-forward", ()),
                            "skip-backward" => window.emit("menu-skip-backward", ()),
                            "volume-up" => window.emit("menu-volume-up", ()),
                            "volume-down" => window.emit("menu-volume-down", ()),
                            _ => Ok(()),
                        };
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_feeds,
            commands::add_feed,
            commands::refresh_feed,
            commands::remove_feed,
            commands::query_items_page,
            commands::get_item_details,
            commands::mark_read,
            commands::save_playback,
            commands::load_reader_content,
            commands::get_items_by_ids,
            commands::save_playback_session,
            commands::load_playback_session,
            commands::clear_playback_session,
            commands::set_feed_sort_order,
            commands::list_stations,
            commands::create_station,
            commands::update_station,
            commands::delete_station,
            commands::query_station_episodes,
            commands::audio_play,
            commands::audio_pause,
            commands::audio_resume,
            commands::audio_toggle,
            commands::audio_stop,
            commands::audio_seek,
            commands::audio_set_volume,
            commands::audio_set_speed,
            commands::audio_get_state,
            commands::audio_list_output_devices,
            commands::audio_get_output_device,
            commands::audio_set_output_device,
            commands::audio_play_with_queue,
            commands::audio_queue_enqueue,
            commands::audio_queue_play_next,
            commands::audio_queue_remove,
            commands::audio_queue_move_up,
            commands::audio_queue_move_down,
            commands::audio_queue_clear,
            commands::audio_queue_get_state,
            commands::audio_queue_set,
            commands::save_playback_context,
            commands::load_playback_context,
            commands::extract_cover_palette
        ]);

    let app = builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        // Handle dock click to recreate window on macOS
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen {
            has_visible_windows,
            ..
        } = event
        {
            if !has_visible_windows {
                if let Some(window) = _app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
    });
}
