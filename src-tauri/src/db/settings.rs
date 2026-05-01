use super::AppResult;
use super::connection::open_connection;
use crate::models::AppSettingsRecord;
use chrono::Utc;
use rusqlite::params;
use std::path::Path;

pub const DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES: i64 = 5 * 1024 * 1024 * 1024;
pub const DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP: bool = false;

fn normalize_max_audio_cache_size_bytes(value: i64) -> AppResult<i64> {
    if value <= 0 {
        return Err("Audio cache size must be greater than 0 bytes.".to_string());
    }
    Ok(value)
}

fn ensure_app_settings_row(connection: &rusqlite::Connection) -> AppResult<()> {
    connection
        .execute(
            "INSERT INTO app_settings (
		        id,
		        max_audio_cache_size_bytes,
		        mini_player_always_on_top,
		        updated_at
		     )
		     VALUES (1, ?1, ?2, ?3)
		     ON CONFLICT(id) DO NOTHING",
            params![
                DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES,
                DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP,
                Utc::now().to_rfc3339()
            ],
        )
        .map_err(|error| format!("Failed to ensure app settings row: {error}"))?;

    Ok(())
}

pub fn load_app_settings(db_path: &Path) -> AppResult<AppSettingsRecord> {
    let connection = open_connection(db_path)?;
    ensure_app_settings_row(&connection)?;

    let (max_audio_cache_size_bytes, mini_player_always_on_top) = connection
        .query_row(
            "SELECT max_audio_cache_size_bytes, mini_player_always_on_top
		     FROM app_settings
		     WHERE id = 1",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, bool>(1)?)),
        )
        .map_err(|error| format!("Failed to load app settings: {error}"))?;

    Ok(AppSettingsRecord {
        max_audio_cache_size_bytes: normalize_max_audio_cache_size_bytes(
            max_audio_cache_size_bytes,
        )?,
        mini_player_always_on_top,
    })
}

pub fn save_app_settings(
    db_path: &Path,
    settings: &AppSettingsRecord,
) -> AppResult<AppSettingsRecord> {
    let connection = open_connection(db_path)?;
    ensure_app_settings_row(&connection)?;

    let max_audio_cache_size_bytes =
        normalize_max_audio_cache_size_bytes(settings.max_audio_cache_size_bytes)?;
    let mini_player_always_on_top = settings.mini_player_always_on_top;

    connection
        .execute(
            "INSERT INTO app_settings (
		        id,
		        max_audio_cache_size_bytes,
		        mini_player_always_on_top,
		        updated_at
		     )
		     VALUES (1, ?1, ?2, ?3)
		     ON CONFLICT(id) DO UPDATE SET
		        max_audio_cache_size_bytes = excluded.max_audio_cache_size_bytes,
		        mini_player_always_on_top = excluded.mini_player_always_on_top,
		        updated_at = excluded.updated_at",
            params![
                max_audio_cache_size_bytes,
                mini_player_always_on_top,
                Utc::now().to_rfc3339()
            ],
        )
        .map_err(|error| format!("Failed to save app settings: {error}"))?;

    Ok(AppSettingsRecord {
        max_audio_cache_size_bytes,
        mini_player_always_on_top,
    })
}
