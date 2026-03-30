mod commands;
mod db;
mod feed_ingest;
mod models;

use db::DatabaseState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
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
            commands::list_items,
            commands::mark_read,
            commands::save_playback
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
