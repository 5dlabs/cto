//! Intake orchestrator - Complete intake workflow from PRD to documentation.
//!
//! This module provides a unified intake process that:
//! 1. Parses PRD to generate tasks
//! 2. Optionally analyzes task complexity
//! 3. Optionally expands tasks into subtasks
//! 4. Adds agent routing hints
//! 5. Generates per-task documentation (XML, MD, acceptance criteria)
//! 6. Saves tasks.json

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::ai::schemas::ComplexityReport;
use crate::entities::TaskStatus;
use crate::errors::{TasksError, TasksResult};
use crate::storage::Storage;

use super::docs::{generate_all_docs, DocsGenerationResult};
use super::routing::infer_agent_hint_str;
use super::AIDomain;

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

        // 1. Read PRD content
        let prd_content = tokio::fs::read_to_string(&config.prd_path)
            .await
            .map_err(|e| TasksError::FileReadError {
                path: config.prd_path.display().to_string(),
                reason: e.to_string(),
            })?;

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
        eprintln!("\n📋 Step 1/4: Parsing PRD to generate ~{} tasks...", config.num_tasks);
        tracing::info!("Parsing PRD to generate ~{} tasks...", config.num_tasks);
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

        eprintln!("  ✅ Generated {} tasks", tasks.len());
        tracing::info!("Generated {} tasks", tasks.len());

        // 4. Analyze complexity if requested
        let complexity_report = if config.analyze {
            eprintln!("\n📊 Step 2/4: Analyzing task complexity...");
            tracing::info!("Analyzing task complexity...");
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

            eprintln!("  ✅ Complexity analysis complete");
            Some(report)
        } else {
            eprintln!("\n⏭️  Step 2/4: Skipping complexity analysis");
            None
        };

        // 5. Expand tasks into subtasks if requested
        let mut subtasks_count = 0;
        if config.expand {
            let tasks_to_expand: Vec<_> = tasks.iter()
                .filter(|t| t.status != TaskStatus::Done && t.subtasks.is_empty())
                .map(|t| t.id.clone())
                .collect();
            
            eprintln!("\n🔄 Step 3/4: Expanding {} tasks into subtasks...", tasks_to_expand.len());
            tracing::info!("Expanding tasks into subtasks...");

            for (idx, task) in tasks.iter_mut().enumerate() {
                // Skip done tasks
                if task.status == TaskStatus::Done {
                    continue;
                }

                // Skip if already has subtasks
                if !task.subtasks.is_empty() {
                    continue;
                }

                eprint!("  → Task {}/{}: {}... ", idx + 1, tasks_to_expand.len(), task.id);
                
                match self
                    .ai_domain
                    .expand_task(
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
                        eprintln!("{} subtasks", subtasks.len());
                        subtasks_count += subtasks.len();
                        task.subtasks = subtasks;
                        total_input_tokens += expand_usage.input_tokens;
                        total_output_tokens += expand_usage.output_tokens;
                    }
                    Err(e) => {
                        eprintln!("⚠️  failed: {}", e);
                        tracing::warn!("Failed to expand task {}: {}", task.id, e);
                    }
                }
            }
            eprintln!("  ✅ Generated {} subtasks total", subtasks_count);
        } else {
            eprintln!("\n⏭️  Step 3/4: Skipping task expansion");
        }

        // 6. Add agent routing hints
        eprintln!("\n🏷️  Adding agent routing hints...");
        tracing::info!("Adding agent routing hints...");
        for task in &mut tasks {
            if task.agent_hint.is_none() {
                task.agent_hint =
                    Some(infer_agent_hint_str(&task.title, &task.description).to_string());
            }
        }
        eprintln!("  ✅ Agent hints assigned");

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
        eprintln!("\n📄 Step 4/4: Generating per-task documentation...");
        tracing::info!("Generating per-task documentation...");
        let docs_dir = config.output_dir.join("docs");
        let docs_result = generate_all_docs(&tasks, &docs_dir).await?;

        eprintln!("  ✅ Created {} task documentation directories", docs_result.task_dirs_created);
        
        eprintln!("\n════════════════════════════════════════════════════════════════");
        eprintln!("✅ Intake Complete!");
        eprintln!("════════════════════════════════════════════════════════════════");
        eprintln!("Summary:");
        eprintln!("  • Tasks generated: {}", tasks.len());
        eprintln!("  • Subtasks generated: {}", subtasks_count);
        eprintln!("  • Documentation dirs: {}", docs_result.task_dirs_created);
        eprintln!("  • Tokens used: {} in, {} out", total_input_tokens, total_output_tokens);
        eprintln!();
        
        tracing::info!(
            "Intake complete: {} tasks, {} subtasks, {} doc dirs",
            tasks.len(),
            subtasks_count,
            docs_result.task_dirs_created
        );

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
