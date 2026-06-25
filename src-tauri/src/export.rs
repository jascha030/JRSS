use crate::db::AppResult;
use crate::models::FeedItemRecord;
use lofty::file::TaggedFileExt;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::tag::{ItemKey, TagType};
use reqwest::blocking::Client;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const DOWNLOAD_TIMEOUT_SECS: u64 = 300;
const MAX_FILENAME_LEN: usize = 120;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgressEvent {
    pub feed_id: String,
    pub item_id: String,
    pub title: String,
    pub status: ExportStatus,
    pub message: Option<String>,
    pub current: usize,
    pub total: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportStatus {
    Started,
    Done,
    Error,
}

pub fn export_feed_to_directory(
    app: &AppHandle,
    feed_id: String,
    target_dir: PathBuf,
    feed_title: String,
    feed_image_url: Option<String>,
    items: Vec<FeedItemRecord>,
) -> AppResult<usize> {
    let client = build_client()?;
    let total = items.len();
    let mut exported_count = 0;

    let cover_art_data = feed_image_url
        .as_ref()
        .and_then(|url| download_cover_art(&client, url).ok());

    let db_state = app.state::<crate::db::DatabaseState>();
    let db_path = db_state.db_path();

    for (index, item) in items.iter().enumerate() {
        let enclosure = match &item.media_enclosure {
            Some(e) => e,
            None => continue,
        };

        let file_name = build_safe_filename(&item.title, &enclosure.url);
        let file_path = resolve_unique_path(&target_dir, &file_name);

        if file_path.exists()
            && crate::db::get_exported_file_for_item(&db_path, &item.id)
                .ok()
                .flatten()
                .is_some()
        {
            emit_progress(
                app,
                &feed_id,
                &item.id,
                &item.title,
                ExportStatus::Done,
                Some("Already exported".to_string()),
                index + 1,
                total,
            );
            continue;
        }

        emit_progress(
            app,
            &feed_id,
            &item.id,
            &item.title,
            ExportStatus::Started,
            None,
            index + 1,
            total,
        );

        match download_audio(&client, &enclosure.url, &file_path) {
            Ok(()) => {
                if let Err(error) = write_id3_tags(
                    &file_path,
                    &item.title,
                    &feed_title,
                    item.episode_number,
                    cover_art_data.as_deref(),
                ) {
                    log::warn!("Failed to write ID3 tags for {}: {}", item.id, error);
                }

                if let Err(error) =
                    crate::db::upsert_exported_file(&db_path, &item.id, &feed_id, file_path.to_str().unwrap_or(""))
                {
                    log::warn!("Failed to record exported file for {}: {}", item.id, error);
                }

                exported_count += 1;
                emit_progress(
                    app,
                    &feed_id,
                    &item.id,
                    &item.title,
                    ExportStatus::Done,
                    None,
                    index + 1,
                    total,
                );
            }
            Err(error) => {
                log::error!("Failed to download {}: {}", enclosure.url, error);
                emit_progress(
                    app,
                    &feed_id,
                    &item.id,
                    &item.title,
                    ExportStatus::Error,
                    Some(error),
                    index + 1,
                    total,
                );
            }
        }
    }

    Ok(exported_count)
}

fn build_client() -> AppResult<Client> {
    Client::builder()
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .user_agent("JRSS/0.0.1")
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))
}

fn download_audio(client: &Client, url: &str, dest: &Path) -> AppResult<()> {
    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("Download request failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Download request failed with status {status}."));
    }

    let bytes = response
        .bytes()
        .map_err(|error| format!("Failed to read download response: {error}"))?;

    std::fs::write(dest, bytes).map_err(|error| format!("Failed to write audio file: {error}"))?;

    Ok(())
}

fn download_cover_art(client: &Client, url: &str) -> AppResult<Vec<u8>> {
    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("Cover art download failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Cover art download failed with status {status}."));
    }

    response
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|error| format!("Failed to read cover art response: {error}"))
}

fn build_safe_filename(title: &str, url: &str) -> String {
    let extension = Path::new(url)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("mp3");

    let safe_extension = if extension.is_empty() { "mp3" } else { extension };

    let sanitized: String = title
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | ' ' | '-' | '_' | '.' | ',' | '!' | '?' | '&'
            | '(' | ')' | '[' | ']' | '{' | '}' => c,
            _ => '_',
        })
        .collect();

    let trimmed = sanitized.trim();
    let base = if trimmed.len() > MAX_FILENAME_LEN {
        &trimmed[..MAX_FILENAME_LEN]
    } else {
        trimmed
    };

    format!("{}.{}", base.trim_end_matches('.'), safe_extension)
}

fn resolve_unique_path(dir: &Path, file_name: &str) -> PathBuf {
	dir.join(file_name)
}

fn write_id3_tags(
    path: &Path,
    title: &str,
    album: &str,
    track_number: Option<i64>,
    cover_art: Option<&[u8]>,
) -> AppResult<()> {
    let mut tagged_file = lofty::read_from_path(path)
        .map_err(|error| format!("Failed to read audio file for tagging: {error}"))?;

    let tag = tagged_file
        .primary_tag_mut()
        .map(|t| t.clone())
        .or_else(|| tagged_file.first_tag_mut().map(|t| t.clone()));

    let mut tag = match tag {
        Some(t) => t,
        None => lofty::tag::Tag::new(TagType::Id3v2),
    };

    tag.set_title(title.into());
    tag.set_artist(album.into());
    tag.set_album(album.into());

    if let Some(number) = track_number {
        tag.insert_text(ItemKey::TrackNumber, number.to_string());
    }

    if let Some(data) = cover_art {
        let mime = infer_image_mime(data);
        let picture = Picture::new_unchecked(
            PictureType::CoverFront,
            Some(mime),
            None,
            data.to_vec(),
        );
        tag.push_picture(picture);
    }

    tagged_file.insert_tag(tag);

    let write_options = lofty::config::WriteOptions::default();
    tagged_file
        .save_to_path(path, write_options)
        .map_err(|error| format!("Failed to save tagged file: {error}"))?;

    Ok(())
}

fn infer_image_mime(data: &[u8]) -> MimeType {
    if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        MimeType::Png
    } else if data.starts_with(b"\xff\xd8\xff") {
        MimeType::Jpeg
    } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        MimeType::Gif
    } else if data.starts_with(b"RIFF") && data.get(8..12) == Some(b"WEBP") {
        MimeType::Unknown("image/webp".into())
    } else {
        MimeType::Jpeg
    }
}

fn emit_progress(
    app: &AppHandle,
    feed_id: &str,
    item_id: &str,
    title: &str,
    status: ExportStatus,
    message: Option<String>,
    current: usize,
    total: usize,
) {
    let event = ExportProgressEvent {
        feed_id: feed_id.to_string(),
        item_id: item_id.to_string(),
        title: title.to_string(),
        status,
        message,
        current,
        total,
    };
    let _ = app.emit("export-progress", event);
}
