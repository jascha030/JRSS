use super::AppResult;
use super::connection::open_connection;
use rusqlite::Connection;
use std::path::Path;

pub fn initialize_database(db_path: &Path) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
		.execute_batch(
			"CREATE TABLE IF NOT EXISTS feeds (
			 	id TEXT PRIMARY KEY,
			 	url TEXT NOT NULL UNIQUE,
			 	title TEXT NOT NULL,
			 	description TEXT NOT NULL,
			 	kind TEXT NOT NULL CHECK(kind IN ('article', 'media')),
			 	site_url TEXT,
			 	created_at TEXT NOT NULL,
			 	last_fetched_at TEXT
			 );

			 CREATE TABLE IF NOT EXISTS items (
			 	id TEXT PRIMARY KEY,
			 	feed_id TEXT NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
			 	external_id TEXT NOT NULL,
			 	title TEXT NOT NULL,
			 	url TEXT NOT NULL,
			 	summary TEXT NOT NULL,
			 	preview_text TEXT NOT NULL DEFAULT '',
			 	summary_text TEXT,
			 	summary_html TEXT,
			 	content_text TEXT,
			 	content_html TEXT,
			 	reader_status TEXT NOT NULL DEFAULT 'unfetched' CHECK(reader_status IN ('unfetched', 'ready', 'failed')),
			 	reader_title TEXT,
			 	reader_byline TEXT,
			 	reader_excerpt TEXT,
			 	reader_content_html TEXT,
			 	reader_content_text TEXT,
			 	reader_fetched_at TEXT,
			 	published_at TEXT NOT NULL,
			 	read INTEGER NOT NULL DEFAULT 0,
			 	favorite INTEGER NOT NULL DEFAULT 0,
			 	enclosure_url TEXT,
			 	enclosure_mime_type TEXT,
			 	enclosure_size_bytes INTEGER,
			 	enclosure_duration_seconds INTEGER,
			 	UNIQUE(feed_id, external_id)
			 );

			 CREATE TABLE IF NOT EXISTS playback_state (
			 	item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
			 	position_seconds INTEGER NOT NULL DEFAULT 0,
			 	updated_at TEXT NOT NULL
			 );

			 CREATE TABLE IF NOT EXISTS playback_session (
			 	id INTEGER PRIMARY KEY CHECK(id = 1),
			 	data_json TEXT NOT NULL,
			 	updated_at TEXT NOT NULL
			 );

			 CREATE TABLE IF NOT EXISTS playback_context (
			 	id INTEGER PRIMARY KEY CHECK(id = 1),
			 	data_json TEXT NOT NULL,
			 	updated_at TEXT NOT NULL
			 );

			 CREATE TABLE IF NOT EXISTS app_settings (
		 		id INTEGER PRIMARY KEY CHECK(id = 1),
		 		max_audio_cache_size_bytes INTEGER NOT NULL,
		 		mini_player_always_on_top INTEGER NOT NULL DEFAULT 0,
		 		auto_refresh_interval_minutes INTEGER NOT NULL DEFAULT 60,
		 		skip_forward_seconds INTEGER NOT NULL DEFAULT 15,
		 		skip_backward_seconds INTEGER NOT NULL DEFAULT 15,
		 		updated_at TEXT NOT NULL
		 	 );

			 CREATE INDEX IF NOT EXISTS idx_items_feed_id_published_at_id
			 	ON items(feed_id, published_at DESC, id DESC);
			 CREATE INDEX IF NOT EXISTS idx_items_published_at_id
			 	ON items(published_at DESC, id DESC);
			 CREATE INDEX IF NOT EXISTS idx_items_unread_published_at_id
			 	ON items(published_at DESC, id DESC)
			 	WHERE read = 0;
			 CREATE INDEX IF NOT EXISTS idx_items_podcast_published_at_id
			 	ON items(published_at DESC, id DESC)
			 	WHERE enclosure_url IS NOT NULL;",
		)
		.map_err(|error| format!("Failed to initialize SQLite schema: {error}"))?;

    ensure_item_content_columns(&connection)?;
    ensure_item_favorite_column(&connection)?;
    ensure_item_favorite_index(&connection)?;
    ensure_item_image_url_column(&connection)?;
    ensure_feed_sort_order_column(&connection)?;
    ensure_feed_image_url_column(&connection)?;
    backfill_preview_text(&connection)?;
    migrate_feed_kind_values(&connection)?;
    ensure_stations_tables(&connection)?;
    ensure_app_settings_columns(&connection)?;
    super::settings::ensure_app_settings_row(&connection)?;

    Ok(())
}

fn ensure_app_settings_columns(connection: &Connection) -> AppResult<()> {
    let mut statement = connection
        .prepare("PRAGMA table_info(app_settings)")
        .map_err(|error| format!("Failed to inspect SQLite app settings columns: {error}"))?;
    let existing_columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to read SQLite app settings columns: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to collect SQLite app settings columns: {error}"))?;

    if existing_columns
        .iter()
        .all(|column| column != "mini_player_always_on_top")
    {
        connection
			.execute(
				"ALTER TABLE app_settings ADD COLUMN mini_player_always_on_top INTEGER NOT NULL DEFAULT 0",
				[]
			)
			.map_err(|error| {
				format!("Failed to add SQLite app settings mini player column: {error}")
			})?;
    }

    if existing_columns
        .iter()
        .all(|column| column != "auto_refresh_interval_minutes")
    {
        connection
			.execute(
				"ALTER TABLE app_settings ADD COLUMN auto_refresh_interval_minutes INTEGER NOT NULL DEFAULT 60",
				[]
			)
			.map_err(|error| {
				format!("Failed to add SQLite app settings auto refresh column: {error}")
			})?;
    }

    if existing_columns
        .iter()
        .all(|column| column != "color_scheme")
    {
        connection
            .execute(
                "ALTER TABLE app_settings ADD COLUMN color_scheme TEXT NOT NULL DEFAULT 'system'",
                [],
            )
            .map_err(|error| {
                format!("Failed to add SQLite app settings color_scheme column: {error}")
            })?;
    }

    if existing_columns
        .iter()
        .all(|column| column != "accent_color")
    {
        connection
            .execute("ALTER TABLE app_settings ADD COLUMN accent_color TEXT", [])
            .map_err(|error| {
                format!("Failed to add SQLite app settings accent_color column: {error}")
            })?;
    }

    if existing_columns
        .iter()
        .all(|column| column != "skip_forward_seconds")
    {
        connection
            .execute(
                "ALTER TABLE app_settings ADD COLUMN skip_forward_seconds INTEGER NOT NULL DEFAULT 15",
                [],
            )
            .map_err(|error| {
                format!("Failed to add SQLite app settings skip_forward_seconds column: {error}")
            })?;
    }

    if existing_columns
        .iter()
        .all(|column| column != "skip_backward_seconds")
    {
        connection
            .execute(
                "ALTER TABLE app_settings ADD COLUMN skip_backward_seconds INTEGER NOT NULL DEFAULT 15",
                [],
            )
            .map_err(|error| {
                format!("Failed to add SQLite app settings skip_backward_seconds column: {error}")
            })?;
    }

    if existing_columns.iter().all(|column| column != "theme_name") {
        connection
            .execute("ALTER TABLE app_settings ADD COLUMN theme_name TEXT", [])
            .map_err(|error| {
                format!("Failed to add SQLite app settings theme_name column: {error}")
            })?;
    }

    Ok(())
}

fn ensure_item_content_columns(connection: &Connection) -> AppResult<()> {
    let mut statement = connection
        .prepare("PRAGMA table_info(items)")
        .map_err(|error| format!("Failed to inspect SQLite item columns: {error}"))?;
    let existing_columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to read SQLite item columns: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to collect SQLite item columns: {error}"))?;

    let columns_to_add = [
        (
            "preview_text",
            "ALTER TABLE items ADD COLUMN preview_text TEXT NOT NULL DEFAULT ''",
        ),
        (
            "summary_text",
            "ALTER TABLE items ADD COLUMN summary_text TEXT",
        ),
        (
            "summary_html",
            "ALTER TABLE items ADD COLUMN summary_html TEXT",
        ),
        (
            "content_text",
            "ALTER TABLE items ADD COLUMN content_text TEXT",
        ),
        (
            "content_html",
            "ALTER TABLE items ADD COLUMN content_html TEXT",
        ),
        (
            "reader_status",
            "ALTER TABLE items ADD COLUMN reader_status TEXT NOT NULL DEFAULT 'unfetched' CHECK(reader_status IN ('unfetched', 'ready', 'failed'))",
        ),
        (
            "reader_title",
            "ALTER TABLE items ADD COLUMN reader_title TEXT",
        ),
        (
            "reader_byline",
            "ALTER TABLE items ADD COLUMN reader_byline TEXT",
        ),
        (
            "reader_excerpt",
            "ALTER TABLE items ADD COLUMN reader_excerpt TEXT",
        ),
        (
            "reader_content_html",
            "ALTER TABLE items ADD COLUMN reader_content_html TEXT",
        ),
        (
            "reader_content_text",
            "ALTER TABLE items ADD COLUMN reader_content_text TEXT",
        ),
        (
            "reader_fetched_at",
            "ALTER TABLE items ADD COLUMN reader_fetched_at TEXT",
        ),
    ];

    for (column, sql) in columns_to_add {
        if !existing_columns.iter().any(|c| c == column) {
            connection
                .execute(sql, [])
                .map_err(|error| format!("Failed to add items.{column} column: {error}"))?;
        }
    }

    connection
		.execute(
			"UPDATE items SET reader_status = ?1 WHERE reader_status IS NULL OR trim(reader_status) = ''",
			["unfetched"],
		)
		.map_err(|error| format!("Failed to backfill items.reader_status values: {error}"))?;

    Ok(())
}

fn ensure_item_favorite_column(connection: &Connection) -> AppResult<()> {
    let mut statement = connection
        .prepare("PRAGMA table_info(items)")
        .map_err(|error| format!("Failed to inspect SQLite item columns: {error}"))?;
    let existing_columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to read SQLite item columns: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to collect SQLite item columns: {error}"))?;

    if !existing_columns.iter().any(|column| column == "favorite") {
        connection
            .execute(
                "ALTER TABLE items ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0",
                [],
            )
            .map_err(|error| format!("Failed to add items.favorite column: {error}"))?;
    }

    Ok(())
}

fn ensure_item_favorite_index(connection: &Connection) -> AppResult<()> {
    connection
		.execute(
			"CREATE INDEX IF NOT EXISTS idx_items_favorite_published_at_id ON items(published_at DESC, id DESC) WHERE favorite = 1",
			[]
		)
		.map_err(|error| format!("Failed to create favorite items index: {error}"))?;

    Ok(())
}

fn ensure_feed_sort_order_column(connection: &Connection) -> AppResult<()> {
    let mut statement = connection
        .prepare("PRAGMA table_info(feeds)")
        .map_err(|error| format!("Failed to inspect SQLite feed columns: {error}"))?;
    let existing_columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to read SQLite feed columns: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to collect SQLite feed columns: {error}"))?;

    if !existing_columns.iter().any(|column| column == "sort_order") {
        connection
            .execute("ALTER TABLE feeds ADD COLUMN sort_order TEXT", [])
            .map_err(|error| format!("Failed to add feeds.sort_order column: {error}"))?;
    }

    Ok(())
}

fn ensure_feed_image_url_column(connection: &Connection) -> AppResult<()> {
    let mut statement = connection
        .prepare("PRAGMA table_info(feeds)")
        .map_err(|error| format!("Failed to inspect SQLite feed columns: {error}"))?;
    let existing_columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to read SQLite feed columns: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to collect SQLite feed columns: {error}"))?;

    if !existing_columns.iter().any(|column| column == "image_url") {
        connection
            .execute("ALTER TABLE feeds ADD COLUMN image_url TEXT", [])
            .map_err(|error| format!("Failed to add feeds.image_url column: {error}"))?;
    }

    Ok(())
}

fn ensure_item_image_url_column(connection: &Connection) -> AppResult<()> {
    let mut statement = connection
        .prepare("PRAGMA table_info(items)")
        .map_err(|error| format!("Failed to inspect SQLite item columns: {error}"))?;
    let existing_columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to read SQLite item columns: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to collect SQLite item columns: {error}"))?;

    if !existing_columns.iter().any(|column| column == "image_url") {
        connection
            .execute("ALTER TABLE items ADD COLUMN image_url TEXT", [])
            .map_err(|error| format!("Failed to add items.image_url column: {error}"))?;
    }

    Ok(())
}

fn backfill_preview_text(connection: &Connection) -> AppResult<()> {
    const PREVIEW_TEXT_BACKFILL_QUERY: &str =
		"UPDATE items
		 SET preview_text = CASE
			 WHEN content_text IS NOT NULL AND trim(content_text) <> '' THEN substr(trim(content_text), 1, 420)
			 WHEN summary_text IS NOT NULL AND trim(summary_text) <> '' THEN substr(trim(summary_text), 1, 420)
			 WHEN trim(summary) <> '' THEN substr(trim(summary), 1, 420)
			 ELSE 'No summary or content available.'
		 END
		 WHERE preview_text IS NULL OR trim(preview_text) = ''";

    connection
        .execute(PREVIEW_TEXT_BACKFILL_QUERY, [])
        .map_err(|error| format!("Failed to backfill items.preview_text values: {error}"))?;

    Ok(())
}

/// Migrate legacy feed kind values: 'rss' → 'article', 'podcast' → 'media'.
/// Rebuilds the feeds table to update the CHECK constraint.
fn migrate_feed_kind_values(connection: &Connection) -> AppResult<()> {
    let table_sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'feeds'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("Failed to read feeds table schema: {error}"))?;

    if !table_sql.contains("'rss'") && !table_sql.contains("'podcast'") {
        return Ok(());
    }

    connection
		.execute_batch(
			"PRAGMA foreign_keys = OFF;
		     PRAGMA legacy_alter_table = ON;

		     ALTER TABLE feeds RENAME TO feeds_old;

		     CREATE TABLE feeds (
		         id TEXT PRIMARY KEY,
		         url TEXT NOT NULL UNIQUE,
		         title TEXT NOT NULL,
		         description TEXT NOT NULL,
		         kind TEXT NOT NULL CHECK(kind IN ('article', 'media')),
		         site_url TEXT,
		         image_url TEXT,
		         sort_order TEXT,
		         created_at TEXT NOT NULL,
		         last_fetched_at TEXT
		     );

		     INSERT INTO feeds (id, url, title, description, kind, site_url, image_url, sort_order, created_at, last_fetched_at)
		     SELECT id, url, title, description,
		            CASE kind WHEN 'rss' THEN 'article' WHEN 'podcast' THEN 'media' ELSE kind END,
		            site_url, image_url, sort_order, created_at, last_fetched_at
		     FROM feeds_old;

		     DROP TABLE feeds_old;

		     PRAGMA legacy_alter_table = OFF;
		     PRAGMA foreign_keys = ON;",
		)
		.map_err(|error| format!("Failed to migrate feed kind values: {error}"))?;

    Ok(())
}

fn ensure_stations_tables(connection: &Connection) -> AppResult<()> {
    connection
		.execute_batch(
			"CREATE TABLE IF NOT EXISTS stations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                episode_filter TEXT NOT NULL DEFAULT 'all' CHECK(episode_filter IN ('all', 'unplayed')),
                sort_order TEXT NOT NULL DEFAULT 'newest_first' CHECK(sort_order IN ('newest_first', 'oldest_first')),
                sort_order_position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                gradient TEXT NOT NULL DEFAULT 'emerald'
            );

            CREATE TABLE IF NOT EXISTS station_feeds (
                station_id TEXT NOT NULL REFERENCES stations(id) ON DELETE CASCADE,
                feed_id TEXT NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
                PRIMARY KEY (station_id, feed_id)
            );",
		)
		.map_err(|error| format!("Failed to create stations tables: {error}"))?;

    ensure_stations_columns(connection)?;

    Ok(())
}

fn ensure_stations_columns(connection: &Connection) -> AppResult<()> {
    let mut statement = connection
        .prepare("PRAGMA table_info(stations)")
        .map_err(|error| format!("Failed to inspect SQLite stations columns: {error}"))?;
    let existing_columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to read SQLite stations columns: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to collect SQLite stations columns: {error}"))?;

    if existing_columns.iter().all(|c| c != "gradient") {
        connection
            .execute(
                "ALTER TABLE stations ADD COLUMN gradient TEXT NOT NULL DEFAULT 'emerald'",
                [],
            )
            .map_err(|error| format!("Failed to add stations.gradient column: {error}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmpdb() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("test.db");
        (dir, db_path)
    }

    #[test]
    fn initialize_is_idempotent() {
        let (_dir, db_path) = tmpdb();
        initialize_database(&db_path).expect("first init");
        initialize_database(&db_path).expect("second init must not fail");
    }

    #[test]
    fn core_tables_exist_after_init() {
        let (_dir, db_path) = tmpdb();
        initialize_database(&db_path).expect("init");
        let conn = open_connection(&db_path).expect("open");

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        for table in [
            "feeds",
            "items",
            "playback_state",
            "stations",
            "station_feeds",
        ] {
            assert!(
                table_names.iter().any(|t| t == table),
                "expected table '{table}' to exist; got: {table_names:?}"
            );
        }
    }
}
