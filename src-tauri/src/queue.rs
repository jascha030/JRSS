//! Queue management in Rust — survives UI destruction.
//!
//! The queue has two segments: `manual` (user-explicit additions) and `auto`
//! (context continuation after the manual queue exhausts). This mirrors the
//! frontend's `manualQueue` and `autoQueue` but lives here so playback can
//! continue without any frontend window present.
//!
//! ## Thread safety
//!
//! All queue operations happen on the audio thread (via channel commands),
//! so no locking is needed. The queue state is only accessed from that thread.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A queued audio item — minimal info needed to play it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedItem {
    pub item_id: String,
    pub url: String,
    pub title: String,
    pub duration_seconds: f64,
}

/// The queue state — two segments as described above.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueState {
    pub manual: VecDeque<QueuedItem>,
    pub auto: VecDeque<QueuedItem>,
    /// Currently playing item (if any).
    pub current: Option<QueuedItem>,
}

impl QueueState {
    /// Add an item to the end of the manual queue.
    pub fn enqueue(&mut self, item: QueuedItem) {
        // Dedupe: remove from both queues if present.
        self.manual.retain(|i| i.item_id != item.item_id);
        self.auto.retain(|i| i.item_id != item.item_id);

        self.manual.push_back(item);
    }

    /// Insert an item at the front of the manual queue (play next).
    pub fn play_next(&mut self, item: QueuedItem) {
        self.manual.retain(|i| i.item_id != item.item_id);
        self.auto.retain(|i| i.item_id != item.item_id);

        self.manual.push_front(item);
    }

    /// Remove a specific item from whichever queue contains it.
    pub fn remove(&mut self, item_id: &str) -> bool {
        if let Some(pos) = self.manual.iter().position(|i| i.item_id == item_id) {
            self.manual.remove(pos);
            return true;
        }
        if let Some(pos) = self.auto.iter().position(|i| i.item_id == item_id) {
            self.auto.remove(pos);
            return true;
        }
        false
    }

    /// Move an item up one position within its segment.
    pub fn move_up(&mut self, item_id: &str) -> bool {
        // Try manual queue first.
        if let Some(pos) = self.manual.iter().position(|i| i.item_id == item_id) {
            if pos > 0 {
                self.manual.swap(pos, pos - 1);
                return true;
            }
            // If at position 0 of manual, move to end of manual (promote to "most recent manual").
            if pos == 0 && self.manual.len() > 1 {
                let item = self.manual.remove(pos).unwrap();
                self.manual.push_back(item);
                return true;
            }
            return false;
        }

        // Try auto queue.
        if let Some(pos) = self.auto.iter().position(|i| i.item_id == item_id) {
            if pos > 0 {
                self.auto.swap(pos, pos - 1);
                return true;
            }
            // If at position 0 of auto, promote to end of manual.
            if pos == 0 {
                if let Some(item) = self.auto.remove(pos) {
                    self.manual.push_back(item);
                    return true;
                }
            }
        }
        false
    }

    /// Move an item down one position within its segment.
    pub fn move_down(&mut self, item_id: &str) -> bool {
        // Try manual queue.
        if let Some(pos) = self.manual.iter().position(|i| i.item_id == item_id) {
            if pos < self.manual.len() - 1 {
                self.manual.swap(pos, pos + 1);
                return true;
            }
            // If at last position of manual, demote to start of auto.
            if pos == self.manual.len() - 1 && !self.auto.is_empty() {
                let item = self.manual.remove(pos).unwrap();
                self.auto.push_front(item);
                return true;
            }
            return false;
        }

        // Try auto queue.
        if let Some(pos) = self.auto.iter().position(|i| i.item_id == item_id) {
            if pos < self.auto.len() - 1 {
                self.auto.swap(pos, pos + 1);
                return true;
            }
        }
        false
    }

    /// Clear both queues.
    pub fn clear(&mut self) {
        self.manual.clear();
        self.auto.clear();
    }

    /// Get the next item to play (drains manual first, then auto).
    /// Returns the item and sets it as current.
    pub fn shift_next(&mut self) -> Option<QueuedItem> {
        // Try manual first.
        if let Some(item) = self.manual.pop_front() {
            self.current = Some(item.clone());
            return Some(item);
        }

        // Then auto.
        if let Some(item) = self.auto.pop_front() {
            self.current = Some(item.clone());
            return Some(item);
        }

        None
    }

    /// Get current item.
    pub fn current_item(&self) -> Option<&QueuedItem> {
        self.current.as_ref()
    }

    /// Replace the entire queue with new items (all go to manual).
    pub fn set_queue(&mut self, items: Vec<QueuedItem>) {
        if items.is_empty() {
            self.manual.clear();
            self.auto.clear();
            self.current = None;
            return;
        }
        // First item becomes current, rest go to manual
        let mut items: VecDeque<QueuedItem> = items.into();
        let first = items.pop_front().unwrap();
        self.current = Some(first);
        self.manual = items;
        self.auto.clear();
    }

    /// Reconstruct queue from persisted session data.
    pub fn restore_from_session(
        &mut self,
        manual_ids: Vec<String>,
        auto_ids: Vec<String>,
        get_item: impl Fn(String) -> Option<QueuedItem>,
    ) {
        let mut get_item = get_item;
        self.manual = manual_ids
            .into_iter()
            .filter_map(|id| get_item(id))
            .collect();
        self.auto = auto_ids.into_iter().filter_map(|id| get_item(id)).collect();
    }

    /// Serialize queue state for SQLite persistence.
    pub fn to_session_parts(&self) -> (Vec<String>, Vec<String>) {
        let manual: Vec<String> = self.manual.iter().map(|i| i.item_id.clone()).collect();
        let auto: Vec<String> = self.auto.iter().map(|i| i.item_id.clone()).collect();
        (manual, auto)
    }

    /// Total queue length (manual + auto).
    pub fn len(&self) -> usize {
        self.manual.len() + self.auto.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Queue-changed event payload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueChangedEvent {
    pub current: Option<QueuedItem>,
    pub manual_count: usize,
    pub auto_count: usize,
}

impl QueueState {
    pub fn to_event(&self) -> QueueChangedEvent {
        QueueChangedEvent {
            current: self.current.clone(),
            manual_count: self.manual.len(),
            auto_count: self.auto.len(),
        }
    }
}
