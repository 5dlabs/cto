//! AI-powered digest analysis.
//!
//! Analyzes research entries to generate actionable recommendations
//! for the CTO platform.
//!
//! NOTE: Full AI analysis requires the intake crate. For now, returns
//! basic summaries without AI enrichment.

use serde::{Deserialize, Serialize};

use crate::storage::{DigestContent, IndexEntry, MarkdownReader};

/// Analysis result for a batch of research entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestAnalysis {
    pub summary: String,
    pub high_priority: Vec<ActionItem>,
    pub worth_investigating: Vec<ActionItem>,
    pub tips_and_tricks: Vec<String>,
    pub overall_recommendation: String,
}

/// Action item from digest analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub title: String,
    pub description: String,
    pub related_entries: Vec<String>,
    pub effort: String,
    pub impact: String,
}

/// Analyzes research entries for the digest.
pub struct DigestAnalyzer;

/// Entry with optional rich content loaded from markdown file.
pub struct RichEntry<'a> {
    pub entry: &'a IndexEntry,
    pub content: Option<DigestContent>,
}

impl DigestAnalyzer {
    /// Create a new digest analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Load rich content for entries by reading their markdown files.
    pub fn load_rich_entries<'a>(
        entries: &[&'a IndexEntry],
        base_dir: &std::path::Path,
    ) -> Vec<RichEntry<'a>> {
        entries
            .iter()
            .map(|entry| {
                let content = if entry.path.is_empty() {
                    None
                } else {
                    let full_path = base_dir.join(&entry.path);
                    MarkdownReader::load_digest_content(&full_path).ok()
                };
                RichEntry { entry, content }
            })
            .collect()
    }

    /// Analyze entries with full document content.
    ///
    /// NOTE: AI analysis disabled (requires intake dependency).
    /// Returns a basic summary without AI enrichment.
    pub async fn analyze_rich(&self, entries: &[RichEntry<'_>]) -> Result<DigestAnalysis, anyhow::Error> {
        let high_priority: Vec<ActionItem> = entries
            .iter()
            .filter(|e| e.entry.score >= 0.7)
            .map(|e| ActionItem {
                title: format!("High-relevance entry from @{}", e.entry.author),
                description: e.entry.preview.clone(),
                related_entries: vec![e.entry.id.clone()],
                effort: "unknown".to_string(),
                impact: if e.entry.score >= 0.9 { "high".to_string() } else { "medium".to_string() },
            })
            .collect();

        let worth_investigating: Vec<ActionItem> = entries
            .iter()
            .filter(|e| e.entry.score >= 0.4 && e.entry.score < 0.7)
            .map(|e| ActionItem {
                title: format!("Entry from @{}", e.entry.author),
                description: e.entry.preview.clone(),
                related_entries: vec![e.entry.id.clone()],
                effort: "unknown".to_string(),
                impact: "low".to_string(),
            })
            .collect();

        let tips: Vec<String> = vec![];

        let summary = format!(
            "Found {} research entries ({} high priority, {} worth investigating)",
            entries.len(),
            high_priority.len(),
            worth_investigating.len()
        );

        let recommendation = if high_priority.is_empty() {
            "No high-priority items this cycle.".to_string()
        } else {
            format!(
                "Review {} high-priority items for potential implementation.",
                high_priority.len()
            )
        };

        Ok(DigestAnalysis {
            summary,
            high_priority,
            worth_investigating,
            tips_and_tricks: tips,
            overall_recommendation: recommendation,
        })
    }
}

impl Default for DigestAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
