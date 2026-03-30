use crate::models::{FeedItemRecord, FeedRecord, MediaEnclosureRecord, ParsedFeed};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
use sha1_smol::Sha1;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub type AppResult<T> = Result<T, String>;

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

        open_connection(&state.db_path)?;

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
			 PRAGMA journal_mode = WAL;

			 CREATE TABLE IF NOT EXISTS feeds (
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
			 	published_at TEXT NOT NULL,
			 	read INTEGER NOT NULL DEFAULT 0,
			 	saved INTEGER NOT NULL DEFAULT 0,
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

			 CREATE INDEX IF NOT EXISTS idx_items_feed_id_published_at
			 	ON items(feed_id, published_at DESC);
			 CREATE INDEX IF NOT EXISTS idx_items_published_at
			 	ON items(published_at DESC);",
        )
        .map_err(|error| format!("Failed to initialize SQLite schema: {error}"))?;

    Ok(connection)
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
    let enclosure_url: Option<String> = row.get(8)?;
    let enclosure_mime_type: Option<String> = row.get(9)?;
    let enclosure_size_bytes: Option<i64> = row.get(10)?;
    let enclosure_duration_seconds: Option<i64> = row.get(11)?;

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
        published_at: row.get(5)?,
        read: row.get::<_, i64>(6)? != 0,
        saved: row.get::<_, i64>(7)? != 0,
        playback_position_seconds: row.get(12)?,
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

pub fn list_items(db_path: &Path, feed_id: Option<&str>) -> AppResult<Vec<FeedItemRecord>> {
    let connection = open_connection(db_path)?;

    let query =
        "SELECT i.id, i.feed_id, i.title, i.url, i.summary, i.published_at, i.read, i.saved,
			 i.enclosure_url, i.enclosure_mime_type, i.enclosure_size_bytes, i.enclosure_duration_seconds,
			 COALESCE(p.position_seconds, 0)
		 FROM items i
		 LEFT JOIN playback_state p ON p.item_id = i.id";

    let query = if feed_id.is_some() {
        format!("{query} WHERE i.feed_id = ?1 ORDER BY i.published_at DESC, i.id DESC")
    } else {
        format!("{query} ORDER BY i.published_at DESC, i.id DESC")
    };

    let mut statement = connection
        .prepare(&query)
        .map_err(|error| format!("Failed to prepare item query: {error}"))?;

    let rows = if let Some(feed_id) = feed_id {
        statement.query_map([feed_id], map_item_row)
    } else {
        statement.query_map([], map_item_row)
    }
    .map_err(|error| format!("Failed to list items: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read items: {error}"))
}

pub fn remove_feed(db_path: &Path, id: &str) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute("DELETE FROM feeds WHERE id = ?1", [id])
        .map_err(|error| format!("Failed to remove feed: {error}"))?;

    Ok(())
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
					published_at,
					read,
					saved,
					enclosure_url,
					enclosure_mime_type,
					enclosure_size_bytes,
					enclosure_duration_seconds
				)
				VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 0, ?8, ?9, ?10, ?11)
				ON CONFLICT(id) DO UPDATE SET
					title = excluded.title,
					url = excluded.url,
					summary = excluded.summary,
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
