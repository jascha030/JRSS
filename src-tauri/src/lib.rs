mod audio;
mod commands;
mod db;
mod feed_ingest;
mod models;
mod queue;
mod reader_extract;

use audio::AudioState;
use db::DatabaseState;
use tauri::Manager;

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
            commands::audio_queue_set
        ]);

    let app = builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        // Handle dock click to recreate window on macOS
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen {
            has_visible_windows, ..
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
