//! Expand task prompt template.
//!
//! Breaks down a task into detailed subtasks.

use serde::Serialize;

use crate::entities::Task;

use super::PromptTemplate;

/// Context for expand-task prompt.
#[derive(Debug, Clone, Serialize)]
pub struct ExpandTaskContext {
    /// Number of subtasks to generate
    pub subtask_count: i32,
    /// The task to expand
    pub task: TaskSummary,
    /// Starting ID for new subtasks
    pub next_subtask_id: i32,
    /// Use research mode
    pub use_research: bool,
    /// Expansion prompt from complexity report
    pub expansion_prompt: Option<String>,
    /// Additional context
    pub additional_context: String,
    /// Complexity analysis reasoning
    pub complexity_reasoning_context: String,
    /// Project context
    pub gathered_context: String,
    /// Project root path
    pub project_root: String,
}

/// Simplified task representation for prompts.
#[derive(Debug, Clone, Serialize)]
pub struct TaskSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub details: String,
}

impl From<&Task> for TaskSummary {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id.clone(),
            title: task.title.clone(),
            description: task.description.clone(),
            details: task.details.clone(),
        }
    }
}

impl Default for ExpandTaskContext {
    fn default() -> Self {
        Self {
            subtask_count: 5,
            task: TaskSummary {
                id: String::new(),
                title: String::new(),
                description: String::new(),
                details: String::new(),
            },
            next_subtask_id: 1,
            use_research: false,
            expansion_prompt: None,
            additional_context: String::new(),
            complexity_reasoning_context: String::new(),
            gathered_context: String::new(),
            project_root: String::new(),
        }
    }
}

/// Get the expand-task template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new("expand-task", SYSTEM_PROMPT, USER_PROMPT)
        .with_description("Break down a task into detailed subtasks")
}

const SYSTEM_PROMPT: &str = r#"You are an AI assistant helping with task breakdown for software development. Break down high-level tasks into specific, actionable subtasks that can be implemented sequentially.{{#if use_research}}

You have access to current best practices and latest technical information to provide research-backed subtask generation.{{/if}}

IMPORTANT: Your response MUST be a JSON object with a "subtasks" property containing an array of subtask objects. Each subtask must include ALL of the following fields:
- id: MUST be sequential integers starting EXACTLY from {{next_subtask_id}}. First subtask id={{next_subtask_id}}, second id={{next_subtask_id}}+1, etc. DO NOT use any other numbering pattern!
- title: A clear, actionable title (5-200 characters)
- description: A detailed description (minimum 10 characters)
- dependencies: An array of subtask IDs this subtask depends on (can be empty [])
- details: Implementation details (minimum 20 characters)
- status: Must be "pending" for new subtasks
- testStrategy: Testing approach (can be null)

You may optionally include a "metadata" object. Do not include any other top-level properties."#;

const USER_PROMPT: &str = r"Break down this task into {{#if (gt subtask_count 0)}}exactly {{subtask_count}}{{else}}an appropriate number of{{/if}} specific subtasks:

Task ID: {{task.id}}
Title: {{task.title}}
Description: {{task.description}}
Current details: {{#if task.details}}{{task.details}}{{else}}None{{/if}}{{#if expansion_prompt}}

Expansion guidance: {{expansion_prompt}}{{/if}}{{#if additional_context}}

Additional context: {{additional_context}}{{/if}}{{#if complexity_reasoning_context}}

Complexity Analysis Reasoning: {{complexity_reasoning_context}}{{/if}}{{#if gathered_context}}

# Project Context

{{gathered_context}}{{/if}}

CRITICAL: You MUST use sequential IDs starting from {{next_subtask_id}}. The first subtask MUST have id={{next_subtask_id}}, the second MUST have id={{next_subtask_id}}+1, and so on. Do NOT use parent task ID in subtask numbering!";
