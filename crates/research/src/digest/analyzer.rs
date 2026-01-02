//! AI-powered digest analysis.
//!
//! Analyzes research entries to generate actionable recommendations
//! for the CTO platform.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use tasks::ai::{parse_ai_response, AIMessage, AIProvider, GenerateOptions};

use crate::storage::{DigestContent, IndexEntry, MarkdownReader};

/// Analysis result for a batch of research entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestAnalysis {
    /// Executive summary of the research batch.
    pub summary: String,
    /// High-priority items that should be implemented.
    pub high_priority: Vec<ActionItem>,
    /// Items worth investigating further.
    pub worth_investigating: Vec<ActionItem>,
    /// General tips and best practices discovered.
    pub tips_and_tricks: Vec<String>,
    /// Overall recommendation for the platform.
    pub overall_recommendation: String,
}

/// An actionable item from the analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    /// Title of the action item.
    pub title: String,
    /// Description of what to implement.
    pub description: String,
    /// Which entries this relates to.
    pub related_entries: Vec<String>,
    /// Estimated effort (low/medium/high).
    pub effort: String,
    /// Potential impact (low/medium/high).
    pub impact: String,
}

/// Analyzes research entries for the digest.
pub struct DigestAnalyzer {
    provider: Arc<dyn AIProvider>,
    model: String,
}

/// Entry with optional rich content loaded from markdown file.
pub struct RichEntry<'a> {
    /// The index entry with basic metadata.
    pub entry: &'a IndexEntry,
    /// Optional full content from the markdown file.
    pub content: Option<DigestContent>,
}

impl DigestAnalyzer {
    /// Create a new digest analyzer.
    pub fn new(provider: Arc<dyn AIProvider>, model: String) -> Self {
        Self { provider, model }
    }

    /// Load rich content for entries by reading their markdown files.
    ///
    /// Returns entries paired with their full content (if readable).
    pub fn load_rich_entries<'a>(
        entries: &[&'a IndexEntry],
        base_dir: &Path,
    ) -> Vec<RichEntry<'a>> {
        entries
            .iter()
            .map(|entry| {
                let content = if !entry.path.is_empty() {
                    let full_path = base_dir.join(&entry.path);
                    MarkdownReader::load_digest_content(&full_path).ok()
                } else {
                    None
                };
                RichEntry {
                    entry,
                    content,
                }
            })
            .collect()
    }

    /// Analyze entries with full document content for richer insights.
    ///
    /// This method reads the full markdown files to include:
    /// - Complete tweet text (not just 100 char preview)
    /// - Implementation ideas from AI analysis
    /// - Reasoning and context
    /// - Enriched content from scraped links
    pub async fn analyze_rich(
        &self,
        entries: &[RichEntry<'_>],
    ) -> Result<DigestAnalysis> {
        let entries_context: Vec<String> = entries
            .iter()
            .map(|rich| {
                let e = rich.entry;
                let categories: Vec<_> = e.categories.iter().map(ToString::to_string).collect();

                // Use full content if available, otherwise fall back to preview
                if let Some(content) = &rich.content {
                    let text = if content.full_text.is_empty() {
                        &e.preview
                    } else {
                        &content.full_text
                    };

                    let mut entry_str = format!(
                        "### @{author} (score: {score:.2})\n**Categories**: {categories}\n\n**Tweet**: {text}",
                        author = e.author,
                        score = e.score,
                        categories = categories.join(", "),
                        text = text,
                    );

                    // Add implementation ideas if present
                    if !content.implementation_ideas.is_empty() {
                        entry_str.push_str("\n\n**Implementation Ideas**:\n");
                        for idea in &content.implementation_ideas {
                            entry_str.push_str(&format!("- {idea}\n"));
                        }
                    }

                    // Add reasoning if present
                    if !content.reasoning.is_empty() {
                        entry_str.push_str(&format!("\n**Analysis**: {}\n", content.reasoning));
                    }

                    // Add enriched content summary (first 500 chars)
                    if !content.enriched_content.is_empty() {
                        let enriched_summary: String = content
                            .enriched_content
                            .iter()
                            .take(5)
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .join(" ");
                        let truncated = if enriched_summary.len() > 500 {
                            format!("{}...", &enriched_summary[..500])
                        } else {
                            enriched_summary
                        };
                        entry_str.push_str(&format!("\n**From linked content**: {truncated}\n"));
                    }

                    entry_str
                } else {
                    // Fall back to basic format
                    format!(
                        "- @{author} (score: {score:.2}, categories: {categories}): {preview}",
                        author = e.author,
                        score = e.score,
                        categories = categories.join(", "),
                        preview = e.preview,
                    )
                }
            })
            .collect();

        self.run_analysis(&entries_context.join("\n\n---\n\n")).await
    }

    /// Analyze a batch of research entries (basic mode - uses preview only).
    pub async fn analyze(&self, entries: &[&IndexEntry]) -> Result<DigestAnalysis> {
        // Build context from entries
        let entries_context: Vec<String> = entries
            .iter()
            .map(|e| {
                let categories: Vec<_> = e.categories.iter().map(ToString::to_string).collect();
                format!(
                    "- @{author} (score: {score:.2}, categories: {categories}): {preview}",
                    author = e.author,
                    score = e.score,
                    categories = categories.join(", "),
                    preview = e.preview,
                )
            })
            .collect();

        self.run_analysis(&entries_context.join("\n")).await
    }

    /// Run the AI analysis with the given entries context.
    async fn run_analysis(&self, entries_context: &str) -> Result<DigestAnalysis> {
        let prompt = format!(
            r#"Analyze these research entries from Twitter/X bookmarks and provide actionable recommendations for the CTO platform.

## Platform Context
The CTO platform is a multi-agent software development system built with:
- Rust for core services
- Kubernetes for orchestration
- Argo Workflows for task automation
- MCP (Model Context Protocol) tools for AI integration
- AI agents (Rex, Blaze, Tess, etc.) for automated coding

## Research Entries to Analyze
{entries}

## Task
Analyze these entries and identify:
1. What can we BUILD or INTEGRATE into the CTO platform?
2. What's worth investigating further?
3. Any tips, tricks, or best practices we should adopt?

Respond with JSON:
{{
  "summary": "<2-3 sentence executive summary of what was found>",
  "high_priority": [
    {{
      "title": "<action item title>",
      "description": "<what to implement and how>",
      "related_entries": ["<@author references>"],
      "effort": "<low|medium|high>",
      "impact": "<low|medium|high>"
    }}
  ],
  "worth_investigating": [
    {{
      "title": "<item title>",
      "description": "<what to look into>",
      "related_entries": ["<@author references>"],
      "effort": "<low|medium|high>",
      "impact": "<low|medium|high>"
    }}
  ],
  "tips_and_tricks": ["<practical tips discovered>"],
  "overall_recommendation": "<1-2 sentence recommendation for this batch>"
}}

Be specific and actionable. Focus on implementation potential, not just interesting reading."#,
            entries = entries_context,
        );

        let messages = vec![AIMessage::system(SYSTEM_PROMPT), AIMessage::user(prompt)];

        let options = GenerateOptions {
            temperature: Some(0.4),
            max_tokens: Some(2000),
            json_mode: true,
            ..Default::default()
        };

        let response = self
            .provider
            .generate_text(&self.model, &messages, &options)
            .await?;

        let analysis: DigestAnalysis = parse_ai_response(&response)?;
        Ok(analysis)
    }
}

const SYSTEM_PROMPT: &str = r#"You are a CTO analyzing research content for implementation potential in a multi-agent software development platform.

Your role is to:
1. Identify actionable items that could improve the platform
2. Prioritize based on effort vs impact
3. Provide specific, implementable recommendations
4. Distinguish between "build now" and "investigate later"

Be concise, specific, and focus on what can actually be implemented.
Always respond with valid JSON."#;
