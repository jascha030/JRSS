//! Download management for audio files.

use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use super::cache::{cache_complete_marker_path, enforce_cache_size_limit};

/// Tracks download progress and completion.
pub struct DownloadMeta {
    pub bytes_written: AtomicU64,
    pub total_size: AtomicU64,
    pub complete: AtomicBool,
    pub cancelled: AtomicBool,
}

impl DownloadMeta {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            bytes_written: AtomicU64::new(0),
            total_size: AtomicU64::new(0),
            complete: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
        })
    }
}

/// Download audio from URL to file with progress tracking.
pub fn download_to_file(
    url: &str,
    path: &Path,
    meta: &DownloadMeta,
    cache_dir: &Path,
    protected_paths: &[std::path::PathBuf],
    max_cache_size_bytes: u64,
) -> Result<(), String> {
    if let Err(e) =
        enforce_cache_size_limit(cache_dir, path, protected_paths, 0, max_cache_size_bytes)
    {
        log::warn!("Failed to enforce cache size limit: {e}");
    }

    let dl_start = Instant::now();
    let marker_path = cache_complete_marker_path(path);
    let _ = std::fs::remove_file(&marker_path);

    crate::rate_limit::throttle_request(url);

    // Use a configured HTTP client with timeout and user agent
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .user_agent("JRSS/1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = {
        let mut attempt = 0;
        loop {
            attempt += 1;
            let resp = client
                .get(url)
                .send()
                .map_err(|e| format!("HTTP request failed: {e}"))?;

            let status = resp.status();
            if status.is_success() {
                break resp;
            }

            if status.as_u16() == 429 && attempt < 3 {
                let backoff = std::time::Duration::from_secs(2_u64.pow(attempt - 1));
                if let Some(retry_after) = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                {
                    log::warn!("Download hit 429, respecting Retry-After: {}s", retry_after);
                    std::thread::sleep(std::time::Duration::from_secs(retry_after));
                } else {
                    log::warn!(
                        "Download hit 429, backing off for {:?} (attempt {}/{})",
                        backoff,
                        attempt,
                        3
                    );
                    std::thread::sleep(backoff);
                }
                continue;
            }

            return Err(format!("HTTP {}", status));
        }
    };

    // Capture Content-Length for total size
    if let Some(content_length) = response.content_length() {
        meta.total_size.store(content_length, Ordering::Release);
        log::debug!("Download content-length: {} bytes", content_length);

        if let Err(e) =
            enforce_cache_size_limit(
                cache_dir,
                path,
                protected_paths,
                content_length,
                max_cache_size_bytes,
            )
        {
            log::warn!("Failed to enforce cache size limit with content length: {e}");
        }
    }

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| format!("Failed to open file for writing: {e}"))?;
    let mut file = io::BufWriter::new(file);

    let mut reader = response;
    let mut buf = [0u8; 32 * 1024];

    loop {
        if meta.cancelled.load(Ordering::Acquire) {
            return Ok(());
        }

        match reader.read(&mut buf) {
            Ok(0) => {
                file.flush().map_err(|e| format!("Flush failed: {e}"))?;
                std::fs::write(&marker_path, b"complete")
                    .map_err(|e| format!("Failed to write cache completion marker: {e}"))?;
                meta.complete.store(true, Ordering::Release);
                let elapsed = dl_start.elapsed();
                let bytes = meta.bytes_written.load(Ordering::Acquire);
                log::info!(
                    "Download complete: {} bytes in {:?} ({:.2} KB/s)",
                    bytes,
                    elapsed,
                    bytes as f64 / 1024.0 / elapsed.as_secs_f64()
                );
                return Ok(());
            }
            Ok(n) => {
                file.write_all(&buf[..n])
                    .map_err(|e| format!("Write failed: {e}"))?;
                meta.bytes_written.fetch_add(n as u64, Ordering::Release);
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(format!("Read error: {e}")),
        }
    }
}
