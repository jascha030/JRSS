use super::AppResult;
use super::connection::open_connection;
use super::rows::{map_item_list_row, map_item_row};
use crate::models::{
    FeedItemRecord, FeedListItemRecord, ItemListSection, ItemPageQueryRecord, ItemPageRecord,
    ReaderContentRecord,
};
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use std::collections::HashMap;
use std::path::Path;

const ITEM_SELECT_QUERY: &str =
	"SELECT i.id, i.feed_id, i.title, i.url, i.summary, i.preview_text, i.summary_text, i.summary_html,
			 i.content_text, i.content_html, i.reader_status, i.reader_title, i.reader_byline,
			 i.reader_excerpt, i.reader_content_html, i.reader_content_text, i.reader_fetched_at,
			 i.published_at, i.read, i.favorite, i.enclosure_url, i.enclosure_mime_type,
			 i.enclosure_size_bytes, i.enclosure_duration_seconds, COALESCE(p.position_seconds, 0),
			 i.image_url
		 FROM items i
		 LEFT JOIN playback_state p ON p.item_id = i.id";

pub const ITEM_LIST_SELECT_QUERY: &str = "SELECT i.id, i.feed_id, i.title, i.url, i.summary,
			 i.preview_text,
			 i.reader_status, i.reader_title, i.reader_byline, i.reader_excerpt, i.reader_fetched_at,
			 i.published_at, i.read, i.favorite, i.enclosure_url, i.enclosure_mime_type,
			 i.enclosure_size_bytes, i.enclosure_duration_seconds, COALESCE(p.position_seconds, 0),
			 i.image_url
			 FROM items i
			 LEFT JOIN playback_state p ON p.item_id = i.id";

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

pub fn mark_favorite(db_path: &Path, item_id: &str, favorite: bool) -> AppResult<()> {
    let connection = open_connection(db_path)?;

    connection
        .execute(
            "UPDATE items SET favorite = ?2 WHERE id = ?1",
            params![item_id, if favorite { 1_i64 } else { 0_i64 }],
        )
        .map_err(|error| format!("Failed to update favorite state: {error}"))?;

    Ok(())
}

pub fn mark_favorite_batch(db_path: &Path, item_ids: &[String], favorite: bool) -> AppResult<()> {
    if item_ids.is_empty() {
        return Ok(());
    }

    let mut connection = open_connection(db_path)?;
    let tx = connection
        .transaction()
        .map_err(|error| format!("Failed to begin transaction: {error}"))?;

    let placeholders: Vec<String> = (2..=item_ids.len() + 1).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "UPDATE items SET favorite = ?1 WHERE id IN ({})",
        placeholders.join(", ")
    );

    let favorite_value = if favorite { 1_i64 } else { 0_i64 };
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    params.push(&favorite_value as &dyn rusqlite::ToSql);
    for id in item_ids {
        params.push(id as &dyn rusqlite::ToSql);
    }

    tx.execute(&sql, params.as_slice())
        .map_err(|error| format!("Failed to batch update favorite state: {error}"))?;

    tx.commit()
        .map_err(|error| format!("Failed to commit batch favorite update: {error}"))?;

    Ok(())
}

pub fn mark_read_batch(db_path: &Path, item_ids: &[String], read: bool) -> AppResult<()> {
    if item_ids.is_empty() {
        return Ok(());
    }

    let mut connection = open_connection(db_path)?;
    let tx = connection
        .transaction()
        .map_err(|error| format!("Failed to begin transaction: {error}"))?;

    let placeholders: Vec<String> = (2..=item_ids.len() + 1).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "UPDATE items SET read = ?1 WHERE id IN ({})",
        placeholders.join(", ")
    );

    let read_value = if read { 1_i64 } else { 0_i64 };
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    params.push(&read_value as &dyn rusqlite::ToSql);
    for id in item_ids {
        params.push(id as &dyn rusqlite::ToSql);
    }

    tx.execute(&sql, params.as_slice())
        .map_err(|error| format!("Failed to batch update read state: {error}"))?;

    tx.commit()
        .map_err(|error| format!("Failed to commit batch read update: {error}"))?;

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

pub fn update_item_duration(db_path: &Path, item_id: &str, duration_seconds: i64) -> AppResult<()> {
    let connection = open_connection(db_path)?;
    let safe_duration = duration_seconds.max(0);

    connection
        .execute(
            "UPDATE items SET enclosure_duration_seconds = ?2 WHERE id = ?1",
            params![item_id, safe_duration],
        )
        .map_err(|error| format!("Failed to update item duration: {error}"))?;

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
                "ready",
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
            params![item_id, "failed", Utc::now().to_rfc3339()],
        )
        .map_err(|error| format!("Failed to save reader failure state: {error}"))?;

    Ok(())
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

/// Legacy query API — prefer `query_items` for new code.
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

    const ITEM_LIST_FILTER_QUERY: &str = " WHERE (?1 IS NULL OR i.feed_id = ?1)
        AND (?2 <> 'unread' OR i.read = 0)
        AND (?2 <> 'media' OR i.enclosure_url IS NOT NULL)
        AND (?2 <> 'favorites' OR i.favorite = 1)";

    let search_clause = if search_term.is_some() {
        " AND (i.title LIKE ?3 COLLATE NOCASE OR i.preview_text LIKE ?3 COLLATE NOCASE OR i.content_text LIKE ?3 COLLATE NOCASE)"
    } else {
        ""
    };

    let count_sql = format!("SELECT COUNT(*) FROM items i{ITEM_LIST_FILTER_QUERY}{search_clause}");
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

/// Unified query for items that handles feed, station, and section views.
pub fn query_items(
    db_path: &Path,
    query: &crate::models::ItemsQueryRecord,
) -> AppResult<ItemPageRecord> {
    let connection = open_connection(db_path)?;
    let safe_limit = query.limit.clamp(1, 500);
    let safe_offset = query.offset.max(0);

    // Determine feed_ids and station metadata
    let (feed_ids, episode_filter): (Vec<String>, Option<String>) = if let Some(ref station_id) =
        query.station_id
    {
        // Station view: get feeds and episode filter from station
        use super::rows::map_station_row;

        let station = connection
                .query_row(
                    "SELECT id, name, episode_filter, sort_order, sort_order_position, created_at, gradient
				     FROM stations WHERE id = ?1",
                    [station_id],
                    map_station_row,
                )
                .optional()
                .map_err(|error| format!("Failed to query station: {error}"))?
                .ok_or_else(|| "Station not found.".to_string())?;

        let feed_ids = super::stations::get_station_feed_ids(&connection, station_id)?;
        (feed_ids, Some(station.episode_filter))
    } else {
        // Feed or section view: use feed_id if set, otherwise all feeds
        let feed_ids = if let Some(ref feed_id) = query.feed_id {
            vec![feed_id.clone()]
        } else {
            // Get all feed IDs for section views
            let mut stmt = connection
                .prepare("SELECT id FROM feeds")
                .map_err(|error| format!("Failed to prepare feed list: {error}"))?;
            let ids: Result<Vec<String>, _> = stmt
                .query_map([], |row| row.get(0))
                .map_err(|error| format!("Failed to query feeds: {error}"))?
                .collect();
            ids.map_err(|error| format!("Failed to read feeds: {error}"))?
        };
        (feed_ids, None)
    };

    if feed_ids.is_empty() {
        return Ok(ItemPageRecord {
            items: Vec::new(),
            total_count: 0,
        });
    }

    // Build query clauses
    let placeholders: Vec<String> = (1..=feed_ids.len()).map(|i| format!("?{i}")).collect();
    let feed_filter = format!("i.feed_id IN ({})", placeholders.join(", "));

    // Episode filter: use station's filter, or derive from section
    let episode_clause = match (episode_filter.as_deref(), query.section) {
        (Some("unplayed"), _) | (None, ItemListSection::Unread) => " AND i.read = 0",
        _ => "",
    };

    // Media filter for section
    let media_clause = if query.section == ItemListSection::Media {
        " AND i.enclosure_url IS NOT NULL"
    } else {
        ""
    };

    let favorites_clause = if query.section == ItemListSection::Favorites {
        " AND i.favorite = 1"
    } else {
        ""
    };

    // Search clause
    let search_pattern = query
        .search
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    // Determine sort order
    let order_by = query.sort_order.order_by_clause();

    // Build and execute count query
    let search_param_idx = feed_ids.len() + 1;
    let section_clauses = format!("{episode_clause}{media_clause}{favorites_clause}");

    let total_count: i64 = if let Some(_pattern) = &search_pattern {
        let count_sql = format!(
            "SELECT COUNT(*) FROM items i WHERE {feed_filter}{section_clauses} AND (i.title LIKE ?{search_param_idx} COLLATE NOCASE OR i.preview_text LIKE ?{search_param_idx} COLLATE NOCASE OR i.content_text LIKE ?{search_param_idx} COLLATE NOCASE)"
        );
        let mut count_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for fid in &feed_ids {
            count_params.push(Box::new(fid.clone()));
        }
        count_params.push(Box::new(_pattern.clone()));
        let count_refs: Vec<&dyn rusqlite::ToSql> =
            count_params.iter().map(|p| p.as_ref()).collect();

        connection
            .query_row(&count_sql, count_refs.as_slice(), |row| row.get(0))
            .map_err(|error| format!("Failed to count items: {error}"))?
    } else {
        let count_sql = format!(
            "SELECT COUNT(*) FROM items i WHERE {feed_filter}{episode_clause}{media_clause}{favorites_clause}"
        );
        let count_params: Vec<Box<dyn rusqlite::ToSql>> = feed_ids
            .iter()
            .map(|fid| Box::new(fid.clone()) as Box<dyn rusqlite::ToSql>)
            .collect();
        let count_refs: Vec<&dyn rusqlite::ToSql> =
            count_params.iter().map(|p| p.as_ref()).collect();

        connection
            .query_row(&count_sql, count_refs.as_slice(), |row| row.get(0))
            .map_err(|error| format!("Failed to count items: {error}"))?
    };

    // Build and execute page query
    let limit_idx = if search_pattern.is_some() {
        feed_ids.len() + 2
    } else {
        feed_ids.len() + 1
    };
    let offset_idx = limit_idx + 1;

    let page_sql = if let Some(_pattern) = &search_pattern {
        format!(
            "{ITEM_LIST_SELECT_QUERY} WHERE {feed_filter}{section_clauses} AND (i.title LIKE ?{search_param_idx} COLLATE NOCASE OR i.preview_text LIKE ?{search_param_idx} COLLATE NOCASE OR i.content_text LIKE ?{search_param_idx} COLLATE NOCASE) ORDER BY {order_by} LIMIT ?{limit_idx} OFFSET ?{offset_idx}"
        )
    } else {
        format!(
            "{ITEM_LIST_SELECT_QUERY} WHERE {feed_filter}{section_clauses} ORDER BY {order_by} LIMIT ?{limit_idx} OFFSET ?{offset_idx}"
        )
    };

    let mut page_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    for fid in &feed_ids {
        page_params.push(Box::new(fid.clone()));
    }
    if let Some(ref pattern) = search_pattern {
        page_params.push(Box::new(pattern.clone()));
    }
    page_params.push(Box::new(safe_limit));
    page_params.push(Box::new(safe_offset));
    let page_refs: Vec<&dyn rusqlite::ToSql> = page_params.iter().map(|p| p.as_ref()).collect();

    let mut statement = connection
        .prepare(&page_sql)
        .map_err(|error| format!("Failed to prepare items query: {error}"))?;

    let items = statement
        .query_map(page_refs.as_slice(), map_item_list_row)
        .map_err(|error| format!("Failed to query items: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read items: {error}"))?;

    Ok(ItemPageRecord { items, total_count })
}

pub fn get_unread_counts_by_feed(db_path: &Path) -> AppResult<HashMap<String, i64>> {
    let connection = open_connection(db_path)?;
    let mut statement = connection
        .prepare("SELECT feed_id, COUNT(*) FROM items WHERE read = 0 GROUP BY feed_id")
        .map_err(|error| format!("Failed to prepare unread counts query: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            let feed_id: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((feed_id, count))
        })
        .map_err(|error| format!("Failed to query unread counts: {error}"))?;

    let mut result = HashMap::new();
    for row in rows {
        let (feed_id, count) =
            row.map_err(|error| format!("Failed to read unread count: {error}"))?;
        result.insert(feed_id, count);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::feeds::upsert_feed_snapshot;
    use crate::db::schema::initialize_database;
    use crate::models::{ParsedFeed, ParsedFeedItem};
    use tempfile::TempDir;

    fn tmpdb() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("test.db");
        initialize_database(&db_path).expect("init");
        (dir, db_path)
    }

    /// Insert a feed with one article item; return the derived item ID.
    fn insert_item(db_path: &Path) -> String {
        insert_item_with_external_id(db_path, "ext-1", "Article 1")
    }

    fn insert_item_with_external_id(db_path: &Path, external_id: &str, title: &str) -> String {
        let feed = upsert_feed_snapshot(
            db_path,
            "https://example.com/rss",
            ParsedFeed {
                title: "Feed".to_string(),
                description: String::new(),
                site_url: None,
                image_url: None,
                kind: "article".to_string(),
                items: vec![ParsedFeedItem {
                    external_id: external_id.to_string(),
                    title: title.to_string(),
                    url: "https://example.com/1".to_string(),
                    summary: String::new(),
                    preview_text: String::new(),
                    summary_text: None,
                    summary_html: None,
                    content_text: None,
                    content_html: None,
                    published_at: "2024-01-01T00:00:00Z".to_string(),
                    media_enclosure: None,
                    image_url: None,
                }],
            },
        )
        .unwrap();
        // Mirror the stable_hash + build_item_id logic from feeds.rs.
        format!(
            "item-{}",
            sha1_smol::Sha1::from(format!("{}:{}", feed.id, external_id)).digest()
        )
    }

    #[test]
    fn get_item_by_id_returns_item() {
        let (_dir, db_path) = tmpdb();
        let item_id = insert_item(&db_path);
        let item = get_item_by_id(&db_path, &item_id).unwrap().unwrap();
        assert_eq!(item.title, "Article 1");
        assert!(!item.read);
        assert_eq!(item.playback_position_seconds, 0);
    }

    #[test]
    fn get_item_by_id_returns_none_for_unknown() {
        let (_dir, db_path) = tmpdb();
        assert!(get_item_by_id(&db_path, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn mark_read_true_sets_flag() {
        let (_dir, db_path) = tmpdb();
        let item_id = insert_item(&db_path);
        mark_read(&db_path, &item_id, true).unwrap();
        let item = get_item_by_id(&db_path, &item_id).unwrap().unwrap();
        assert!(item.read);
    }

    #[test]
    fn mark_read_false_clears_flag() {
        let (_dir, db_path) = tmpdb();
        let item_id = insert_item(&db_path);
        mark_read(&db_path, &item_id, true).unwrap();
        mark_read(&db_path, &item_id, false).unwrap();
        let item = get_item_by_id(&db_path, &item_id).unwrap().unwrap();
        assert!(!item.read);
    }

    #[test]
    fn mark_favorite_true_sets_flag() {
        let (_dir, db_path) = tmpdb();
        let item_id = insert_item(&db_path);
        mark_favorite(&db_path, &item_id, true).unwrap();
        let item = get_item_by_id(&db_path, &item_id).unwrap().unwrap();
        assert!(item.favorite);
    }

    #[test]
    fn mark_favorite_false_clears_flag() {
        let (_dir, db_path) = tmpdb();
        let item_id = insert_item(&db_path);
        mark_favorite(&db_path, &item_id, true).unwrap();
        mark_favorite(&db_path, &item_id, false).unwrap();
        let item = get_item_by_id(&db_path, &item_id).unwrap().unwrap();
        assert!(!item.favorite);
    }

    #[test]
    fn mark_favorite_batch_sets_flags() {
        let (_dir, db_path) = tmpdb();
        let first_item_id = insert_item(&db_path);
        let second_item_id = insert_item_with_external_id(&db_path, "ext-2", "Article 2");
        mark_favorite_batch(
            &db_path,
            &[first_item_id.clone(), second_item_id.clone()],
            true,
        )
        .unwrap();
        assert!(
            get_item_by_id(&db_path, &first_item_id)
                .unwrap()
                .unwrap()
                .favorite
        );
        assert!(
            get_item_by_id(&db_path, &second_item_id)
                .unwrap()
                .unwrap()
                .favorite
        );
    }

    #[test]
    fn save_playback_persists_position() {
        let (_dir, db_path) = tmpdb();
        let item_id = insert_item(&db_path);
        save_playback(&db_path, &item_id, 42).unwrap();
        let item = get_item_by_id(&db_path, &item_id).unwrap().unwrap();
        assert_eq!(item.playback_position_seconds, 42);
    }

    #[test]
    fn save_playback_negative_clamped_to_zero() {
        let (_dir, db_path) = tmpdb();
        let item_id = insert_item(&db_path);
        save_playback(&db_path, &item_id, -5).unwrap();
        let item = get_item_by_id(&db_path, &item_id).unwrap().unwrap();
        assert_eq!(item.playback_position_seconds, 0);
    }

    #[test]
    fn save_playback_upserts_on_second_call() {
        let (_dir, db_path) = tmpdb();
        let item_id = insert_item(&db_path);
        save_playback(&db_path, &item_id, 10).unwrap();
        save_playback(&db_path, &item_id, 77).unwrap();
        let item = get_item_by_id(&db_path, &item_id).unwrap().unwrap();
        assert_eq!(item.playback_position_seconds, 77);
    }
}
