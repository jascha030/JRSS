//! StreamingFile — a growing file that blocks reads at the download edge.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use super::download::DownloadMeta;

pub struct StreamingFile {
    file: File,
    cursor: u64,
    meta: Arc<DownloadMeta>,
}

impl StreamingFile {
    pub fn open(path: &Path, meta: Arc<DownloadMeta>) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            file,
            cursor: 0,
            meta,
        })
    }
}

impl Read for StreamingFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            // Check for cancellation first
            if self.meta.cancelled.load(Ordering::Acquire) {
                return Ok(0);
            }

            let available = self.meta.bytes_written.load(Ordering::Acquire);
            let is_complete = self.meta.complete.load(Ordering::Acquire);

            if self.cursor < available {
                let readable = (available - self.cursor) as usize;
                let to_read = buf.len().min(readable);
                self.file.seek(SeekFrom::Start(self.cursor))?;
                let n = self.file.read(&mut buf[..to_read])?;
                self.cursor += n as u64;
                return Ok(n);
            }

            if is_complete {
                return Ok(0);
            }

            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Seek for StreamingFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let available = self.meta.bytes_written.load(Ordering::Acquire);
        let is_complete = self.meta.complete.load(Ordering::Acquire);
        let total_size = self.meta.total_size.load(Ordering::Acquire);

        // When the download is still in progress, clamp seeks to the bytes
        // that are actually on disk. Using total_size while incomplete would
        // let the decoder seek into undownloaded territory and read garbage.
        let effective_size = if is_complete && total_size > 0 {
            total_size
        } else {
            available
        };

        let new_pos = match pos {
            SeekFrom::Start(n) => n.min(effective_size),
            SeekFrom::Current(n) => {
                if n >= 0 {
                    self.cursor.saturating_add(n as u64).min(effective_size)
                } else {
                    self.cursor.saturating_sub(n.unsigned_abs())
                }
            }
            SeekFrom::End(n) => {
                if n >= 0 {
                    effective_size.saturating_add(n as u64)
                } else {
                    effective_size.saturating_sub(n.unsigned_abs())
                }
            }
        };
        self.cursor = new_pos;
        Ok(new_pos)
    }
}
