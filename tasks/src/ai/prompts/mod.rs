//! Prompt template system for AI operations.
//!
//! This module provides:
//! - Handlebars-based prompt templates
//! - Pre-defined templates for task operations
//! - Dynamic template rendering with context

use handlebars::Handlebars;
use serde::Serialize;
use std::collections::HashMap;

use crate::errors::{TasksError, TasksResult};

// Template modules
mod parse_prd;
mod expand_task;
mod analyze_complexity;
mod add_task;
mod update_task;
mod update_subtask;
mod update_tasks;

// Re-export context types (not the template() functions to avoid ambiguity)
pub use parse_prd::ParsePrdContext;
pub use expand_task::{ExpandTaskContext, TaskSummary};
pub use analyze_complexity::AnalyzeComplexityContext;
pub use add_task::AddTaskContext;
pub use update_task::UpdateTaskContext;
pub use update_subtask::UpdateSubtaskContext;
pub use update_tasks::UpdateTasksContext;

/// A prompt template with system and user messages.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// Template ID
    pub id: String,
    /// Template version
    pub version: String,
    /// Description
    pub description: String,
    /// System prompt template
    pub system: String,
    /// User prompt template
    pub user: String,
}

impl PromptTemplate {
    /// Create a new prompt template.
    pub fn new(
        id: impl Into<String>,
        system: impl Into<String>,
        user: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            version: "1.0.0".to_string(),
            description: String::new(),
            system: system.into(),
            user: user.into(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Render the template with the given context.
    pub fn render<T: Serialize>(&self, context: &T) -> TasksResult<(String, String)> {
        let mut handlebars = create_handlebars();
        
        handlebars
            .register_template_string("system", &self.system)
            .map_err(|e| TasksError::Ai(format!("Invalid system template: {e}")))?;
        
        handlebars
            .register_template_string("user", &self.user)
            .map_err(|e| TasksError::Ai(format!("Invalid user template: {e}")))?;

        let system = handlebars
            .render("system", context)
            .map_err(|e| TasksError::Ai(format!("Failed to render system prompt: {e}")))?;
        
        let user = handlebars
            .render("user", context)
            .map_err(|e| TasksError::Ai(format!("Failed to render user prompt: {e}")))?;

        Ok((system, user))
    }
}

/// Create a Handlebars instance with custom helpers.
fn create_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    
    // Disable HTML escaping for prompts
    handlebars.register_escape_fn(handlebars::no_escape);
    
    // Register custom helpers
    
    // Helper: {{#if (gt numTasks 0)}}
    handlebars.register_helper(
        "gt",
        Box::new(|h: &handlebars::Helper,
                  _: &Handlebars,
                  _: &handlebars::Context,
                  _: &mut handlebars::RenderContext,
                  out: &mut dyn handlebars::Output| {
            let a = h.param(0)
                .and_then(|v| v.value().as_i64())
                .unwrap_or(0);
            let b = h.param(1)
                .and_then(|v| v.value().as_i64())
                .unwrap_or(0);
            out.write(if a > b { "true" } else { "" })?;
            Ok(())
        }),
    );

    // Helper: {{#if (not value)}}
    handlebars.register_helper(
        "not",
        Box::new(|h: &handlebars::Helper,
                  _: &Handlebars,
                  _: &handlebars::Context,
                  _: &mut handlebars::RenderContext,
                  out: &mut dyn handlebars::Output| {
            let value = h.param(0)
                .map(|v| v.value().as_bool().unwrap_or(false))
                .unwrap_or(false);
            out.write(if !value { "true" } else { "" })?;
            Ok(())
        }),
    );

    // Helper: {{{json value}}}
    handlebars.register_helper(
        "json",
        Box::new(|h: &handlebars::Helper,
                  _: &Handlebars,
                  _: &handlebars::Context,
                  _: &mut handlebars::RenderContext,
                  out: &mut dyn handlebars::Output| {
            if let Some(param) = h.param(0) {
                let json = serde_json::to_string_pretty(param.value())
                    .unwrap_or_else(|_| "null".to_string());
                out.write(&json)?;
            }
            Ok(())
        }),
    );

    handlebars
}

/// Prompt manager for loading and rendering templates.
pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptManager {
    /// Create a new prompt manager with default templates.
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // Register default templates
        manager.register(parse_prd::template());
        manager.register(expand_task::template());
        manager.register(analyze_complexity::template());
        manager.register(add_task::template());
        manager.register(update_task::template());
        manager.register(update_subtask::template());
        manager.register(update_tasks::template());

        manager
    }

    /// Register a template.
    pub fn register(&mut self, template: PromptTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Get a template by ID.
    pub fn get(&self, id: &str) -> Option<&PromptTemplate> {
        self.templates.get(id)
    }

    /// Render a template with context.
    pub fn render<T: Serialize>(&self, id: &str, context: &T) -> TasksResult<(String, String)> {
        let template = self.get(id).ok_or_else(|| {
            TasksError::Ai(format!("Template '{}' not found", id))
        })?;
        template.render(context)
    }

    /// List all template IDs.
    pub fn template_ids(&self) -> Vec<&str> {
        self.templates.keys().map(String::as_str).collect()
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_template_rendering() {
        let template = PromptTemplate::new(
            "test",
            "You are a {{role}}",
            "{{#if useResearch}}Research: {{/if}}{{prompt}}",
        );

        let context = json!({
            "role": "helpful assistant",
            "useResearch": true,
            "prompt": "Hello world"
        });

        let (system, user) = template.render(&context).unwrap();
        
        assert_eq!(system, "You are a helpful assistant");
        assert_eq!(user, "Research: Hello world");
    }

    #[test]
    fn test_json_helper() {
        let template = PromptTemplate::new(
            "test",
            "System",
            "Tasks: {{{json tasks}}}",
        );

        let context = json!({
            "tasks": [
                {"id": 1, "title": "Task 1"},
                {"id": 2, "title": "Task 2"}
            ]
        });

        let (_, user) = template.render(&context).unwrap();
        
        assert!(user.contains("\"id\": 1"));
        assert!(user.contains("\"title\": \"Task 1\""));
    }

    #[test]
    fn test_prompt_manager() {
        let manager = PromptManager::new();
        
        // Should have default templates registered
        assert!(manager.get("parse-prd").is_some());
        assert!(manager.get("expand-task").is_some());
        assert!(manager.get("analyze-complexity").is_some());
    }
}

