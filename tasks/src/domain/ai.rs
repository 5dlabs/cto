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
    prompts::{
        AddTaskContext, AnalyzeComplexityContext, ExpandTaskContext, ParsePrdContext, TaskSummary,
        UpdateSubtaskContext, UpdateTaskContext, UpdateTasksContext,
    },
    schemas::{
        AddTaskResponse, AnalyzeComplexityResponse, ComplexityReport, ExpandTaskResponse,
        GeneratedSubtask, GeneratedTask, ParsePrdResponse, UpdateTaskResponse, UpdateTasksResponse,
    },
    AIMessage, AIProvider, GenerateOptions, PromptManager, ProviderRegistry, TokenUsage,
};
use crate::entities::{Subtask, Task, TaskPriority, TaskStatus};
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
            "claude-sonnet-4-20250514"
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

        let options = GenerateOptions {
            temperature: Some(0.7),
            max_tokens: Some(16000),
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
    pub async fn expand_task(
        &self,
        task: &Task,
        subtask_count: Option<i32>,
        research: bool,
        additional_context: Option<&str>,
        complexity_report: Option<&ComplexityReport>,
        model: Option<&str>,
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

        // Convert to Subtask entities
        #[allow(clippy::cast_sign_loss)]
        let subtasks: Vec<Subtask> = parsed
            .subtasks
            .into_iter()
            .map(|gs| {
                let mut subtask = Subtask::new(gs.id as u32, &task.id, gs.title, gs.description);
                subtask.status = gs.status.unwrap_or(TaskStatus::Pending);
                subtask.dependencies = gs.dependencies.into_iter().map(|d| d.to_string()).collect();
                subtask.details = gs.details.unwrap_or_default();
                subtask.test_strategy = gs.test_strategy.unwrap_or_default();
                subtask
            })
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

    /// Add a new task using AI.
    pub async fn add_task(
        &self,
        prompt: &str,
        priority: Option<TaskPriority>,
        dependencies: Option<Vec<i32>>,
        research: bool,
        model: Option<&str>,
    ) -> TasksResult<(Task, TokenUsage)> {
        let provider = self.get_provider(model)?;
        let model_id = model.unwrap_or_else(|| Self::get_default_model(provider.as_ref()));

        // Get existing tasks for context
        let existing_tasks = self.storage.load_tasks(None).await?;
        let next_id = existing_tasks
            .iter()
            .filter_map(|t| t.id.parse::<i32>().ok())
            .max()
            .unwrap_or(0)
            + 1;

        // Create task summary for context
        let tasks_summary: Vec<serde_json::Value> = existing_tasks
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "title": t.title,
                    "status": format!("{:?}", t.status).to_lowercase(),
                    "dependencies": t.dependencies,
                })
            })
            .collect();

        let context = AddTaskContext {
            prompt: prompt.to_string(),
            new_task_id: next_id,
            existing_tasks: serde_json::json!(tasks_summary),
            gathered_context: format!(
                "Existing tasks in the project:\n{}",
                serde_json::to_string_pretty(&tasks_summary).unwrap_or_default()
            ),
            context_from_args: String::new(),
            priority: priority
                .map_or_else(|| "medium".to_string(), |p| format!("{p:?}").to_lowercase()),
            dependencies: dependencies.unwrap_or_default(),
            use_research: research,
            project_root: String::new(),
        };

        let template = self
            .prompts
            .get("add-task")
            .ok_or_else(|| TasksError::Ai("add-task template not found".to_string()))?;

        let (system, user) = template.render(&context)?;

        let messages = vec![AIMessage::system(system), AIMessage::user(user)];

        let options = GenerateOptions {
            temperature: Some(0.7),
            max_tokens: Some(4000),
            json_mode: true,
            ..Default::default()
        };

        let response = provider
            .generate_text(model_id, &messages, &options)
            .await?;
        let parsed: AddTaskResponse = parse_ai_response(&response)?;

        let mut task = Task::new(next_id.to_string(), parsed.title, parsed.description);
        task.details = parsed.details;
        task.test_strategy = parsed.test_strategy;
        task.dependencies = parsed
            .dependencies
            .into_iter()
            .map(|d| d.to_string())
            .collect();
        task.priority = priority.unwrap_or(TaskPriority::Medium);

        Ok((task, response.usage))
    }

    /// Update a single task with AI assistance.
    pub async fn update_task(
        &self,
        task: &Task,
        update_prompt: &str,
        append_mode: bool,
        research: bool,
        model: Option<&str>,
    ) -> TasksResult<(Task, TokenUsage)> {
        let provider = self.get_provider(model)?;
        let model_id = model.unwrap_or_else(|| Self::get_default_model(provider.as_ref()));

        let task_json = serde_json::to_string_pretty(task)?;

        let context = UpdateTaskContext {
            task: serde_json::to_value(task)?,
            task_json,
            update_prompt: update_prompt.to_string(),
            append_mode,
            use_research: research,
            current_details: if task.details.is_empty() {
                "(No existing details)".to_string()
            } else {
                task.details.clone()
            },
            gathered_context: String::new(),
            project_root: String::new(),
        };

        let template = self
            .prompts
            .get("update-task")
            .ok_or_else(|| TasksError::Ai("update-task template not found".to_string()))?;

        let (system, user) = template.render(&context)?;

        let messages = vec![AIMessage::system(system), AIMessage::user(user)];

        let options = GenerateOptions {
            temperature: Some(0.7),
            max_tokens: Some(8000),
            // In append mode, we want plain text; otherwise JSON for structured response
            json_mode: !append_mode,
            ..Default::default()
        };

        let response = provider
            .generate_text(model_id, &messages, &options)
            .await?;

        if append_mode {
            // In append mode, the AI returns just the text to append
            let new_details = response.text.trim();
            let mut updated_task = task.clone();

            // Append with timestamp
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
            let separator = if updated_task.details.is_empty() {
                ""
            } else {
                "\n\n---\n\n"
            };
            updated_task.details = format!(
                "{}{}[{}] {}",
                updated_task.details, separator, timestamp, new_details
            );

            Ok((updated_task, response.usage))
        } else {
            let parsed: UpdateTaskResponse = parse_ai_response(&response)?;
            let updated_task = Self::generated_task_to_task(parsed.task);
            Ok((updated_task, response.usage))
        }
    }

    /// Update a subtask by appending information.
    pub async fn update_subtask(
        &self,
        parent_task: &Task,
        subtask: &Subtask,
        update_prompt: &str,
        research: bool,
        model: Option<&str>,
    ) -> TasksResult<(String, TokenUsage)> {
        let provider = self.get_provider(model)?;
        let model_id = model.unwrap_or_else(|| Self::get_default_model(provider.as_ref()));

        let context = UpdateSubtaskContext {
            subtask: serde_json::to_value(subtask)?,
            parent_task: serde_json::to_value(parent_task)?,
            update_prompt: update_prompt.to_string(),
            use_research: research,
            current_details: if subtask.details.is_empty() {
                "(No existing details)".to_string()
            } else {
                subtask.details.clone()
            },
            gathered_context: String::new(),
            project_root: String::new(),
        };

        let template = self
            .prompts
            .get("update-subtask")
            .ok_or_else(|| TasksError::Ai("update-subtask template not found".to_string()))?;

        let (system, user) = template.render(&context)?;

        let messages = vec![AIMessage::system(system), AIMessage::user(user)];

        let options = GenerateOptions {
            temperature: Some(0.7),
            max_tokens: Some(4000),
            ..Default::default()
        };

        let response = provider
            .generate_text(model_id, &messages, &options)
            .await?;

        // The response is plain text to append
        Ok((response.text.trim().to_string(), response.usage))
    }

    /// Batch update tasks starting from a specific ID.
    pub async fn update_tasks(
        &self,
        from_id: i32,
        update_prompt: &str,
        research: bool,
        model: Option<&str>,
    ) -> TasksResult<(Vec<Task>, TokenUsage)> {
        let provider = self.get_provider(model)?;
        let model_id = model.unwrap_or_else(|| Self::get_default_model(provider.as_ref()));

        // Get tasks from the given ID onwards
        let all_tasks = self.storage.load_tasks(None).await?;
        let tasks_to_update: Vec<_> = all_tasks
            .iter()
            .filter(|t| {
                t.id.parse::<i32>().ok().is_some_and(|id| id >= from_id)
                    && t.status != TaskStatus::Done
            })
            .collect();

        let context = UpdateTasksContext {
            tasks: serde_json::to_value(&tasks_to_update)?,
            from_id,
            update_prompt: update_prompt.to_string(),
            use_research: research,
            gathered_context: String::new(),
            project_root: String::new(),
        };

        let template = self
            .prompts
            .get("update-tasks")
            .ok_or_else(|| TasksError::Ai("update-tasks template not found".to_string()))?;

        let (system, user) = template.render(&context)?;

        let messages = vec![AIMessage::system(system), AIMessage::user(user)];

        let options = GenerateOptions {
            temperature: Some(0.7),
            max_tokens: Some(16000),
            json_mode: true,
            ..Default::default()
        };

        let response = provider
            .generate_text(model_id, &messages, &options)
            .await?;
        let parsed: UpdateTasksResponse = parse_ai_response(&response)?;

        let tasks: Vec<Task> = parsed
            .tasks
            .into_iter()
            .map(Self::generated_task_to_task)
            .collect();

        Ok((tasks, response.usage))
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
        };

        let task = AIDomain::generated_task_to_task(generated);

        assert_eq!(task.id, "1");
        assert_eq!(task.title, "Test task");
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_generated_task_with_subtasks_conversion() {
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
                },
                GeneratedSubtask {
                    id: 2,
                    title: "Subtask 2".to_string(),
                    description: "Second subtask".to_string(),
                    details: None,
                    test_strategy: None,
                    dependencies: vec![1],
                    status: None,
                },
            ],
        };

        let task = AIDomain::generated_task_to_task(generated);

        assert_eq!(task.id, "1");
        assert_eq!(task.subtasks.len(), 2);
        assert_eq!(task.subtasks[0].id, 1);
        assert_eq!(task.subtasks[0].title, "Subtask 1");
        assert_eq!(task.subtasks[0].parent_id, "1");
        assert_eq!(task.subtasks[0].details, "Subtask details");
        assert_eq!(task.subtasks[1].id, 2);
        assert_eq!(task.subtasks[1].title, "Subtask 2");
        assert_eq!(task.subtasks[1].dependencies, vec!["1"]);
    }
}
