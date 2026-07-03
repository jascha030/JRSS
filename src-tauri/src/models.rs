use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ItemListSection {
    #[default]
    All,
    Unread,
    Media,
    Favorites,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ItemSortOrder {
    #[default]
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

/// Unified items query that handles all cases: feed, station, or section views.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsQueryRecord {
    /// If set, query items from this specific feed only
    pub feed_id: Option<String>,
    /// If set, query items from this station's feeds
    pub station_id: Option<String>,
    /// Section filter: all, unread, or media (only used when feed_id is None)
    #[serde(default)]
    pub section: ItemListSection,
    /// Pagination offset
    pub offset: i64,
    /// Pagination limit
    pub limit: i64,
    /// Optional search term (filters title, preview_text, content_text)
    pub search: Option<String>,
    /// Sort order for results
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
pub struct PodcastSearchResultRecord {
    pub name: String,
    pub artist: String,
    pub feed_url: String,
    pub artwork_url: Option<String>,
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
    pub image_url: Option<String>,
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
    pub favorite: bool,
    pub playback_position_seconds: i64,
    pub media_enclosure: Option<MediaEnclosureRecord>,
    pub image_url: Option<String>,
    pub episode_number: Option<i64>,
    pub season_number: Option<i64>,
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
    pub favorite: bool,
    pub playback_position_seconds: i64,
    pub media_enclosure: Option<MediaEnclosureRecord>,
    pub image_url: Option<String>,
    pub episode_number: Option<i64>,
    pub season_number: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ParsedFeed {
    pub title: String,
    pub description: String,
    pub site_url: Option<String>,
    pub image_url: Option<String>,
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
    pub image_url: Option<String>,
    pub episode_number: Option<i64>,
    pub season_number: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemLocalStatusRecord {
    pub item_id: String,
    pub is_cached: bool,
    pub is_exported: bool,
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StationEpisodeFilter {
    All,
    Unplayed,
}

impl StationEpisodeFilter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Unplayed => "unplayed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StationRecord {
    pub id: String,
    pub name: String,
    pub episode_filter: String,
    pub sort_order: String,
    pub sort_order_position: i64,
    pub created_at: String,
    pub gradient: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StationWithFeedsRecord {
    #[serde(flatten)]
    pub station: StationRecord,
    pub feed_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStationInput {
    pub name: String,
    pub episode_filter: StationEpisodeFilter,
    pub sort_order: ItemSortOrder,
    pub feed_ids: Vec<String>,
    pub gradient: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStationInput {
    pub id: String,
    pub name: Option<String>,
    pub episode_filter: Option<StationEpisodeFilter>,
    pub sort_order: Option<ItemSortOrder>,
    pub feed_ids: Option<Vec<String>>,
    pub gradient: Option<String>,
}

/// UI color scheme preference. Serialises as lowercase (`"system"`, `"light"`, `"dark"`)
/// to match the frontend `ColorScheme` type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    /// Follow the OS `prefers-color-scheme` setting.
    #[default]
    System,
    Light,
    Dark,
}

impl ColorScheme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

impl std::str::FromStr for ColorScheme {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "system" => Ok(Self::System),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            // Unrecognised values (e.g. from a future version being downgraded) fall back to
            // the default rather than hard-failing, keeping old databases openable.
            _ => Err(()),
        }
    }
}

impl ToSql for ColorScheme {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for ColorScheme {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(|s| s.parse().unwrap_or_default())
    }
}

/// Frontend-managed playback context for feed/station tracking.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackContextRecord {
    pub context_type: String,
    pub id: String,
}

/// Persisted playback session — stored as an opaque JSON blob in SQLite.
/// The frontend owns the shape; the backend just stores and returns it.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSessionRecord {
    pub current_item_id: Option<String>,
    pub position_seconds: i64,
    pub duration_seconds: i64,
    pub history_queue: Vec<String>,
    pub manual_queue: Vec<String>,
    pub auto_queue: Vec<String>,
    pub playback_context: Option<PlaybackContextRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsRecord {
    pub max_audio_cache_size_bytes: i64,
    pub max_image_cache_size_bytes: i64,
    pub mini_player_always_on_top: bool,
    /// Interval between automatic background feed refreshes, in minutes.
    /// `0` disables auto-refresh.
    pub auto_refresh_interval_minutes: i64,
    /// UI color scheme preference.
    pub color_scheme: ColorScheme,
    /// Custom accent color override as a CSS hex string (e.g. `"#4f46e5"`), or `null` to use the
    /// theme default.
    pub accent_color: Option<String>,
    pub skip_forward_seconds: i64,
    pub skip_backward_seconds: i64,
    /// Filename of the active user theme (e.g. `"my-theme.css"`), or `null` to use the built-in
    /// default.
    pub theme_name: Option<String>,
}
