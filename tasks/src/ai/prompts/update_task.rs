//! Update task prompt template.
//!
//! Updates a single task with new information.

use serde::Serialize;

use super::PromptTemplate;

/// Context for update-task prompt.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateTaskContext {
    /// The task to update (as JSON)
    pub task: serde_json::Value,
    /// JSON string of the task
    pub task_json: String,
    /// Description of changes to apply
    pub update_prompt: String,
    /// Whether to append to details or do full update
    pub append_mode: bool,
    /// Use research mode
    pub use_research: bool,
    /// Current task details for context
    pub current_details: String,
    /// Additional project context
    pub gathered_context: String,
    /// Project root path
    pub project_root: String,
}

impl Default for UpdateTaskContext {
    fn default() -> Self {
        Self {
            task: serde_json::json!({}),
            task_json: String::new(),
            update_prompt: String::new(),
            append_mode: false,
            use_research: false,
            current_details: "(No existing details)".to_string(),
            gathered_context: String::new(),
            project_root: String::new(),
        }
    }
}

/// Get the update-task template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new(
        "update-task",
        SYSTEM_PROMPT,
        USER_PROMPT,
    ).with_description("Update a single task with new information")
}

const SYSTEM_PROMPT: &str = r#"You are an AI assistant helping to update a software development task based on new context.{{#if use_research}} You have access to current best practices and latest technical information to provide research-backed updates.{{/if}}
You will be given a task and a prompt describing changes or new implementation details.
Your job is to update the task to reflect these changes, while preserving its basic structure.

Guidelines:
1. VERY IMPORTANT: NEVER change the title of the task - keep it exactly as is
2. Maintain the same ID, status, and dependencies unless specifically mentioned in the prompt{{#if use_research}}
3. Research and update the description, details, and test strategy with current best practices
4. Include specific versions, libraries, and approaches that are current and well-tested{{/if}}{{#if (not use_research)}}
3. Update the description, details, and test strategy to reflect the new information
4. Do not change anything unnecessarily - just adapt what needs to change based on the prompt{{/if}}
5. Return the complete updated task
6. VERY IMPORTANT: Preserve all subtasks marked as "done" or "completed" - do not modify their content
7. For tasks with completed subtasks, build upon what has already been done rather than rewriting everything
8. If an existing completed subtask needs to be changed/undone based on the new context, DO NOT modify it directly
9. Instead, add a new subtask that clearly indicates what needs to be changed or replaced
10. Use the existence of completed subtasks as an opportunity to make new subtasks more specific and targeted
11. Ensure any new subtasks have unique IDs that don't conflict with existing ones
12. CRITICAL: For subtask IDs, use ONLY numeric values (1, 2, 3, etc.) NOT strings ("1", "2", "3")
13. CRITICAL: Subtask IDs should start from 1 and increment sequentially (1, 2, 3...) - do NOT use parent task ID as prefix{{#if use_research}}
14. Include links to documentation or resources where helpful
15. Focus on practical, implementable solutions using current technologies{{/if}}

The changes described in the prompt should be thoughtfully applied to make the task more accurate and actionable."#;

const USER_PROMPT: &str = r#"Here is the task to update{{#if use_research}} with research-backed information{{/if}}:
{{{task_json}}}

Please {{#if use_research}}research and {{/if}}update this task based on the following {{#if use_research}}context:
{{update_prompt}}

Incorporate current best practices, latest stable versions, and proven approaches.{{/if}}{{#if (not use_research)}}new context:
{{update_prompt}}{{/if}}

IMPORTANT: {{#if use_research}}Preserve any subtasks marked as "done" or "completed".{{/if}}{{#if (not use_research)}}In the task JSON above, any subtasks with "status": "done" or "status": "completed" should be preserved exactly as is. Build your changes around these completed items.{{/if}}
{{#if gathered_context}}

# Project Context

{{gathered_context}}
{{/if}}

Return the complete updated task{{#if use_research}} with research-backed improvements{{/if}}.

IMPORTANT: Your response must be a JSON object with a single property named "task" containing the updated task object."#;

