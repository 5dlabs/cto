//! Markdown file generation with YAML frontmatter.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::PathBuf;

use crate::analysis::RelevanceResult;
use crate::enrichment::EnrichedLink;
use crate::twitter::Bookmark;

/// A complete research entry ready for storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEntry {
    /// Original bookmark.
    pub bookmark: Bookmark,
    /// Relevance analysis.
    pub relevance: RelevanceResult,
    /// Enriched content from links.
    #[serde(default)]
    pub enriched: Vec<EnrichedLink>,
    /// Generated summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Tags for search.
    #[serde(default)]
    pub tags: Vec<String>,
    /// When this was processed.
    pub processed_at: DateTime<Utc>,
}

impl ResearchEntry {
    /// Create a new research entry.
    #[must_use]
    pub fn new(bookmark: Bookmark, relevance: RelevanceResult) -> Self {
        Self {
            bookmark,
            relevance,
            enriched: Vec::new(),
            summary: None,
            tags: Vec::new(),
            processed_at: Utc::now(),
        }
    }

    /// Add enriched links.
    #[must_use]
    pub fn with_enriched(mut self, enriched: Vec<EnrichedLink>) -> Self {
        self.enriched = enriched;
        self
    }

    /// Add a summary.
    #[must_use]
    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }

    /// Generate tags from categories and topics.
    pub fn generate_tags(&mut self) {
        let mut tags = Vec::new();

        // Add categories as tags
        for cat in &self.relevance.categories {
            tags.push(cat.to_string());
        }

        // Add topics as tags (lowercase, hyphenated)
        for topic in &self.relevance.topics {
            let tag = topic.to_lowercase().replace(' ', "-");
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }

        self.tags = tags;
    }
}

/// Writes research entries as markdown files.
pub struct MarkdownWriter {
    base_dir: PathBuf,
}

impl MarkdownWriter {
    /// Create a new writer with the given base directory.
    #[must_use]
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Write a research entry to a markdown file.
    pub fn write(&self, entry: &ResearchEntry) -> Result<PathBuf> {
        let path = self.entry_path(entry);

        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = self.render_markdown(entry);
        std::fs::write(&path, content)?;

        tracing::info!(path = %path.display(), "Wrote research entry");
        Ok(path)
    }

    /// Get the path for a research entry.
    fn entry_path(&self, entry: &ResearchEntry) -> PathBuf {
        let date = entry.processed_at.format("%Y/%m/%d");
        self.base_dir
            .join(date.to_string())
            .join(format!("{}.md", entry.bookmark.id))
    }

    /// Render a research entry as markdown with frontmatter.
    #[allow(clippy::unused_self)]
    fn render_markdown(&self, entry: &ResearchEntry) -> String {
        let mut md = String::new();

        // YAML frontmatter
        md.push_str("---\n");
        let _ = writeln!(md, "id: \"{}\"", entry.bookmark.id);
        let _ = writeln!(md, "author: \"@{}\"", entry.bookmark.author.handle);
        let _ = writeln!(
            md,
            "posted_at: \"{}\"",
            entry.bookmark.posted_at.to_rfc3339()
        );
        let _ = writeln!(md, "processed_at: \"{}\"", entry.processed_at.to_rfc3339());
        let _ = writeln!(md, "implementation_score: {:.2}", entry.relevance.score);
        let _ = writeln!(
            md,
            "implementation_priority: \"{}\"",
            implementation_label(entry.relevance.score)
        );

        // Implementation Ideas
        if !entry.relevance.implementation_ideas.is_empty() {
            md.push_str("implementation_ideas:\n");
            for idea in &entry.relevance.implementation_ideas {
                // Escape quotes and newlines for valid YAML
                let escaped = idea.replace('"', "'").replace('\n', "\\n");
                let _ = writeln!(md, "  - \"{escaped}\"");
            }
        }

        // Categories
        if !entry.relevance.categories.is_empty() {
            md.push_str("categories:\n");
            for cat in &entry.relevance.categories {
                let _ = writeln!(md, "  - {cat}");
            }
        }

        // Topics
        if !entry.relevance.topics.is_empty() {
            md.push_str("topics:\n");
            for topic in &entry.relevance.topics {
                let _ = writeln!(md, "  - {topic}");
            }
        }

        // Tags
        if !entry.tags.is_empty() {
            md.push_str("tags:\n");
            for tag in &entry.tags {
                let _ = writeln!(md, "  - {tag}");
            }
        }

        md.push_str("---\n\n");

        // Title
        let title = generate_title(entry);
        let _ = writeln!(md, "# {title}\n");

        // Original Tweet
        md.push_str("## Original Tweet\n\n");
        let _ = writeln!(md, "> {}\n", entry.bookmark.text.replace('\n', "\n> "));
        let _ = writeln!(
            md,
            "**Author**: {} ({})  ",
            entry.bookmark.author.at_handle(),
            entry.bookmark.author.name
        );
        let _ = writeln!(
            md,
            "**Posted**: {}\n",
            entry.bookmark.posted_at.format("%B %d, %Y at %I:%M %p")
        );

        // Analysis
        md.push_str("## Analysis\n\n");
        let _ = writeln!(md, "{}\n", entry.relevance.reasoning);
        let _ = writeln!(
            md,
            "**Implementation Potential**: {} ({:.2})\n",
            implementation_label(entry.relevance.score),
            entry.relevance.score
        );

        // Implementation Ideas
        if !entry.relevance.implementation_ideas.is_empty() {
            md.push_str("### 💡 Implementation Ideas\n\n");
            for idea in &entry.relevance.implementation_ideas {
                let _ = writeln!(md, "- {idea}");
            }
            md.push('\n');
        }

        // Summary
        if let Some(summary) = &entry.summary {
            md.push_str("## Summary\n\n");
            md.push_str(summary);
            md.push_str("\n\n");
        }

        // Enriched content (full scraped content from links)
        if !entry.enriched.is_empty() {
            md.push_str("---\n\n");
            md.push_str("## Supporting Content\n\n");
            for (i, link) in entry.enriched.iter().enumerate() {
                if i > 0 {
                    md.push_str("\n---\n\n");
                }

                if let Some(title) = &link.title {
                    let _ = writeln!(md, "### {title}\n");
                } else {
                    let _ = writeln!(md, "### Source {}\n", i + 1);
                }
                let _ = writeln!(md, "**URL**: <{}>\n", link.url);

                // Include full content
                if !link.content.is_empty() {
                    md.push_str(&link.content);
                    md.push_str("\n\n");
                }
            }
        }

        // References section at the bottom
        md.push_str("---\n\n");
        md.push_str("## References\n\n");

        // Tweet link
        let _ = writeln!(
            md,
            "- **Original Tweet**: <https://x.com/{}/status/{}>",
            entry.bookmark.author.handle, entry.bookmark.id
        );

        // Enriched source URLs
        for link in &entry.enriched {
            if let Some(title) = &link.title {
                let _ = writeln!(md, "- **{}**: <{}>", title, link.url);
            } else {
                let _ = writeln!(md, "- <{}>", link.url);
            }
        }

        md.push_str("\n---\n\n");
        md.push_str("*Curated by CTO Research Pipeline*\n");

        md
    }
}

/// Generate a title for the research entry.
fn generate_title(entry: &ResearchEntry) -> String {
    // Try to use first topic, or category, or generic
    if let Some(topic) = entry.relevance.topics.first() {
        format!("Research: {topic}")
    } else if let Some(cat) = entry.relevance.categories.first() {
        format!("Research: {}", cat.to_string().to_uppercase())
    } else {
        "Research Entry".to_string()
    }
}

/// Get a label for the implementation potential score.
fn implementation_label(score: f32) -> &'static str {
    if score >= 0.9 {
        "🚀 Build This"
    } else if score >= 0.7 {
        "🔍 Worth Investigating"
    } else if score >= 0.5 {
        "📌 Low Priority"
    } else {
        "📖 Not Actionable"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::Category;
    use crate::twitter::Author;

    #[test]
    fn test_entry_path() {
        let writer = MarkdownWriter::new(PathBuf::from("/data/research"));
        let entry = ResearchEntry::new(
            Bookmark::new(
                "123456".to_string(),
                Author::new("test".to_string(), "Test".to_string()),
                "Test tweet".to_string(),
                Utc::now(),
            ),
            RelevanceResult {
                score: 0.8,
                reasoning: "Relevant".to_string(),
                categories: vec![Category::Rust],
                topics: vec!["testing".to_string()],
                should_enrich: false,
                implementation_ideas: vec!["Test idea".to_string()],
            },
        );

        let path = writer.entry_path(&entry);
        assert!(path.to_string_lossy().contains("123456.md"));
    }
}
