//! Skill learner - extracts SOPs from successful task executions.

use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::{debug, info, warn};

use crate::models::{AgentType, Skill, TaskRecord, ToolStep};

use super::complexity::ComplexityFilter;

/// LLM client trait for SOP extraction.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate a description of when this skill should be used.
    async fn generate_use_when(&self, task: &TaskRecord, tool_sequence: &[ToolStep]) -> Result<String>;

    /// Generate an embedding vector for a skill.
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
}

/// Skill learner that converts successful tasks to reusable skills.
pub struct SkillLearner {
    /// Complexity filter for determining learnability.
    complexity_filter: ComplexityFilter,
}

impl SkillLearner {
    /// Create a new skill learner.
    #[must_use]
    pub fn new(complexity_filter: ComplexityFilter) -> Self {
        Self { complexity_filter }
    }

    /// Create with default complexity filter.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ComplexityFilter::default())
    }

    /// Check if a task should be learned.
    #[must_use]
    pub fn should_learn(&self, task: &TaskRecord) -> bool {
        self.complexity_filter.should_learn(task)
    }

    /// Extract a skill from a successful task.
    pub async fn extract_skill(
        &self,
        task: &TaskRecord,
        space_id: uuid::Uuid,
        llm_client: Option<&dyn LlmClient>,
    ) -> Result<Option<Skill>> {
        // Check if task is learnable
        if !self.should_learn(task) {
            debug!(task_id = %task.id, "Task not learnable, skipping");
            return Ok(None);
        }

        info!(
            task_id = %task.id,
            tool_calls = task.tool_calls.len(),
            "Extracting skill from task"
        );

        // Extract tool sequence as steps
        let tool_sops = self.extract_tool_steps(task);

        if tool_sops.is_empty() {
            warn!(task_id = %task.id, "No tool steps extracted, skipping");
            return Ok(None);
        }

        // Determine agent type
        let agent = self.determine_agent(task);

        // Generate use_when description
        let use_when = if let Some(client) = llm_client {
            client
                .generate_use_when(task, &tool_sops)
                .await
                .unwrap_or_else(|e| {
                    warn!(error = %e, "Failed to generate use_when, using fallback");
                    self.fallback_use_when(task)
                })
        } else {
            self.fallback_use_when(task)
        };

        // Create skill
        let mut skill = Skill::new(use_when, agent, tool_sops, space_id);

        // Add preferences from task
        for pref in &task.user_preferences {
            skill.add_preference(pref);
        }

        // Set complexity score
        let complexity = self.complexity_filter.calculate_complexity(task);
        skill = skill.with_complexity(complexity);

        // Generate embedding if LLM client available
        if let Some(client) = llm_client {
            let embed_text = format!("{} {}", skill.use_when, task.description);
            match client.generate_embedding(&embed_text).await {
                Ok(embedding) => {
                    skill = skill.with_embedding(embedding);
                }
                Err(e) => {
                    warn!(error = %e, "Failed to generate embedding");
                }
            }
        }

        info!(
            skill_id = %skill.id,
            use_when = %skill.use_when,
            steps = skill.tool_sops.len(),
            "Successfully extracted skill"
        );

        Ok(Some(skill))
    }

    /// Extract tool steps from task's tool calls.
    fn extract_tool_steps(&self, task: &TaskRecord) -> Vec<ToolStep> {
        let mut steps = Vec::new();
        let mut seen_patterns: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (idx, tool_call) in task.tool_calls.iter().enumerate() {
            // Create a pattern key to avoid exact duplicates
            let pattern_key = format!("{}:{}", tool_call.tool_name, tool_call.success);

            // Skip if we've seen this exact pattern recently (within 3 steps)
            if seen_patterns.contains(&pattern_key) && idx > 0 {
                let last_same = steps
                    .iter()
                    .rev()
                    .take(3)
                    .any(|s: &ToolStep| s.tool_name == tool_call.tool_name);
                if last_same {
                    continue;
                }
            }

            seen_patterns.insert(pattern_key);

            // Generate action description
            let action = self.generate_action_description(&tool_call.tool_name, &tool_call.arguments);

            let step = ToolStep::new(
                (steps.len() + 1) as u32,
                &tool_call.tool_name,
                action,
            );

            steps.push(step);
        }

        steps
    }

    /// Generate an action description for a tool call.
    fn generate_action_description(&self, tool_name: &str, _arguments: &str) -> String {
        // Map common tools to descriptions
        match tool_name {
            "read_file" => "Read file contents to understand existing code".to_string(),
            "write_file" => "Write or update file with implementation".to_string(),
            "search_files" => "Search codebase for relevant patterns".to_string(),
            "list_directory" => "List directory contents to understand structure".to_string(),
            "directory_tree" => "View directory tree for project overview".to_string(),
            "git_status" => "Check git status for changes".to_string(),
            "git_diff" => "View diff of changes".to_string(),
            "git_log" => "Review commit history".to_string(),
            "git_show" => "Examine specific commit details".to_string(),
            "brave_search" | "brave_web_search" => "Search web for documentation or examples".to_string(),
            _ => format!("Execute {tool_name}"),
        }
    }

    /// Determine the agent type from task metadata.
    fn determine_agent(&self, task: &TaskRecord) -> AgentType {
        if let Some(ref agent) = task.agent {
            agent.parse().unwrap_or(AgentType::Rex)
        } else {
            // Try to infer from tool usage
            let tools = task.unique_tools();
            if tools.iter().any(|t| t.contains("react") || t.contains("component")) {
                AgentType::Blaze
            } else {
                AgentType::Rex // Default to Rex
            }
        }
    }

    /// Generate a fallback use_when description without LLM.
    fn fallback_use_when(&self, task: &TaskRecord) -> String {
        let tools = task.unique_tools();
        let tool_summary = if tools.len() <= 3 {
            tools.join(", ")
        } else {
            format!("{} and {} more tools", tools[..3].join(", "), tools.len() - 3)
        };

        format!(
            "{} (using {})",
            task.description,
            tool_summary
        )
    }
}

impl Default for SkillLearner {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Mock LLM client for testing.
#[cfg(test)]
pub struct MockLlmClient;

#[cfg(test)]
#[async_trait]
impl LlmClient for MockLlmClient {
    async fn generate_use_when(&self, task: &TaskRecord, _tool_sequence: &[ToolStep]) -> Result<String> {
        Ok(format!("Implementing: {}", task.description))
    }

    async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1; 1536])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ToolCallRecord;
    use uuid::Uuid;

    fn create_learnable_task() -> TaskRecord {
        let mut task = TaskRecord::new("1", "Implement HTTP handler");
        task.agent = Some("rex".to_string());
        task.start();

        // Add tool calls
        task.add_tool_call(ToolCallRecord::new("1", "read_file", r#"{"path": "src/main.rs"}"#));
        task.add_tool_call(ToolCallRecord::new("2", "search_files", r#"{"query": "handler"}"#));
        task.add_tool_call(ToolCallRecord::new("3", "write_file", r#"{"path": "src/handler.rs"}"#));
        task.add_tool_call(ToolCallRecord::new("4", "git_status", "{}"));
        task.add_tool_call(ToolCallRecord::new("5", "read_file", r#"{"path": "Cargo.toml"}"#));

        task.add_preference("Use async/await");
        task.add_progress("Implemented the handler");

        task.complete(true);
        task
    }

    #[test]
    fn test_learner_creation() {
        let learner = SkillLearner::with_defaults();
        assert!(std::mem::size_of_val(&learner) > 0);
    }

    #[tokio::test]
    async fn test_extract_skill_without_llm() {
        let learner = SkillLearner::new(ComplexityFilter {
            min_duration_secs: 0,
            complexity_threshold: 0.2,
            ..Default::default()
        });

        let task = create_learnable_task();
        let space_id = Uuid::new_v4();

        let skill = learner.extract_skill(&task, space_id, None).await.unwrap();

        assert!(skill.is_some());
        let skill = skill.unwrap();
        assert_eq!(skill.agent, AgentType::Rex);
        assert!(!skill.tool_sops.is_empty());
        assert!(!skill.preferences.is_empty());
    }

    #[tokio::test]
    async fn test_extract_skill_with_mock_llm() {
        let learner = SkillLearner::new(ComplexityFilter {
            min_duration_secs: 0,
            complexity_threshold: 0.2,
            ..Default::default()
        });

        let task = create_learnable_task();
        let space_id = Uuid::new_v4();
        let mock_client = MockLlmClient;

        let skill = learner
            .extract_skill(&task, space_id, Some(&mock_client))
            .await
            .unwrap();

        assert!(skill.is_some());
        let skill = skill.unwrap();
        assert!(skill.use_when.contains("Implementing"));
        assert!(skill.embedding.is_some());
    }

    #[test]
    fn test_tool_step_extraction() {
        let learner = SkillLearner::with_defaults();
        let task = create_learnable_task();

        let steps = learner.extract_tool_steps(&task);

        // Should have deduplicated steps
        assert!(!steps.is_empty());
        assert!(steps.iter().any(|s| s.tool_name == "read_file"));
        assert!(steps.iter().any(|s| s.tool_name == "write_file"));
    }

    #[test]
    fn test_action_descriptions() {
        let learner = SkillLearner::with_defaults();

        let desc = learner.generate_action_description("read_file", "{}");
        assert!(desc.contains("Read file"));

        let desc = learner.generate_action_description("unknown_tool", "{}");
        assert!(desc.contains("unknown_tool"));
    }
}
