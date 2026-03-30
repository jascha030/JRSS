use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedRecord {
    pub id: String,
    pub title: String,
    pub url: String,
    pub description: String,
    pub kind: String,
    pub site_url: Option<String>,
    pub created_at: String,
    pub last_fetched_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaEnclosureRecord {
    pub url: String,
    pub mime_type: String,
    pub size_bytes: Option<i64>,
    pub duration_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedItemRecord {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub url: String,
    pub summary: String,
    pub published_at: String,
    pub read: bool,
    pub saved: bool,
    pub playback_position_seconds: i64,
    pub media_enclosure: Option<MediaEnclosureRecord>,
}

#[derive(Debug, Clone)]
pub struct ParsedFeed {
    pub title: String,
    pub description: String,
    pub site_url: Option<String>,
    pub kind: String,
    pub items: Vec<ParsedFeedItem>,
}

#[derive(Debug, Clone)]
pub struct ParsedFeedItem {
    pub external_id: String,
    pub title: String,
    pub url: String,
    pub summary: String,
    pub published_at: String,
    pub media_enclosure: Option<MediaEnclosureRecord>,
}
