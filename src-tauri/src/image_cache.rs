use rusqlite::{params, Connection};
use sha1_smol::Sha1;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

const ORIGINALS_DIR: &str = "originals";
const THUMBNAILS_DIR: &str = "thumbnails";
const METADATA_DB: &str = "meta.db";

pub struct ImageCache {
    cache_dir: PathBuf,
}

impl ImageCache {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let app_data = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {e}"))?;
        let cache_dir = app_data.join("image_cache");
        Self::from_dir(cache_dir)
    }

    pub fn new_from_path(cache_dir: &Path) -> Result<Self, String> {
        Self::from_dir(cache_dir.to_path_buf())
    }

    fn from_dir(dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(dir.join(ORIGINALS_DIR))
            .map_err(|e| format!("Failed to create originals dir: {e}"))?;
        std::fs::create_dir_all(dir.join(THUMBNAILS_DIR))
            .map_err(|e| format!("Failed to create thumbnails dir: {e}"))?;

        let cache = Self { cache_dir: dir };
        cache.initialize_metadata_db()?;

        Ok(cache)
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    fn metadata_db_path(&self) -> PathBuf {
        self.cache_dir.join(METADATA_DB)
    }

    fn originals_dir(&self) -> PathBuf {
        self.cache_dir.join(ORIGINALS_DIR)
    }

    fn thumbnails_dir(&self) -> PathBuf {
        self.cache_dir.join(THUMBNAILS_DIR)
    }

    fn open_metadata_db(&self) -> Result<Connection, String> {
        Connection::open(self.metadata_db_path())
            .map_err(|e| format!("Failed to open image cache metadata DB: {e}"))
    }

    fn initialize_metadata_db(&self) -> Result<(), String> {
        let conn = self.open_metadata_db()?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")
            .map_err(|e| format!("Failed to enable WAL mode for image cache metadata DB: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS images (
                url_hash TEXT PRIMARY KEY,
                original_url TEXT NOT NULL UNIQUE,
                content_type TEXT,
                file_ext TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                width INTEGER,
                height INTEGER,
                access_count INTEGER NOT NULL DEFAULT 1,
                last_accessed INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS thumbnails (
                url_hash TEXT NOT NULL REFERENCES images(url_hash) ON DELETE CASCADE,
                size INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (url_hash, size)
            );",
        )
        .map_err(|e| format!("Failed to initialize image cache metadata DB: {e}"))?;
        Ok(())
    }

    pub fn url_hash(url: &str) -> String {
        Sha1::from(url).digest().to_string()
    }

    pub fn ensure_cached(&self, url: &str) -> Result<PathBuf, String> {
        let hash = Self::url_hash(url);
        let conn = self.open_metadata_db()?;

        if let Some(path) = self.get_cached_path_internal(&conn, &hash)? {
            self.touch_access_time(&conn, &hash)?;
            return Ok(path);
        }

        let client = crate::feed_ingest::build_http_client("JRSS/0.0.1 Image Cache")?;
        let response = client
            .get(url)
            .send()
            .map_err(|e| format!("Failed to fetch image for cache: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("Image fetch failed with status {status}"));
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);

        let bytes = response
            .bytes()
            .map_err(|e| format!("Failed to read image response: {e}"))?;

        let file_ext = guess_image_extension(&content_type, url);
        let file_name = format!("{hash}.{file_ext}");
        let file_path = self.originals_dir().join(&file_name);

        std::fs::write(&file_path, &bytes)
            .map_err(|e| format!("Failed to write cached image: {e}"))?;

        let file_size = bytes.len() as i64;
        let (width, height) = decode_image_dimensions(&bytes);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO images (url_hash, original_url, content_type, file_ext, file_size, width, height, access_count, last_accessed, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)",
            params![hash, url, content_type, file_ext, file_size, width, height, now],
        )
        .map_err(|e| format!("Failed to record image cache metadata: {e}"))?;

        Ok(file_path)
    }

    fn get_cached_path_internal(
        &self,
        conn: &Connection,
        hash: &str,
    ) -> Result<Option<PathBuf>, String> {
        let result = conn.query_row(
            "SELECT file_ext FROM images WHERE url_hash = ?1",
            params![hash],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(ext) => {
                let path = self.originals_dir().join(format!("{hash}.{ext}"));
                if path.exists() {
                    Ok(Some(path))
                } else {
                    let _ = conn.execute("DELETE FROM images WHERE url_hash = ?1", params![hash]);
                    Ok(None)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to query image cache: {e}")),
        }
    }

    fn touch_access_time(&self, conn: &Connection, hash: &str) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        conn.execute(
            "UPDATE images SET access_count = access_count + 1, last_accessed = ?1 WHERE url_hash = ?2",
            params![now, hash],
        )
        .map_err(|e| format!("Failed to update access time: {e}"))?;
        Ok(())
    }

    pub fn get_or_create_thumbnail(&self, url: &str, size: u32) -> Result<PathBuf, String> {
        let hash = Self::url_hash(url);
        let conn = self.open_metadata_db()?;

        let thumb_result = conn.query_row(
            "SELECT file_path FROM thumbnails WHERE url_hash = ?1 AND size = ?2",
            params![hash, size as i64],
            |row| row.get::<_, String>(0),
        );

        if let Ok(path_str) = thumb_result {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                self.touch_access_time(&conn, &hash)?;
                return Ok(path);
            }
            let _ = conn.execute(
                "DELETE FROM thumbnails WHERE url_hash = ?1 AND size = ?2",
                params![hash, size as i64],
            );
        }

        let original_path = self.ensure_cached(url)?;

        let img = image::open(&original_path)
            .map_err(|e| format!("Failed to decode cached image for thumbnail: {e}"))?;

        let thumbnail = img.resize(size, size, image::imageops::FilterType::Triangle);

        let thumb_file_name = format!("{hash}_{size}x{size}.jpg");
        let thumb_path = self.thumbnails_dir().join(&thumb_file_name);

        // SAFETY: JPEG format is universally supported for thumbnails.
        thumbnail
            .save_with_format(&thumb_path, image::ImageFormat::Jpeg)
            .map_err(|e| format!("Failed to save thumbnail: {e}"))?;

        let file_size = std::fs::metadata(&thumb_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO thumbnails (url_hash, size, file_path, file_size, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                hash,
                size as i64,
                thumb_path.to_string_lossy().to_string(),
                file_size,
                now
            ],
        )
        .map_err(|e| format!("Failed to record thumbnail metadata: {e}"))?;

        Ok(thumb_path)
    }

    pub fn get_cached_dimensions(&self, url: &str) -> Result<Option<(u32, u32)>, String> {
        let conn = self.open_metadata_db()?;
        let hash = Self::url_hash(url);

        let result = conn.query_row(
            "SELECT width, height FROM images WHERE url_hash = ?1 AND width IS NOT NULL AND height IS NOT NULL",
            params![hash],
            |row| Ok((row.get::<_, u32>(0)?, row.get::<_, u32>(1)?)),
        );

        match result {
            Ok(dims) => Ok(Some(dims)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to query cached dimensions: {e}")),
        }
    }

}

fn guess_image_extension(content_type: &Option<String>, url: &str) -> String {
    if let Some(ct) = content_type {
        match ct.as_str() {
            "image/jpeg" | "image/jpg" => return "jpg".to_string(),
            "image/png" => return "png".to_string(),
            "image/gif" => return "gif".to_string(),
            "image/webp" => return "webp".to_string(),
            _ => {}
        }
    }

    let url_lower = url.to_lowercase();
    if url_lower.ends_with(".png") {
        "png"
    } else if url_lower.ends_with(".gif") {
        "gif"
    } else if url_lower.ends_with(".webp") {
        "webp"
    } else {
        "jpg"
    }
    .to_string()
}

fn decode_image_dimensions(bytes: &[u8]) -> (Option<u32>, Option<u32>) {
    match image::load_from_memory(bytes) {
        Ok(img) => (Some(img.width()), Some(img.height())),
        Err(_) => (None, None),
    }
}
