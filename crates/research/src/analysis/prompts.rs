//! Prompt template management.

use anyhow::Result;
use handlebars::Handlebars;
use serde::Serialize;
use std::path::Path;

/// Manages Handlebars prompt templates.
pub struct PromptManager {
    handlebars: Handlebars<'static>,
}

impl PromptManager {
    /// Create a new prompt manager with embedded templates.
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register embedded templates
        handlebars.register_template_string("relevance", RELEVANCE_TEMPLATE)?;
        handlebars.register_template_string("categorize", CATEGORIZE_TEMPLATE)?;
        handlebars.register_template_string("summarize", SUMMARIZE_TEMPLATE)?;

        Ok(Self { handlebars })
    }

    /// Create a prompt manager loading templates from a directory.
    pub fn from_dir(dir: &Path) -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Load templates from directory
        let templates = ["relevance", "categorize", "summarize"];
        for name in templates {
            let path = dir.join(format!("{name}.hbs"));
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                handlebars.register_template_string(name, &content)?;
            }
        }

        Ok(Self { handlebars })
    }

    /// Render a template with the given data.
    pub fn render<T: Serialize>(&self, template: &str, data: &T) -> Result<String> {
        let result = self.handlebars.render(template, data)?;
        Ok(result)
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default PromptManager")
    }
}

/// Relevance analysis prompt template.
const RELEVANCE_TEMPLATE: &str = r#"Analyze this tweet for IMPLEMENTATION POTENTIAL in our multi-agent software development platform.

## Platform Context
The CTO platform orchestrates AI coding agents (Rex, Blaze, Tess, etc.) for automated
software development, using Kubernetes, Argo Workflows, MCP tools, and Rust.

We're looking for content that has **potential to be implemented** in our platform - not just
interesting reads, but actionable ideas, patterns, tools, or techniques we could actually build.

## High-Value Implementation Areas
- New MCP tools or integrations we could add
- Agent orchestration patterns we could adopt
- Rust crates or libraries we could incorporate
- Kubernetes operators or controllers we could build
- Workflow automation improvements
- Developer tooling we could integrate
- AI/LLM techniques that would improve our agents

## Lower Priority (Still Track)
- General industry news or announcements
- High-level thought leadership without specifics
- Tools in languages/stacks we don't use
- Already well-known patterns we've implemented

## Tweet to Analyze
Author: @{{author}}
Content: {{tweet_text}}
{{#if urls}}
Links:
{{#each urls}}
- {{this}}
{{/each}}
{{/if}}

## Task
Evaluate this for implementation potential. Respond with JSON:
{
  "score": <0.0-1.0 where 0.9+ = "we should build this", 0.7-0.9 = "worth investigating", 0.5-0.7 = "interesting but low priority", <0.5 = "not actionable">,
  "reasoning": "<explain WHAT could be implemented and HOW it would benefit the platform>",
  "categories": ["<matching categories>"],
  "topics": ["<key topics/technologies>"],
  "should_enrich": <true if links likely contain implementation details worth scraping>,
  "implementation_ideas": ["<specific things we could build or integrate>"]
}
"#;

/// Topic categorization prompt template.
const CATEGORIZE_TEMPLATE: &str = r"Categorize this tweet content into one or more categories.

Categories:
- agents: AI/LLM agent patterns, autonomous systems
- rust: Rust language, crates, tooling
- infrastructure: Kubernetes, cloud, DevOps
- tooling: Developer tools, MCP, IDEs
- architecture: Software design patterns
- devops: CI/CD, automation, deployment
- security: Security practices, vulnerabilities
- research: Academic papers, studies
- announcements: Product launches, releases
- other: Doesn't fit other categories

Tweet: {{text}}

Respond with JSON array of matching categories.
";

/// Content summary prompt template.
const SUMMARIZE_TEMPLATE: &str = r"Summarize this research item for a technical audience.

## Original Tweet
{{tweet_text}}

{{#if enriched_content}}
## Linked Content
{{#each enriched_content}}
### {{this.title}}
{{this.excerpt}}
{{/each}}
{{/if}}

Write a 2-3 sentence summary highlighting key technical insights.
";
