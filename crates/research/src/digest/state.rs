//! Digest state tracking.
//!
//! Tracks when the last digest was sent and which entries haven't been included yet.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// State for tracking digest sends.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DigestState {
    /// When the last digest email was sent.
    pub last_digest_at: Option<DateTime<Utc>>,
    /// Entry IDs that have been processed but not yet included in a digest.
    pub entries_since_digest: Vec<String>,
    /// Total digests sent (for stats).
    pub total_digests_sent: u64,
}

impl DigestState {
    /// Load state from a JSON file, creating default if not exists.
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let state: Self = serde_json::from_str(&content)?;
            Ok(state)
        } else {
            Ok(Self::default())
        }
    }

    /// Save state to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add an entry ID to the pending list.
    pub fn add_entry(&mut self, entry_id: String) {
        if !self.entries_since_digest.contains(&entry_id) {
            self.entries_since_digest.push(entry_id);
        }
    }

    /// Get the number of pending entries.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.entries_since_digest.len()
    }

    /// Mark a digest as sent, clearing the pending entries.
    pub fn mark_digest_sent(&mut self) {
        self.last_digest_at = Some(Utc::now());
        self.entries_since_digest.clear();
        self.total_digests_sent += 1;
    }

    /// Check if enough time has passed for a scheduled digest (e.g., 24 hours).
    #[must_use]
    pub fn is_scheduled_time(&self, hours_between: u64) -> bool {
        match self.last_digest_at {
            Some(last) => {
                let elapsed = Utc::now().signed_duration_since(last);
                elapsed.num_hours() >= hours_between as i64
            }
            None => true, // Never sent, so yes
        }
    }

    /// Get entry IDs pending digest, consuming them.
    pub fn take_pending_entries(&mut self) -> Vec<String> {
        std::mem::take(&mut self.entries_since_digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entry() {
        let mut state = DigestState::default();
        state.add_entry("123".to_string());
        state.add_entry("456".to_string());
        state.add_entry("123".to_string()); // Duplicate

        assert_eq!(state.pending_count(), 2);
        assert!(state.entries_since_digest.contains(&"123".to_string()));
        assert!(state.entries_since_digest.contains(&"456".to_string()));
    }

    #[test]
    fn test_mark_digest_sent() {
        let mut state = DigestState::default();
        state.add_entry("123".to_string());
        state.add_entry("456".to_string());

        state.mark_digest_sent();

        assert_eq!(state.pending_count(), 0);
        assert!(state.last_digest_at.is_some());
        assert_eq!(state.total_digests_sent, 1);
    }

    #[test]
    fn test_is_scheduled_time_never_sent() {
        let state = DigestState::default();
        assert!(state.is_scheduled_time(24));
    }

    #[test]
    fn test_take_pending_entries() {
        let mut state = DigestState::default();
        state.add_entry("123".to_string());
        state.add_entry("456".to_string());

        let entries = state.take_pending_entries();

        assert_eq!(entries.len(), 2);
        assert_eq!(state.pending_count(), 0);
    }
}

