use super::AppResult;
use super::connection::open_connection;
use crate::models::{PlaybackContextRecord, PlaybackSessionRecord};
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use std::path::Path;

pub fn save_playback_session(db_path: &Path, session: &PlaybackSessionRecord) -> AppResult<()> {
    let connection = open_connection(db_path)?;
    let data_json = serde_json::to_string(session)
        .map_err(|error| format!("Failed to serialize playback session: {error}"))?;

    connection
        .execute(
            "INSERT INTO playback_session (id, data_json, updated_at)
		     VALUES (1, ?1, ?2)
		     ON CONFLICT(id) DO UPDATE SET
                data_json = excluded.data_json,
                updated_at = excluded.updated_at",
            params![data_json, Utc::now().to_rfc3339()],
        )
        .map_err(|error| format!("Failed to save playback session: {error}"))?;

    Ok(())
}

pub fn load_playback_session(db_path: &Path) -> AppResult<Option<PlaybackSessionRecord>> {
    let connection = open_connection(db_path)?;

    let json_opt: Option<String> = connection
        .query_row(
            "SELECT data_json FROM playback_session WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Failed to load playback session: {error}"))?;

    match json_opt {
        Some(json) => {
            let session: PlaybackSessionRecord = serde_json::from_str(&json)
                .map_err(|error| format!("Failed to deserialize playback session: {error}"))?;
            Ok(Some(session))
        }
        None => Ok(None),
    }
}

pub fn clear_playback_session(db_path: &Path) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute("DELETE FROM playback_session WHERE id = 1", [])
        .map_err(|error| format!("Failed to clear playback session: {error}"))?;

    Ok(())
}

pub fn save_playback_context(
    db_path: &Path,
    context: Option<&PlaybackContextRecord>,
) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    if let Some(ctx) = context {
        let json = serde_json::to_string(ctx)
            .map_err(|error| format!("Failed to serialize playback context: {error}"))?;
        connection
            .execute(
                "INSERT INTO playback_context (id, data_json, updated_at)
		         VALUES (1, ?1, datetime('now'))
		         ON CONFLICT(id) DO UPDATE SET
		         data_json = excluded.data_json,
		         updated_at = excluded.updated_at",
                params![json],
            )
            .map_err(|error| format!("Failed to save playback context: {error}"))?;
    } else {
        connection
            .execute("DELETE FROM playback_context WHERE id = 1", [])
            .map_err(|error| format!("Failed to clear playback context: {error}"))?;
    }

    Ok(())
}

pub fn load_playback_context(db_path: &Path) -> AppResult<Option<PlaybackContextRecord>> {
    let connection = open_connection(db_path)?;

    let result: Option<String> = connection
        .query_row(
            "SELECT data_json FROM playback_context WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Failed to load playback context: {error}"))?;

    match result {
        Some(json) => {
            let context: PlaybackContextRecord = serde_json::from_str(&json)
                .map_err(|error| format!("Failed to deserialize playback context: {error}"))?;
            Ok(Some(context))
        }
        None => Ok(None),
    }
}
