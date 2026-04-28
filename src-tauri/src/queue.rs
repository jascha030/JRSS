//! Queue management in Rust — survives UI destruction.
//!
//! The queue has three segments: `history` (previously played items), `manual`
//! (user-explicit additions), and `auto` (context continuation after the manual
//! queue exhausts). This mirrors the frontend's `playbackHistory`, `manualQueue`,
//! and `autoQueue` but lives here so playback can continue without any frontend
//! window present.
//!
//! ## Thread safety
//!
//! All queue operations happen on the audio thread (via channel commands),
//! so no locking is needed. The queue state is only accessed from that thread.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const MAX_HISTORY_SIZE: usize = 50;

/// A queued audio item — minimal info needed to play it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedItem {
    pub item_id: String,
    pub url: String,
    pub title: String,
    pub duration_seconds: f64,
}

/// The queue state — three segments: history, manual, and auto.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueState {
    /// Previously played items (most recent first).
    pub history: VecDeque<QueuedItem>,
    pub manual: VecDeque<QueuedItem>,
    pub auto: VecDeque<QueuedItem>,
    /// Currently playing item (if any).
    pub current: Option<QueuedItem>,
}

impl QueueState {
    /// Replace current + all queue segments in one shot.
    pub fn replace(
        &mut self,
        current: Option<QueuedItem>,
        manual: Vec<QueuedItem>,
        auto: Vec<QueuedItem>,
    ) {
        self.current = current;
        self.manual = manual.into();
        self.auto = auto.into();
    }

    /// Replace all queue segments including history (used during session restore).
    pub fn replace_with_history(
        &mut self,
        history: Vec<QueuedItem>,
        current: Option<QueuedItem>,
        manual: Vec<QueuedItem>,
        auto: Vec<QueuedItem>,
    ) {
        self.history = history.into();
        self.current = current;
        self.manual = manual.into();
        self.auto = auto.into();
    }

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

    /// Clear the playback history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Clear the current item without touching queued items.
    pub fn clear_current(&mut self) {
        self.current = None;
    }

    /// Push an item to the front of history (most recent first).
    /// Removes duplicates and maintains max size.
    fn push_history(&mut self, item: QueuedItem) {
        // Remove existing entry if present (to avoid duplicates)
        self.history.retain(|i| i.item_id != item.item_id);

        // Add to front
        self.history.push_front(item);

        // Trim to max size
        while self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_back();
        }
    }

    /// Get the next item to play (drains manual first, then auto).
    /// Moves the current item to history before advancing.
    /// Returns the item and sets it as current.
    pub fn shift_next(&mut self) -> Option<QueuedItem> {
        // Move current item to history before advancing
        if let Some(current) = self.current.take() {
            self.push_history(current);
        }

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

    /// Peek at the next item without removing it (manual first, then auto).
    pub fn peek_next(&self) -> Option<&QueuedItem> {
        self.manual.front().or_else(|| self.auto.front())
    }

    /// Serialize queue state for SQLite persistence.
    pub fn to_session_parts(&self) -> (Vec<String>, Vec<String>, Vec<String>) {
        let history: Vec<String> = self.history.iter().map(|i| i.item_id.clone()).collect();
        let manual: Vec<String> = self.manual.iter().map(|i| i.item_id.clone()).collect();
        let auto: Vec<String> = self.auto.iter().map(|i| i.item_id.clone()).collect();
        (history, manual, auto)
    }

    /// Total queue length (manual + auto). Does not include history.
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
    pub history: Vec<QueuedItem>,
    pub current: Option<QueuedItem>,
    pub manual: Vec<QueuedItem>,
    pub auto: Vec<QueuedItem>,
}

impl QueueState {
    pub fn to_event(&self) -> QueueChangedEvent {
        QueueChangedEvent {
            history: self.history.iter().cloned().collect(),
            current: self.current.clone(),
            manual: self.manual.iter().cloned().collect(),
            auto: self.auto.iter().cloned().collect(),
        }
    }
}
