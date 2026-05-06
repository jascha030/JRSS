use super::AppResult;
use super::connection::open_connection;
use super::rows::map_station_row;
use crate::models::{
    CreateStationInput, ItemPageRecord, ItemSortOrder, StationWithFeedsRecord, UpdateStationInput,
};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use std::path::Path;

fn build_station_id() -> String {
    format!("station-{}", uuid::Uuid::new_v4())
}

pub fn get_station_feed_ids(connection: &Connection, station_id: &str) -> AppResult<Vec<String>> {
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

    let station = crate::models::StationRecord {
        id: station_id.clone(),
        name: input.name.clone(),
        episode_filter: input.episode_filter.as_str().to_string(),
        sort_order: match input.sort_order {
            ItemSortOrder::NewestFirst => "newest_first".to_string(),
            ItemSortOrder::OldestFirst => "oldest_first".to_string(),
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
            ItemSortOrder::NewestFirst => "newest_first".to_string(),
            ItemSortOrder::OldestFirst => "oldest_first".to_string(),
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
    search: Option<&str>,
) -> AppResult<ItemPageRecord> {
    use super::items::ITEM_LIST_SELECT_QUERY;
    use crate::db::rows::map_item_list_row;

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

    // Search clause
    let search_pattern = search
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));

    let order_by = if station.sort_order == "oldest_first" {
        "i.published_at ASC, i.id ASC"
    } else {
        "i.published_at DESC, i.id DESC"
    };

    // Count
    let search_param_idx = feed_ids.len() + 1;
    let count_sql = if let Some(ref _pattern) = search_pattern {
        format!(
            "SELECT COUNT(*) FROM items i WHERE {feed_filter}{episode_clause}{enclosure_clause} AND (i.title LIKE ?{search_param_idx} COLLATE NOCASE OR i.preview_text LIKE ?{search_param_idx} COLLATE NOCASE OR i.content_text LIKE ?{search_param_idx} COLLATE NOCASE)"
        )
    } else {
        format!(
            "SELECT COUNT(*) FROM items i WHERE {feed_filter}{episode_clause}{enclosure_clause}"
        )
    };
    let mut count_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    for fid in &feed_ids {
        count_params.push(Box::new(fid.clone()));
    }
    if let Some(ref pattern) = search_pattern {
        count_params.push(Box::new(pattern.clone()));
    }
    let count_refs: Vec<&dyn rusqlite::ToSql> = count_params.iter().map(|p| p.as_ref()).collect();

    let total_count: i64 = connection
        .query_row(&count_sql, count_refs.as_slice(), |row| row.get(0))
        .map_err(|error| format!("Failed to count station episodes: {error}"))?;

    // Page query
    let limit_idx = if search_pattern.is_some() {
        feed_ids.len() + 2
    } else {
        feed_ids.len() + 1
    };
    let offset_idx = limit_idx + 1;

    let page_sql = if search_pattern.is_some() {
        format!(
            "{ITEM_LIST_SELECT_QUERY} WHERE {feed_filter}{episode_clause}{enclosure_clause} AND (i.title LIKE ?{search_param_idx} COLLATE NOCASE OR i.preview_text LIKE ?{search_param_idx} COLLATE NOCASE OR i.content_text LIKE ?{search_param_idx} COLLATE NOCASE) ORDER BY {order_by} LIMIT ?{limit_idx} OFFSET ?{offset_idx}"
        )
    } else {
        format!(
            "{ITEM_LIST_SELECT_QUERY} WHERE {feed_filter}{episode_clause}{enclosure_clause} ORDER BY {order_by} LIMIT ?{limit_idx} OFFSET ?{offset_idx}"
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
        .map_err(|error| format!("Failed to prepare station episodes query: {error}"))?;

    let items = statement
        .query_map(page_refs.as_slice(), map_item_list_row)
        .map_err(|error| format!("Failed to query station episodes: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read station episodes: {error}"))?;

    Ok(ItemPageRecord { items, total_count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::db::schema::initialize_database;
    use crate::models::{CreateStationInput, ItemSortOrder, StationEpisodeFilter, UpdateStationInput};

    fn tmpdb() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("test.db");
        initialize_database(&db_path).expect("init");
        (dir, db_path)
    }

    fn make_input(name: &str) -> CreateStationInput {
        CreateStationInput {
            name: name.to_string(),
            episode_filter: StationEpisodeFilter::All,
            sort_order: ItemSortOrder::NewestFirst,
            feed_ids: Vec::new(),
        }
    }

    #[test]
    fn create_station_succeeds() {
        let (_dir, db_path) = tmpdb();
        let station = create_station(&db_path, &make_input("My Station")).unwrap();
        assert_eq!(station.station.name, "My Station");
        assert!(station.station.id.starts_with("station-"));
    }

    #[test]
    fn list_stations_returns_all_created() {
        let (_dir, db_path) = tmpdb();
        create_station(&db_path, &make_input("Station A")).unwrap();
        create_station(&db_path, &make_input("Station B")).unwrap();
        assert_eq!(list_stations(&db_path).unwrap().len(), 2);
    }

    #[test]
    fn list_stations_empty_on_fresh_db() {
        let (_dir, db_path) = tmpdb();
        assert!(list_stations(&db_path).unwrap().is_empty());
    }

    #[test]
    fn delete_station_removes_it() {
        let (_dir, db_path) = tmpdb();
        let station = create_station(&db_path, &make_input("My Station")).unwrap();
        delete_station(&db_path, &station.station.id).unwrap();
        assert!(list_stations(&db_path).unwrap().is_empty());
    }

    #[test]
    fn update_station_changes_name() {
        let (_dir, db_path) = tmpdb();
        let station = create_station(&db_path, &make_input("Original")).unwrap();
        update_station(
            &db_path,
            &UpdateStationInput {
                id: station.station.id.clone(),
                name: Some("Updated".to_string()),
                episode_filter: None,
                sort_order: None,
                feed_ids: None,
            },
        ).unwrap();
        let stations = list_stations(&db_path).unwrap();
        assert_eq!(stations[0].station.name, "Updated");
    }

    #[test]
    fn station_sort_order_position_increments() {
        let (_dir, db_path) = tmpdb();
        let s1 = create_station(&db_path, &make_input("First")).unwrap();
        let s2 = create_station(&db_path, &make_input("Second")).unwrap();
        assert!(s2.station.sort_order_position > s1.station.sort_order_position);
    }
}
