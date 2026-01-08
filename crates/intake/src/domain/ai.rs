//! AI Domain - High-level AI operations for task management.
//!
//! This module provides AI-powered operations:
//! - Parse PRD into tasks
//! - Expand tasks into subtasks
//! - Analyze task complexity
//! - Update tasks with AI assistance

use std::sync::Arc;

use crate::ai::{
    parse_ai_response,
    prompts::{AnalyzeComplexityContext, ExpandTaskContext, ParsePrdContext, TaskSummary},
    schemas::{
        AnalyzeComplexityResponse, ComplexityReport, ExpandTaskResponse, GeneratedDecisionPoint,
        GeneratedSubtask, GeneratedTask, ParsePrdResponse,
    },
    AIMessage, AIProvider, GenerateOptions, PromptManager, ProviderRegistry, TokenUsage,
};
use crate::entities::{DecisionPoint, Subtask, Task, TaskPriority, TaskStatus};
use crate::errors::{TasksError, TasksResult};
use crate::storage::Storage;

/// AI Domain for AI-powered task operations.
pub struct AIDomain {
    storage: Arc<dyn Storage>,
    registry: ProviderRegistry,
    prompts: PromptManager,
}

impl AIDomain {
    /// Create a new AI domain.
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            storage,
            registry: ProviderRegistry::default(),
            prompts: PromptManager::default(),
        }
    }

    /// Create with a custom provider registry.
    pub fn with_registry(storage: Arc<dyn Storage>, registry: ProviderRegistry) -> Self {
        Self {
            storage,
            registry,
            prompts: PromptManager::default(),
        }
    }

    /// Get a configured provider or return an error.
    fn get_provider(&self, model: Option<&str>) -> TasksResult<Arc<dyn AIProvider>> {
        if let Some(model) = model {
            self.registry
                .get_for_model(model)
                .ok_or_else(|| TasksError::ModelNotSupported {
                    model: model.to_string(),
                })
        } else {
            self.registry.require_any()
        }
    }

    /// Get the default model for a provider.
    fn get_default_model(provider: &dyn AIProvider) -> &str {
        if provider.name() == "anthropic" {
            "claude-sonnet-4-5-20250514"
        } else {
            "gpt-4o"
        }
    }

    /// Parse a PRD file and generate tasks.
    pub async fn parse_prd(
        &self,
        prd_content: &str,
        prd_path: &str,
        num_tasks: Option<i32>,
        research: bool,
        model: Option<&str>,
    ) -> TasksResult<(Vec<Task>, TokenUsage)> {
        let provider = self.get_provider(model)?;
        let model_id = model.unwrap_or_else(|| Self::get_default_model(provider.as_ref()));

        // Get the next task ID
        let existing_tasks = self.storage.load_tasks(None).await?;
        let next_id = existing_tasks
            .iter()
            .filter_map(|t| t.id.parse::<i32>().ok())
            .max()
            .unwrap_or(0)
            + 1;

        let context = ParsePrdContext {
            num_tasks: num_tasks.unwrap_or(10),
            next_id,
            research,
            prd_content: prd_content.to_string(),
            prd_path: prd_path.to_string(),
            default_task_priority: "medium".to_string(),
            project_root: String::new(),
        };

        let template = self
            .prompts
            .get("parse-prd")
            .ok_or_else(|| TasksError::Ai("parse-prd template not found".to_string()))?;

        let (system, user) = template.render(&context)?;

        let messages = vec![AIMessage::system(system), AIMessage::user(user)];

        // Use maximum output tokens to ensure complete response
        // Claude 4.5 models support up to 64k output tokens (128k with extended thinking)
        // We set to 64k to allow complete task generation without truncation
        let options = GenerateOptions {
            temperature: Some(0.7),
            max_tokens: Some(64_000),
            json_mode: true,
            ..Default::default()
        };

        let response = provider
            .generate_text(model_id, &messages, &options)
            .await?;
        let parsed: ParsePrdResponse = parse_ai_response(&response)?;

        // Convert generated tasks to Task entities
        let tasks: Vec<Task> = parsed
            .tasks
            .into_iter()
            .map(Self::generated_task_to_task)
            .collect();

        Ok((tasks, response.usage))
    }

    /// Expand a task into subtasks.
    ///
    /// # Arguments
    /// * `task` - The task to expand
    /// * `subtask_count` - Optional target number of subtasks
    /// * `research` - Use research mode for better subtask generation
    /// * `additional_context` - Additional context to guide subtask generation
    /// * `complexity_report` - Optional complexity report for guided expansion
    /// * `model` - AI model to use
    ///
    /// # Returns
    /// A tuple of (subtasks, token_usage)
    pub async fn expand_task(
        &self,
        task: &Task,
        subtask_count: Option<i32>,
        research: bool,
        additional_context: Option<&str>,
        complexity_report: Option<&ComplexityReport>,
        model: Option<&str>,
    ) -> TasksResult<(Vec<Subtask>, TokenUsage)> {
        self.expand_task_internal(
            task,
            subtask_count,
            research,
            additional_context,
            complexity_report,
            model,
            false, // enable_subagents
        )
        .await
    }

    /// Expand a task into subtasks with subagent support.
    ///
    /// This variant enables subagent-aware expansion, which:
    /// - Generates `subagentType` for each subtask (implementer, reviewer, tester, etc.)
    /// - Sets `parallelizable` flags based on dependency analysis
    /// - Computes `executionLevel` for parallel grouping
    ///
    /// # Arguments
    /// * `task` - The task to expand
    /// * `subtask_count` - Optional target number of subtasks
    /// * `research` - Use research mode for better subtask generation
    /// * `additional_context` - Additional context to guide subtask generation
    /// * `complexity_report` - Optional complexity report for guided expansion
    /// * `model` - AI model to use
    ///
    /// # Returns
    /// A tuple of (subtasks, token_usage) where subtasks have execution levels computed
    pub async fn expand_task_with_subagents(
        &self,
        task: &Task,
        subtask_count: Option<i32>,
        research: bool,
        additional_context: Option<&str>,
        complexity_report: Option<&ComplexityReport>,
        model: Option<&str>,
    ) -> TasksResult<(Vec<Subtask>, TokenUsage)> {
        let (mut subtasks, usage) = self
            .expand_task_internal(
                task,
                subtask_count,
                research,
                additional_context,
                complexity_report,
                model,
                true, // enable_subagents
            )
            .await?;

        // Compute execution levels for parallel grouping
        let levels = super::compute_subtask_execution_levels(&mut subtasks);
        tracing::info!(
            task_id = %task.id,
            total_subtasks = levels.stats.total_subtasks,
            execution_levels = levels.stats.total_levels,
            max_parallelism = levels.stats.max_parallelism,
            "Computed subtask execution levels"
        );

        Ok((subtasks, usage))
    }

    /// Internal expand task implementation.
    #[allow(clippy::too_many_arguments)]
    async fn expand_task_internal(
        &self,
        task: &Task,
        subtask_count: Option<i32>,
        research: bool,
        additional_context: Option<&str>,
        complexity_report: Option<&ComplexityReport>,
        model: Option<&str>,
        enable_subagents: bool,
    ) -> TasksResult<(Vec<Subtask>, TokenUsage)> {
        let provider = self.get_provider(model)?;
        let model_id = model.unwrap_or_else(|| Self::get_default_model(provider.as_ref()));

        // Get expansion prompt from complexity report if available
        let (expansion_prompt, recommended_count, reasoning) = complexity_report
            .and_then(|r| r.get_task_analysis(task.id.parse().ok()?))
            .map_or((None, None, None), |a| {
                (
                    Some(a.expansion_prompt.clone()),
                    Some(a.recommended_subtasks),
                    Some(a.reasoning.clone()),
                )
            });

        let count = subtask_count.or(recommended_count).unwrap_or(5);

        // Get next subtask ID
        let next_id = task.subtasks.iter().map(|s| s.id).max().unwrap_or(0) + 1;

        #[allow(clippy::cast_possible_wrap)]
        let context = ExpandTaskContext {
            subtask_count: count,
            task: TaskSummary::from(task),
            next_subtask_id: next_id as i32,
            use_research: research,
            expansion_prompt,
            additional_context: additional_context.unwrap_or("").to_string(),
            complexity_reasoning_context: reasoning.unwrap_or_default(),
            gathered_context: String::new(),
            project_root: String::new(),
            enable_subagents,
        };

        let template = self
            .prompts
            .get("expand-task")
            .ok_or_else(|| TasksError::Ai("expand-task template not found".to_string()))?;

        let (system, user) = template.render(&context)?;

        let messages = vec![AIMessage::system(system), AIMessage::user(user)];

        let options = GenerateOptions {
            temperature: Some(0.7),
            max_tokens: Some(8000),
            json_mode: true,
            ..Default::default()
        };

        let response = provider
            .generate_text(model_id, &messages, &options)
            .await?;
        let parsed: ExpandTaskResponse = parse_ai_response(&response)?;

        // Convert to Subtask entities using the helper that handles subagent fields
        let subtasks: Vec<Subtask> = parsed
            .subtasks
            .into_iter()
            .map(|gs| Self::generated_subtask_to_subtask(gs, &task.id))
            .collect();

        Ok((subtasks, response.usage))
    }

    /// Analyze task complexity.
    pub async fn analyze_complexity(
        &self,
        tasks: &[Task],
        threshold: i32,
        research: bool,
        model: Option<&str>,
    ) -> TasksResult<(ComplexityReport, TokenUsage)> {
        let provider = self.get_provider(model)?;
        let model_id = model.unwrap_or_else(|| Self::get_default_model(provider.as_ref()));

        // Filter to non-done tasks
        let pending_tasks: Vec<_> = tasks
            .iter()
            .filter(|t| t.status != TaskStatus::Done)
            .collect();

        let context = AnalyzeComplexityContext {
            tasks: serde_json::to_value(&pending_tasks)?,
            gathered_context: String::new(),
            threshold,
            use_research: research,
            project_root: String::new(),
        };

        let template = self
            .prompts
            .get("analyze-complexity")
            .ok_or_else(|| TasksError::Ai("analyze-complexity template not found".to_string()))?;

        let (system, user) = template.render(&context)?;

        let messages = vec![AIMessage::system(system), AIMessage::user(user)];

        let options = GenerateOptions {
            temperature: Some(0.5),
            max_tokens: Some(8000),
            json_mode: true,
            ..Default::default()
        };

        let response = provider
            .generate_text(model_id, &messages, &options)
            .await?;
        let parsed: AnalyzeComplexityResponse = parse_ai_response(&response)?;

        let report = ComplexityReport::new(
            ".tasks/tasks.json",
            model_id,
            threshold,
            parsed.complexity_analysis,
        );

        Ok((report, response.usage))
    }

    /// Expand all pending tasks into subtasks based on complexity or defaults.
    ///
    /// # Arguments
    /// * `num_subtasks` - Optional target number of subtasks per task
    /// * `force` - Force regeneration of subtasks for tasks that already have them
    /// * `research` - Enable research-backed subtask generation
    /// * `additional_context` - Additional context to guide subtask generation
    /// * `complexity_report` - Optional complexity report for guided expansion
    /// * `model` - AI model to use
    pub async fn expand_all(
        &self,
        num_subtasks: Option<i32>,
        force: bool,
        research: bool,
        additional_context: Option<&str>,
        complexity_report: Option<&ComplexityReport>,
        model: Option<&str>,
    ) -> TasksResult<(Vec<Task>, TokenUsage)> {
        let mut all_tasks = self.storage.load_tasks(None).await?;
        let mut total_usage = TokenUsage::default();
        let mut expanded_count = 0;

        // Find tasks that need expansion
        for task in &mut all_tasks {
            // Skip non-pending tasks
            if task.status != TaskStatus::Pending {
                continue;
            }

            // Skip tasks that already have subtasks unless force is set
            if !task.subtasks.is_empty() && !force {
                continue;
            }

            // Get subtask count from complexity report or use default
            let subtask_count = num_subtasks.or_else(|| {
                complexity_report
                    .and_then(|r| r.get_task_analysis(task.id.parse().ok()?))
                    .map(|a| a.recommended_subtasks)
            });

            // Skip if complexity report says 0 subtasks needed
            if subtask_count == Some(0) {
                continue;
            }

            // Expand the task
            let (subtasks, usage) = self
                .expand_task(
                    task,
                    subtask_count,
                    research,
                    additional_context,
                    complexity_report,
                    model,
                )
                .await?;

            // Clear existing subtasks if force is set
            if force {
                task.subtasks.clear();
            }

            // Add new subtasks
            task.subtasks.extend(subtasks);
            total_usage.input_tokens += usage.input_tokens;
            total_usage.output_tokens += usage.output_tokens;
            total_usage.total_tokens += usage.total_tokens;
            expanded_count += 1;
        }

        tracing::info!(expanded_count, "Expanded tasks into subtasks");

        Ok((all_tasks, total_usage))
    }

    /// Convert a generated task to a Task entity.
    #[allow(clippy::cast_sign_loss)]
    fn generated_task_to_task(gt: GeneratedTask) -> Task {
        let task_id = gt.id.to_string();
        let subtasks = gt
            .subtasks
            .into_iter()
            .map(|gs| Self::generated_subtask_to_subtask(gs, &task_id))
            .collect();
        let decision_points = gt
            .decision_points
            .into_iter()
            .map(Self::generated_decision_point_to_decision_point)
            .collect();
        Task {
            id: task_id,
            title: gt.title,
            description: gt.description,
            status: gt.status.unwrap_or(TaskStatus::Pending),
            priority: gt.priority.unwrap_or(TaskPriority::Medium),
            dependencies: gt.dependencies.into_iter().map(|d| d.to_string()).collect(),
            details: gt.details.unwrap_or_default(),
            test_strategy: gt.test_strategy.unwrap_or_default(),
            subtasks,
            created_at: None,
            updated_at: None,
            effort: None,
            actual_effort: None,
            tags: Vec::new(),
            assignee: None,
            complexity: None,
            agent_hint: None,
            decision_points,
        }
    }

    /// Convert a generated decision point to a DecisionPoint entity.
    fn generated_decision_point_to_decision_point(gdp: GeneratedDecisionPoint) -> DecisionPoint {
        DecisionPoint {
            id: gdp.id,
            category: gdp.category,
            description: gdp.description,
            options: gdp.options,
            requires_approval: gdp.requires_approval,
            constraints: gdp.constraints,
            constraint_type: gdp.constraint_type,
        }
    }

    /// Convert a generated subtask to a Subtask entity.
    #[allow(clippy::cast_sign_loss)]
    fn generated_subtask_to_subtask(gs: GeneratedSubtask, parent_id: &str) -> Subtask {
        let mut subtask = Subtask::new(gs.id as u32, parent_id, gs.title, gs.description);
        subtask.status = gs.status.unwrap_or(TaskStatus::Pending);
        subtask.dependencies = gs.dependencies.into_iter().map(|d| d.to_string()).collect();
        subtask.details = gs.details.unwrap_or_default();
        subtask.test_strategy = gs.test_strategy.unwrap_or_default();

        // Set subagent fields if provided by AI
        subtask.subagent_type = gs.subagent_type;
        subtask.parallelizable = gs.parallelizable.unwrap_or(false);

        subtask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would go here, but they require actual API keys
    // Unit tests for the conversion functions can be added

    #[test]
    fn test_generated_task_conversion() {
        let storage = Arc::new(crate::storage::FileStorage::new("."));
        let _domain = AIDomain::new(storage);

        let generated = GeneratedTask {
            id: 1,
            title: "Test task".to_string(),
            description: "Test description".to_string(),
            details: Some("Test details".to_string()),
            test_strategy: Some("Test strategy".to_string()),
            priority: Some(TaskPriority::High),
            dependencies: vec![],
            status: Some(TaskStatus::Pending),
            subtasks: vec![],
            decision_points: vec![],
        };

        let task = AIDomain::generated_task_to_task(generated);

        assert_eq!(task.id, "1");
        assert_eq!(task.title, "Test task");
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_generated_task_with_subtasks_conversion() {
        use crate::entities::SubagentType;

        let generated = GeneratedTask {
            id: 1,
            title: "Test task".to_string(),
            description: "Test description".to_string(),
            details: Some("Test details".to_string()),
            test_strategy: Some("Test strategy".to_string()),
            priority: Some(TaskPriority::High),
            dependencies: vec![],
            status: Some(TaskStatus::Pending),
            subtasks: vec![
                GeneratedSubtask {
                    id: 1,
                    title: "Subtask 1".to_string(),
                    description: "First subtask".to_string(),
                    details: Some("Subtask details".to_string()),
                    test_strategy: Some("Subtask test".to_string()),
                    dependencies: vec![],
                    status: Some(TaskStatus::Pending),
                    subagent_type: Some(SubagentType::Implementer),
                    parallelizable: Some(true),
                },
                GeneratedSubtask {
                    id: 2,
                    title: "Subtask 2".to_string(),
                    description: "Second subtask".to_string(),
                    details: None,
                    test_strategy: None,
                    dependencies: vec![1],
                    status: None,
                    subagent_type: None,
                    parallelizable: None,
                },
            ],
            decision_points: vec![],
        };

        let task = AIDomain::generated_task_to_task(generated);

        assert_eq!(task.id, "1");
        assert_eq!(task.subtasks.len(), 2);
        assert_eq!(task.subtasks[0].id, 1);
        assert_eq!(task.subtasks[0].title, "Subtask 1");
        assert_eq!(task.subtasks[0].parent_id, "1");
        assert_eq!(task.subtasks[0].details, "Subtask details");
        assert_eq!(
            task.subtasks[0].subagent_type,
            Some(SubagentType::Implementer)
        );
        assert!(task.subtasks[0].parallelizable);
        assert_eq!(task.subtasks[1].id, 2);
        assert_eq!(task.subtasks[1].title, "Subtask 2");
        assert_eq!(task.subtasks[1].dependencies, vec!["1"]);
        assert!(task.subtasks[1].subagent_type.is_none());
        assert!(!task.subtasks[1].parallelizable);
    }
}
