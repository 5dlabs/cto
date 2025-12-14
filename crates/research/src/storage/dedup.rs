//! Duplicate detection for research entries.

use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Tracks processed tweet IDs to avoid duplicates.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DedupTracker {
    /// Set of processed tweet IDs.
    processed_ids: HashSet<String>,
}

impl DedupTracker {
    /// Load tracker from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let tracker: Self = serde_json::from_str(&content)?;
            Ok(tracker)
        } else {
            Ok(Self::default())
        }
    }

    /// Save tracker to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Check if a tweet ID has been processed.
    pub fn is_duplicate(&self, id: &str) -> bool {
        self.processed_ids.contains(id)
    }

    /// Mark a tweet ID as processed.
    pub fn mark_processed(&mut self, id: &str) {
        self.processed_ids.insert(id.to_string());
    }

    /// Get the count of processed IDs.
    pub fn count(&self) -> usize {
        self.processed_ids.len()
    }
}
