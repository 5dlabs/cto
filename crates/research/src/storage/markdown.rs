//! Markdown file generation and parsing with YAML frontmatter.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::{Path, PathBuf};

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

/// Content extracted from a markdown research file for digest purposes.
#[derive(Debug, Clone, Default)]
pub struct DigestContent {
    /// Full tweet text (not truncated).
    pub full_text: String,
    /// Implementation ideas from the analysis.
    pub implementation_ideas: Vec<String>,
    /// AI analysis reasoning.
    pub reasoning: String,
    /// Topics identified.
    pub topics: Vec<String>,
    /// Content from enriched/scraped links.
    pub enriched_content: Vec<String>,
}

/// Reads and parses markdown research files.
pub struct MarkdownReader;

impl MarkdownReader {
    /// Load digest-relevant content from a markdown file.
    ///
    /// Extracts the full tweet text, implementation ideas, reasoning,
    /// and enriched link content that can be used for more informative digests.
    pub fn load_digest_content(path: &Path) -> Result<DigestContent> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read markdown file: {}", path.display()))?;

        let mut digest = DigestContent::default();

        // Parse frontmatter if present
        if let Some(after_start) = content.strip_prefix("---") {
            if let Some(end_idx) = after_start.find("---") {
                let frontmatter = &after_start[..end_idx];
                digest.parse_frontmatter(frontmatter);
            }
        }

        // Extract sections from the markdown body
        digest.parse_body(&content);

        Ok(digest)
    }
}

impl DigestContent {
    /// Parse YAML frontmatter for implementation_ideas and topics.
    fn parse_frontmatter(&mut self, frontmatter: &str) {
        let mut in_ideas = false;
        let mut in_topics = false;

        for line in frontmatter.lines() {
            let trimmed = line.trim();

            // Detect section starts
            if trimmed.starts_with("implementation_ideas:") {
                in_ideas = true;
                in_topics = false;
                continue;
            }
            if trimmed.starts_with("topics:") {
                in_topics = true;
                in_ideas = false;
                continue;
            }
            // End sections on new keys
            if !trimmed.starts_with('-') && !trimmed.starts_with(' ') && trimmed.contains(':') {
                in_ideas = false;
                in_topics = false;
            }

            // Extract list items
            if trimmed.starts_with("- ") || trimmed.starts_with("  - ") {
                let value = trimmed
                    .trim_start_matches('-')
                    .trim()
                    .trim_matches('"')
                    .replace("\\n", "\n");

                if in_ideas && !value.is_empty() {
                    self.implementation_ideas.push(value);
                } else if in_topics && !value.is_empty() {
                    self.topics.push(value);
                }
            }
        }
    }

    /// Parse the markdown body for tweet text, analysis, and enriched content.
    fn parse_body(&mut self, content: &str) {
        let mut current_section = "";
        let mut in_quote = false;
        let mut quote_lines: Vec<String> = Vec::new();

        for line in content.lines() {
            // Track section headers
            if line.starts_with("## ") {
                current_section = line.trim_start_matches('#').trim();
                in_quote = false;
                continue;
            }
            if line.starts_with("### ") {
                // Sub-section - check for supporting content titles
                if current_section == "Supporting Content" {
                    // Start of a new enriched section
                    continue;
                }
            }

            match current_section {
                "Original Tweet" => {
                    // Capture blockquote content
                    if line.starts_with("> ") {
                        in_quote = true;
                        quote_lines.push(line.trim_start_matches("> ").to_string());
                    } else if in_quote && !line.is_empty() && !line.starts_with("**") {
                        quote_lines.push(line.to_string());
                    } else if in_quote {
                        in_quote = false;
                        self.full_text = quote_lines.join("\n");
                        quote_lines.clear();
                    }
                }
                "Analysis" => {
                    // Capture analysis reasoning (skip metadata lines)
                    if !line.starts_with("**") && !line.is_empty() && !line.starts_with('#') {
                        if !self.reasoning.is_empty() {
                            self.reasoning.push('\n');
                        }
                        self.reasoning.push_str(line);
                    }
                }
                "Supporting Content" => {
                    // Capture enriched content (excluding headers and URLs)
                    if !line.starts_with('#')
                        && !line.starts_with("**URL**")
                        && !line.starts_with("---")
                        && !line.is_empty()
                    {
                        self.enriched_content.push(line.to_string());
                    }
                }
                _ => {}
            }
        }

        // Handle case where quote wasn't terminated
        if !quote_lines.is_empty() {
            self.full_text = quote_lines.join("\n");
        }
    }

    /// Check if this content has meaningful data beyond the basics.
    #[must_use]
    pub fn has_rich_content(&self) -> bool {
        !self.implementation_ideas.is_empty()
            || !self.reasoning.is_empty()
            || !self.enriched_content.is_empty()
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

    #[test]
    fn test_digest_content_parse_frontmatter() {
        let frontmatter = r#"
id: "123"
implementation_ideas:
  - "Build a cache layer"
  - "Add rate limiting"
topics:
  - "caching"
  - "performance"
categories:
  - rust
"#;

        let mut content = DigestContent::default();
        content.parse_frontmatter(frontmatter);

        assert_eq!(content.implementation_ideas.len(), 2);
        assert_eq!(content.implementation_ideas[0], "Build a cache layer");
        assert_eq!(content.topics.len(), 2);
        assert!(content.topics.contains(&"caching".to_string()));
    }

    #[test]
    fn test_digest_content_parse_body() {
        let body = r#"---
id: "123"
---

# Research: Testing

## Original Tweet

> This is the full tweet text that should be captured.
> It spans multiple lines.

**Author**: @test

## Analysis

This is the analysis reasoning that explains why this tweet is relevant.

## Supporting Content

This is enriched content from a linked article.
It has multiple paragraphs.

---
"#;

        let mut content = DigestContent::default();
        content.parse_body(body);

        assert!(content.full_text.contains("full tweet text"));
        assert!(content.reasoning.contains("analysis reasoning"));
        assert!(!content.enriched_content.is_empty());
    }

    #[test]
    fn test_digest_content_has_rich_content() {
        let mut content = DigestContent::default();
        assert!(!content.has_rich_content());

        content.implementation_ideas.push("Test idea".to_string());
        assert!(content.has_rich_content());

        content.implementation_ideas.clear();
        content.reasoning = "Test reasoning".to_string();
        assert!(content.has_rich_content());
    }
}
