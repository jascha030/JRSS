use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tauri::Manager;

/// Information about a discovered theme file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeInfo {
    pub name: String,
    pub filename: String,
}

/// The directory where the default theme is always stored.
fn default_themes_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| format!("Failed to resolve app config dir: {e}"))
        .map(|d| d.join("themes"))
}

/// The XDG config directory for custom themes, if `$XDG_CONFIG_HOME` is set.
fn xdg_themes_dir() -> Option<PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(|p| PathBuf::from(p).join("jrss").join("themes"))
}

/// The directory specified by `$JRSS_CONFIG_DIR`, if set.
fn env_themes_dir() -> Option<PathBuf> {
    std::env::var("JRSS_CONFIG_DIR")
        .ok()
        .map(|p| PathBuf::from(p).join("themes"))
}

/// All directories to scan for theme files, in search order.
///
/// Priority:
/// 1. `$JRSS_CONFIG_DIR/themes` (if set)
/// 2. `$XDG_CONFIG_HOME/jrss/themes` (if set and exists)
/// 3. `app_config_dir()/themes` (always, holds the default theme)
fn theme_search_dirs(app: &tauri::AppHandle) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(dir) = env_themes_dir() {
        dirs.push(dir);
    }

    if let Some(dir) = xdg_themes_dir() {
        if dir.exists() {
            dirs.push(dir);
        }
    }

    if let Ok(dir) = default_themes_dir(app) {
        dirs.push(dir);
    }

    dirs
}

/// Ensure the default themes directory exists.
fn ensure_default_themes_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = default_themes_dir(app)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create themes dir: {e}"))?;
    Ok(dir)
}

/// Discover all `.css` theme files in the given directory.
fn discover_themes_in_dir(dir: &Path) -> Result<Vec<ThemeInfo>, String> {
    let mut themes = Vec::new();

    let entries = std::fs::read_dir(dir).map_err(|e| format!("Failed to read themes dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("css") {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read theme file {filename}: {e}"))?;

        let name = parse_theme_name(&content);
        let name = name.unwrap_or_else(|| filename.clone());

        themes.push(ThemeInfo { name, filename });
    }

    Ok(themes)
}

/// Load the raw CSS content of a theme file by filename.
fn load_theme_from_dirs(dirs: &[PathBuf], filename: &str) -> Result<String, String> {
    for dir in dirs {
        let path = dir.join(filename);
        if path.exists() {
            return std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to load theme {filename}: {e}"));
        }
    }
    Err(format!("Theme file not found: {filename}"))
}

/// Write the embedded default theme to `app_config_dir()/themes/default.css`.
/// Always overwrites so updates to the embedded CSS are picked up.
pub fn ensure_default_theme(app: &tauri::AppHandle) -> Result<(), String> {
    let dir = ensure_default_themes_dir(app)?;
    let default_path = dir.join("default.css");

    const DEFAULT_CSS: &str = include_str!("../../src/lib/assets/themes/default.css");

    std::fs::write(&default_path, DEFAULT_CSS)
        .map_err(|e| format!("Failed to write default theme: {e}"))?;

    Ok(())
}

/// Parse `name` from the first CSS comment block.
///
/// Expected format:
/// ```css
/// /*
/// name: My Theme
/// */
/// ```
fn parse_theme_name(content: &str) -> Option<String> {
    let trimmed = content.trim_start();

    let start = trimmed.find("/*")?;
    let after_start = &trimmed[start + 2..];
    let end = after_start.find("*/")?;

    let comment = &after_start[..end];

    for line in comment.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("name:") {
            return Some(rest.trim().to_string());
        }
    }

    None
}

#[tauri::command]
pub async fn cmd_discover_themes(app: tauri::AppHandle) -> Result<Vec<ThemeInfo>, String> {
    ensure_default_theme(&app)?;

    let dirs = theme_search_dirs(&app);
    let mut all_themes = Vec::new();
    let mut seen = HashSet::new();

    // Search in priority order; first occurrence wins.
    for dir in dirs {
        if !dir.exists() {
            continue;
        }

        let themes = discover_themes_in_dir(&dir)?;
        for theme in themes {
            // Skip the shipped default — it is already the implicit fallback.
            if theme.filename == "default.css" {
                continue;
            }
            if seen.insert(theme.filename.clone()) {
                all_themes.push(theme);
            }
        }
    }

    all_themes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(all_themes)
}

#[tauri::command]
pub async fn cmd_load_theme(filename: String, app: tauri::AppHandle) -> Result<String, String> {
    let dirs = theme_search_dirs(&app);
    load_theme_from_dirs(&dirs, &filename)
}
