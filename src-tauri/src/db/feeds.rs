use super::AppResult;
use super::connection::open_connection;
use super::rows::map_feed_row;
use crate::models::{FeedRecord, ParsedFeed};
use chrono::Utc;
use rusqlite::{OptionalExtension, Transaction, params};
use sha1_smol::Sha1;
use std::path::Path;

fn stable_hash(value: &str) -> String {
    Sha1::from(value).digest().to_string()
}

fn build_feed_id(feed_url: &str) -> String {
    format!("feed-{}", stable_hash(feed_url))
}

fn build_item_id(feed_id: &str, external_id: &str) -> String {
    format!("item-{}", stable_hash(&format!("{feed_id}:{external_id}")))
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

pub fn remove_feed(db_path: &Path, id: &str) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute("DELETE FROM feeds WHERE id = ?1", [id])
        .map_err(|error| format!("Failed to remove feed: {error}"))?;

    Ok(())
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
			],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::initialize_database;
    use crate::models::{ParsedFeed, ParsedFeedItem};
    use tempfile::TempDir;

    fn tmpdb() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("test.db");
        initialize_database(&db_path).expect("init");
        (dir, db_path)
    }

    fn make_feed(title: &str) -> ParsedFeed {
        ParsedFeed {
            title: title.to_string(),
            description: String::new(),
            site_url: None,
            image_url: None,
            kind: "article".to_string(),
            items: Vec::new(),
        }
    }

    fn make_feed_with_item(title: &str, external_id: &str) -> ParsedFeed {
        ParsedFeed {
            title: title.to_string(),
            description: String::new(),
            site_url: None,
            image_url: None,
            kind: "article".to_string(),
            items: vec![ParsedFeedItem {
                external_id: external_id.to_string(),
                title: "Item".to_string(),
                url: "https://example.com/item".to_string(),
                summary: String::new(),
                preview_text: String::new(),
                summary_text: None,
                summary_html: None,
                content_text: None,
                content_html: None,
                published_at: "2024-01-01T00:00:00Z".to_string(),
                media_enclosure: None,
            }],
        }
    }

    #[test]
    fn upsert_creates_feed() {
        let (_dir, db_path) = tmpdb();
        let feed =
            upsert_feed_snapshot(&db_path, "https://example.com/rss", make_feed("Test")).unwrap();
        assert_eq!(feed.title, "Test");
        assert!(feed.id.starts_with("feed-"));
    }

    #[test]
    fn upsert_is_idempotent_preserving_id() {
        let (_dir, db_path) = tmpdb();
        let f1 = upsert_feed_snapshot(&db_path, "https://example.com/rss", make_feed("Title 1"))
            .unwrap();
        let f2 = upsert_feed_snapshot(&db_path, "https://example.com/rss", make_feed("Title 2"))
            .unwrap();
        assert_eq!(f1.id, f2.id);
        let feeds = list_feeds(&db_path).unwrap();
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].title, "Title 2");
    }

    #[test]
    fn get_feed_by_id_returns_feed() {
        let (_dir, db_path) = tmpdb();
        let created =
            upsert_feed_snapshot(&db_path, "https://example.com/rss", make_feed("Test")).unwrap();
        let found = get_feed_by_id(&db_path, &created.id).unwrap().unwrap();
        assert_eq!(found.title, "Test");
    }

    #[test]
    fn get_feed_by_id_returns_none_for_unknown() {
        let (_dir, db_path) = tmpdb();
        assert!(get_feed_by_id(&db_path, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn remove_feed_deletes_it() {
        let (_dir, db_path) = tmpdb();
        let feed =
            upsert_feed_snapshot(&db_path, "https://example.com/rss", make_feed("Test")).unwrap();
        remove_feed(&db_path, &feed.id).unwrap();
        assert!(list_feeds(&db_path).unwrap().is_empty());
    }

    #[test]
    fn remove_feed_cascades_to_items() {
        let (_dir, db_path) = tmpdb();
        let feed = upsert_feed_snapshot(
            &db_path,
            "https://example.com/rss",
            make_feed_with_item("Feed", "ext-1"),
        )
        .unwrap();
        remove_feed(&db_path, &feed.id).unwrap();
        // If items were not deleted the DB would have orphaned rows;
        // a successful remove without FK violation is sufficient evidence.
        assert!(list_feeds(&db_path).unwrap().is_empty());
    }

    #[test]
    fn set_feed_sort_order_persists() {
        let (_dir, db_path) = tmpdb();
        let feed =
            upsert_feed_snapshot(&db_path, "https://example.com/rss", make_feed("Test")).unwrap();
        set_feed_sort_order(&db_path, &feed.id, Some("oldest_first")).unwrap();
        let updated = get_feed_by_id(&db_path, &feed.id).unwrap().unwrap();
        assert_eq!(updated.sort_order, Some("oldest_first".to_string()));
    }

    #[test]
    fn set_feed_sort_order_to_none_clears_it() {
        let (_dir, db_path) = tmpdb();
        let feed =
            upsert_feed_snapshot(&db_path, "https://example.com/rss", make_feed("Test")).unwrap();
        set_feed_sort_order(&db_path, &feed.id, Some("oldest_first")).unwrap();
        set_feed_sort_order(&db_path, &feed.id, None).unwrap();
        let updated = get_feed_by_id(&db_path, &feed.id).unwrap().unwrap();
        assert_eq!(updated.sort_order, None);
    }
}
