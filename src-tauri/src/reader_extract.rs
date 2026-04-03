use crate::db::AppResult;
use crate::models::ReaderContentRecord;
use ammonia::{Builder, UrlRelative};
use chrono::Utc;
use html_escape::decode_html_entities;
use readability::{extract, ExtractOptions};
use regex::Regex;
use reqwest::blocking::Client;
use std::io::Cursor;
use std::sync::OnceLock;
use std::time::Duration;
use url::Url;

const ARTICLE_ACCEPT_HEADER: &str = "text/html, application/xhtml+xml;q=0.9, */*;q=0.8";
const READER_EXCERPT_MAX_CHARS: usize = 280;

pub fn fetch_reader_content(
    article_url: &str,
    fallback_title: &str,
) -> AppResult<ReaderContentRecord> {
    let normalized_url = normalize_article_url(article_url)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("JRSS/0.0.1 Reader")
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))?;

    let response = client
        .get(normalized_url)
        .header(reqwest::header::ACCEPT, ARTICLE_ACCEPT_HEADER)
        .send()
        .map_err(|error| format!("Failed to fetch article: {error}"))?;

    let status = response.status();

    if !status.is_success() {
        return Err(format!("Article request failed with status {status}."));
    }

    let final_url = response.url().clone();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    if !looks_like_html_response(content_type.as_deref()) {
        return Err("Original article did not return an HTML page.".to_string());
    }

    let html = response
        .text()
        .map_err(|error| format!("Failed to read article response body: {error}"))?;

    if html.trim().is_empty() {
        return Err("Original article page was empty.".to_string());
    }

    extract_reader_content_from_html(&html, &final_url, fallback_title)
}

fn extract_reader_content_from_html(
    html: &str,
    final_url: &Url,
    fallback_title: &str,
) -> AppResult<ReaderContentRecord> {
    let mut cursor = Cursor::new(html.as_bytes());
    let readable = extract(&mut cursor, &final_url, ExtractOptions::default())
        .map_err(|error| format!("Failed to extract readable article content: {error}"))?;
    let content_html = sanitize_reader_html(&readable.content, final_url);
    let content_text = normalize_reader_text(&readable.text);

    if content_html.is_none() && content_text.is_none() {
        return Err("Could not extract readable article content from that page.".to_string());
    }

    Ok(ReaderContentRecord {
        title: clean_metadata_value(&readable.title)
            .or_else(|| extract_title(&html))
            .or_else(|| clean_metadata_value(fallback_title))
            .unwrap_or_else(|| "Untitled article".to_string()),
        byline: extract_byline(&html),
        excerpt: extract_excerpt(&html)
            .or_else(|| content_text.as_deref().and_then(build_excerpt_from_text)),
        content_html,
        content_text,
        fetched_at: Utc::now().to_rfc3339(),
    })
}

fn normalize_article_url(article_url: &str) -> AppResult<Url> {
    let parsed_url =
        Url::parse(article_url).map_err(|_| "Item URL is not a valid article URL.".to_string())?;

    match parsed_url.scheme() {
        "http" | "https" => Ok(parsed_url),
        _ => Err("Reader Mode only supports http and https article URLs.".to_string()),
    }
}

fn looks_like_html_response(content_type: Option<&str>) -> bool {
    content_type.is_none_or(|value| {
        let normalized = value.trim().to_ascii_lowercase();

        normalized.contains("text/html") || normalized.contains("application/xhtml+xml")
    })
}

fn sanitize_reader_html(candidate: &str, base_url: &Url) -> Option<String> {
    if candidate.trim().is_empty() {
        return None;
    }

    let mut sanitizer = Builder::default();
    sanitizer
        .url_relative(UrlRelative::RewriteWithBase(base_url.clone()))
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

fn extract_title(html: &str) -> Option<String> {
    extract_meta_content(html, &[("property", "og:title"), ("name", "twitter:title")]).or_else(
        || {
            html_title_regex()
                .and_then(|regex| regex.captures(html))
                .and_then(|captures| captures.get(1))
                .and_then(|value| clean_metadata_value(value.as_str()))
        },
    )
}

fn extract_byline(html: &str) -> Option<String> {
    extract_meta_content(
        html,
        &[
            ("name", "author"),
            ("property", "author"),
            ("property", "article:author"),
            ("name", "parsely-author"),
            ("name", "dc.creator"),
            ("name", "twitter:creator"),
        ],
    )
}

fn extract_excerpt(html: &str) -> Option<String> {
    extract_meta_content(
        html,
        &[
            ("name", "description"),
            ("property", "og:description"),
            ("name", "twitter:description"),
        ],
    )
    .and_then(|value| build_excerpt_from_text(&value))
}

fn extract_meta_content(html: &str, candidates: &[(&str, &str)]) -> Option<String> {
    let Some(meta_tag_regex) = meta_tag_regex() else {
        return None;
    };

    for meta_tag in meta_tag_regex.find_iter(html) {
        let attributes = parse_tag_attributes(meta_tag.as_str());
        let content = attributes
            .iter()
            .find(|(name, _)| name == "content")
            .and_then(|(_, value)| clean_metadata_value(value));
        let Some(content) = content else {
            continue;
        };

        if candidates.iter().any(|(attribute, expected)| {
            attributes
                .iter()
                .any(|(name, value)| name == attribute && value.eq_ignore_ascii_case(expected))
        }) {
            return Some(content);
        }
    }

    None
}

fn parse_tag_attributes(tag_html: &str) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    let bytes = tag_html.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len()
            && (bytes[index].is_ascii_whitespace()
                || matches!(bytes[index], b'<' | b'>' | b'/' | b'?'))
        {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        let name_start = index;

        while index < bytes.len() && is_attribute_name_char(bytes[index]) {
            index += 1;
        }

        if name_start == index {
            index += 1;
            continue;
        }

        let name = tag_html[name_start..index].trim().to_ascii_lowercase();

        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() || bytes[index] != b'=' {
            continue;
        }

        index += 1;

        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        let value = match bytes[index] {
            b'\'' | b'"' => {
                let quote = bytes[index];
                index += 1;
                let value_start = index;

                while index < bytes.len() && bytes[index] != quote {
                    index += 1;
                }

                let value = tag_html[value_start..index].trim().to_string();

                if index < bytes.len() {
                    index += 1;
                }

                value
            }
            _ => {
                let value_start = index;

                while index < bytes.len()
                    && !bytes[index].is_ascii_whitespace()
                    && !matches!(bytes[index], b'>' | b'/')
                {
                    index += 1;
                }

                tag_html[value_start..index].trim().to_string()
            }
        };

        if !name.is_empty() {
            attributes.push((name, value));
        }
    }

    attributes
}

fn normalize_reader_text(candidate: &str) -> Option<String> {
    let decoded = decode_html_entities(candidate).to_string();
    let mut lines = Vec::new();
    let mut last_line_blank = false;

    for line in decoded.lines() {
        let normalized_line = normalize_inline_whitespace(line);

        if normalized_line.is_empty() {
            if !last_line_blank && !lines.is_empty() {
                lines.push(String::new());
            }

            last_line_blank = true;
            continue;
        }

        lines.push(normalized_line);
        last_line_blank = false;
    }

    let normalized = lines.join("\n").trim().to_string();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn clean_metadata_value(candidate: &str) -> Option<String> {
    let decoded = decode_html_entities(candidate).to_string();
    let normalized = normalize_inline_whitespace(&decoded);

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn build_excerpt_from_text(candidate: &str) -> Option<String> {
    let normalized = normalize_inline_whitespace(candidate);

    if normalized.is_empty() {
        return None;
    }

    if normalized.chars().count() <= READER_EXCERPT_MAX_CHARS {
        return Some(normalized);
    }

    let mut end_index = 0;
    let mut char_count = 0;

    for (index, character) in normalized.char_indices() {
        if char_count == READER_EXCERPT_MAX_CHARS {
            break;
        }

        char_count += 1;
        end_index = index + character.len_utf8();
    }

    let truncated = normalized[..end_index]
        .trim_end_matches(|character: char| {
            character.is_whitespace() || ",.;:!?".contains(character)
        })
        .to_string();

    if truncated.is_empty() {
        None
    } else {
        Some(format!("{truncated}..."))
    }
}

fn normalize_inline_whitespace(candidate: &str) -> String {
    candidate.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_attribute_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'.' | b'-')
}

fn meta_tag_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?is)<meta\b[^>]*>").ok())
        .as_ref()
}

fn html_title_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?is)<title[^>]*>(.*?)</title>").ok())
        .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tag_attributes_handles_supported_html_forms() {
        let attributes = parse_tag_attributes(
            r#"<meta property='og:title' content="Hello world" data-id=abc123>"#,
        );

        assert!(attributes
            .iter()
            .any(|(name, value)| name == "property" && value == "og:title"));
        assert!(attributes
            .iter()
            .any(|(name, value)| name == "content" && value == "Hello world"));
        assert!(attributes
            .iter()
            .any(|(name, value)| name == "data-id" && value == "abc123"));
    }

    #[test]
    fn extract_reader_content_from_html_sanitizes_and_extracts_metadata() {
        let article_url = Url::parse("https://example.com/posts/test").expect("valid url");
        let html = r#"
            <html>
                <head>
                    <title>Fallback title</title>
                    <meta name="author" content="Jane Doe">
                    <meta name="description" content="Example article description for reader mode.">
                </head>
                <body>
                    <article>
                        <h1>Readable article heading</h1>
                        <p>Hello <script>alert('xss')</script><a href="/relative-link">reader mode</a>.</p>
                    </article>
                </body>
            </html>
        "#;

        let reader_content = extract_reader_content_from_html(html, &article_url, "Feed title")
            .expect("reader content extracted");

        assert_eq!(reader_content.byline.as_deref(), Some("Jane Doe"));
        assert_eq!(
            reader_content.excerpt.as_deref(),
            Some("Example article description for reader mode.")
        );

        let content_html = reader_content.content_html.expect("sanitized html");
        assert!(!content_html.contains("<script"));
        assert!(content_html.contains("https://example.com/relative-link"));
        assert!(reader_content.content_text.is_some());
    }
}
