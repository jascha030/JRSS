use super::AppResult;
use rusqlite::Connection;
use std::path::Path;

pub fn open_connection(db_path: &Path) -> AppResult<Connection> {
    let connection = Connection::open(db_path)
        .map_err(|error| format!("Failed to open SQLite database: {error}"))?;

    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
			 PRAGMA journal_mode = WAL;",
        )
        .map_err(|error| format!("Failed to configure SQLite connection: {error}"))?;

    Ok(connection)
}
