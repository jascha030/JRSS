use crate::db::AppResult;
use crate::models::{MediaEnclosureRecord, ParsedFeed, ParsedFeedItem};
use ammonia::Builder;
use atom_syndication::{Entry as AtomEntry, Feed as AtomFeed, Link as AtomLink, Text, TextType};
use chrono::{DateTime, Utc};
use html_escape::decode_html_entities;
use regex::Regex;
use reqwest::blocking::Client;
use rss::{Channel as RssChannel, Item as RssItem};
use serde::Deserialize;
use std::io::Cursor;
use std::sync::OnceLock;
use std::time::Duration;
use url::Url;

const APPLE_LOOKUP_URL: &str = "https://itunes.apple.com/lookup";
const APPLE_LOOKUP_ACCEPT_HEADER: &str = "application/json";
const FEED_ACCEPT_HEADER: &str =
    "application/atom+xml, application/rss+xml, application/xml, text/xml;q=0.9, */*;q=0.8";
const SUMMARY_PREVIEW_MAX_CHARS: usize = 420;

#[derive(Debug, Clone)]
pub struct ResolvedFeedInput {
    pub feed_url: String,
    source: FeedInputSource,
}

impl ResolvedFeedInput {
    fn new(feed_url: String, source: FeedInputSource) -> Self {
        Self { feed_url, source }
    }

    pub fn map_fetch_error(&self, error: String) -> String {
        if self.source != FeedInputSource::ApplePodcasts {
            return error;
        }

        if is_feed_fetch_error(&error) {
            return format!("Resolved Apple Podcasts feed could not be fetched. {error}");
        }

        format!("Resolved Apple Podcasts feed could not be parsed. {error}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeedInputSource {
    DirectUrl,
    ApplePodcasts,
}

#[derive(Debug, Deserialize)]
struct AppleLookupResponse {
    results: Vec<AppleLookupResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppleLookupResult {
    kind: Option<String>,
    feed_url: Option<String>,
}

pub fn resolve_feed_input(input: &str) -> AppResult<ResolvedFeedInput> {
    let candidate = input.trim();

    if candidate.is_empty() {
        return Err("Enter a feed URL, Apple Podcasts URL, or Apple Podcasts ID.".to_string());
    }

    if let Some(podcast_id) = extract_raw_apple_podcast_id(candidate) {
        let feed_url = lookup_apple_podcast_feed_url(&podcast_id)?;
        return Ok(ResolvedFeedInput::new(
            feed_url,
            FeedInputSource::ApplePodcasts,
        ));
    }

    let parsed_url = Url::parse(candidate).map_err(|_| {
        "Enter a valid feed URL, Apple Podcasts URL, or Apple Podcasts ID.".to_string()
    })?;

    if is_apple_podcasts_url(&parsed_url) {
        let podcast_id = extract_apple_podcast_id_from_url(&parsed_url)
            .ok_or_else(|| "Could not extract an Apple Podcasts ID from this URL.".to_string())?;
        let feed_url = lookup_apple_podcast_feed_url(&podcast_id)?;

        return Ok(ResolvedFeedInput::new(
            feed_url,
            FeedInputSource::ApplePodcasts,
        ));
    }

    Ok(ResolvedFeedInput::new(
        normalize_feed_url(candidate)?,
        FeedInputSource::DirectUrl,
    ))
}

pub fn normalize_feed_url(url: &str) -> AppResult<String> {
    let parsed_url = Url::parse(url).map_err(|_| "Enter a valid feed URL.".to_string())?;
    let scheme = parsed_url.scheme();

    if scheme != "http" && scheme != "https" {
        return Err("Feed URLs must use http or https.".to_string());
    }

    Ok(parsed_url.to_string())
}

pub fn fetch_and_parse_feed(feed_url: &str) -> AppResult<ParsedFeed> {
    let client = build_http_client()?;

    let response = client
        .get(feed_url)
        .header(reqwest::header::ACCEPT, FEED_ACCEPT_HEADER)
        .send()
        .map_err(|error| format!("Failed to fetch feed: {error}"))?;

    let status = response.status();

    if !status.is_success() {
        return Err(format!("Feed request failed with status {status}."));
    }

    let bytes = response
        .bytes()
        .map_err(|error| format!("Failed to read feed response body: {error}"))?;

    parse_feed(&bytes, feed_url)
}

fn build_http_client() -> AppResult<Client> {
    Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("JRSS/0.0.1")
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))
}

fn extract_raw_apple_podcast_id(input: &str) -> Option<String> {
    let candidate = input.trim();

    if candidate.is_empty()
        || !candidate
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return None;
    }

    Some(candidate.to_string())
}

fn is_apple_podcasts_url(url: &Url) -> bool {
    matches!(
        url.host_str(),
        Some("podcasts.apple.com") | Some("itunes.apple.com")
    )
}

fn extract_apple_podcast_id_from_url(url: &Url) -> Option<String> {
    apple_podcast_id_regex()
        .captures(url.path())
        .and_then(|captures| captures.get(1))
        .map(|capture| capture.as_str().to_string())
}

fn lookup_apple_podcast_feed_url(podcast_id: &str) -> AppResult<String> {
    let client = build_http_client()?;
    let lookup_url = Url::parse_with_params(APPLE_LOOKUP_URL, &[("id", podcast_id)])
        .map_err(|error| format!("Failed to build Apple Podcasts lookup URL: {error}"))?;
    let response = client
        .get(lookup_url)
        .header(reqwest::header::ACCEPT, APPLE_LOOKUP_ACCEPT_HEADER)
        .send()
        .map_err(|error| format!("Apple Podcasts lookup request failed: {error}"))?;
    let status = response.status();

    if !status.is_success() {
        return Err(format!(
            "Apple Podcasts lookup request failed with status {status}."
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|error| format!("Failed to read Apple Podcasts lookup response: {error}"))?;
    let lookup_response =
        serde_json::from_slice::<AppleLookupResponse>(&bytes).map_err(|error| {
            format!("Apple Podcasts returned an unreadable lookup response: {error}")
        })?;

    select_apple_lookup_feed_url(lookup_response)
}

fn select_apple_lookup_feed_url(response: AppleLookupResponse) -> AppResult<String> {
    if response.results.is_empty() {
        return Err("Apple Podcasts did not return any results for this show.".to_string());
    }

    if let Some(feed_url) = response
        .results
        .first()
        .and_then(normalized_lookup_feed_url)
    {
        return Ok(feed_url);
    }

    if let Some(feed_url) = response
        .results
        .iter()
        .filter(|result| result.kind.as_deref() == Some("podcast"))
        .find_map(normalized_lookup_feed_url)
    {
        return Ok(feed_url);
    }

    Err("Apple Podcasts did not return a public RSS feed for this show.".to_string())
}

fn normalized_lookup_feed_url(result: &AppleLookupResult) -> Option<String> {
    result
        .feed_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| normalize_feed_url(value).ok())
}

fn is_feed_fetch_error(error: &str) -> bool {
    error.starts_with("Failed to fetch feed:")
        || error.starts_with("Feed request failed with status")
        || error.starts_with("Failed to read feed response body:")
}

fn parse_feed(xml_bytes: &[u8], feed_url: &str) -> AppResult<ParsedFeed> {
    if let Ok(channel) = RssChannel::read_from(Cursor::new(xml_bytes)) {
        return Ok(parse_rss(channel, feed_url));
    }

    if let Ok(feed) = AtomFeed::read_from(Cursor::new(xml_bytes)) {
        return Ok(parse_atom(feed, feed_url));
    }

    Err("Unsupported or malformed RSS/Atom feed.".to_string())
}

fn parse_rss(channel: RssChannel, feed_url: &str) -> ParsedFeed {
    let items = channel
        .items()
        .iter()
        .map(|item| parse_rss_item(item, feed_url))
        .collect::<Vec<_>>();
    let has_audio = items.iter().any(|item| item.media_enclosure.is_some());
    let is_podcast = has_audio || channel.itunes_ext().is_some();

    let site_url = resolve_optional_url(Some(channel.link()), feed_url);

    let image_url = channel
        .itunes_ext()
        .and_then(|itunes| itunes.image())
        .and_then(|url| resolve_optional_url(Some(url), feed_url))
        .or_else(|| {
            channel
                .image()
                .and_then(|image| resolve_optional_url(Some(image.url()), feed_url))
        })
        .or_else(|| favicon_url_from_origin(site_url.as_deref(), feed_url));

    ParsedFeed {
        title: fallback_string(
            Some(channel.title()),
            Url::parse(feed_url)
                .ok()
                .and_then(|url| url.host_str().map(str::to_string))
                .as_deref(),
            "Untitled feed",
        ),
        description: clean_text(channel.description())
            .unwrap_or_else(|| "No description provided.".to_string()),
        site_url,
        image_url,
        kind: if is_podcast {
            "podcast".to_string()
        } else {
            "rss".to_string()
        },
        items,
    }
}

fn parse_rss_item(item: &RssItem, feed_url: &str) -> ParsedFeedItem {
    let published_at = item
        .pub_date()
        .map(parse_datetime_string)
        .unwrap_or_else(now_iso_string);
    let link_url =
        resolve_optional_url(item.link(), feed_url).unwrap_or_else(|| feed_url.to_string());
    let title = fallback_string(item.title(), None, "Untitled item");
    let content_text = item.content().and_then(clean_text);
    let content_html = item
        .content()
        .and_then(raw_html_value)
        .filter(|value| looks_like_html(value))
        .and_then(|html| sanitize_feed_html(&html));
    let summary_text = extract_rss_summary_text(item);
    let summary_html = extract_rss_summary_html(item).and_then(|html| sanitize_feed_html(&html));
    let summary = item
        .content()
        .and_then(clean_summary)
        .or_else(|| item.description().and_then(clean_summary))
        .or_else(|| {
            item.itunes_ext()
                .and_then(|itunes| itunes.summary())
                .and_then(clean_summary)
        })
        .unwrap_or_else(|| "No summary provided.".to_string());
    let preview_text = build_preview_text(
        content_text.as_deref(),
        summary_text.as_deref(),
        Some(summary.as_str()),
    );
    let enclosure = parse_rss_enclosure(item, feed_url);
    let external_id = item
        .guid()
        .map(|guid| guid.value())
        .map(str::to_string)
        .or_else(|| item.link().map(str::to_string))
        .unwrap_or_else(|| format!("{title}-{published_at}"));

    ParsedFeedItem {
        external_id,
        title,
        url: link_url,
        summary,
        preview_text,
        summary_text,
        summary_html,
        content_text,
        content_html,
        published_at,
        media_enclosure: enclosure,
    }
}

fn extract_rss_summary_text(item: &RssItem) -> Option<String> {
    item.description().and_then(clean_text).or_else(|| {
        item.itunes_ext()
            .and_then(|itunes| itunes.summary())
            .and_then(clean_text)
    })
}

fn extract_rss_summary_html(item: &RssItem) -> Option<String> {
    item.description()
        .and_then(raw_html_value)
        .filter(|value| looks_like_html(value))
}

fn parse_rss_enclosure(item: &RssItem, feed_url: &str) -> Option<MediaEnclosureRecord> {
    let duration_seconds = item
        .itunes_ext()
        .and_then(|itunes| itunes.duration())
        .and_then(parse_duration_seconds);

    item.enclosure().and_then(|enclosure| {
        build_enclosure(
            Some(enclosure.url()),
            feed_url,
            Some(enclosure.mime_type()),
            Some(enclosure.length()),
            duration_seconds,
        )
    })
}

fn parse_atom(feed: AtomFeed, feed_url: &str) -> ParsedFeed {
    let items = feed
        .entries()
        .iter()
        .map(|entry| parse_atom_entry(entry, feed_url))
        .collect::<Vec<_>>();
    let has_audio = items.iter().any(|item| item.media_enclosure.is_some());

    let site_url = select_atom_link(feed.links(), feed_url, "alternate");

    let image_url = feed
        .logo()
        .and_then(|url| resolve_optional_url(Some(url), feed_url))
        .or_else(|| {
            feed.icon()
                .and_then(|url| resolve_optional_url(Some(url), feed_url))
        })
        .or_else(|| favicon_url_from_origin(site_url.as_deref(), feed_url));

    ParsedFeed {
        title: fallback_string(Some(feed.title().as_str()), None, "Untitled feed"),
        description: feed
            .subtitle()
            .and_then(|subtitle| clean_text(subtitle.as_str()))
            .unwrap_or_else(|| "No description provided.".to_string()),
        site_url,
        image_url,
        kind: if has_audio {
            "podcast".to_string()
        } else {
            "rss".to_string()
        },
        items,
    }
}

fn parse_atom_entry(entry: &AtomEntry, feed_url: &str) -> ParsedFeedItem {
    let published_at = entry
        .published()
        .map(|published| published.to_rfc3339())
        .unwrap_or_else(|| entry.updated().to_rfc3339());
    let link_url = select_atom_link(entry.links(), feed_url, "alternate")
        .unwrap_or_else(|| feed_url.to_string());
    let title = fallback_string(Some(entry.title().as_str()), None, "Untitled item");
    let summary_text = extract_atom_summary_text(entry);
    let summary_html = extract_atom_summary_html(entry).and_then(|html| sanitize_feed_html(&html));
    let content_text = extract_atom_content_text(entry);
    let content_html = extract_atom_content_html(entry).and_then(|html| sanitize_feed_html(&html));
    let summary = extract_atom_summary(entry).unwrap_or_else(|| "No summary provided.".to_string());
    let preview_text = build_preview_text(
        content_text.as_deref(),
        summary_text.as_deref(),
        Some(summary.as_str()),
    );
    let enclosure = entry
        .links()
        .iter()
        .find_map(|link| parse_atom_enclosure(link, feed_url));

    ParsedFeedItem {
        external_id: if entry.id().trim().is_empty() {
            format!("{title}-{published_at}")
        } else {
            entry.id().to_string()
        },
        title,
        url: link_url,
        summary,
        preview_text,
        summary_text,
        summary_html,
        content_text,
        content_html,
        published_at,
        media_enclosure: enclosure,
    }
}

fn parse_atom_enclosure(link: &AtomLink, feed_url: &str) -> Option<MediaEnclosureRecord> {
    if link.rel() != "enclosure" {
        return None;
    }

    build_enclosure(
        Some(link.href()),
        feed_url,
        link.mime_type(),
        link.length(),
        None,
    )
}

fn extract_atom_summary(entry: &AtomEntry) -> Option<String> {
    entry
        .summary()
        .and_then(|summary| clean_summary(summary.as_str()))
        .or_else(|| {
            entry
                .content()
                .and_then(|content| content.value())
                .and_then(clean_summary)
        })
}

fn extract_atom_summary_text(entry: &AtomEntry) -> Option<String> {
    entry
        .summary()
        .and_then(|summary| clean_text(summary.as_str()))
}

fn extract_atom_summary_html(entry: &AtomEntry) -> Option<String> {
    entry.summary().and_then(raw_atom_text_html)
}

fn extract_atom_content_text(entry: &AtomEntry) -> Option<String> {
    entry
        .content()
        .and_then(|content| content.value())
        .and_then(clean_text)
}

fn extract_atom_content_html(entry: &AtomEntry) -> Option<String> {
    entry.content().and_then(|content| {
        let raw_value = content.value().and_then(raw_html_value)?;
        let content_type = content
            .content_type()
            .unwrap_or("text")
            .trim()
            .to_ascii_lowercase();

        if matches!(content_type.as_str(), "html" | "xhtml") || looks_like_html(&raw_value) {
            Some(raw_value)
        } else {
            None
        }
    })
}

fn build_enclosure(
    url: Option<&str>,
    base_url: &str,
    mime_type: Option<&str>,
    size_bytes: Option<&str>,
    duration_seconds: Option<i64>,
) -> Option<MediaEnclosureRecord> {
    let resolved_url = resolve_optional_url(url, base_url)?;

    if !is_audio_mime_type(mime_type) && !is_likely_audio_url(&resolved_url) {
        return None;
    }

    Some(MediaEnclosureRecord {
        url: resolved_url.clone(),
        mime_type: mime_type
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| guess_audio_mime_type(&resolved_url)),
        size_bytes: size_bytes.and_then(parse_i64),
        duration_seconds,
    })
}

fn select_atom_link(links: &[AtomLink], feed_url: &str, rel: &str) -> Option<String> {
    links
        .iter()
        .find(|link| {
            let link_rel = link.rel();
            if rel == "alternate" {
                link_rel.is_empty() || link_rel == "alternate"
            } else {
                link_rel == rel
            }
        })
        .and_then(|link| resolve_optional_url(Some(link.href()), feed_url))
}

fn resolve_optional_url(candidate: Option<&str>, base_url: &str) -> Option<String> {
    let candidate = candidate?.trim();

    if candidate.is_empty() {
        return None;
    }

    Url::parse(candidate)
        .map(|url| url.to_string())
        .or_else(|_| {
            Url::parse(base_url)
                .and_then(|base| base.join(candidate))
                .map(|url| url.to_string())
        })
        .ok()
}

/// Derives a Google Favicon Service URL from the site URL or feed URL origin.
/// Used as a last-resort image when the feed itself provides no explicit image.
fn favicon_url_from_origin(site_url: Option<&str>, feed_url: &str) -> Option<String> {
    let domain = site_url
        .and_then(|url| Url::parse(url).ok())
        .or_else(|| Url::parse(feed_url).ok())
        .and_then(|url| url.host_str().map(str::to_string))?;

    Some(format!(
        "https://www.google.com/s2/favicons?domain={domain}&sz=128"
    ))
}

fn parse_datetime_string(candidate: &str) -> String {
    DateTime::parse_from_rfc3339(candidate)
        .map(|date| date.with_timezone(&Utc).to_rfc3339())
        .or_else(|_| {
            DateTime::parse_from_rfc2822(candidate)
                .map(|date| date.with_timezone(&Utc).to_rfc3339())
        })
        .unwrap_or_else(|_| now_iso_string())
}

fn parse_duration_seconds(candidate: &str) -> Option<i64> {
    let candidate = candidate.trim();

    if candidate.is_empty() {
        return None;
    }

    if let Ok(total_seconds) = candidate.parse::<i64>() {
        return Some(total_seconds);
    }

    let segments = candidate
        .split(':')
        .map(str::trim)
        .map(parse_i64)
        .collect::<Option<Vec<_>>>()?;

    match segments.as_slice() {
        [minutes, seconds] => Some(minutes * 60 + seconds),
        [hours, minutes, seconds] => Some(hours * 3600 + minutes * 60 + seconds),
        _ => None,
    }
}

fn clean_summary(candidate: &str) -> Option<String> {
    let normalized = normalize_preview_text(candidate, true)?;

    Some(truncate_preview(&normalized, SUMMARY_PREVIEW_MAX_CHARS))
}

fn clean_text(candidate: &str) -> Option<String> {
    normalize_preview_text(candidate, false)
}

fn build_preview_text(
    content_text: Option<&str>,
    summary_text: Option<&str>,
    summary: Option<&str>,
) -> String {
    content_text
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| truncate_preview(value, SUMMARY_PREVIEW_MAX_CHARS))
        .or_else(|| {
            summary_text
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| truncate_preview(value, SUMMARY_PREVIEW_MAX_CHARS))
        })
        .or_else(|| {
            summary
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| truncate_preview(value, SUMMARY_PREVIEW_MAX_CHARS))
        })
        .unwrap_or_else(|| "No summary or content available.".to_string())
}

fn raw_html_value(candidate: &str) -> Option<String> {
    let decoded = decode_entities(candidate);
    let trimmed = decoded.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn raw_atom_text_html(summary: &Text) -> Option<String> {
    let raw_value = raw_html_value(summary.as_str())?;

    if matches!(summary.r#type, TextType::Html | TextType::Xhtml) || looks_like_html(&raw_value) {
        Some(raw_value)
    } else {
        None
    }
}

fn looks_like_html(candidate: &str) -> bool {
    candidate.contains('<') && candidate.contains('>')
}

fn sanitize_feed_html(candidate: &str) -> Option<String> {
    if candidate.trim().is_empty() {
        return None;
    }

    let mut sanitizer = Builder::default();
    sanitizer
        .add_tag_attributes("a", &["target"])
        .set_tag_attribute_value("a", "target", "_blank");

    let sanitized = sanitizer.clean(candidate).to_string();
    let trimmed = sanitized.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_preview_text(candidate: &str, suppress_heavy_blocks: bool) -> Option<String> {
    let decoded = decode_entities(candidate);
    let without_heavy_blocks = if suppress_heavy_blocks {
        heavy_html_block_regex()
            .replace_all(&decoded, " ")
            .into_owned()
    } else {
        decoded
    };
    let structured_text = preserve_html_structure(&without_heavy_blocks);
    let without_tags = html_tag_regex().replace_all(&structured_text, " ");
    let without_github_noise = remove_github_noise(&without_tags);
    let normalized = normalize_preview_lines(&without_github_noise);

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn decode_entities(candidate: &str) -> String {
    let mut decoded = candidate.to_string();

    for _ in 0..2 {
        let next = decode_html_entities(&decoded).to_string();

        if next == decoded {
            break;
        }

        decoded = next;
    }

    decoded
}

fn preserve_html_structure(candidate: &str) -> String {
    let with_list_items = li_open_regex().replace_all(candidate, "\n• ");
    let with_list_breaks = li_close_regex().replace_all(&with_list_items, "\n");
    let with_line_breaks = br_regex().replace_all(&with_list_breaks, "\n");
    let with_paragraph_breaks = paragraph_close_regex().replace_all(&with_line_breaks, "\n");

    heading_close_regex()
        .replace_all(&with_paragraph_breaks, "\n")
        .into_owned()
}

fn remove_github_noise(candidate: &str) -> String {
    let without_pr_metadata = github_pr_metadata_regex().replace_all(candidate, " ");

    github_hash_regex()
        .replace_all(&without_pr_metadata, " ")
        .into_owned()
}

fn normalize_preview_lines(candidate: &str) -> String {
    let mut normalized_lines: Vec<String> = Vec::new();
    let mut previous_blank = false;

    for raw_line in candidate.lines() {
        let trimmed_line = raw_line
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        if trimmed_line.is_empty() {
            if !previous_blank && !normalized_lines.is_empty() {
                normalized_lines.push(String::new());
                previous_blank = true;
            }

            continue;
        }

        normalized_lines.push(trimmed_line);
        previous_blank = false;
    }

    while normalized_lines.last().is_some_and(String::is_empty) {
        normalized_lines.pop();
    }

    normalized_lines.join("\n")
}

fn html_tag_regex() -> &'static Regex {
    static HTML_TAG_REGEX: OnceLock<Regex> = OnceLock::new();

    HTML_TAG_REGEX.get_or_init(|| Regex::new(r"(?is)<[^>]+>").expect("valid HTML tag regex"))
}

fn apple_podcast_id_regex() -> &'static Regex {
    static APPLE_PODCAST_ID_REGEX: OnceLock<Regex> = OnceLock::new();

    APPLE_PODCAST_ID_REGEX
        .get_or_init(|| Regex::new(r"(?i)\bid(\d+)\b").expect("valid Apple podcast ID regex"))
}

fn li_open_regex() -> &'static Regex {
    static LI_OPEN_REGEX: OnceLock<Regex> = OnceLock::new();

    LI_OPEN_REGEX
        .get_or_init(|| Regex::new(r"(?is)<li\b[^>]*>").expect("valid list item open regex"))
}

fn li_close_regex() -> &'static Regex {
    static LI_CLOSE_REGEX: OnceLock<Regex> = OnceLock::new();

    LI_CLOSE_REGEX.get_or_init(|| Regex::new(r"(?is)</li>").expect("valid list item close regex"))
}

fn br_regex() -> &'static Regex {
    static BR_REGEX: OnceLock<Regex> = OnceLock::new();

    BR_REGEX.get_or_init(|| Regex::new(r"(?is)<br\s*/?>").expect("valid br regex"))
}

fn paragraph_close_regex() -> &'static Regex {
    static PARAGRAPH_CLOSE_REGEX: OnceLock<Regex> = OnceLock::new();

    PARAGRAPH_CLOSE_REGEX
        .get_or_init(|| Regex::new(r"(?is)</p>").expect("valid paragraph close regex"))
}

fn heading_close_regex() -> &'static Regex {
    static HEADING_CLOSE_REGEX: OnceLock<Regex> = OnceLock::new();

    HEADING_CLOSE_REGEX
        .get_or_init(|| Regex::new(r"(?is)</h[1-6]>").expect("valid heading close regex"))
}

fn heavy_html_block_regex() -> &'static Regex {
    static HEAVY_HTML_BLOCK_REGEX: OnceLock<Regex> = OnceLock::new();

    HEAVY_HTML_BLOCK_REGEX.get_or_init(|| {
        Regex::new(r"(?is)<(?:pre|details)\b[^>]*>.*?</(?:pre|details)>")
            .expect("valid heavy HTML block regex")
    })
}

fn github_hash_regex() -> &'static Regex {
    static GITHUB_HASH_REGEX: OnceLock<Regex> = OnceLock::new();

    GITHUB_HASH_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\b[0-9a-f]{7,40}\b").expect("valid GitHub hash cleanup regex")
    })
}

fn github_pr_metadata_regex() -> &'static Regex {
    static GITHUB_PR_METADATA_REGEX: OnceLock<Regex> = OnceLock::new();

    GITHUB_PR_METADATA_REGEX.get_or_init(|| {
        Regex::new(r"\s*\(\s*#\d+(?:\s+by\s+@[^)]+)?\s*\)").expect("valid GitHub PR metadata regex")
    })
}

fn truncate_preview(candidate: &str, max_chars: usize) -> String {
    if candidate.chars().count() <= max_chars {
        return candidate.to_string();
    }

    let lines = candidate.lines().collect::<Vec<_>>();
    let mut preview_lines: Vec<&str> = Vec::new();
    let mut current_chars = 0_usize;
    let mut bullet_count = 0_usize;

    for (index, line) in lines.iter().enumerate() {
        let separator_chars = if preview_lines.is_empty() { 0 } else { 1 };
        let line_chars = line.chars().count();

        if current_chars + separator_chars + line_chars > max_chars {
            break;
        }

        preview_lines.push(line);
        current_chars += separator_chars + line_chars;

        if line.trim_start().starts_with('•') {
            bullet_count += 1;
        }

        if bullet_count >= 3 && index + 1 < lines.len() && current_chars >= max_chars / 2 {
            break;
        }
    }

    if preview_lines.is_empty() {
        let truncated_at = candidate
            .char_indices()
            .nth(max_chars)
            .map(|(index, _)| index)
            .unwrap_or(candidate.len());

        return format!("{}...", candidate[..truncated_at].trim_end());
    }

    let preview = preview_lines.join("\n");

    if preview.chars().count() == candidate.chars().count() {
        return preview;
    }

    format!("{}\n...", preview.trim_end())
}

fn fallback_string(primary: Option<&str>, secondary: Option<&str>, fallback: &str) -> String {
    primary
        .and_then(clean_text)
        .or_else(|| secondary.and_then(clean_text))
        .unwrap_or_else(|| fallback.to_string())
}

fn is_audio_mime_type(mime_type: Option<&str>) -> bool {
    mime_type.is_some_and(|value| value.to_ascii_lowercase().starts_with("audio/"))
}

fn is_likely_audio_url(url: &str) -> bool {
    url.ends_with(".mp3")
        || url.ends_with(".m4a")
        || url.ends_with(".aac")
        || url.ends_with(".ogg")
        || url.ends_with(".oga")
        || url.ends_with(".wav")
        || url.ends_with(".flac")
        || url.contains(".mp3?")
        || url.contains(".m4a?")
        || url.contains(".aac?")
        || url.contains(".ogg?")
        || url.contains(".oga?")
        || url.contains(".wav?")
        || url.contains(".flac?")
}

fn guess_audio_mime_type(url: &str) -> String {
    let lowered_url = url.to_ascii_lowercase();

    if lowered_url.contains(".m4a") {
        return "audio/mp4".to_string();
    }

    if lowered_url.contains(".aac") {
        return "audio/aac".to_string();
    }

    if lowered_url.contains(".ogg") || lowered_url.contains(".oga") {
        return "audio/ogg".to_string();
    }

    if lowered_url.contains(".wav") {
        return "audio/wav".to_string();
    }

    if lowered_url.contains(".flac") {
        return "audio/flac".to_string();
    }

    "audio/mpeg".to_string()
}

fn parse_i64(candidate: &str) -> Option<i64> {
    candidate.trim().parse::<i64>().ok()
}

fn now_iso_string() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_raw_apple_podcast_id_accepts_numeric_ids() {
        assert_eq!(
            extract_raw_apple_podcast_id("1840442757"),
            Some("1840442757".to_string())
        );
        assert_eq!(
            extract_raw_apple_podcast_id(" 1840442757 "),
            Some("1840442757".to_string())
        );
    }

    #[test]
    fn extract_apple_podcast_id_from_url_reads_show_id() {
        let url = Url::parse("https://podcasts.apple.com/us/podcast/show-name/id1840442757")
            .expect("valid Apple Podcasts URL");

        assert!(is_apple_podcasts_url(&url));
        assert_eq!(
            extract_apple_podcast_id_from_url(&url),
            Some("1840442757".to_string())
        );
    }

    #[test]
    fn extract_apple_podcast_id_from_url_returns_none_without_id() {
        let url = Url::parse("https://podcasts.apple.com/us/podcast/show-name")
            .expect("valid Apple Podcasts URL");

        assert_eq!(extract_apple_podcast_id_from_url(&url), None);
    }

    #[test]
    fn select_apple_lookup_feed_url_prefers_first_result_feed_url() {
        let response = AppleLookupResponse {
            results: vec![AppleLookupResult {
                kind: Some("podcast".to_string()),
                feed_url: Some("https://example.com/feed.xml".to_string()),
            }],
        };

        assert_eq!(
            select_apple_lookup_feed_url(response),
            Ok("https://example.com/feed.xml".to_string())
        );
    }

    #[test]
    fn select_apple_lookup_feed_url_falls_back_to_later_podcast_result() {
        let response = AppleLookupResponse {
            results: vec![
                AppleLookupResult {
                    kind: Some("ebook".to_string()),
                    feed_url: None,
                },
                AppleLookupResult {
                    kind: Some("podcast".to_string()),
                    feed_url: Some("https://example.com/podcast.xml".to_string()),
                },
            ],
        };

        assert_eq!(
            select_apple_lookup_feed_url(response),
            Ok("https://example.com/podcast.xml".to_string())
        );
    }

    #[test]
    fn select_apple_lookup_feed_url_errors_without_public_feed_url() {
        let response = AppleLookupResponse {
            results: vec![AppleLookupResult {
                kind: Some("podcast".to_string()),
                feed_url: None,
            }],
        };

        assert_eq!(
            select_apple_lookup_feed_url(response),
            Err("Apple Podcasts did not return a public RSS feed for this show.".to_string())
        );
    }
}
