mod audio;
mod commands;
mod db;
mod feed_ingest;
mod models;
mod reader_extract;

use audio::AudioState;
use db::DatabaseState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
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
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
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
            commands::audio_get_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
