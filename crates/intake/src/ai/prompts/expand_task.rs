//! Expand task prompt template.
//!
//! Breaks down a task into detailed subtasks with subagent-aware metadata
//! for parallel execution.

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
    /// Enable subagent-aware expansion for parallel execution
    #[serde(default)]
    pub enable_subagents: bool,
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
            enable_subagents: false,
        }
    }
}

/// Get the expand-task template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new("expand-task", SYSTEM_PROMPT, USER_PROMPT)
        .with_description("Break down a task into detailed subtasks")
}

const SYSTEM_PROMPT: &str = r#"You are an AI assistant helping with task breakdown for software development. Break down high-level tasks into specific, actionable subtasks that can be implemented{{#if enable_subagents}} in parallel by specialized subagents{{else}} sequentially{{/if}}.{{#if use_research}}

You have access to current best practices and latest technical information to provide research-backed subtask generation.{{/if}}

IMPORTANT: Your response MUST be a JSON object with a "subtasks" property containing an array of subtask objects. Each subtask must include ALL of the following fields:
- id: MUST be sequential integers starting EXACTLY from {{next_subtask_id}}. First subtask id={{next_subtask_id}}, second id={{next_subtask_id}}+1, etc. DO NOT use any other numbering pattern!
- title: A clear, actionable title (5-200 characters)
- description: A detailed description (minimum 10 characters)
- dependencies: An array of subtask IDs this subtask depends on (can be empty [])
- details: Implementation details (minimum 20 characters)
- status: Must be "pending" for new subtasks
- testStrategy: Testing approach (can be null){{#if enable_subagents}}
- subagentType: The type of specialized subagent to handle this subtask. MUST be one of:
  - "implementer": Write/implement code (default for most coding subtasks)
  - "reviewer": Review code quality, patterns, and best practices
  - "tester": Write and run tests
  - "documenter": Write documentation
  - "researcher": Research and exploration tasks
  - "debugger": Debug issues and fix bugs
- parallelizable: Boolean indicating if this subtask can run in parallel with others at the same dependency level (true for independent work, false for coordination-required tasks){{/if}}

You may optionally include a "metadata" object. Do not include any other top-level properties.

CRITICAL OUTPUT FORMAT:
- Return ONLY the JSON object. No markdown formatting, no explanatory text before or after.
- Do NOT explain your reasoning. Do NOT summarize the subtasks. Do NOT include any text outside the JSON.
- Start your response with { and end with }{{#if enable_subagents}}

## Subagent Optimization Guidelines

When breaking down tasks for subagent execution:
1. **Maximize parallelism**: Group independent work units that can run simultaneously
2. **Minimize dependencies**: Only add dependencies when strictly necessary
3. **Match subagent types to work**: Use implementer for coding, tester for tests, etc.
4. **Consider context isolation**: Each subagent works in isolation, so subtasks should be self-contained
5. **Plan review phases**: Include reviewer subtasks after implementation phases{{/if}}"#;

const USER_PROMPT: &str = r"Break down this task into {{#if (gt subtask_count 0)}}exactly {{subtask_count}}{{else}}an appropriate number of{{/if}} specific subtasks{{#if enable_subagents}} optimized for parallel subagent execution{{/if}}:

Task ID: {{task.id}}
Title: {{task.title}}
Description: {{task.description}}
Current details: {{#if task.details}}{{task.details}}{{else}}None{{/if}}{{#if expansion_prompt}}

Expansion guidance: {{expansion_prompt}}{{/if}}{{#if additional_context}}

Additional context: {{additional_context}}{{/if}}{{#if complexity_reasoning_context}}

Complexity Analysis Reasoning: {{complexity_reasoning_context}}{{/if}}{{#if gathered_context}}

# Project Context

{{gathered_context}}{{/if}}

CRITICAL: You MUST use sequential IDs starting from {{next_subtask_id}}. The first subtask MUST have id={{next_subtask_id}}, the second MUST have id={{next_subtask_id}}+1, and so on. Do NOT use parent task ID in subtask numbering!{{#if enable_subagents}}

SUBAGENT REQUIREMENTS:
- Include subagentType for EVERY subtask (implementer, reviewer, tester, documenter, researcher, or debugger)
- Set parallelizable=true for subtasks that can run concurrently with others at the same dependency level
- Minimize dependencies to maximize parallel execution potential
- Group related implementation work so multiple implementer subagents can work simultaneously
- Include at least one reviewer subtask after implementation subtasks
- Include tester subtasks for validation work{{/if}}

OUTPUT: Return ONLY valid JSON. No explanations, no summaries, no markdown. Start with { and end with }.";
