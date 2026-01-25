//! Relevance analysis using AI providers.
//!
//! This module provides multi-dimensional scoring for research content,
//! evaluating technical fit, impact, effort, urgency, and strategic alignment.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use intake::ai::{parse_ai_response, AIMessage, AIProvider, GenerateOptions};

use super::categories::Category;
use super::prompts::PromptManager;
use crate::context::PlatformContext;
use crate::twitter::Bookmark;

/// Represents an installable asset (skill or MCP server) detected in content.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallableAsset {
    /// GitHub repository URL for the asset.
    pub github_url: String,
    /// Human-readable name of the asset.
    pub name: String,
    /// Confidence score for this detection (0.0-1.0).
    pub confidence: f32,
}

/// Multi-dimensional feature scoring.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureScore {
    /// How well it fits our current stack (0-10).
    pub technical_fit: u8,
    /// Potential impact on platform capability (0-10).
    pub impact: u8,
    /// Estimated effort (1=hours, 5=week, 10=months).
    pub effort: u8,
    /// Urgency based on market/competitor activity (0-10).
    pub urgency: u8,
    /// Alignment with current roadmap (0-10).
    pub strategic_alignment: u8,
}

impl FeatureScore {
    /// Calculate overall score from dimensions (0.0-1.0).
    ///
    /// Formula weights impact and strategic alignment higher,
    /// penalizes high effort, and adds bonus for urgency.
    #[must_use]
    pub fn overall(&self) -> f32 {
        // Weight: impact (30%), technical_fit (25%), strategic (25%), urgency (10%), effort penalty (10%)
        let base = (f32::from(self.impact) * 0.30
            + f32::from(self.technical_fit) * 0.25
            + f32::from(self.strategic_alignment) * 0.25
            + f32::from(self.urgency) * 0.10)
            / 10.0;

        // Effort penalty: high effort reduces score
        let effort_penalty = (f32::from(self.effort) - 1.0) * 0.02;

        (base - effort_penalty).clamp(0.0, 1.0)
    }

    /// Get the computed priority based on overall score and dimensions.
    #[must_use]
    pub fn priority(&self) -> Priority {
        let overall = self.overall();

        // Critical if high impact + high urgency + reasonable effort
        if self.impact >= 8 && self.urgency >= 8 && self.effort <= 5 {
            return Priority::Critical;
        }

        // Use overall score for other priorities
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

    /// Check if this needs more investigation.
    #[must_use]
    pub fn needs_research(&self) -> bool {
        // Needs research if high impact but unclear technical fit or effort
        self.impact >= 6 && (self.technical_fit <= 4 || self.effort == 0)
    }
}

/// Priority classification for features.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Implement this week - high impact + urgent.
    Critical,
    /// This month - high overall score.
    High,
    /// This quarter - moderate priority.
    Medium,
    /// Nice to have - low priority.
    Low,
    /// Needs more investigation.
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

impl Default for Priority {
    fn default() -> Self {
        Self::Research
    }
}

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
    /// Multi-dimensional feature scoring.
    #[serde(default)]
    pub feature_score: FeatureScore,
    /// Computed priority based on scoring.
    #[serde(default)]
    pub priority: Priority,
    /// Agents that would be affected by this feature.
    #[serde(default)]
    pub affected_agents: Vec<String>,
    /// Detected installable skill (GitHub repo with SKILL.md).
    #[serde(default)]
    pub installable_skill: Option<InstallableAsset>,
    /// Detected installable MCP server.
    #[serde(default)]
    pub installable_mcp_server: Option<InstallableAsset>,
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
        self.score >= 0.9 || self.priority == Priority::Critical || self.priority == Priority::High
    }

    /// Check if this is worth investigating (0.7+).
    #[must_use]
    pub fn is_worth_investigating(&self) -> bool {
        self.score >= 0.7 || self.priority <= Priority::Medium
    }

    /// Get a formatted priority label.
    #[must_use]
    pub fn priority_label(&self) -> String {
        if self.score >= 0.9 {
            "🚀 Build This".to_string()
        } else if self.score >= 0.7 {
            "🔍 Worth Investigating".to_string()
        } else if self.score >= 0.5 {
            "📌 Low Priority".to_string()
        } else {
            "📰 Not Actionable".to_string()
        }
    }
}

/// Raw installable asset from AI response.
#[derive(Debug, Deserialize, Default)]
struct RawInstallableAsset {
    #[serde(default)]
    github_url: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    confidence: f32,
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
    #[serde(default)]
    feature_score: Option<RawFeatureScore>,
    #[serde(default)]
    affected_agents: Vec<String>,
    #[serde(default)]
    installable_skill: Option<RawInstallableAsset>,
    #[serde(default)]
    installable_mcp_server: Option<RawInstallableAsset>,
}

/// Raw feature score from AI response.
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

/// Analyzes content relevance using an AI provider.
pub struct RelevanceAnalyzer {
    provider: Arc<dyn AIProvider>,
    prompts: PromptManager,
    model: String,
    context: Option<PlatformContext>,
}

impl RelevanceAnalyzer {
    /// Create a new analyzer with the given AI provider.
    pub fn new(provider: Arc<dyn AIProvider>, model: String) -> Result<Self> {
        let prompts = PromptManager::new()?;
        Ok(Self {
            provider,
            prompts,
            model,
            context: None,
        })
    }

    /// Create a new analyzer with platform context for context-aware analysis.
    pub fn with_context(
        provider: Arc<dyn AIProvider>,
        model: String,
        context: PlatformContext,
    ) -> Result<Self> {
        let prompts = PromptManager::new()?;
        Ok(Self {
            provider,
            prompts,
            model,
            context: Some(context),
        })
    }

    /// Set the platform context for analysis.
    pub fn set_context(&mut self, context: PlatformContext) {
        self.context = Some(context);
    }

    /// Analyze a bookmark for relevance.
    pub async fn analyze(&self, bookmark: &Bookmark) -> Result<RelevanceResult> {
        let prompt_data = serde_json::json!({
            "author": bookmark.author.handle,
            "tweet_text": bookmark.text,
            "urls": bookmark.urls,
        });

        let prompt = self.prompts.render("relevance", &prompt_data)?;

        // Build system prompt with optional platform context
        let system_prompt = if let Some(ctx) = &self.context {
            format!(
                "{}\n\n## Platform Context\n\n{}",
                SYSTEM_PROMPT,
                ctx.to_prompt_context()
            )
        } else {
            SYSTEM_PROMPT.to_string()
        };

        let messages = vec![AIMessage::system(&system_prompt), AIMessage::user(prompt)];

        let options = GenerateOptions {
            temperature: Some(0.3),
            max_tokens: Some(1500),
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

        // Build feature score
        let feature_score = raw.feature_score.map_or_else(
            || {
                // Infer from simple score if feature_score not provided
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

        // Determine affected agents based on topics and context
        let affected_agents = if raw.affected_agents.is_empty() {
            Self::infer_affected_agents(&raw.topics, &categories)
        } else {
            raw.affected_agents
        };

        // Convert installable assets
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
            affected_agents,
            installable_skill,
            installable_mcp_server,
        })
    }

    /// Infer which agents would be affected based on topics.
    fn infer_affected_agents(topics: &[String], categories: &[Category]) -> Vec<String> {
        let mut agents = Vec::new();

        let topics_lower: Vec<String> = topics.iter().map(|t| t.to_lowercase()).collect();
        let topic_str = topics_lower.join(" ");

        // Check for language/framework matches
        if topic_str.contains("rust") || topic_str.contains("axum") || topic_str.contains("tokio") {
            agents.push("Rex".to_string());
        }
        if topic_str.contains("go") || topic_str.contains("grpc") || topic_str.contains("chi") {
            agents.push("Grizz".to_string());
        }
        if topic_str.contains("react")
            || topic_str.contains("next")
            || topic_str.contains("typescript")
            || topic_str.contains("tailwind")
        {
            agents.push("Blaze".to_string());
        }
        if topic_str.contains("node") || topic_str.contains("bun") || topic_str.contains("elysia") {
            agents.push("Nova".to_string());
        }
        if topic_str.contains("expo") || topic_str.contains("react native") {
            agents.push("Tap".to_string());
        }
        if topic_str.contains("electron") || topic_str.contains("desktop") {
            agents.push("Spark".to_string());
        }
        if topic_str.contains("unity") || topic_str.contains("xr") || topic_str.contains("vr") {
            agents.push("Vex".to_string());
        }

        // Check categories for support agent relevance
        for cat in categories {
            match cat {
                Category::Research => agents.push("Tess".to_string()),
                Category::Security => agents.push("Cipher".to_string()),
                Category::DevOps | Category::Infrastructure => {
                    agents.push("Bolt".to_string());
                    agents.push("Atlas".to_string());
                }
                _ => {}
            }
        }

        // Code quality topics
        if topic_str.contains("code review")
            || topic_str.contains("lint")
            || topic_str.contains("quality")
        {
            agents.push("Cleo".to_string());
        }

        agents.sort();
        agents.dedup();
        agents
    }
}

const SYSTEM_PROMPT: &str = r#"You are a CTO evaluating technical content for IMPLEMENTATION POTENTIAL in a multi-agent software development platform.

Your task is to evaluate tweets and determine:
1. Could we actually BUILD or INTEGRATE this into our platform?
2. What SPECIFIC features or improvements could we implement?
3. Is this actionable, or just interesting reading?
4. Whether linked content likely contains implementation details worth scraping
5. Whether the content references an INSTALLABLE SKILL or MCP SERVER

## Scoring Dimensions

Provide multi-dimensional scoring:

1. **technical_fit** (0-10): How well does this fit our current tech stack (Rust, React, Kubernetes)?
2. **impact** (0-10): Potential improvement to platform capabilities
3. **effort** (1-10): Implementation effort (1=hours, 5=week, 10=months)
4. **urgency** (0-10): Market pressure, competitive advantage
5. **strategic_alignment** (0-10): Alignment with roadmap and vision

## Overall Score Guide

- 0.9+ = "We should build this" - clear implementation path, high value
- 0.7-0.9 = "Worth investigating" - potential value, needs more research  
- 0.5-0.7 = "Interesting but low priority" - tangentially useful
- <0.5 = "Not actionable" - news, opinions, or irrelevant tech

## Installable Asset Detection

Detect content that references installable assets:

### Installable Skills
A skill is a GitHub repository containing a SKILL.md file that provides capabilities to AI agents.
Look for:
- Links to GitHub repos that mention "skill", "agent skill", or "SKILL.md"
- Repos in known skill directories (e.g., .factory/skills/, .cursor/skills/)
- Content describing agent capabilities that can be installed

### Installable MCP Servers
An MCP (Model Context Protocol) server provides tools and resources to AI agents.
Look for:
- Links to GitHub repos that mention "MCP server", "mcp-server-", or "@modelcontextprotocol"
- Repos that implement the MCP protocol for tool integration
- Content describing MCP tools or server implementations

For each detected asset, provide:
- github_url: The full GitHub repository URL
- name: Human-readable name of the asset
- confidence: How confident you are this is an installable asset (0.0-1.0)

## Response Format

Always respond with valid JSON including:
- score (0.0-1.0)
- reasoning (string)
- categories (array of strings)
- topics (array of strings)
- should_enrich (boolean)
- implementation_ideas (array of strings)
- feature_score (object with technical_fit, impact, effort, urgency, strategic_alignment)
- affected_agents (array of agent names that would implement this: Rex, Blaze, Nova, Tess, etc.)
- installable_skill (object with github_url, name, confidence - or null if not detected)
- installable_mcp_server (object with github_url, name, confidence - or null if not detected)

Be specific about WHAT we could implement and HOW."#;

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
        assert!(overall < 1.0);
    }

    #[test]
    fn test_feature_score_priority_critical() {
        let score = FeatureScore {
            technical_fit: 8,
            impact: 9,
            effort: 3,
            urgency: 9,
            strategic_alignment: 8,
        };
        assert_eq!(score.priority(), Priority::Critical);
    }

    #[test]
    fn test_feature_score_priority_research() {
        let score = FeatureScore {
            technical_fit: 3,
            impact: 3,
            effort: 8,
            urgency: 2,
            strategic_alignment: 3,
        };
        assert_eq!(score.priority(), Priority::Research);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical < Priority::High);
        assert!(Priority::High < Priority::Medium);
        assert!(Priority::Medium < Priority::Low);
        assert!(Priority::Low < Priority::Research);
    }

    #[test]
    fn test_relevance_result_backward_compat() {
        let result = RelevanceResult {
            score: 0.85,
            reasoning: "Test".to_string(),
            categories: vec![],
            topics: vec![],
            should_enrich: true,
            implementation_ideas: vec![],
            feature_score: FeatureScore::default(),
            priority: Priority::default(),
            affected_agents: vec![],
            installable_skill: None,
            installable_mcp_server: None,
        };
        assert!(result.is_worth_investigating());
    }

    #[test]
    fn test_installable_asset() {
        let asset = InstallableAsset {
            github_url: "https://github.com/example/skill".to_string(),
            name: "example-skill".to_string(),
            confidence: 0.9,
        };
        assert!((asset.confidence - 0.9).abs() < f32::EPSILON);
    }
}
