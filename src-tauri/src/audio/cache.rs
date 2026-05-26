//! Audio cache management — paths, LRU eviction, marker files.

use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Represents a cached file with its metadata for LRU eviction.
struct CacheFile {
    path: PathBuf,
    size: u64,
    accessed: std::time::SystemTime,
}

/// Get the audio cache directory path.
pub fn get_audio_cache_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let cache_dir = app_data.join("audio_cache");
    ensure_audio_cache_dir(&cache_dir)?;
    Ok(cache_dir)
}

pub fn clear_audio_cache(app: &tauri::AppHandle) -> Result<(), String> {
    let cache_dir = get_audio_cache_path(app)?;
    let entries =
        std::fs::read_dir(&cache_dir).map_err(|e| format!("Failed to read cache dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read cache entry: {e}"))?;
        let path = entry.path();
        let is_cache_file = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "mp3" || ext == "complete")
            .unwrap_or(false);

        if is_cache_file {
            let _ = std::fs::remove_file(&path);
        }
    }

    Ok(())
}

fn ensure_audio_cache_dir(cache_dir: &Path) -> Result<(), String> {
    if !cache_dir.exists() {
        std::fs::create_dir_all(cache_dir)
            .map_err(|e| format!("Failed to create audio cache dir: {e}"))?;
    }
    Ok(())
}

/// Get the path to the cache completion marker file.
pub fn cache_complete_marker_path(path: &Path) -> PathBuf {
    match path.file_name().and_then(|name| name.to_str()) {
        Some(file_name) => path.with_file_name(format!("{file_name}.complete")),
        None => path.with_extension("complete"),
    }
}

/// Check if a cache file is complete (has marker and non-zero size).
pub fn is_cache_complete(path: &Path) -> bool {
    if !path.exists() {
        log::debug!("Cache file does not exist: {:?}", path);
        return false;
    }

    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            log::debug!("Failed to read cache file metadata: {:?} - {}", path, e);
            return false;
        }
    };

    if !metadata.is_file() {
        log::debug!("Cache path is not a file: {:?}", path);
        return false;
    }

    let marker_path = cache_complete_marker_path(path);
    if !marker_path.exists() {
        log::debug!("Cache marker does not exist: {:?}", marker_path);
        return false;
    }

    let size = metadata.len();
    let is_complete = size > 0;
    log::debug!(
        "Cache file check: {:?} - size={} bytes, complete={}",
        path,
        size,
        is_complete
    );
    is_complete
}

/// Enforce the cache size limit by removing least-recently-used files.
/// Call this before downloading a new file to make room if needed.
pub fn enforce_cache_size_limit(
    cache_dir: &Path,
    current_download_path: &Path,
    protected_paths: &[PathBuf],
    new_file_bytes: u64,
    max_cache_size_bytes: u64,
) -> Result<(), String> {
    let mut files: Vec<CacheFile> = Vec::new();
    let mut evictable_files_size: u64 = 0;
    let mut retained_files_size: u64 = 0;

    // Read directory and collect all cached audio files
    let entries = match std::fs::read_dir(cache_dir) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Failed to read cache directory: {e}");
            return Ok(());
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if !metadata.is_file() {
            continue;
        }

        // Only consider .mp3 files (not .complete markers)
        if path.extension().and_then(|e| e.to_str()) != Some("mp3") {
            continue;
        }

        let size = metadata.len();
        let accessed = metadata.accessed().unwrap_or_else(|_| {
            metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        if path == current_download_path || protected_paths.iter().any(|protected| protected == &path) {
            retained_files_size = retained_files_size.saturating_add(size);
            continue;
        }

        evictable_files_size = evictable_files_size.saturating_add(size);
        files.push(CacheFile {
            path,
            size,
            accessed,
        });
    }

    let projected_download_size = if new_file_bytes > 0 {
        new_file_bytes
    } else {
        retained_files_size
    };

    if projected_download_size > max_cache_size_bytes {
        log::warn!(
            "Audio file exceeds configured cache limit: file={} MB, limit={} MB",
            projected_download_size / (1024 * 1024),
            max_cache_size_bytes / (1024 * 1024)
        );
    }

    let projected_size = evictable_files_size
        .saturating_add(retained_files_size)
        .saturating_add(projected_download_size);

    if projected_size <= max_cache_size_bytes {
        log::debug!(
            "Cache size: {} MB / {} MB - no cleanup needed",
            projected_size / (1024 * 1024),
            max_cache_size_bytes / (1024 * 1024)
        );
        return Ok(());
    }

    // Sort by last accessed time (oldest first)
    files.sort_by(|a, b| a.accessed.cmp(&b.accessed));

    let mut freed: u64 = 0;
    let mut removed_count: usize = 0;
    let target_size = max_cache_size_bytes
        .saturating_sub(retained_files_size)
        .saturating_sub(projected_download_size);

    for file in files {
        if evictable_files_size.saturating_sub(freed) <= target_size {
            break;
        }

        // Remove the file and its complete marker
        let marker_path = cache_complete_marker_path(&file.path);

        if let Err(e) = std::fs::remove_file(&file.path) {
            log::warn!("Failed to remove cached file {:?}: {e}", file.path);
            continue;
        }
        let _ = std::fs::remove_file(&marker_path);

        freed = freed.saturating_add(file.size);
        removed_count += 1;

        log::info!(
            "Evicted from cache: {:?} ({} MB)",
            file.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown"),
            file.size / (1024 * 1024)
        );
    }

    log::info!(
        "Cache cleanup complete: removed {} files, freed {} MB",
        removed_count,
        freed / (1024 * 1024)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::enforce_cache_size_limit;
    use std::fs;

    #[test]
    fn cleanup_keeps_protected_cache_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp_dir.path();
        let active_path = cache_dir.join("active.mp3");
        let old_path = cache_dir.join("old.mp3");
        let incoming_path = cache_dir.join("incoming.mp3");

        fs::write(&active_path, vec![0_u8; 4]).expect("write active");
        fs::write(&old_path, vec![0_u8; 4]).expect("write old");

        enforce_cache_size_limit(
            cache_dir,
            &incoming_path,
            std::slice::from_ref(&active_path),
            4,
            8,
        )
        .expect("cleanup succeeds");

        assert!(active_path.exists(), "protected file should remain");
        assert!(!old_path.exists(), "unprotected file should be evicted");
    }
}

/// Generate a stable hash for cache filenames using SHA1.
pub fn hash_item_id(item_id: &str) -> String {
    use sha1_smol::Sha1;
    Sha1::from(item_id).digest().to_string()
}

/// Get the cache size limit from app settings.
pub fn get_audio_cache_size_limit_bytes(app: &AppHandle) -> u64 {
    let db_state = app.state::<crate::db::DatabaseState>();
    let db_path = db_state.db_path();

    match crate::db::load_app_settings(&db_path) {
        Ok(settings) => match u64::try_from(settings.max_audio_cache_size_bytes) {
            Ok(max_audio_cache_size_bytes) if max_audio_cache_size_bytes > 0 => {
                max_audio_cache_size_bytes
            }
            Ok(_) | Err(_) => {
                log::warn!(
                    "Invalid audio cache size setting, falling back to default {} MB",
                    (crate::db::DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES as u64) / (1024 * 1024)
                );
                crate::db::DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES as u64
            }
        },
        Err(error) => {
            log::warn!("Failed to load audio cache size setting, falling back to default: {error}");
            crate::db::DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES as u64
        }
    }
}

/// Clean up a failed playback start by removing temp files.
pub fn cleanup_failed_playback_start(meta: &super::download::DownloadMeta, temp_path: &Path) {
    use std::sync::atomic::Ordering;
    meta.cancelled.store(true, Ordering::Release);
    meta.complete.store(true, Ordering::Release);
    let _ = std::fs::remove_file(temp_path);
    let _ = std::fs::remove_file(cache_complete_marker_path(temp_path));
}
