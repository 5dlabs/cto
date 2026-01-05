//! Scope up prompt template.
//!
//! Increases task complexity by consolidating or expanding tasks.

use serde::Serialize;

use super::PromptTemplate;

/// Context for scope-up prompt.
#[derive(Debug, Clone, Serialize)]
pub struct ScopeUpContext {
    /// Tasks to scope up
    pub tasks: serde_json::Value,
    /// Scoping strength: "light", "regular", or "heavy"
    pub strength: String,
    /// Custom prompt for specific adjustments
    pub prompt: Option<String>,
    /// Complexity threshold (tasks below this may be candidates for merging)
    pub threshold: i32,
    /// Use research mode
    pub use_research: bool,
    /// Project context
    pub gathered_context: String,
    /// Project root path
    pub project_root: String,
}

impl Default for ScopeUpContext {
    fn default() -> Self {
        Self {
            tasks: serde_json::json!([]),
            strength: "regular".to_string(),
            prompt: None,
            threshold: 3,
            use_research: false,
            gathered_context: String::new(),
            project_root: String::new(),
        }
    }
}

/// Get the scope-up template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new("scope-up", SYSTEM_PROMPT, USER_PROMPT)
        .with_description("Increase task complexity by consolidating or expanding tasks")
}

const SYSTEM_PROMPT: &str = r#"You are an AI assistant specializing in task consolidation and scope management for software development projects.

Your job is to INCREASE the complexity/scope of tasks by:
1. Merging overly granular tasks into cohesive, larger tasks
2. Adding related functionality that logically belongs together
3. Consolidating scattered implementation details into unified tasks
4. Expanding minimal tasks to include proper error handling, edge cases, and production readiness

Scoping strength levels:
- **light**: Minor consolidation - merge only trivially small tasks that clearly belong together
- **regular**: Balanced approach - merge related tasks, add necessary production features
- **heavy**: Aggressive consolidation - maximize cohesion, merge all related functionality, add comprehensive features{{#if use_research}}

You have access to current best practices and latest technical information to provide research-backed recommendations.{{/if}}

IMPORTANT: Your response MUST be a JSON object with a "tasks" property containing an array of task objects.
Each task must include:
- id: Integer task ID (use the lowest ID from merged tasks, or generate new sequential IDs)
- title: Clear, actionable title reflecting the expanded scope
- description: Comprehensive description
- details: Detailed implementation instructions
- testStrategy: Comprehensive testing approach
- priority: "low", "medium", "high", or "critical"
- dependencies: Array of task IDs this task depends on
- status: Keep original status or "pending" for merged tasks

For merged tasks:
- Use the lowest ID from the merged set
- Combine descriptions and details thoughtfully
- Update all dependencies that referenced merged tasks
- Mark as "pending" if any source task was pending

You may optionally include a "metadata" object with:
- mergedTaskIds: Array of task IDs that were merged into this task
- addedScope: Description of any scope that was added
- consolidationReason: Why tasks were merged"#;

const USER_PROMPT: &str = r#"Increase the scope/consolidate the following tasks using **{{strength}}** scoping strength.

## Tasks to Scope Up

{{{json tasks}}}

## Consolidation Threshold

Tasks with complexity score <= {{threshold}} are candidates for merging with related tasks.{{#if prompt}}

## Custom Guidance

{{prompt}}{{/if}}{{#if gathered_context}}

## Project Context

{{gathered_context}}{{/if}}

## Instructions

1. Analyze tasks for consolidation opportunities:
   - Tasks that are logically related and should be implemented together
   - Tasks that are too granular to be useful standalone
   - Tasks missing production-ready features
2. Apply {{strength}} scoping strategy:
{{#if (eq strength "light")}}   - Merge only trivially small tasks (complexity <= 2)
   - Keep most tasks separate
   - Add only essential missing features{{/if}}{{#if (eq strength "regular")}}   - Merge related tasks with complexity <= {{threshold}}
   - Add proper error handling and logging
   - Include reasonable edge case handling{{/if}}{{#if (eq strength "heavy")}}   - Aggressively consolidate related functionality
   - Merge all tasks that share common concerns
   - Add comprehensive production features
   - Include monitoring, observability, and resilience{{/if}}
3. Return the consolidated task list
4. Ensure all dependencies are correctly updated

Return the scoped-up tasks as JSON."#;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_scope_up_template_renders() {
        let template = template();
        let context = ScopeUpContext {
            tasks: json!([
                {"id": 1, "title": "Add user model", "complexity": 2},
                {"id": 2, "title": "Add user validation", "complexity": 2},
                {"id": 3, "title": "Add user serialization", "complexity": 1}
            ]),
            strength: "regular".to_string(),
            prompt: Some("Consolidate user-related tasks".to_string()),
            threshold: 3,
            use_research: false,
            gathered_context: String::new(),
            project_root: String::new(),
        };

        let (system, user) = template.render(&context).unwrap();

        assert!(system.contains("INCREASE the complexity"));
        assert!(user.contains("regular"));
        assert!(user.contains("Add user model"));
        assert!(user.contains("Consolidate user-related tasks"));
    }

    #[test]
    fn test_scope_up_heavy_strength() {
        let template = template();
        let context = ScopeUpContext {
            strength: "heavy".to_string(),
            ..Default::default()
        };

        let (_, user) = template.render(&context).unwrap();
        assert!(user.contains("Aggressively consolidate"));
    }
}
