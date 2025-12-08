//! Research index for cataloging entries.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::analysis::Category;

/// Summary entry for the index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    /// Tweet ID.
    pub id: String,
    /// Author handle.
    pub author: String,
    /// First 100 chars of tweet text.
    pub preview: String,
    /// Relevance score.
    pub score: f32,
    /// Categories.
    pub categories: Vec<Category>,
    /// When processed.
    pub processed_at: DateTime<Utc>,
    /// Path to full markdown file.
    pub path: String,
}

/// Research index that tracks all entries.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchIndex {
    /// Map of tweet ID to index entry.
    pub entries: HashMap<String, IndexEntry>,
    /// Last updated timestamp.
    pub updated_at: Option<DateTime<Utc>>,
}

impl ResearchIndex {
    /// Load index from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let index: Self = serde_json::from_str(&content)?;
            Ok(index)
        } else {
            Ok(Self::default())
        }
    }

    /// Save index to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add an entry to the index.
    pub fn add(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.id.clone(), entry);
        self.updated_at = Some(Utc::now());
    }

    /// Check if an entry exists.
    pub fn contains(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    /// Get entries by category.
    pub fn by_category(&self, category: Category) -> Vec<&IndexEntry> {
        self.entries
            .values()
            .filter(|e| e.categories.contains(&category))
            .collect()
    }

    /// Get recent entries sorted by date.
    pub fn recent(&self, limit: usize) -> Vec<&IndexEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| b.processed_at.cmp(&a.processed_at));
        entries.into_iter().take(limit).collect()
    }

    /// Search entries by text.
    pub fn search(&self, query: &str) -> Vec<&IndexEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .values()
            .filter(|e| {
                e.preview.to_lowercase().contains(&query_lower)
                    || e.author.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}
