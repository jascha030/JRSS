use crate::models::{
    FeedItemRecord, FeedListItemRecord, FeedRecord, ItemPageQueryRecord, ItemPageRecord,
    MediaEnclosureRecord, ParsedFeed, ReaderContentRecord,
};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
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
        AND (?2 <> 'podcasts' OR i.enclosure_url IS NOT NULL)";

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
			 	kind TEXT NOT NULL CHECK(kind IN ('rss', 'podcast')),
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
    backfill_preview_text(&connection)?;

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

fn backfill_preview_text(connection: &Connection) -> AppResult<()> {
    connection
        .execute(PREVIEW_TEXT_BACKFILL_QUERY, [])
        .map_err(|error| format!("Failed to backfill items.preview_text values: {error}"))?;

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
        created_at: row.get(6)?,
        last_fetched_at: row.get(7)?,
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
            "SELECT id, title, url, description, kind, site_url, created_at, last_fetched_at
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
            "SELECT id, title, url, description, kind, site_url, created_at, last_fetched_at
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
            "SELECT id, title, url, description, kind, site_url, created_at, last_fetched_at
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
    let count_query = format!("{ITEM_LIST_COUNT_QUERY}{ITEM_LIST_FILTER_QUERY}");
    let total_count = connection
        .query_row(
            &count_query,
            params![query.feed_id.as_deref(), query.section.as_str()],
            |row| row.get(0),
        )
        .map_err(|error| format!("Failed to count items: {error}"))?;

    let page_query = format!(
        "{ITEM_LIST_SELECT_QUERY}{ITEM_LIST_FILTER_QUERY} ORDER BY i.published_at DESC, i.id DESC LIMIT ?3 OFFSET ?4"
    );
    let mut statement = connection
        .prepare(&page_query)
        .map_err(|error| format!("Failed to prepare paged item query: {error}"))?;
    let rows = statement
        .query_map(
            params![
                query.feed_id.as_deref(),
                query.section.as_str(),
                safe_limit,
                safe_offset
            ],
            map_item_list_row,
        )
        .map_err(|error| format!("Failed to query item page: {error}"))?;
    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read paged items: {error}"))?;

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
        created_at: existing_feed
            .as_ref()
            .map(|feed| feed.created_at.clone())
            .unwrap_or_else(|| fetched_at.clone()),
        last_fetched_at: Some(fetched_at),
    };

    transaction
		.execute(
			"INSERT INTO feeds (id, url, title, description, kind, site_url, created_at, last_fetched_at)
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
			 ON CONFLICT(id) DO UPDATE SET
			 	url = excluded.url,
			 	title = excluded.title,
			 	description = excluded.description,
			 	kind = excluded.kind,
			 	site_url = excluded.site_url,
			 	last_fetched_at = excluded.last_fetched_at",
			params![
				next_feed.id,
				next_feed.url,
				next_feed.title,
				next_feed.description,
				next_feed.kind,
				next_feed.site_url,
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
