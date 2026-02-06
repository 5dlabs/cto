//! Relevance analysis using Anthropic API.
//!
//! Direct Anthropic API calls - no intake-agent dependency.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::categories::Category;
use super::prompts::PromptManager;
use crate::anthropic::{AnthropicClient, Message, Role};
use crate::context::PlatformContext;
use crate::twitter::Bookmark;

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
    #[serde(default)]
    feature_score: Option<RawFeatureScore>,
    #[serde(default)]
    affected_agents: Vec<String>,
    #[serde(default)]
    installable_skill: Option<RawInstallableAsset>,
    #[serde(default)]
    installable_mcp_server: Option<RawInstallableAsset>,
}

#[derive(Debug, Deserialize, Default)]
struct RawFeatureScore {
    #[serde(default)]
    technical_fit: u8,
    #[serde(default)]
    impact: u8,
    #[serde(default)]
    effort: u8,
    #[serde(default)]
    urgency: u8,
    #[serde(default)]
    strategic_alignment: u8,
}

#[derive(Debug, Deserialize, Default)]
struct RawInstallableAsset {
    #[serde(default)]
    github_url: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallableAsset {
    pub github_url: String,
    pub name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureScore {
    pub technical_fit: u8,
    pub impact: u8,
    pub effort: u8,
    pub urgency: u8,
    pub strategic_alignment: u8,
}

impl FeatureScore {
    #[must_use]
    pub fn overall(&self) -> f32 {
        let base = (f32::from(self.impact) * 0.30
            + f32::from(self.technical_fit) * 0.25
            + f32::from(self.strategic_alignment) * 0.25
            + f32::from(self.urgency) * 0.10)
            / 10.0;
        let effort_penalty = (f32::from(self.effort) - 1.0) * 0.02;
        (base - effort_penalty).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn priority(&self) -> Priority {
        let overall = self.overall();
        if self.impact >= 8 && self.urgency >= 8 && self.effort <= 5 {
            return Priority::Critical;
        }
        if overall >= 0.8 {
            Priority::High
        } else if overall >= 0.6 {
            Priority::Medium
        } else if overall >= 0.4 {
            Priority::Low
        } else {
            Priority::Research
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    #[default]
    Research,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "🔴 Critical"),
            Self::High => write!(f, "🟠 High"),
            Self::Medium => write!(f, "🟡 Medium"),
            Self::Low => write!(f, "🟢 Low"),
            Self::Research => write!(f, "🔬 Research"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceResult {
    pub score: f32,
    pub reasoning: String,
    pub categories: Vec<Category>,
    pub topics: Vec<String>,
    pub should_enrich: bool,
    #[serde(default)]
    pub implementation_ideas: Vec<String>,
    #[serde(default)]
    pub feature_score: FeatureScore,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub affected_agents: Vec<String>,
    #[serde(default)]
    pub installable_skill: Option<InstallableAsset>,
    #[serde(default)]
    pub installable_mcp_server: Option<InstallableAsset>,
}

impl RelevanceResult {
    #[must_use]
    pub fn is_relevant(&self, min_score: f32) -> bool {
        self.score >= min_score
    }

    #[must_use]
    pub fn is_high_priority(&self) -> bool {
        self.score >= 0.9 || self.priority == Priority::Critical || self.priority == Priority::High
    }

    #[must_use]
    pub fn is_worth_investigating(&self) -> bool {
        self.score >= 0.7 || self.priority <= Priority::Medium
    }
}

pub struct RelevanceAnalyzer {
    client: AnthropicClient,
    prompts: PromptManager,
    model: String,
    context: Option<PlatformContext>,
}

impl RelevanceAnalyzer {
    pub fn new(client: AnthropicClient, model: String) -> Result<Self> {
        let prompts = PromptManager::new()?;
        Ok(Self {
            client,
            prompts,
            model,
            context: None,
        })
    }

    pub fn with_context(client: AnthropicClient, model: String, context: PlatformContext) -> Result<Self> {
        let prompts = PromptManager::new()?;
        Ok(Self {
            client,
            prompts,
            model,
            context: Some(context),
        })
    }

    pub async fn analyze(&self, bookmark: &Bookmark) -> Result<RelevanceResult> {
        let prompt_data = serde_json::json!({
            "author": bookmark.author.handle,
            "tweet_text": bookmark.text,
            "urls": bookmark.urls,
        });

        let prompt = self.prompts.render("relevance", &prompt_data)?;

        let system_prompt = if let Some(ctx) = &self.context {
            format!("{}\n\n## Platform Context\n\n{}", SYSTEM_PROMPT, ctx.to_prompt_context())
        } else {
            SYSTEM_PROMPT.to_string()
        };

        let messages = vec![
            Message { role: Role::System, content: system_prompt },
            Message { role: Role::User, content: prompt },
        ];

        let response = self
            .client
            .message(&self.model, &messages, 1500, Some(0.3))
            .await
            .context("Anthropic API call failed")?;

        let text = response
            .content
            .first()
            .and_then(|c| c.text.clone())
            .context("Empty response from Anthropic")?;

        let raw: RawRelevanceResponse = crate::anthropic::parse_json_response(&text)
            .context("Failed to parse AI response as JSON")?;

        let categories: Vec<Category> = raw
            .categories
            .iter()
            .filter_map(|s| Category::parse(s))
            .collect();

        let feature_score = raw.feature_score.map_or_else(
            || {
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let score_scaled = (raw.score * 10.0) as u8;
                FeatureScore {
                    technical_fit: score_scaled.min(10),
                    impact: score_scaled.min(10),
                    effort: 5,
                    urgency: if raw.score >= 0.9 { 8 } else { 5 },
                    strategic_alignment: score_scaled.min(10),
                }
            },
            |fs| FeatureScore {
                technical_fit: fs.technical_fit.min(10),
                impact: fs.impact.min(10),
                effort: fs.effort.clamp(1, 10),
                urgency: fs.urgency.min(10),
                strategic_alignment: fs.strategic_alignment.min(10),
            },
        );

        let priority = feature_score.priority();

        let installable_skill = raw.installable_skill.and_then(|asset| {
            if asset.github_url.is_empty() {
                None
            } else {
                Some(InstallableAsset {
                    github_url: asset.github_url,
                    name: asset.name,
                    confidence: asset.confidence.clamp(0.0, 1.0),
                })
            }
        });

        let installable_mcp_server = raw.installable_mcp_server.and_then(|asset| {
            if asset.github_url.is_empty() {
                None
            } else {
                Some(InstallableAsset {
                    github_url: asset.github_url,
                    name: asset.name,
                    confidence: asset.confidence.clamp(0.0, 1.0),
                })
            }
        });

        Ok(RelevanceResult {
            score: raw.score.clamp(0.0, 1.0),
            reasoning: raw.reasoning,
            categories,
            topics: raw.topics,
            should_enrich: raw.should_enrich,
            implementation_ideas: raw.implementation_ideas,
            feature_score,
            priority,
            affected_agents: raw.affected_agents,
            installable_skill,
            installable_mcp_server,
        })
    }
}

const SYSTEM_PROMPT: &str = r#"You are a CTO evaluating technical content for IMPLEMENTATION POTENTIAL.

Your task is to evaluate tweets and determine:
1. Could we BUILD or INTEGRATE this into our platform?
2. What SPECIFIC features could we implement?
3. Is this actionable?

## Scoring

Provide multi-dimensional scoring:
- technical_fit (0-10): How well it fits our stack (Rust, React, Kubernetes)
- impact (0-10): Potential improvement
- effort (1-10): Implementation effort
- urgency (0-10): Market pressure, competitive advantage
- strategic_alignment (0-10): Alignment with roadmap

## Score Guide

- 0.9+ = "We should build this"
- 0.7-0.9 = "Worth investigating"
- 0.5-0.7 = "Interesting but low priority"
- <0.5 = "Not actionable"

## Response Format

Always respond with valid JSON:
- score (0.0-1.0)
- reasoning (string)
- categories (array)
- topics (array)
- should_enrich (boolean)
- implementation_ideas (array)
- feature_score (object with technical_fit, impact, effort, urgency, strategic_alignment)
- affected_agents (array)
- installable_skill (object or null)
- installable_mcp_server (object or null)
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_score_overall() {
        let score = FeatureScore {
            technical_fit: 8,
            impact: 9,
            effort: 3,
            urgency: 7,
            strategic_alignment: 8,
        };
        let overall = score.overall();
        assert!(overall >= 0.69, "Expected overall >= 0.69, got {overall}");
    }
}
