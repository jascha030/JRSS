use crate::models::{
    CreateStationInput, FeedItemRecord, FeedListItemRecord, FeedRecord, ItemPageQueryRecord,
    ItemPageRecord, MediaEnclosureRecord, ParsedFeed, PlaybackContextRecord, PlaybackSessionRecord,
    ReaderContentRecord, StationRecord, StationWithFeedsRecord, UpdateStationInput,
};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};
use sha1_smol::Sha1;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub type AppResult<T> = Result<T, String>;

const READER_STATUS_UNFETCHED: &str = "unfetched";
const READER_STATUS_READY: &str = "ready";
const READER_STATUS_FAILED: &str = "failed";
const ITEM_SELECT_QUERY: &str =
    "SELECT i.id, i.feed_id, i.title, i.url, i.summary, i.preview_text, i.summary_text, i.summary_html,
             i.content_text, i.content_html, i.reader_status, i.reader_title, i.reader_byline,
             i.reader_excerpt, i.reader_content_html, i.reader_content_text, i.reader_fetched_at,
             i.published_at, i.read, i.enclosure_url, i.enclosure_mime_type,
             i.enclosure_size_bytes, i.enclosure_duration_seconds, COALESCE(p.position_seconds, 0)
         FROM items i
         LEFT JOIN playback_state p ON p.item_id = i.id";

const ITEM_LIST_SELECT_QUERY: &str =
    "SELECT i.id, i.feed_id, i.title, i.url, i.summary,
             i.preview_text,
             i.reader_status, i.reader_title, i.reader_byline, i.reader_excerpt, i.reader_fetched_at,
             i.published_at, i.read, i.enclosure_url, i.enclosure_mime_type,
             i.enclosure_size_bytes, i.enclosure_duration_seconds, COALESCE(p.position_seconds, 0)
             FROM items i
             LEFT JOIN playback_state p ON p.item_id = i.id";

const ITEM_LIST_FILTER_QUERY: &str = " WHERE (?1 IS NULL OR i.feed_id = ?1)
        AND (?2 <> 'unread' OR i.read = 0)
        AND (?2 <> 'media' OR i.enclosure_url IS NOT NULL)";

const ITEM_LIST_COUNT_QUERY: &str = "SELECT COUNT(*) FROM items i";

const PREVIEW_TEXT_BACKFILL_QUERY: &str =
    "UPDATE items
        SET preview_text = CASE
            WHEN content_text IS NOT NULL AND trim(content_text) <> '' THEN substr(trim(content_text), 1, 420)
            WHEN summary_text IS NOT NULL AND trim(summary_text) <> '' THEN substr(trim(summary_text), 1, 420)
            WHEN trim(summary) <> '' THEN substr(trim(summary), 1, 420)
            ELSE 'No summary or content available.'
        END
      WHERE preview_text IS NULL OR trim(preview_text) = ''";

pub struct DatabaseState {
    db_path: PathBuf,
}

impl DatabaseState {
    pub fn new(app: &AppHandle) -> AppResult<Self> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| format!("Failed to resolve app data directory: {error}"))?;

        fs::create_dir_all(&app_data_dir)
            .map_err(|error| format!("Failed to create app data directory: {error}"))?;

        let state = Self {
            db_path: app_data_dir.join("jrss.sqlite3"),
        };

        initialize_database(&state.db_path)?;

        Ok(state)
    }

    pub fn db_path(&self) -> PathBuf {
        self.db_path.clone()
    }
}

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
    ensure_feed_sort_order_column(&connection)?;
    ensure_feed_image_url_column(&connection)?;
    backfill_preview_text(&connection)?;
    migrate_feed_kind_values(&connection)?;
    ensure_stations_tables(&connection)?;

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

    if !existing_columns
        .iter()
        .any(|column| column == "preview_text")
    {
        connection
            .execute(
                "ALTER TABLE items ADD COLUMN preview_text TEXT NOT NULL DEFAULT ''",
                [],
            )
            .map_err(|error| format!("Failed to add items.preview_text column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "summary_text")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN summary_text TEXT", [])
            .map_err(|error| format!("Failed to add items.summary_text column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "summary_html")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN summary_html TEXT", [])
            .map_err(|error| format!("Failed to add items.summary_html column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "content_text")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN content_text TEXT", [])
            .map_err(|error| format!("Failed to add items.content_text column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "content_html")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN content_html TEXT", [])
            .map_err(|error| format!("Failed to add items.content_html column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "reader_status")
    {
        connection
            .execute(
                "ALTER TABLE items ADD COLUMN reader_status TEXT NOT NULL DEFAULT 'unfetched' CHECK(reader_status IN ('unfetched', 'ready', 'failed'))",
                [],
            )
            .map_err(|error| format!("Failed to add items.reader_status column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "reader_title")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN reader_title TEXT", [])
            .map_err(|error| format!("Failed to add items.reader_title column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "reader_byline")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN reader_byline TEXT", [])
            .map_err(|error| format!("Failed to add items.reader_byline column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "reader_excerpt")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN reader_excerpt TEXT", [])
            .map_err(|error| format!("Failed to add items.reader_excerpt column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "reader_content_html")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN reader_content_html TEXT", [])
            .map_err(|error| format!("Failed to add items.reader_content_html column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "reader_content_text")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN reader_content_text TEXT", [])
            .map_err(|error| format!("Failed to add items.reader_content_text column: {error}"))?;
    }

    if !existing_columns
        .iter()
        .any(|column| column == "reader_fetched_at")
    {
        connection
            .execute("ALTER TABLE items ADD COLUMN reader_fetched_at TEXT", [])
            .map_err(|error| format!("Failed to add items.reader_fetched_at column: {error}"))?;
    }

    connection
        .execute(
            "UPDATE items SET reader_status = ?1 WHERE reader_status IS NULL OR trim(reader_status) = ''",
            [READER_STATUS_UNFETCHED],
        )
        .map_err(|error| format!("Failed to backfill items.reader_status values: {error}"))?;

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

fn backfill_preview_text(connection: &Connection) -> AppResult<()> {
    connection
        .execute(PREVIEW_TEXT_BACKFILL_QUERY, [])
        .map_err(|error| format!("Failed to backfill items.preview_text values: {error}"))?;

    Ok(())
}

/// Migrate legacy feed kind values: 'rss' → 'article', 'podcast' → 'media'.
/// Rebuilds the feeds table to update the CHECK constraint.
/// Idempotent — no-op when the constraint already uses the new values.
fn migrate_feed_kind_values(connection: &Connection) -> AppResult<()> {
    // Check if migration is needed by inspecting the table's CREATE statement
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

fn stable_hash(value: &str) -> String {
    Sha1::from(value).digest().to_string()
}

fn build_feed_id(feed_url: &str) -> String {
    format!("feed-{}", stable_hash(feed_url))
}

fn build_item_id(feed_id: &str, external_id: &str) -> String {
    format!("item-{}", stable_hash(&format!("{feed_id}:{external_id}")))
}

fn map_feed_row(row: &Row<'_>) -> rusqlite::Result<FeedRecord> {
    Ok(FeedRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        url: row.get(2)?,
        description: row.get(3)?,
        kind: row.get(4)?,
        site_url: row.get(5)?,
        image_url: row.get(6)?,
        created_at: row.get(7)?,
        last_fetched_at: row.get(8)?,
        sort_order: row.get(9)?,
    })
}

fn map_item_row(row: &Row<'_>) -> rusqlite::Result<FeedItemRecord> {
    let enclosure_url: Option<String> = row.get(19)?;
    let enclosure_mime_type: Option<String> = row.get(20)?;
    let enclosure_size_bytes: Option<i64> = row.get(21)?;
    let enclosure_duration_seconds: Option<i64> = row.get(22)?;

    let media_enclosure = match (enclosure_url, enclosure_mime_type) {
        (Some(url), Some(mime_type)) => Some(MediaEnclosureRecord {
            url,
            mime_type,
            size_bytes: enclosure_size_bytes,
            duration_seconds: enclosure_duration_seconds,
        }),
        _ => None,
    };

    Ok(FeedItemRecord {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        title: row.get(2)?,
        url: row.get(3)?,
        summary: row.get(4)?,
        preview_text: row.get(5)?,
        summary_text: row.get(6)?,
        summary_html: row.get(7)?,
        content_text: row.get(8)?,
        content_html: row.get(9)?,
        reader_status: row.get(10)?,
        reader_title: row.get(11)?,
        reader_byline: row.get(12)?,
        reader_excerpt: row.get(13)?,
        reader_content_html: row.get(14)?,
        reader_content_text: row.get(15)?,
        reader_fetched_at: row.get(16)?,
        published_at: row.get(17)?,
        read: row.get::<_, i64>(18)? != 0,
        playback_position_seconds: row.get(23)?,
        media_enclosure,
    })
}

fn map_item_list_row(row: &Row<'_>) -> rusqlite::Result<FeedListItemRecord> {
    let enclosure_url: Option<String> = row.get(13)?;
    let enclosure_mime_type: Option<String> = row.get(14)?;
    let enclosure_size_bytes: Option<i64> = row.get(15)?;
    let enclosure_duration_seconds: Option<i64> = row.get(16)?;

    let media_enclosure = match (enclosure_url, enclosure_mime_type) {
        (Some(url), Some(mime_type)) => Some(MediaEnclosureRecord {
            url,
            mime_type,
            size_bytes: enclosure_size_bytes,
            duration_seconds: enclosure_duration_seconds,
        }),
        _ => None,
    };

    Ok(FeedListItemRecord {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        title: row.get(2)?,
        url: row.get(3)?,
        summary: row.get(4)?,
        preview_text: row.get(5)?,
        reader_status: row.get(6)?,
        reader_title: row.get(7)?,
        reader_byline: row.get(8)?,
        reader_excerpt: row.get(9)?,
        reader_fetched_at: row.get(10)?,
        published_at: row.get(11)?,
        read: row.get::<_, i64>(12)? != 0,
        playback_position_seconds: row.get(17)?,
        media_enclosure,
    })
}

fn get_feed_by_url_in_tx(
    transaction: &Transaction<'_>,
    url: &str,
) -> AppResult<Option<FeedRecord>> {
    transaction
        .query_row(
            "SELECT id, title, url, description, kind, site_url, image_url, created_at, last_fetched_at, sort_order
			 FROM feeds
			 WHERE url = ?1",
            [url],
            map_feed_row,
        )
        .optional()
        .map_err(|error| format!("Failed to query feed by URL: {error}"))
}

pub fn get_feed_by_id(db_path: &Path, id: &str) -> AppResult<Option<FeedRecord>> {
    let connection = open_connection(db_path)?;

    connection
        .query_row(
            "SELECT id, title, url, description, kind, site_url, image_url, created_at, last_fetched_at, sort_order
			 FROM feeds
			 WHERE id = ?1",
            [id],
            map_feed_row,
        )
        .optional()
        .map_err(|error| format!("Failed to query feed by ID: {error}"))
}

pub fn list_feeds(db_path: &Path) -> AppResult<Vec<FeedRecord>> {
    let connection = open_connection(db_path)?;
    let mut statement = connection
        .prepare(
            "SELECT id, title, url, description, kind, site_url, image_url, created_at, last_fetched_at, sort_order
			 FROM feeds
			 ORDER BY lower(title), title",
        )
        .map_err(|error| format!("Failed to prepare feed query: {error}"))?;

    let feeds = statement
        .query_map([], map_feed_row)
        .map_err(|error| format!("Failed to list feeds: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read feeds: {error}"))?;

    Ok(feeds)
}

pub fn query_items_page(db_path: &Path, query: &ItemPageQueryRecord) -> AppResult<ItemPageRecord> {
    let connection = open_connection(db_path)?;
    let safe_limit = query.limit.clamp(1, 500);
    let safe_offset = query.offset.max(0);

    let search_term = query
        .search
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let search_clause = if search_term.is_some() {
        " AND (i.title LIKE ?3 COLLATE NOCASE OR i.preview_text LIKE ?3 COLLATE NOCASE OR i.content_text LIKE ?3 COLLATE NOCASE)"
    } else {
        ""
    };

    let count_sql = format!("{ITEM_LIST_COUNT_QUERY}{ITEM_LIST_FILTER_QUERY}{search_clause}");
    let total_count: i64 = if let Some(ref pattern) = search_term {
        connection
            .query_row(
                &count_sql,
                params![query.feed_id.as_deref(), query.section.as_str(), pattern],
                |row| row.get(0),
            )
            .map_err(|error| format!("Failed to count items: {error}"))?
    } else {
        connection
            .query_row(
                &count_sql,
                params![query.feed_id.as_deref(), query.section.as_str()],
                |row| row.get(0),
            )
            .map_err(|error| format!("Failed to count items: {error}"))?
    };

    let order_by = query.sort_order.order_by_clause();

    let page_sql = if search_term.is_some() {
        format!(
            "{ITEM_LIST_SELECT_QUERY}{ITEM_LIST_FILTER_QUERY}{search_clause} ORDER BY {order_by} LIMIT ?4 OFFSET ?5"
        )
    } else {
        format!(
            "{ITEM_LIST_SELECT_QUERY}{ITEM_LIST_FILTER_QUERY} ORDER BY {order_by} LIMIT ?3 OFFSET ?4"
        )
    };

    let mut statement = connection
        .prepare(&page_sql)
        .map_err(|error| format!("Failed to prepare paged item query: {error}"))?;

    let items = if let Some(ref pattern) = search_term {
        statement
            .query_map(
                params![
                    query.feed_id.as_deref(),
                    query.section.as_str(),
                    pattern,
                    safe_limit,
                    safe_offset
                ],
                map_item_list_row,
            )
            .map_err(|error| format!("Failed to query item page: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Failed to read paged items: {error}"))?
    } else {
        statement
            .query_map(
                params![
                    query.feed_id.as_deref(),
                    query.section.as_str(),
                    safe_limit,
                    safe_offset
                ],
                map_item_list_row,
            )
            .map_err(|error| format!("Failed to query item page: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Failed to read paged items: {error}"))?
    };

    Ok(ItemPageRecord { items, total_count })
}

pub fn remove_feed(db_path: &Path, id: &str) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute("DELETE FROM feeds WHERE id = ?1", [id])
        .map_err(|error| format!("Failed to remove feed: {error}"))?;

    Ok(())
}

pub fn get_item_by_id(db_path: &Path, id: &str) -> AppResult<Option<FeedItemRecord>> {
    let connection = open_connection(db_path)?;

    connection
        .query_row(
            &format!("{ITEM_SELECT_QUERY} WHERE i.id = ?1"),
            [id],
            map_item_row,
        )
        .optional()
        .map_err(|error| format!("Failed to query item by ID: {error}"))
}

pub fn mark_read(db_path: &Path, item_id: &str, read: bool) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute(
            "UPDATE items SET read = ?2 WHERE id = ?1",
            params![item_id, if read { 1_i64 } else { 0_i64 }],
        )
        .map_err(|error| format!("Failed to update read state: {error}"))?;

    Ok(())
}

pub fn save_playback(db_path: &Path, item_id: &str, position_seconds: i64) -> AppResult<()> {
    let connection = open_connection(db_path)?;
    let safe_position = position_seconds.max(0);

    connection
        .execute(
            "INSERT INTO playback_state (item_id, position_seconds, updated_at)
			 VALUES (?1, ?2, ?3)
			 ON CONFLICT(item_id) DO UPDATE SET
			 	position_seconds = excluded.position_seconds,
			 	updated_at = excluded.updated_at",
            params![item_id, safe_position, Utc::now().to_rfc3339()],
        )
        .map_err(|error| format!("Failed to save playback state: {error}"))?;

    Ok(())
}

pub fn save_reader_content(
    db_path: &Path,
    item_id: &str,
    reader_content: &ReaderContentRecord,
) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute(
            "UPDATE items
             SET reader_status = ?2,
                 reader_title = ?3,
                 reader_byline = ?4,
                 reader_excerpt = ?5,
                 reader_content_html = ?6,
                 reader_content_text = ?7,
                 reader_fetched_at = ?8
             WHERE id = ?1",
            params![
                item_id,
                READER_STATUS_READY,
                reader_content.title,
                reader_content.byline,
                reader_content.excerpt,
                reader_content.content_html,
                reader_content.content_text,
                reader_content.fetched_at,
            ],
        )
        .map_err(|error| format!("Failed to save reader content: {error}"))?;

    Ok(())
}

pub fn save_reader_failure(db_path: &Path, item_id: &str) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute(
            "UPDATE items
             SET reader_status = ?2,
                 reader_title = NULL,
                 reader_byline = NULL,
                 reader_excerpt = NULL,
                 reader_content_html = NULL,
                 reader_content_text = NULL,
                 reader_fetched_at = ?3
             WHERE id = ?1",
            params![item_id, READER_STATUS_FAILED, Utc::now().to_rfc3339()],
        )
        .map_err(|error| format!("Failed to save reader failure state: {error}"))?;

    Ok(())
}

pub fn upsert_feed_snapshot(
    db_path: &Path,
    normalized_url: &str,
    parsed_feed: ParsedFeed,
) -> AppResult<FeedRecord> {
    let mut connection = open_connection(db_path)?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Failed to open SQLite transaction: {error}"))?;

    let existing_feed = get_feed_by_url_in_tx(&transaction, normalized_url)?;
    let fetched_at = Utc::now().to_rfc3339();
    let next_feed = FeedRecord {
        id: existing_feed
            .as_ref()
            .map(|feed| feed.id.clone())
            .unwrap_or_else(|| build_feed_id(normalized_url)),
        title: parsed_feed.title,
        url: normalized_url.to_string(),
        description: parsed_feed.description,
        kind: parsed_feed.kind,
        site_url: parsed_feed.site_url,
        image_url: parsed_feed.image_url,
        created_at: existing_feed
            .as_ref()
            .map(|feed| feed.created_at.clone())
            .unwrap_or_else(|| fetched_at.clone()),
        last_fetched_at: Some(fetched_at),
        sort_order: existing_feed
            .as_ref()
            .and_then(|feed| feed.sort_order.clone()),
    };

    transaction
		.execute(
			"INSERT INTO feeds (id, url, title, description, kind, site_url, image_url, created_at, last_fetched_at)
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
			 ON CONFLICT(id) DO UPDATE SET
			 	url = excluded.url,
			 	title = excluded.title,
			 	description = excluded.description,
			 	kind = excluded.kind,
			 	site_url = excluded.site_url,
			 	image_url = excluded.image_url,
			 	last_fetched_at = excluded.last_fetched_at",
			params![
				next_feed.id,
				next_feed.url,
				next_feed.title,
				next_feed.description,
				next_feed.kind,
				next_feed.site_url,
				next_feed.image_url,
				next_feed.created_at,
				next_feed.last_fetched_at
			]
		)
		.map_err(|error| format!("Failed to upsert feed: {error}"))?;

    for parsed_item in parsed_feed.items {
        let item_id = build_item_id(&next_feed.id, &parsed_item.external_id);
        let media_enclosure = parsed_item.media_enclosure;

        transaction
            .execute(
                "INSERT INTO items (
					id,
					feed_id,
					external_id,
					title,
					url,
					summary,
					preview_text,
					summary_text,
					summary_html,
					content_text,
					content_html,
					published_at,
					read,
					enclosure_url,
					enclosure_mime_type,
					enclosure_size_bytes,
					enclosure_duration_seconds
				)
				VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, ?13, ?14, ?15, ?16)
				ON CONFLICT(id) DO UPDATE SET
					title = excluded.title,
					url = excluded.url,
					summary = excluded.summary,
					preview_text = excluded.preview_text,
					summary_text = excluded.summary_text,
					summary_html = excluded.summary_html,
					content_text = excluded.content_text,
					content_html = excluded.content_html,
					published_at = excluded.published_at,
					enclosure_url = excluded.enclosure_url,
					enclosure_mime_type = excluded.enclosure_mime_type,
					enclosure_size_bytes = excluded.enclosure_size_bytes,
					enclosure_duration_seconds = excluded.enclosure_duration_seconds",
                params![
                    item_id,
                    next_feed.id,
                    parsed_item.external_id,
                    parsed_item.title,
                    parsed_item.url,
                    parsed_item.summary,
                    parsed_item.preview_text,
                    parsed_item.summary_text,
                    parsed_item.summary_html,
                    parsed_item.content_text,
                    parsed_item.content_html,
                    parsed_item.published_at,
                    media_enclosure
                        .as_ref()
                        .map(|enclosure| enclosure.url.clone()),
                    media_enclosure
                        .as_ref()
                        .map(|enclosure| enclosure.mime_type.clone()),
                    media_enclosure
                        .as_ref()
                        .and_then(|enclosure| enclosure.size_bytes),
                    media_enclosure
                        .as_ref()
                        .and_then(|enclosure| enclosure.duration_seconds)
                ],
            )
            .map_err(|error| format!("Failed to upsert feed item: {error}"))?;
    }

    transaction
        .commit()
        .map_err(|error| format!("Failed to commit SQLite transaction: {error}"))?;

    Ok(next_feed)
}

// ---------------------------------------------------------------------------
// Playback session persistence
// ---------------------------------------------------------------------------

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

pub fn get_items_by_ids(db_path: &Path, ids: &[String]) -> AppResult<Vec<FeedListItemRecord>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let connection = open_connection(db_path)?;
    let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "{ITEM_LIST_SELECT_QUERY} WHERE i.id IN ({}) ORDER BY i.published_at DESC, i.id DESC",
        placeholders.join(", ")
    );

    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| format!("Failed to prepare items-by-ids query: {error}"))?;

    let params: Vec<&dyn rusqlite::ToSql> =
        ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
    let items = statement
        .query_map(params.as_slice(), map_item_list_row)
        .map_err(|error| format!("Failed to query items by IDs: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read items by IDs: {error}"))?;

    Ok(items)
}

pub fn set_feed_sort_order(
    db_path: &Path,
    feed_id: &str,
    sort_order: Option<&str>,
) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute(
            "UPDATE feeds SET sort_order = ?2 WHERE id = ?1",
            params![feed_id, sort_order],
        )
        .map_err(|error| format!("Failed to update feed sort order: {error}"))?;

    Ok(())
}

pub fn clear_playback_session(db_path: &Path) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute("DELETE FROM playback_session WHERE id = 1", [])
        .map_err(|error| format!("Failed to clear playback session: {error}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Playback context — frontend-managed context for feed/station
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Stations — podcast playlist grouping
// ---------------------------------------------------------------------------

fn ensure_stations_tables(connection: &Connection) -> AppResult<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS stations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                episode_filter TEXT NOT NULL DEFAULT 'all' CHECK(episode_filter IN ('all', 'unplayed')),
                sort_order TEXT NOT NULL DEFAULT 'newest_first' CHECK(sort_order IN ('newest_first', 'oldest_first')),
                sort_order_position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS station_feeds (
                station_id TEXT NOT NULL REFERENCES stations(id) ON DELETE CASCADE,
                feed_id TEXT NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
                PRIMARY KEY (station_id, feed_id)
            );",
        )
        .map_err(|error| format!("Failed to create stations tables: {error}"))?;

    Ok(())
}

fn build_station_id() -> String {
    format!("station-{}", uuid::Uuid::new_v4())
}

fn map_station_row(row: &Row<'_>) -> rusqlite::Result<StationRecord> {
    Ok(StationRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        episode_filter: row.get(2)?,
        sort_order: row.get(3)?,
        sort_order_position: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn get_station_feed_ids(connection: &Connection, station_id: &str) -> AppResult<Vec<String>> {
    let mut statement = connection
        .prepare("SELECT feed_id FROM station_feeds WHERE station_id = ?1 ORDER BY feed_id")
        .map_err(|error| format!("Failed to prepare station feeds query: {error}"))?;

    let feed_ids = statement
        .query_map([station_id], |row| row.get(0))
        .map_err(|error| format!("Failed to query station feeds: {error}"))?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|error| format!("Failed to read station feeds: {error}"))?;

    Ok(feed_ids)
}

fn set_station_feeds_in_tx(
    transaction: &Transaction<'_>,
    station_id: &str,
    feed_ids: &[String],
) -> AppResult<()> {
    transaction
        .execute(
            "DELETE FROM station_feeds WHERE station_id = ?1",
            [station_id],
        )
        .map_err(|error| format!("Failed to clear station feeds: {error}"))?;

    let mut insert_statement = transaction
        .prepare("INSERT INTO station_feeds (station_id, feed_id) VALUES (?1, ?2)")
        .map_err(|error| format!("Failed to prepare station feed insert: {error}"))?;

    for feed_id in feed_ids {
        insert_statement
            .execute(params![station_id, feed_id])
            .map_err(|error| format!("Failed to insert station feed: {error}"))?;
    }

    Ok(())
}

pub fn list_stations(db_path: &Path) -> AppResult<Vec<StationWithFeedsRecord>> {
    let connection = open_connection(db_path)?;

    let mut statement = connection
        .prepare(
            "SELECT id, name, episode_filter, sort_order, sort_order_position, created_at
             FROM stations
             ORDER BY sort_order_position ASC, lower(name), name",
        )
        .map_err(|error| format!("Failed to prepare stations query: {error}"))?;

    let stations = statement
        .query_map([], map_station_row)
        .map_err(|error| format!("Failed to list stations: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read stations: {error}"))?;

    let mut result = Vec::with_capacity(stations.len());

    for station in stations {
        let feed_ids = get_station_feed_ids(&connection, &station.id)?;
        result.push(StationWithFeedsRecord { station, feed_ids });
    }

    Ok(result)
}

pub fn create_station(
    db_path: &Path,
    input: &CreateStationInput,
) -> AppResult<StationWithFeedsRecord> {
    let mut connection = open_connection(db_path)?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Failed to open transaction: {error}"))?;

    let station_id = build_station_id();
    let now = Utc::now().to_rfc3339();

    // Determine next sort_order_position
    let max_position: i64 = transaction
        .query_row(
            "SELECT COALESCE(MAX(sort_order_position), -1) FROM stations",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("Failed to query max station position: {error}"))?;

    let station = StationRecord {
        id: station_id.clone(),
        name: input.name.clone(),
        episode_filter: input.episode_filter.as_str().to_string(),
        sort_order: match input.sort_order {
            crate::models::ItemSortOrder::NewestFirst => "newest_first".to_string(),
            crate::models::ItemSortOrder::OldestFirst => "oldest_first".to_string(),
        },
        sort_order_position: max_position + 1,
        created_at: now,
    };

    transaction
        .execute(
            "INSERT INTO stations (id, name, episode_filter, sort_order, sort_order_position, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                station.id,
                station.name,
                station.episode_filter,
                station.sort_order,
                station.sort_order_position,
                station.created_at
            ],
        )
        .map_err(|error| format!("Failed to insert station: {error}"))?;

    set_station_feeds_in_tx(&transaction, &station_id, &input.feed_ids)?;

    transaction
        .commit()
        .map_err(|error| format!("Failed to commit station creation: {error}"))?;

    Ok(StationWithFeedsRecord {
        station,
        feed_ids: input.feed_ids.clone(),
    })
}

pub fn update_station(
    db_path: &Path,
    input: &UpdateStationInput,
) -> AppResult<StationWithFeedsRecord> {
    let mut connection = open_connection(db_path)?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Failed to open transaction: {error}"))?;

    // Fetch existing station
    let existing = transaction
        .query_row(
            "SELECT id, name, episode_filter, sort_order, sort_order_position, created_at
             FROM stations WHERE id = ?1",
            [&input.id],
            map_station_row,
        )
        .optional()
        .map_err(|error| format!("Failed to query station: {error}"))?
        .ok_or_else(|| "Station not found.".to_string())?;

    let name = input.name.as_deref().unwrap_or(&existing.name);
    let episode_filter = input
        .episode_filter
        .map(|ef| ef.as_str().to_string())
        .unwrap_or(existing.episode_filter);
    let sort_order = input
        .sort_order
        .map(|so| match so {
            crate::models::ItemSortOrder::NewestFirst => "newest_first".to_string(),
            crate::models::ItemSortOrder::OldestFirst => "oldest_first".to_string(),
        })
        .unwrap_or(existing.sort_order);

    transaction
        .execute(
            "UPDATE stations SET name = ?2, episode_filter = ?3, sort_order = ?4 WHERE id = ?1",
            params![input.id, name, episode_filter, sort_order],
        )
        .map_err(|error| format!("Failed to update station: {error}"))?;

    if let Some(ref feed_ids) = input.feed_ids {
        set_station_feeds_in_tx(&transaction, &input.id, feed_ids)?;
    }

    transaction
        .commit()
        .map_err(|error| format!("Failed to commit station update: {error}"))?;

    // Re-read the updated station
    let updated = connection
        .query_row(
            "SELECT id, name, episode_filter, sort_order, sort_order_position, created_at
             FROM stations WHERE id = ?1",
            [&input.id],
            map_station_row,
        )
        .map_err(|error| format!("Failed to re-read station: {error}"))?;

    let feed_ids = get_station_feed_ids(&connection, &input.id)?;

    Ok(StationWithFeedsRecord {
        station: updated,
        feed_ids,
    })
}

pub fn delete_station(db_path: &Path, id: &str) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute("DELETE FROM stations WHERE id = ?1", [id])
        .map_err(|error| format!("Failed to delete station: {error}"))?;

    Ok(())
}

pub fn query_station_episodes(
    db_path: &Path,
    station_id: &str,
    offset: i64,
    limit: i64,
) -> AppResult<ItemPageRecord> {
    let connection = open_connection(db_path)?;
    let safe_limit = limit.clamp(1, 500);
    let safe_offset = offset.max(0);

    // Load station metadata
    let station = connection
        .query_row(
            "SELECT id, name, episode_filter, sort_order, sort_order_position, created_at
             FROM stations WHERE id = ?1",
            [station_id],
            map_station_row,
        )
        .optional()
        .map_err(|error| format!("Failed to query station: {error}"))?
        .ok_or_else(|| "Station not found.".to_string())?;

    let feed_ids = get_station_feed_ids(&connection, station_id)?;

    if feed_ids.is_empty() {
        return Ok(ItemPageRecord {
            items: Vec::new(),
            total_count: 0,
        });
    }

    let placeholders: Vec<String> = (1..=feed_ids.len()).map(|i| format!("?{i}")).collect();
    let feed_filter = format!("i.feed_id IN ({})", placeholders.join(", "));

    let episode_clause = if station.episode_filter == "unplayed" {
        " AND i.read = 0"
    } else {
        ""
    };

    // Only podcast episodes (items with enclosures)
    let enclosure_clause = " AND i.enclosure_url IS NOT NULL";

    let order_by = if station.sort_order == "oldest_first" {
        "i.published_at ASC, i.id ASC"
    } else {
        "i.published_at DESC, i.id DESC"
    };

    // Count
    let count_sql = format!(
        "SELECT COUNT(*) FROM items i WHERE {feed_filter}{episode_clause}{enclosure_clause}"
    );
    let mut count_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    for fid in &feed_ids {
        count_params.push(Box::new(fid.clone()));
    }
    let count_refs: Vec<&dyn rusqlite::ToSql> = count_params.iter().map(|p| p.as_ref()).collect();

    let total_count: i64 = connection
        .query_row(&count_sql, count_refs.as_slice(), |row| row.get(0))
        .map_err(|error| format!("Failed to count station episodes: {error}"))?;

    // Page query
    let next_param = feed_ids.len() + 1;
    let page_sql = format!(
        "{ITEM_LIST_SELECT_QUERY} WHERE {feed_filter}{episode_clause}{enclosure_clause} ORDER BY {order_by} LIMIT ?{next_param} OFFSET ?{}",
        next_param + 1
    );

    let mut page_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    for fid in &feed_ids {
        page_params.push(Box::new(fid.clone()));
    }
    page_params.push(Box::new(safe_limit));
    page_params.push(Box::new(safe_offset));
    let page_refs: Vec<&dyn rusqlite::ToSql> = page_params.iter().map(|p| p.as_ref()).collect();

    let mut statement = connection
        .prepare(&page_sql)
        .map_err(|error| format!("Failed to prepare station episodes query: {error}"))?;

    let items = statement
        .query_map(page_refs.as_slice(), map_item_list_row)
        .map_err(|error| format!("Failed to query station episodes: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read station episodes: {error}"))?;

    Ok(ItemPageRecord { items, total_count })
}
