use super::AppResult;
use super::connection::open_connection;
use crate::models::{AppSettingsRecord, ColorScheme};
use chrono::Utc;
use rusqlite::params;
use std::path::Path;

pub const DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES: i64 = 5 * 1024 * 1024 * 1024;
pub const DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP: bool = false;
/// Default auto-refresh interval in minutes (1 hour). `0` means disabled.
pub const DEFAULT_AUTO_REFRESH_INTERVAL_MINUTES: i64 = 60;
/// Default color scheme: follow the OS preference.
pub const DEFAULT_COLOR_SCHEME: ColorScheme = ColorScheme::System;
pub const DEFAULT_SKIP_FORWARD_SECONDS: i64 = 15;
pub const DEFAULT_SKIP_BACKWARD_SECONDS: i64 = 15;

fn normalize_max_audio_cache_size_bytes(value: i64) -> AppResult<i64> {
    if value <= 0 {
        return Err("Audio cache size must be greater than 0 bytes.".to_string());
    }
    Ok(value)
}

fn normalize_max_image_cache_size_bytes(value: i64) -> AppResult<i64> {
    if value <= 0 {
        return Err("Image cache size must be greater than 0 bytes.".to_string());
    }
    Ok(value)
}

fn normalize_auto_refresh_interval_minutes(value: i64) -> AppResult<i64> {
    if value < 0 {
        return Err("Auto-refresh interval cannot be negative.".to_string());
    }
    Ok(value)
}

/// Validates that `accent_color`, if present, looks like a CSS hex colour (`#rrggbb` or `#rgb`).
fn normalize_accent_color(value: Option<String>) -> Option<String> {
    value.filter(|c| {
        c.starts_with('#')
            && (c.len() == 7 || c.len() == 4)
            && c[1..].chars().all(|ch| ch.is_ascii_hexdigit())
    })
}

fn normalize_skip_forward_seconds(value: i64) -> i64 {
    value.max(1)
}

fn normalize_skip_backward_seconds(value: i64) -> i64 {
    value.max(1)
}

pub(super) fn ensure_app_settings_row(connection: &rusqlite::Connection) -> AppResult<()> {
    connection
        .execute(
            "INSERT INTO app_settings (
		        id,
		        max_audio_cache_size_bytes,
		        mini_player_always_on_top,
		        color_scheme,
		        skip_forward_seconds,
		        skip_backward_seconds,
		        updated_at
		     )
		     VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)
		     ON CONFLICT(id) DO NOTHING",
            params![
                DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES,
                DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP,
                DEFAULT_COLOR_SCHEME,
                DEFAULT_SKIP_FORWARD_SECONDS,
                DEFAULT_SKIP_BACKWARD_SECONDS,
                Utc::now().to_rfc3339()
            ],
        )
        .map_err(|error| format!("Failed to ensure app settings row: {error}"))?;

    Ok(())
}

pub fn load_app_settings(db_path: &Path) -> AppResult<AppSettingsRecord> {
    let connection = open_connection(db_path)?;
    ensure_app_settings_row(&connection)?;

    let (
        max_audio_cache_size_bytes,
        mini_player_always_on_top,
        auto_refresh_interval_minutes,
        color_scheme,
        accent_color,
        skip_forward_seconds,
        skip_backward_seconds,
        theme_name,
        max_image_cache_size_bytes,
    ) = connection
        .query_row(
            "SELECT max_audio_cache_size_bytes, mini_player_always_on_top, auto_refresh_interval_minutes, color_scheme, accent_color, skip_forward_seconds, skip_backward_seconds, theme_name, max_image_cache_size_bytes
		         FROM app_settings
		         WHERE id = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, bool>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, ColorScheme>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            },
        )
        .map_err(|error| format!("Failed to load app settings: {error}"))?;

    Ok(AppSettingsRecord {
        max_audio_cache_size_bytes: normalize_max_audio_cache_size_bytes(
            max_audio_cache_size_bytes,
        )?,
        mini_player_always_on_top,
        auto_refresh_interval_minutes: normalize_auto_refresh_interval_minutes(
            auto_refresh_interval_minutes,
        )?,
        color_scheme,
        accent_color: normalize_accent_color(accent_color),
        skip_forward_seconds: normalize_skip_forward_seconds(skip_forward_seconds),
        skip_backward_seconds: normalize_skip_backward_seconds(skip_backward_seconds),
        theme_name,
        max_image_cache_size_bytes: normalize_max_image_cache_size_bytes(
            max_image_cache_size_bytes,
        )?,
    })
}

pub fn save_app_settings(
    db_path: &Path,
    settings: &AppSettingsRecord,
) -> AppResult<AppSettingsRecord> {
    let connection = open_connection(db_path)?;
    ensure_app_settings_row(&connection)?;

    let max_audio_cache_size_bytes =
        normalize_max_audio_cache_size_bytes(settings.max_audio_cache_size_bytes)?;
    let mini_player_always_on_top = settings.mini_player_always_on_top;
    let auto_refresh_interval_minutes =
        normalize_auto_refresh_interval_minutes(settings.auto_refresh_interval_minutes)?;
    let color_scheme = settings.color_scheme;
    let accent_color = normalize_accent_color(settings.accent_color.clone());
    let skip_forward_seconds = normalize_skip_forward_seconds(settings.skip_forward_seconds);
    let skip_backward_seconds = normalize_skip_backward_seconds(settings.skip_backward_seconds);
    let theme_name = settings.theme_name.clone();
    let max_image_cache_size_bytes =
        normalize_max_image_cache_size_bytes(settings.max_image_cache_size_bytes)?;

    connection
        .execute(
            "INSERT INTO app_settings (
		        id,
		        max_audio_cache_size_bytes,
		        mini_player_always_on_top,
		        auto_refresh_interval_minutes,
		        color_scheme,
		        accent_color,
		        skip_forward_seconds,
		        skip_backward_seconds,
		        theme_name,
		        max_image_cache_size_bytes,
		        updated_at
		     )
		     VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
		     ON CONFLICT(id) DO UPDATE SET
		        max_audio_cache_size_bytes = excluded.max_audio_cache_size_bytes,
		        mini_player_always_on_top = excluded.mini_player_always_on_top,
		        auto_refresh_interval_minutes = excluded.auto_refresh_interval_minutes,
		        color_scheme = excluded.color_scheme,
		        accent_color = excluded.accent_color,
		        skip_forward_seconds = excluded.skip_forward_seconds,
		        skip_backward_seconds = excluded.skip_backward_seconds,
		        theme_name = excluded.theme_name,
		        max_image_cache_size_bytes = excluded.max_image_cache_size_bytes,
		        updated_at = excluded.updated_at",
            params![
                max_audio_cache_size_bytes,
                mini_player_always_on_top,
                auto_refresh_interval_minutes,
                color_scheme,
                accent_color,
                skip_forward_seconds,
                skip_backward_seconds,
                theme_name,
                max_image_cache_size_bytes,
                Utc::now().to_rfc3339()
            ],
        )
        .map_err(|error| format!("Failed to save app settings: {error}"))?;

    Ok(AppSettingsRecord {
        max_audio_cache_size_bytes,
        mini_player_always_on_top,
        auto_refresh_interval_minutes,
        color_scheme,
        accent_color,
        skip_forward_seconds,
        skip_backward_seconds,
        theme_name,
        max_image_cache_size_bytes,
    })
}
