//! Update tasks prompt template.
//!
//! Batch updates multiple tasks starting from a specific ID.

use serde::Serialize;

use super::PromptTemplate;

/// Context for update-tasks prompt.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateTasksContext {
    /// Tasks to update (as JSON)
    pub tasks: serde_json::Value,
    /// Starting task ID for updates
    pub from_id: i32,
    /// Description of changes to apply
    pub update_prompt: String,
    /// Use research mode
    pub use_research: bool,
    /// Additional project context
    pub gathered_context: String,
    /// Project root path
    pub project_root: String,
}

impl Default for UpdateTasksContext {
    fn default() -> Self {
        Self {
            tasks: serde_json::json!([]),
            from_id: 1,
            update_prompt: String::new(),
            use_research: false,
            gathered_context: String::new(),
            project_root: String::new(),
        }
    }
}

/// Get the update-tasks template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new("update-tasks", SYSTEM_PROMPT, USER_PROMPT)
        .with_description("Batch update multiple tasks based on new context")
}

const SYSTEM_PROMPT: &str = r#"You are an AI assistant helping to update multiple software development tasks based on new context or implementation changes.{{#if use_research}} You have access to current best practices and latest technical information to provide research-backed updates.{{/if}}

You will be given a list of tasks and a prompt describing changes or new implementation details that affect these tasks.
Your job is to update each task to reflect these changes, while preserving its basic structure.

Guidelines:
1. VERY IMPORTANT: NEVER change the title of any task - keep titles exactly as they are
2. Maintain the same ID, status for each task
3. Update dependencies only if the changes require it
4. Update the description, details, and test strategy to reflect the new information
5. Return ALL tasks in the response, both modified and unmodified ones
6. Tasks that are already marked as "done" should generally not be modified unless absolutely necessary
7. For tasks that don't need changes based on the prompt, return them unchanged
8. CRITICAL: For subtask IDs, use ONLY numeric values (1, 2, 3, etc.) NOT strings
9. CRITICAL: Subtask IDs should start from 1 and increment sequentially{{#if use_research}}
10. Include specific versions, libraries, and approaches based on current best practices
11. Consider how the changes affect downstream tasks{{/if}}

The changes described in the prompt should be thoughtfully applied across all affected tasks."#;

const USER_PROMPT: &str = r#"Here are the tasks to update{{#if use_research}} with research-backed information{{/if}}:
{{{json tasks}}}

Please {{#if use_research}}research and {{/if}}update these tasks starting from task ID {{from_id}} based on the following {{#if use_research}}context:
{{update_prompt}}

Incorporate current best practices, latest stable versions, and proven approaches.{{/if}}{{#if (not use_research)}}new context:
{{update_prompt}}{{/if}}
{{#if gathered_context}}

# Project Context

{{gathered_context}}
{{/if}}

Return ALL tasks (modified and unmodified) as a JSON object with a "tasks" property containing an array of task objects."#;
