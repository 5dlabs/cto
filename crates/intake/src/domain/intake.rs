//! Intake orchestrator - Complete intake workflow from PRD to documentation.
//!
//! This module provides a unified intake process that:
//! 1. Parses PRD to generate tasks
//! 2. Optionally analyzes task complexity
//! 3. Optionally expands tasks into subtasks
//! 4. Adds agent routing hints
//! 5. Generates per-task documentation (XML, MD, acceptance criteria)
//! 6. Saves tasks.json
//! 7. Generates cto-config.json with per-agent tools

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::ai::schemas::ComplexityReport;
use crate::entities::{Task, TaskStatus};
use crate::errors::{TasksError, TasksResult};
use crate::progress::{emit_progress, ProgressEvent};
use crate::storage::Storage;

use super::cto_config::{generate_cto_config, save_cto_config};
use super::docs::{generate_all_docs, DocsGenerationResult};
use super::tasks::routing::infer_agent_hint_with_deps_str;
use super::AIDomain;

/// Check if a deploy task already exists in the task list.
/// Looks for tasks with agent_hint="bolt" and title containing "deploy".
#[must_use]
pub fn has_deploy_task(tasks: &[Task]) -> bool {
    tasks.iter().any(|t| {
        let title_lower = t.title.to_lowercase();
        let desc_lower = t.description.to_lowercase();
        let is_bolt = t.agent_hint.as_deref() == Some("bolt");
        let has_deploy_keyword = title_lower.contains("deploy")
            || desc_lower.contains("deploy")
            || title_lower.contains("deployment")
            || desc_lower.contains("deployment");
        is_bolt && has_deploy_keyword
    })
}

/// Create a deploy task that depends on all other tasks.
/// The task is assigned to Bolt and includes standard deployment instructions.
#[must_use]
pub fn create_deploy_task(tasks: &[Task]) -> Task {
    // Get the next task ID (max ID + 1)
    let max_id = tasks
        .iter()
        .filter_map(|t| t.id.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    let new_id = (max_id + 1).to_string();

    // Collect all task IDs as dependencies
    let dependencies: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();

    let mut task = Task::new(
        new_id,
        "Deploy application to production",
        "Deploy the completed application to production environment. \
         This task depends on all implementation tasks being complete.",
    );
    task.dependencies = dependencies;
    task.agent_hint = Some("bolt".to_string());
    task.details = "Deployment steps:\n\
        1. Verify all dependent tasks are complete\n\
        2. Run final integration tests\n\
        3. Build production artifacts\n\
        4. Deploy to staging environment\n\
        5. Run smoke tests\n\
        6. Deploy to production\n\
        7. Verify production deployment\n\
        8. Update deployment documentation"
        .to_string();
    task.test_strategy = "Verify deployment by:\n\
        - Checking health endpoints\n\
        - Running smoke tests against production\n\
        - Verifying all services are operational"
        .to_string();

    task
}

/// Configuration for the intake process.
#[derive(Debug, Clone)]
pub struct IntakeConfig {
    /// Path to the PRD file.
    pub prd_path: PathBuf,

    /// Path to the architecture file (optional).
    pub architecture_path: Option<PathBuf>,

    /// Target number of tasks to generate (0 = auto).
    pub num_tasks: i32,

    /// Whether to expand tasks into subtasks.
    pub expand: bool,

    /// Whether to analyze task complexity.
    pub analyze: bool,

    /// Complexity threshold for expansion recommendations.
    pub complexity_threshold: i32,

    /// Whether to use research mode for AI operations.
    pub research: bool,

    /// AI model to use (None = default).
    pub model: Option<String>,

    /// Output directory for tasks and documentation.
    pub output_dir: PathBuf,

    /// Target repository (e.g., "5dlabs/my-project").
    pub repository: Option<String>,

    /// Service name for the project.
    pub service: Option<String>,

    /// Docs repository URL.
    pub docs_repository: Option<String>,

    /// Project directory within docs repo.
    pub docs_project_directory: Option<String>,

    /// Whether to auto-append a deploy task after all other tasks.
    /// When enabled, a Bolt deploy task is added that depends on all other tasks.
    pub auto_append_deploy_task: bool,

    /// Whether to run the deliberation phase before PRD parsing.
    /// When enabled, the Optimist/Pessimist debate runs first and a resolved
    /// design brief is used as input to parse_prd instead of the raw PRD.
    /// Feature flag — defaults to false until deliberation agents are deployed.
    pub deliberate: bool,

    /// Path to a pre-computed design brief (output of deliberation).
    /// If set and `deliberate` is false, this brief is used directly.
    pub design_brief_path: Option<PathBuf>,
}

impl Default for IntakeConfig {
    fn default() -> Self {
        Self {
            prd_path: PathBuf::from(".tasks/docs/prd.txt"),
            architecture_path: None,
            num_tasks: 15,
            expand: true,
            analyze: true,
            complexity_threshold: 5,
            research: true,
            model: None,
            output_dir: PathBuf::from(".tasks"),
            repository: None,
            service: None,
            docs_repository: None,
            docs_project_directory: None,
            auto_append_deploy_task: false,
            deliberate: false,
            design_brief_path: None,
        }
    }
}

/// Result of the intake process.
#[derive(Debug, Clone)]
pub struct IntakeResult {
    /// Number of tasks generated.
    pub tasks_count: usize,

    /// Number of subtasks generated (if expanded).
    pub subtasks_count: usize,

    /// Complexity report (if analyzed).
    pub complexity_report: Option<ComplexityReport>,

    /// Documentation generation result.
    pub docs_result: DocsGenerationResult,

    /// Path to the generated tasks.json.
    pub tasks_file: PathBuf,

    /// Total input tokens used.
    pub total_input_tokens: u32,

    /// Total output tokens used.
    pub total_output_tokens: u32,
}

/// Intake domain for orchestrating the full intake workflow.
pub struct IntakeDomain {
    storage: Arc<dyn Storage>,
    ai_domain: AIDomain,
}

impl IntakeDomain {
    /// Create a new intake domain.
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            ai_domain: AIDomain::new(Arc::clone(&storage)),
            storage,
        }
    }

    /// Run the complete intake workflow.
    pub async fn run(&self, config: &IntakeConfig) -> TasksResult<IntakeResult> {
        let mut total_input_tokens = 0u32;
        let mut total_output_tokens = 0u32;
        let start_time = std::time::Instant::now();

        // Emit progress config event for Linear sidecar
        let cli_type = std::env::var("TASKS_CLI").unwrap_or_else(|_| "claude".to_string());
        #[allow(clippy::cast_sign_loss)] // num_tasks is always positive from CLI
        emit_progress(&ProgressEvent::config(
            config.model.as_deref().unwrap_or("default"),
            &cli_type,
            config.num_tasks as u32,
            90, // Default acceptance threshold
        ));

        // 1. Read PRD content
        let raw_prd_content = tokio::fs::read_to_string(&config.prd_path)
            .await
            .map_err(|e| TasksError::FileReadError {
                path: config.prd_path.display().to_string(),
                reason: e.to_string(),
            })?;

        // 1a. Deliberation phase (feature-flagged).
        // When enabled, run the Optimist/Pessimist debate and use the
        // resulting design brief as the primary context for task generation.
        // Falls back to the raw PRD if deliberation is disabled or the brief
        // is not available.
        let prd_content = if let Some(ref brief_path) = config.design_brief_path {
            // Pre-computed brief supplied — use it directly
            if brief_path.exists() {
                let brief = tokio::fs::read_to_string(brief_path).await.map_err(|e| {
                    TasksError::FileReadError {
                        path: brief_path.display().to_string(),
                        reason: e.to_string(),
                    }
                })?;
                format!("{brief}\n\n---\n\n## Original PRD\n\n{raw_prd_content}")
            } else {
                tracing::warn!(
                    "design_brief_path {:?} not found — falling back to raw PRD",
                    brief_path
                );
                raw_prd_content.clone()
            }
        } else if config.deliberate {
            // Deliberation is enabled but no pre-computed brief — check for
            // a brief that was already produced in this run.
            let default_brief_path = config.output_dir.join("docs/design-brief.md");
            if default_brief_path.exists() {
                let brief = tokio::fs::read_to_string(&default_brief_path)
                    .await
                    .map_err(|e| TasksError::FileReadError {
                        path: default_brief_path.display().to_string(),
                        reason: e.to_string(),
                    })?;
                tracing::info!("Using design brief from {:?}", default_brief_path);
                format!("{brief}\n\n---\n\n## Original PRD\n\n{raw_prd_content}")
            } else {
                tracing::warn!(
                    "deliberate=true but no design brief found at {:?} — \
                     run the deliberation workflow first, or call the intake \
                     agent with the brief pre-populated. Falling back to raw PRD.",
                    default_brief_path
                );
                raw_prd_content.clone()
            }
        } else {
            raw_prd_content.clone()
        };

        // 2. Read architecture content if provided
        let architecture_content = if let Some(arch_path) = &config.architecture_path {
            if arch_path.exists() {
                Some(tokio::fs::read_to_string(arch_path).await.map_err(|e| {
                    TasksError::FileReadError {
                        path: arch_path.display().to_string(),
                        reason: e.to_string(),
                    }
                })?)
            } else {
                None
            }
        } else {
            None
        };

        // Combine PRD with architecture context
        let full_prd = if let Some(arch) = &architecture_content {
            format!(
                "{}\n\n---\n\n## Architecture Context\n\n{}",
                prd_content, arch
            )
        } else {
            prd_content
        };

        // 3. Parse PRD to generate tasks
        emit_progress(&ProgressEvent::step_started(
            1,
            "Parse PRD and generate tasks",
        ));
        tracing::info!(
            "Step 1/4: Parsing PRD to generate ~{} tasks...",
            config.num_tasks
        );
        let (mut tasks, prd_usage) = self
            .ai_domain
            .parse_prd(
                &full_prd,
                config.prd_path.to_str().unwrap_or(""),
                Some(config.num_tasks),
                config.research,
                config.model.as_deref(),
            )
            .await?;

        total_input_tokens += prd_usage.input_tokens;
        total_output_tokens += prd_usage.output_tokens;

        tracing::info!("Generated {} tasks", tasks.len());
        emit_progress(&ProgressEvent::step_completed(
            1,
            "Parse PRD and generate tasks",
            Some(&format!("{} tasks generated", tasks.len())),
        ));

        // 4. Analyze complexity if requested
        let complexity_report = if config.analyze {
            emit_progress(&ProgressEvent::step_started(2, "Analyze task complexity"));
            tracing::info!("Step 2/4: Analyzing task complexity...");
            let (report, analyze_usage) = self
                .ai_domain
                .analyze_complexity(
                    &tasks,
                    config.complexity_threshold,
                    config.research,
                    config.model.as_deref(),
                )
                .await?;

            total_input_tokens += analyze_usage.input_tokens;
            total_output_tokens += analyze_usage.output_tokens;

            tracing::info!("Complexity analysis complete");
            emit_progress(&ProgressEvent::step_completed(
                2,
                "Analyze task complexity",
                None,
            ));
            Some(report)
        } else {
            tracing::info!("Step 2/4: Skipping complexity analysis");
            emit_progress(&ProgressEvent::step_skipped(
                2,
                "Analyze task complexity",
                "analyze=false",
            ));
            None
        };

        // 5. Expand tasks into subtasks if requested
        let mut subtasks_count = 0;
        if config.expand {
            let tasks_to_expand: Vec<_> = tasks
                .iter()
                .filter(|t| t.status != TaskStatus::Done && t.subtasks.is_empty())
                .map(|t| t.id.clone())
                .collect();

            emit_progress(&ProgressEvent::step_started(
                3,
                "Expand tasks into subtasks",
            ));
            tracing::info!(
                "Step 3/4: Expanding {} tasks into subtasks...",
                tasks_to_expand.len()
            );

            // Track expansion progress for task_progress events
            let total_to_expand = tasks_to_expand.len();
            let mut expanded_count: u32 = 0;

            for (idx, task) in tasks.iter_mut().enumerate() {
                // Skip done tasks
                if task.status == TaskStatus::Done {
                    continue;
                }

                // Skip if already has subtasks
                if !task.subtasks.is_empty() {
                    continue;
                }

                tracing::debug!(
                    "Expanding task {}/{}: {}",
                    idx + 1,
                    total_to_expand,
                    task.id
                );

                // Use expand_task_with_subagents by default for parallel execution support
                // This generates subagentType, executionLevel, and parallelizable fields
                match self
                    .ai_domain
                    .expand_task_with_subagents(
                        task,
                        None, // Use complexity report recommendation
                        config.research,
                        None,
                        complexity_report.as_ref(),
                        config.model.as_deref(),
                    )
                    .await
                {
                    Ok((subtasks, expand_usage)) => {
                        tracing::debug!("Generated {} subtasks", subtasks.len());
                        subtasks_count += subtasks.len();
                        task.subtasks = subtasks;
                        total_input_tokens += expand_usage.input_tokens;
                        total_output_tokens += expand_usage.output_tokens;

                        // Emit task progress for the Linear sidecar to display
                        expanded_count += 1;
                        #[allow(clippy::cast_possible_truncation)]
                        // total_to_expand is realistically < 4B tasks
                        emit_progress(&ProgressEvent::task_progress(
                            expanded_count,
                            total_to_expand as u32,
                        ));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to expand task {}: {}", task.id, e);
                    }
                }
            }
            tracing::info!("Generated {} subtasks total", subtasks_count);
            emit_progress(&ProgressEvent::step_completed(
                3,
                "Expand tasks into subtasks",
                Some(&format!("{} subtasks", subtasks_count)),
            ));
        } else {
            tracing::info!("Step 3/4: Skipping task expansion");
            emit_progress(&ProgressEvent::step_skipped(
                3,
                "Expand tasks into subtasks",
                "expand=false",
            ));
        }

        // 5.5: Normalize agent hints to lowercase
        // LLMs may generate "Bolt" instead of "bolt", or "Rex" instead of "rex"
        // This ensures consistent lowercase agent names before routing validation
        for task in &mut tasks {
            if let Some(ref mut hint) = task.agent_hint {
                let normalized = hint.to_lowercase();
                if *hint != normalized {
                    tracing::debug!(
                        "Normalized agent hint for task {}: '{}' → '{}'",
                        task.id,
                        hint,
                        normalized
                    );
                    *hint = normalized;
                }
            }
        }

        // 6. Add agent routing hints WITH DEPENDENCY AWARENESS
        // Dependencies are the PRIMARY signal - if a task depends on a Tap/Spark/Blaze
        // initialization task, it should inherit that agent.
        //
        // IMPORTANT: We ALWAYS re-validate and potentially override AI-generated hints
        // because the AI model may assign incorrect agents. Our routing logic is the
        // source of truth.
        //
        // NOTE: All tasks (including Task 1) are routed by content-based inference.
        // The PRD prompt guides the AI to make Task 1 infrastructure only when needed
        // (databases, caches, storage), but routing validates based on actual content.
        tracing::info!("Adding agent routing hints with dependency analysis...");

        // First pass: assign hints to tasks without dependencies
        // This ensures dependency targets have hints before we check dependencies
        let mut unroutable_tasks: Vec<String> = Vec::new();

        for task in &mut tasks {
            if task.dependencies.is_empty() {
                match infer_agent_hint_with_deps_str(task, &[]) {
                    Some(inferred) => {
                        if task.agent_hint.as_deref() != Some(inferred) {
                            if task.agent_hint.is_some() {
                                tracing::debug!(
                                    "Task {} hint '{}' overridden to '{}'",
                                    task.id,
                                    task.agent_hint.as_deref().unwrap_or("none"),
                                    inferred
                                );
                            }
                            task.agent_hint = Some(inferred.to_string());
                        }
                    }
                    None => {
                        unroutable_tasks.push(format!(
                            "Task {} '{}': {}",
                            task.id, task.title, task.description
                        ));
                    }
                }
            }
        }

        // Second pass: assign hints considering dependencies
        // Clone tasks for reference since we need to mutate while iterating
        let tasks_snapshot = tasks.clone();
        for task in &mut tasks {
            if let Some(inferred) = infer_agent_hint_with_deps_str(task, &tasks_snapshot) {
                if task.agent_hint.as_deref() != Some(inferred) {
                    if task.agent_hint.is_some() {
                        tracing::debug!(
                            "Task {} hint '{}' overridden to '{}' (dependency-aware)",
                            task.id,
                            task.agent_hint.as_deref().unwrap_or("none"),
                            inferred
                        );
                    }
                    task.agent_hint = Some(inferred.to_string());
                }
            } else {
                // Only add if not already in list from first pass
                let task_info = format!("Task {} '{}': {}", task.id, task.title, task.description);
                if !unroutable_tasks.contains(&task_info) {
                    unroutable_tasks.push(task_info);
                }
            }
        }

        // FAIL if any tasks couldn't be routed
        if !unroutable_tasks.is_empty() {
            return Err(TasksError::ValidationError {
                field: "agent_hint".to_string(),
                reason: format!(
                    "Cannot determine agent for {} task(s). Add routing keywords or explicit agent hints:\n{}",
                    unroutable_tasks.len(),
                    unroutable_tasks.join("\n")
                ),
            });
        }

        tracing::info!(
            "Agent hints assigned via content-based inference with dependency awareness"
        );

        // 6.5: Auto-append deploy task if configured
        if config.auto_append_deploy_task {
            if has_deploy_task(&tasks) {
                tracing::info!("Deploy task already exists, skipping auto-append");
            } else {
                tracing::info!("Auto-appending deploy task (depends on all other tasks)");
                let deploy_task = create_deploy_task(&tasks);
                tasks.push(deploy_task);
            }
        }

        // 7. Save tasks to storage
        let tasks_dir = config.output_dir.join("tasks");
        tokio::fs::create_dir_all(&tasks_dir)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: tasks_dir.display().to_string(),
                reason: e.to_string(),
            })?;

        self.storage.save_tasks(&tasks, None).await?;

        let tasks_file = tasks_dir.join("tasks.json");

        // 8. Save complexity report if generated
        if let Some(ref report) = complexity_report {
            let reports_dir = config.output_dir.join("reports");
            tokio::fs::create_dir_all(&reports_dir).await.map_err(|e| {
                TasksError::FileWriteError {
                    path: reports_dir.display().to_string(),
                    reason: e.to_string(),
                }
            })?;

            let report_path = reports_dir.join("task-complexity-report.json");
            let report_json = serde_json::to_string_pretty(report)?;
            tokio::fs::write(&report_path, &report_json)
                .await
                .map_err(|e| TasksError::FileWriteError {
                    path: report_path.display().to_string(),
                    reason: e.to_string(),
                })?;
        }

        // 9. Generate documentation
        emit_progress(&ProgressEvent::step_started(4, "Generate documentation"));
        tracing::info!("Step 4/4: Generating per-task documentation...");
        let docs_dir = config.output_dir.join("docs");
        let docs_result = generate_all_docs(&tasks, &docs_dir).await?;

        tracing::info!(
            "Created {} task documentation directories",
            docs_result.task_dirs_created
        );
        emit_progress(&ProgressEvent::step_completed(
            4,
            "Generate documentation",
            Some(&format!("{} doc dirs", docs_result.task_dirs_created)),
        ));

        // 10. Generate cto-config.json with per-agent tools
        if config.repository.is_some() || config.service.is_some() {
            tracing::info!("Generating cto-config.json with agent tool configurations...");

            let repository = config
                .repository
                .clone()
                .unwrap_or_else(|| "unknown/unknown".to_string());
            let service = config
                .service
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let docs_repository = config
                .docs_repository
                .clone()
                .unwrap_or_else(|| repository.clone());
            let docs_project_directory = config
                .docs_project_directory
                .clone()
                .unwrap_or_else(|| service.clone());

            let cto_config = generate_cto_config(
                &tasks,
                &repository,
                &service,
                &docs_repository,
                &docs_project_directory,
            );

            // Save cto-config.json in the output directory (project root, not .tasks)
            // The parent of output_dir (.tasks) is typically the project root
            let project_root = config
                .output_dir
                .parent()
                .unwrap_or(&config.output_dir)
                .to_path_buf();

            if let Err(e) = save_cto_config(&cto_config, &project_root).await {
                tracing::warn!("Failed to save cto-config.json: {}", e);
            } else {
                tracing::info!(
                    "Generated cto-config.json with {} agent configurations",
                    cto_config.agents.len()
                );
            }
        } else {
            tracing::info!("Skipping cto-config.json generation (no repository/service provided)");
        }

        tracing::info!(
            "Intake complete: {} tasks, {} subtasks, {} doc dirs",
            tasks.len(),
            subtasks_count,
            docs_result.task_dirs_created
        );

        #[allow(clippy::cast_possible_truncation)] // Task count realistically < 4B
        emit_progress(&ProgressEvent::complete(
            tasks.len() as u32,
            start_time.elapsed().as_secs_f64(),
            true,
            None,
        ));

        Ok(IntakeResult {
            tasks_count: tasks.len(),
            subtasks_count,
            complexity_report,
            docs_result,
            tasks_file,
            total_input_tokens,
            total_output_tokens,
        })
    }
}

/// Convenience function to run intake with default configuration.
pub async fn run_intake(
    storage: Arc<dyn Storage>,
    prd_path: &Path,
    architecture_path: Option<&Path>,
    num_tasks: i32,
    model: Option<&str>,
    output_dir: &Path,
) -> TasksResult<IntakeResult> {
    let config = IntakeConfig {
        prd_path: prd_path.to_path_buf(),
        architecture_path: architecture_path.map(Path::to_path_buf),
        num_tasks,
        model: model.map(String::from),
        output_dir: output_dir.to_path_buf(),
        ..Default::default()
    };

    let domain = IntakeDomain::new(storage);
    domain.run(&config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intake_config_default() {
        let config = IntakeConfig::default();
        assert_eq!(config.num_tasks, 15);
        assert!(config.expand);
        assert!(config.analyze);
        assert!(config.research);
        assert_eq!(config.complexity_threshold, 5);
    }
}
