//! Relevance analysis using AI providers.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use tasks::ai::{parse_ai_response, AIMessage, AIProvider, GenerateOptions};

use super::categories::Category;
use super::prompts::PromptManager;
use crate::twitter::Bookmark;

/// Result of relevance analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceResult {
    /// Implementation potential score (0.0-1.0).
    /// 0.9+ = "we should build this"
    /// 0.7-0.9 = "worth investigating"
    /// 0.5-0.7 = "interesting but low priority"
    /// <0.5 = "not actionable"
    pub score: f32,
    /// Reasoning explaining WHAT could be implemented and HOW.
    pub reasoning: String,
    /// Detected categories.
    pub categories: Vec<Category>,
    /// Key topics/technologies mentioned.
    pub topics: Vec<String>,
    /// Whether links should be enriched.
    pub should_enrich: bool,
    /// Specific implementation ideas for the platform.
    #[serde(default)]
    pub implementation_ideas: Vec<String>,
}

impl RelevanceResult {
    /// Check if the content is relevant based on minimum score.
    #[must_use]
    pub fn is_relevant(&self, min_score: f32) -> bool {
        self.score >= min_score
    }

    /// Check if this has high implementation potential (0.9+).
    #[must_use]
    pub fn is_high_priority(&self) -> bool {
        self.score >= 0.9
    }

    /// Check if this is worth investigating (0.7+).
    #[must_use]
    pub fn is_worth_investigating(&self) -> bool {
        self.score >= 0.7
    }
}

/// Raw response from AI for parsing.
#[derive(Debug, Deserialize)]
struct RawRelevanceResponse {
    score: f32,
    reasoning: String,
    categories: Vec<String>,
    topics: Vec<String>,
    should_enrich: bool,
    #[serde(default)]
    implementation_ideas: Vec<String>,
}

/// Analyzes content relevance using an AI provider.
pub struct RelevanceAnalyzer {
    provider: Arc<dyn AIProvider>,
    prompts: PromptManager,
    model: String,
}

impl RelevanceAnalyzer {
    /// Create a new analyzer with the given AI provider.
    pub fn new(provider: Arc<dyn AIProvider>, model: String) -> Result<Self> {
        let prompts = PromptManager::new()?;
        Ok(Self {
            provider,
            prompts,
            model,
        })
    }

    /// Analyze a bookmark for relevance.
    pub async fn analyze(&self, bookmark: &Bookmark) -> Result<RelevanceResult> {
        let prompt_data = serde_json::json!({
            "author": bookmark.author.handle,
            "tweet_text": bookmark.text,
            "urls": bookmark.urls,
        });

        let prompt = self.prompts.render("relevance", &prompt_data)?;

        let messages = vec![AIMessage::system(SYSTEM_PROMPT), AIMessage::user(prompt)];

        let options = GenerateOptions {
            temperature: Some(0.3),
            max_tokens: Some(1000),
            json_mode: true,
            ..Default::default()
        };

        let response = self
            .provider
            .generate_text(&self.model, &messages, &options)
            .await?;

        let raw: RawRelevanceResponse = parse_ai_response(&response)?;

        // Convert string categories to enum
        let categories: Vec<Category> = raw
            .categories
            .iter()
            .filter_map(|s| Category::parse(s))
            .collect();

        Ok(RelevanceResult {
            score: raw.score.clamp(0.0, 1.0),
            reasoning: raw.reasoning,
            categories,
            topics: raw.topics,
            should_enrich: raw.should_enrich,
            implementation_ideas: raw.implementation_ideas,
        })
    }
}

const SYSTEM_PROMPT: &str = r#"You are a CTO evaluating technical content for IMPLEMENTATION POTENTIAL in a multi-agent software development platform.

Your task is to evaluate tweets and determine:
1. Could we actually BUILD or INTEGRATE this into our platform?
2. What SPECIFIC features or improvements could we implement?
3. Is this actionable, or just interesting reading?
4. Whether linked content likely contains implementation details worth scraping

Score guide:
- 0.9+ = "We should build this" - clear implementation path, high value
- 0.7-0.9 = "Worth investigating" - potential value, needs more research
- 0.5-0.7 = "Interesting but low priority" - tangentially useful
- <0.5 = "Not actionable" - news, opinions, or irrelevant tech

Always respond with valid JSON. Be specific about WHAT we could implement and HOW."#;
