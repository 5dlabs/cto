//! AI-powered digest analysis.
//!
//! Analyzes research entries to generate actionable recommendations
//! for the CTO platform.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use tasks::ai::{parse_ai_response, AIMessage, AIProvider, GenerateOptions};

use crate::storage::IndexEntry;

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

impl DigestAnalyzer {
    /// Create a new digest analyzer.
    pub fn new(provider: Arc<dyn AIProvider>, model: String) -> Self {
        Self { provider, model }
    }

    /// Analyze a batch of research entries.
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
            entries = entries_context.join("\n"),
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

