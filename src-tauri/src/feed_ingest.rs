use crate::db::AppResult;
use crate::models::{MediaEnclosureRecord, ParsedFeed, ParsedFeedItem};
use atom_syndication::{Entry as AtomEntry, Feed as AtomFeed, Link as AtomLink};
use chrono::{DateTime, Utc};
use html_escape::decode_html_entities;
use regex::Regex;
use reqwest::blocking::Client;
use rss::{Channel as RssChannel, Item as RssItem};
use std::io::Cursor;
use std::sync::OnceLock;
use std::time::Duration;
use url::Url;

const FEED_ACCEPT_HEADER: &str =
    "application/atom+xml, application/rss+xml, application/xml, text/xml;q=0.9, */*;q=0.8";
const SUMMARY_PREVIEW_MAX_CHARS: usize = 420;

pub fn normalize_feed_url(url: &str) -> AppResult<String> {
    let parsed_url = Url::parse(url).map_err(|_| "Enter a valid feed URL.".to_string())?;
    let scheme = parsed_url.scheme();

    if scheme != "http" && scheme != "https" {
        return Err("Feed URLs must use http or https.".to_string());
    }

    Ok(parsed_url.to_string())
}

pub fn fetch_and_parse_feed(feed_url: &str) -> AppResult<ParsedFeed> {
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("JRSS/0.0.1")
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))?;

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
        site_url: resolve_optional_url(Some(channel.link()), feed_url),
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
        published_at,
        media_enclosure: enclosure,
    }
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

    ParsedFeed {
        title: fallback_string(Some(feed.title().as_str()), None, "Untitled feed"),
        description: feed
            .subtitle()
            .and_then(|subtitle| clean_text(subtitle.as_str()))
            .unwrap_or_else(|| "No description provided.".to_string()),
        site_url: select_atom_link(feed.links(), feed_url, "alternate"),
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
    let summary = extract_atom_summary(entry).unwrap_or_else(|| "No summary provided.".to_string());
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
