use crate::models::{
    FeedItemRecord, FeedListItemRecord, FeedRecord, MediaEnclosureRecord, StationRecord,
};
use rusqlite::Row;

pub fn map_feed_row(row: &Row<'_>) -> rusqlite::Result<FeedRecord> {
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

pub fn map_item_row(row: &Row<'_>) -> rusqlite::Result<FeedItemRecord> {
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
        image_url: row.get(24)?,
    })
}

pub fn map_item_list_row(row: &Row<'_>) -> rusqlite::Result<FeedListItemRecord> {
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
        image_url: row.get(18)?,
    })
}

pub fn map_station_row(row: &Row<'_>) -> rusqlite::Result<StationRecord> {
    Ok(StationRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        episode_filter: row.get(2)?,
        sort_order: row.get(3)?,
        sort_order_position: row.get(4)?,
        created_at: row.get(5)?,
        gradient: row.get(6)?,
    })
}
