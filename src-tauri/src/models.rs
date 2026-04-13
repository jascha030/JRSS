use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemListSection {
    All,
    Unread,
    Podcasts,
}

impl ItemListSection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Unread => "unread",
            Self::Podcasts => "podcasts",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemSortOrder {
    NewestFirst,
    OldestFirst,
}

impl ItemSortOrder {
    pub fn order_by_clause(self) -> &'static str {
        match self {
            Self::NewestFirst => "i.published_at DESC, i.id DESC",
            Self::OldestFirst => "i.published_at ASC, i.id ASC",
        }
    }
}

impl Default for ItemSortOrder {
    fn default() -> Self {
        Self::NewestFirst
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPageQueryRecord {
    pub feed_id: Option<String>,
    pub section: ItemListSection,
    pub offset: i64,
    pub limit: i64,
    pub search: Option<String>,
    #[serde(default)]
    pub sort_order: ItemSortOrder,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPageRecord {
    pub items: Vec<FeedListItemRecord>,
    pub total_count: i64,
}

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
    pub sort_order: Option<String>,
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
pub struct FeedListItemRecord {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub url: String,
    pub summary: String,
    pub preview_text: String,
    pub reader_status: String,
    pub reader_title: Option<String>,
    pub reader_byline: Option<String>,
    pub reader_excerpt: Option<String>,
    pub reader_fetched_at: Option<String>,
    pub published_at: String,
    pub read: bool,
    pub playback_position_seconds: i64,
    pub media_enclosure: Option<MediaEnclosureRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedItemRecord {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub url: String,
    pub summary: String,
    pub preview_text: String,
    pub summary_text: Option<String>,
    pub summary_html: Option<String>,
    pub content_text: Option<String>,
    pub content_html: Option<String>,
    pub reader_status: String,
    pub reader_title: Option<String>,
    pub reader_byline: Option<String>,
    pub reader_excerpt: Option<String>,
    pub reader_content_html: Option<String>,
    pub reader_content_text: Option<String>,
    pub reader_fetched_at: Option<String>,
    pub published_at: String,
    pub read: bool,
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
    pub preview_text: String,
    pub summary_text: Option<String>,
    pub summary_html: Option<String>,
    pub content_text: Option<String>,
    pub content_html: Option<String>,
    pub published_at: String,
    pub media_enclosure: Option<MediaEnclosureRecord>,
}

#[derive(Debug, Clone)]
pub struct ReaderContentRecord {
    pub title: String,
    pub byline: Option<String>,
    pub excerpt: Option<String>,
    pub content_html: Option<String>,
    pub content_text: Option<String>,
    pub fetched_at: String,
}

/// Persisted playback session — stored as an opaque JSON blob in SQLite.
/// The frontend owns the shape; the backend just stores and returns it.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSessionRecord {
    pub current_item_id: String,
    pub position_seconds: i64,
    pub duration_seconds: i64,
    pub manual_queue: Vec<String>,
    pub auto_queue: Vec<String>,
}
