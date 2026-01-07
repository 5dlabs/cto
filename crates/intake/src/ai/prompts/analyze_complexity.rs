//! Analyze complexity prompt template.
//!
//! Analyzes task complexity and generates expansion recommendations.

use serde::Serialize;

use super::PromptTemplate;

/// Context for analyze-complexity prompt.
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeComplexityContext {
    /// Tasks to analyze (as JSON)
    pub tasks: serde_json::Value,
    /// Additional project context
    pub gathered_context: String,
    /// Complexity threshold for expansion recommendation
    pub threshold: i32,
    /// Use research mode
    pub use_research: bool,
    /// Project root path
    pub project_root: String,
}

impl Default for AnalyzeComplexityContext {
    fn default() -> Self {
        Self {
            tasks: serde_json::json!([]),
            gathered_context: String::new(),
            threshold: 5,
            use_research: false,
            project_root: String::new(),
        }
    }
}

/// Get the analyze-complexity template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new("analyze-complexity", SYSTEM_PROMPT, USER_PROMPT)
        .with_description("Analyze task complexity and generate expansion recommendations")
}

const SYSTEM_PROMPT: &str = r#"You are an expert software architect and project manager analyzing task complexity. Your analysis should consider implementation effort, technical challenges, dependencies, and testing requirements.

IMPORTANT: For each task, provide an analysis object with ALL of the following fields:
- taskId: The ID of the task being analyzed (positive integer)
- taskTitle: The title of the task
- complexityScore: A score from 1-10 indicating complexity
- recommendedSubtasks: Number of subtasks recommended (non-negative integer; 0 if no expansion needed)
- expansionPrompt: A prompt to guide subtask generation
- reasoning: Your reasoning for the complexity score

Your response MUST be a JSON object with a single "complexityAnalysis" property containing an array of these analysis objects. You may optionally include a "metadata" object, but no other top-level properties."#;

const USER_PROMPT: &str = r"Analyze the following tasks to determine their complexity (1-10 scale) and recommend the number of subtasks for expansion. Provide a brief reasoning and an initial expansion prompt for each.{{#if use_research}} Consider current best practices, common implementation patterns, and industry standards in your analysis.{{/if}}

Tasks:
{{{json tasks}}}
{{#if gathered_context}}

# Project Context

{{gathered_context}}
{{/if}}";
