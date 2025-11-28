//! Add task prompt template.
//!
//! Generates a new task from a description.

use serde::Serialize;

use super::PromptTemplate;

/// Context for add-task prompt.
#[derive(Debug, Clone, Serialize)]
pub struct AddTaskContext {
    /// User's task description
    pub prompt: String,
    /// ID for the new task
    pub new_task_id: i32,
    /// List of existing tasks for context
    pub existing_tasks: serde_json::Value,
    /// Context from codebase analysis
    pub gathered_context: String,
    /// Additional context from args
    pub context_from_args: String,
    /// Task priority
    pub priority: String,
    /// Task dependencies
    pub dependencies: Vec<i32>,
    /// Use research mode
    pub use_research: bool,
    /// Project root path
    pub project_root: String,
}

impl Default for AddTaskContext {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            new_task_id: 1,
            existing_tasks: serde_json::json!([]),
            gathered_context: String::new(),
            context_from_args: String::new(),
            priority: "medium".to_string(),
            dependencies: Vec::new(),
            use_research: false,
            project_root: String::new(),
        }
    }
}

/// Get the add-task template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new(
        "add-task",
        SYSTEM_PROMPT,
        USER_PROMPT,
    ).with_description("Generate a new task based on description")
}

const SYSTEM_PROMPT: &str = r#"You are a helpful assistant that creates well-structured tasks for a software development project. Generate a single new task based on the user's description, adhering strictly to the provided JSON schema.

IMPORTANT: Your response MUST be a JSON object with the following structure (no wrapper property, just these fields directly):
{
  "title": "string",
  "description": "string",
  "details": "string",
  "testStrategy": "string",
  "dependencies": [array of numbers]
}

Do not include any other top-level properties. Do NOT wrap this in any additional object.

Pay special attention to dependencies between tasks, ensuring the new task correctly references any tasks it depends on.

When determining dependencies for a new task, follow these principles:
1. Select dependencies based on logical requirements - what must be completed before this task can begin.
2. Prioritize task dependencies that are semantically related to the functionality being built.
3. Consider both direct dependencies (immediately prerequisite) and indirect dependencies.
4. Avoid adding unnecessary dependencies - only include tasks that are genuinely prerequisite.
5. Consider the current status of tasks - prefer completed tasks as dependencies when possible.
6. Pay special attention to foundation tasks (1-5) but don't automatically include them without reason.
7. Recent tasks (higher ID numbers) may be more relevant for newer functionality.

The dependencies array should contain task IDs (numbers) of prerequisite tasks.{{#if use_research}}

Research current best practices and technologies relevant to this task.{{/if}}"#;

const USER_PROMPT: &str = r#"You are generating the details for Task #{{new_task_id}}. Based on the user's request: "{{prompt}}", create a comprehensive new task for a software development project.
      
{{gathered_context}}
      
{{#if use_research}}Research current best practices, technologies, and implementation patterns relevant to this task. {{/if}}Based on the information about existing tasks provided above, include appropriate dependencies in the "dependencies" array. Only include task IDs that this new task directly depends on.
      
Return your answer as a single JSON object matching the schema precisely:
      
{
  "title": "Task title goes here",
  "description": "A concise one or two sentence description of what the task involves",
  "details": "Detailed implementation steps, considerations, code examples, or technical approach",
  "testStrategy": "Specific steps to verify correct implementation and functionality",
  "dependencies": [1, 3] // Example: IDs of tasks that must be completed before this task
}
      
Make sure the details and test strategy are comprehensive and specific{{#if use_research}}, incorporating current best practices from your research{{/if}}. DO NOT include the task ID in the title.
{{#if context_from_args}}{{context_from_args}}{{/if}}"#;

