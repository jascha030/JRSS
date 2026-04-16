mod commands;
mod db;
mod feed_ingest;
mod models;
mod reader_extract;

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
            commands::set_feed_sort_order
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
