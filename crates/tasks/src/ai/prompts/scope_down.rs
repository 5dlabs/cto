//! Scope down prompt template.
//!
//! Decreases task complexity by splitting complex tasks into simpler ones.

use serde::Serialize;

use super::PromptTemplate;

/// Context for scope-down prompt.
#[derive(Debug, Clone, Serialize)]
pub struct ScopeDownContext {
    /// Tasks to scope down
    pub tasks: serde_json::Value,
    /// Scoping strength: "light", "regular", or "heavy"
    pub strength: String,
    /// Custom prompt for specific adjustments
    pub prompt: Option<String>,
    /// Complexity threshold
    pub threshold: i32,
    /// Use research mode
    pub use_research: bool,
    /// Project context
    pub gathered_context: String,
    /// Project root path
    pub project_root: String,
}

impl Default for ScopeDownContext {
    fn default() -> Self {
        Self {
            tasks: serde_json::json!([]),
            strength: "regular".to_string(),
            prompt: None,
            threshold: 5,
            use_research: false,
            gathered_context: String::new(),
            project_root: String::new(),
        }
    }
}

/// Get the scope-down template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new("scope-down", SYSTEM_PROMPT, USER_PROMPT)
        .with_description("Decrease task complexity by splitting into simpler tasks")
}

const SYSTEM_PROMPT: &str = r#"You are an AI assistant specializing in task decomposition and scope management for software development projects.

Your job is to REDUCE the complexity of tasks by:
1. Splitting overly complex tasks into multiple simpler, more focused tasks
2. Removing unnecessary scope or features that can be deferred
3. Breaking down monolithic requirements into incremental deliverables
4. Simplifying acceptance criteria while maintaining core functionality

Scoping strength levels:
- **light**: Minor adjustments - split only the most complex parts, keep most scope intact
- **regular**: Balanced approach - split complex tasks, defer nice-to-have features
- **heavy**: Aggressive reduction - maximize simplicity, defer all non-essential features, break into smallest possible units{{#if use_research}}

You have access to current best practices and latest technical information to provide research-backed recommendations.{{/if}}

IMPORTANT: Your response MUST be a JSON object with a "tasks" property containing an array of task objects.
Each task must include:
- id: Integer task ID (keep original IDs where possible, use new sequential IDs for split tasks)
- title: Clear, actionable title
- description: Concise description
- details: Implementation details
- testStrategy: Testing approach (can be null)
- priority: "low", "medium", "high", or "critical"
- dependencies: Array of task IDs this task depends on
- status: Keep original status or "pending" for new tasks

For split tasks:
- Original task ID should be preserved for the primary/core portion
- New tasks from splits should use IDs starting from the highest existing ID + 1
- Update dependencies to reflect the new task structure
- Mark deferred scope clearly in the description

You may optionally include a "metadata" object with:
- originalTaskIds: Array of task IDs that were modified
- splitDetails: Description of what was split and why
- deferredScope: Features or scope that was removed/deferred"#;

const USER_PROMPT: &str = r#"Reduce the complexity of the following tasks using **{{strength}}** scoping strength.

## Tasks to Scope Down

{{{json tasks}}}

## Complexity Threshold

Tasks with complexity score >= {{threshold}} should be prioritized for scope reduction.{{#if prompt}}

## Custom Guidance

{{prompt}}{{/if}}{{#if gathered_context}}

## Project Context

{{gathered_context}}{{/if}}

## Instructions

1. Analyze each task's complexity and scope
2. Apply {{strength}} scoping strategy:
{{#if (eq strength "light")}}   - Make minor adjustments only
   - Split only tasks with complexity > 8
   - Keep most features intact{{/if}}{{#if (eq strength "regular")}}   - Split tasks with complexity >= {{threshold}}
   - Defer nice-to-have features
   - Focus on core functionality first{{/if}}{{#if (eq strength "heavy")}}   - Aggressively simplify all complex tasks
   - Break into smallest viable units
   - Defer all non-essential features
   - Prioritize MVP approach{{/if}}
3. Return the modified task list with any new tasks from splits
4. Ensure all dependencies are correctly updated

Return the scoped-down tasks as JSON."#;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_scope_down_template_renders() {
        let template = template();
        let context = ScopeDownContext {
            tasks: json!([{
                "id": 1,
                "title": "Complex authentication system",
                "description": "Implement full OAuth2 with SSO, MFA, and social login",
                "complexity": 9
            }]),
            strength: "regular".to_string(),
            prompt: Some("Focus on email/password auth first".to_string()),
            threshold: 5,
            use_research: false,
            gathered_context: String::new(),
            project_root: String::new(),
        };

        let (system, user) = template.render(&context).unwrap();

        assert!(system.contains("REDUCE the complexity"));
        assert!(user.contains("regular"));
        assert!(user.contains("Complex authentication"));
        assert!(user.contains("Focus on email/password auth first"));
    }

    #[test]
    fn test_scope_down_heavy_strength() {
        let template = template();
        let context = ScopeDownContext {
            strength: "heavy".to_string(),
            ..Default::default()
        };

        let (_, user) = template.render(&context).unwrap();
        assert!(user.contains("Aggressively simplify"));
    }
}
