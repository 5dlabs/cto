//! Intake CLI - Two-session workflow for AI-driven development.
//!
//! Session 1: Task planning (PRD -> tasks.json)
//! Session 2: Prompt generation (tasks -> per-task prompts)

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::disallowed_macros)]
#![allow(clippy::uninlined_format_args)]

use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use colored::Colorize;
use config::CtoConfig;

use intake::ai::schemas::ComplexityReport;
use intake::domain::{
    docs::generate_all_docs, infer_agent_hint_with_deps_str, AIDomain, DependencyDomain,
    IntakeConfig, IntakeDomain, TasksDomain,
};
use intake::entities::TaskStatus;
use intake::errors::TasksError;
use intake::storage::{FileStorage, Storage};
use intake::ui;

#[derive(Parser)]
#[command(name = "intake")]
#[command(about = "Intake workflow: PRD to tasks to prompts for AI-driven development", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Project root directory
    #[arg(long, global = true)]
    project: Option<PathBuf>,

    /// Use CLI mode instead of API mode for AI operations.
    /// When enabled, uses external CLI tools (claude, codex, cursor, etc.)
    /// instead of direct API calls.
    #[arg(long = "use-cli", global = true, env = "TASKS_USE_CLI")]
    use_external_provider: bool,

    /// CLI type to use when --use-cli is enabled.
    /// Options: claude, codex, cursor, factory, opencode, gemini, dexter
    #[arg(
        long = "cli-type",
        global = true,
        env = "TASKS_CLI",
        default_value = "claude"
    )]
    provider: String,

    /// Enable verbose output (debug level logging)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable multi-model collaboration (critic/validator pattern).
    /// When enabled, uses one model for generation and another for critique.
    #[arg(long = "multi-model", global = true, env = "TASKS_MULTI_MODEL")]
    multi_model: bool,

    /// Generator provider for multi-model mode (claude, minimax, codex).
    #[arg(long = "generator", global = true, env = "MULTI_MODEL_GENERATOR", default_value = "claude")]
    generator: String,

    /// Critic provider for multi-model mode (claude, minimax, codex).
    #[arg(long = "critic", global = true, env = "MULTI_MODEL_CRITIC", default_value = "minimax")]
    critic: String,
}

#[derive(Subcommand)]
enum Commands {
    /// List all tasks
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,

        /// Include subtasks
        #[arg(long)]
        with_subtasks: bool,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Show details of a specific task
    Show {
        /// Task ID(s), comma-separated
        id: String,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Show the next task to work on
    Next {
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Manage dependencies
    #[command(subcommand)]
    Deps(DepsCommands),

    // =========== AI-Powered Commands ===========
    /// Parse a PRD file and generate tasks
    ParsePrd {
        /// Path to the PRD file
        file: PathBuf,

        /// Number of tasks to generate (0 = auto)
        #[arg(short, long, default_value = "10")]
        num_tasks: i32,

        /// Use research mode for better results
        #[arg(short, long)]
        research: bool,

        /// AI model to use
        #[arg(long)]
        model: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Expand a task into subtasks using AI
    Expand {
        /// Task ID to expand
        #[arg(short, long)]
        id: String,

        /// Number of subtasks to generate
        #[arg(short, long)]
        num: Option<i32>,

        /// Use research mode
        #[arg(short, long)]
        research: bool,

        /// Force replace existing subtasks
        #[arg(short, long)]
        force: bool,

        /// AI model to use
        #[arg(long)]
        model: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Expand all pending tasks
    ExpandAll {
        /// Number of subtasks per task
        #[arg(short, long)]
        num: Option<i32>,

        /// Use research mode
        #[arg(short, long)]
        research: bool,

        /// Force replace existing subtasks
        #[arg(short, long)]
        force: bool,

        /// AI model to use
        #[arg(long)]
        model: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Analyze task complexity
    AnalyzeComplexity {
        /// Complexity threshold (1-10)
        #[arg(short, long, default_value = "5")]
        threshold: i32,

        /// Use research mode
        #[arg(short, long)]
        research: bool,

        /// Output file for report
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// AI model to use
        #[arg(long)]
        model: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// View complexity report
    ComplexityReport {
        /// Report file path
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Generate individual task files
    Generate {
        /// Output directory (default: same as tasks file)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Run complete intake workflow: PRD → tasks → docs
    Intake {
        /// Path to the PRD file
        #[arg(long, default_value = ".tasks/docs/prd.txt")]
        prd: PathBuf,

        /// Path to the architecture file (optional)
        #[arg(long)]
        architecture: Option<PathBuf>,

        /// Number of tasks to generate (0 = auto)
        #[arg(short, long, default_value = "15")]
        num_tasks: i32,

        /// Skip task expansion into subtasks
        #[arg(long)]
        no_expand: bool,

        /// Skip complexity analysis
        #[arg(long)]
        no_analyze: bool,

        /// Complexity threshold (1-10) for expansion
        #[arg(long, default_value = "5")]
        threshold: i32,

        /// Use research mode for AI operations
        #[arg(short, long)]
        research: bool,

        /// AI model to use
        #[arg(long)]
        model: Option<String>,

        /// Output directory (default: .tasks)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target repository (e.g., 5dlabs/my-project) - reads from `REPOSITORY` env if not provided
        #[arg(long, env = "REPOSITORY")]
        repository: Option<String>,

        /// Service name - reads from `SERVICE` env if not provided
        #[arg(long, env = "SERVICE")]
        service: Option<String>,

        /// Docs repository URL - reads from `DOCS_REPOSITORY` env if not provided
        #[arg(long, env = "DOCS_REPOSITORY")]
        docs_repository: Option<String>,

        /// Project directory within docs repo - reads from `PROJECT_NAME` env if not provided
        #[arg(long, env = "PROJECT_NAME")]
        docs_project_directory: Option<String>,
    },

    /// Generate cto-config.json with per-agent tool configurations
    GenerateConfig {
        /// Target repository (e.g., 5dlabs/my-project) - reads from `REPOSITORY` env if not provided
        #[arg(long, env = "REPOSITORY")]
        repository: Option<String>,

        /// Service name - reads from `SERVICE` env if not provided
        #[arg(long, env = "SERVICE")]
        service: Option<String>,

        /// Docs repository - reads from `DOCS_REPOSITORY` env if not provided
        #[arg(long, env = "DOCS_REPOSITORY")]
        docs_repository: Option<String>,

        /// Project directory - reads from `PROJECT_NAME` env if not provided
        #[arg(long, env = "PROJECT_NAME")]
        docs_project_directory: Option<String>,

        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Generate documentation (XML, MD) for all tasks
    GenerateDocs {
        /// Output directory for docs (default: .tasks/docs)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    // =========== Session 2: Prompt Generation Commands ===========
    /// Split tasks.json into individual task-N.json files
    SplitTasks {
        /// Path to tasks.json file (default: .tasks/tasks/tasks.json)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output directory for individual task files (default: .tasks/tasks/)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Sync a task from a Linear issue
    SyncTask {
        /// Linear issue ID (e.g., "TSK-123" or the UUID)
        #[arg(long)]
        issue_id: String,

        /// Project name/identifier
        #[arg(long, default_value = "")]
        project_name: String,

        /// Local task ID to update (defaults to Linear issue identifier)
        #[arg(long)]
        task_id: Option<String>,

        /// Linear API token (reads from `LINEAR_API_KEY` env if not provided)
        #[arg(long, env = "LINEAR_API_KEY")]
        linear_token: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Generate prompts for tasks using AI (Session 2)
    GeneratePrompts {
        /// Process specific task file only
        #[arg(long)]
        task_file: Option<PathBuf>,

        /// Process all task-*.json files in directory
        #[arg(long)]
        all: bool,

        /// Tasks directory containing task-*.json files (default: .tasks/tasks/)
        #[arg(long)]
        tasks_dir: Option<PathBuf>,

        /// Include PRD in context
        #[arg(long, default_value = "true")]
        include_prd: bool,

        /// Include architecture doc in context
        #[arg(long, default_value = "true")]
        include_arch: bool,

        /// Generate code-examples.md for each task
        #[arg(long)]
        with_examples: bool,

        /// Enable task-specific research via MCP tools
        #[arg(long, default_value = "true")]
        research: bool,

        /// Path to PRD file (default: .tasks/docs/prd.txt)
        #[arg(long)]
        prd_path: Option<PathBuf>,

        /// Path to architecture doc (default: .tasks/docs/architecture.md)
        #[arg(long)]
        arch_path: Option<PathBuf>,

        /// Output directory for generated prompts (default: .tasks/docs/)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// CLI to use (claude, cursor, codex)
        #[arg(long, default_value = "claude")]
        cli: String,

        /// Model to use
        #[arg(long)]
        model: Option<String>,

        /// MCP config path
        #[arg(long)]
        mcp_config: Option<PathBuf>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
}

#[derive(Subcommand)]
enum DepsCommands {
    /// Add a dependency
    Add {
        /// Task ID
        #[arg(short, long)]
        id: String,

        /// Dependency task ID
        #[arg(short, long)]
        depends_on: String,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Remove a dependency
    Remove {
        /// Task ID
        #[arg(short, long)]
        id: String,

        /// Dependency task ID
        #[arg(short, long)]
        depends_on: String,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Validate dependencies
    Validate {
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Fix invalid dependencies
    Fix {
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
}

fn get_project_path(cli_path: Option<PathBuf>) -> PathBuf {
    cli_path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[tokio::main]
async fn main() {
    // Parse CLI args first so we can use --verbose flag
    let cli = Cli::parse();

    // Initialize tracing based on --verbose flag or RUST_LOG env var
    // Use compact format without timestamps/targets to match ui::print_info style
    let default_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(default_level.into()),
        )
        .without_time()
        .with_target(cli.verbose) // Show target module when verbose
        .with_level(cli.verbose) // Show log level when verbose
        .init();

    if let Err(e) = run(cli).await {
        ui::print_error(&e.to_string());
        std::process::exit(1);
    }
}

/// Load the model configuration for a CLI type from cto-config.json.
///
/// Searches for cto-config.json in the project path and its parents.
/// Returns the CLI-specific model if configured, otherwise the primary model.
fn load_cli_model_from_config(project_path: &std::path::Path, cli_type: &str) -> Option<String> {
    // Search for cto-config.json in project path and parents
    let config_paths = [
        project_path.join("cto-config.json"),
        project_path.join(".tasks/cto-config.json"),
    ];

    for config_path in &config_paths {
        if config_path.exists() {
            match std::fs::read_to_string(config_path) {
                Ok(content) => match CtoConfig::from_json(&content) {
                    Ok(config) => {
                        let model = config.defaults.intake.models.get_model_for_cli(cli_type);
                        if !model.is_empty() {
                            return Some(model.to_string());
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            path = %config_path.display(),
                            error = %e,
                            "Failed to parse cto-config.json"
                        );
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        path = %config_path.display(),
                        error = %e,
                        "Failed to read cto-config.json"
                    );
                }
            }
        }
    }

    None
}

/// Load multi-model configuration from cto-config.json.
///
/// If multiModel is enabled in the config, sets the appropriate environment variables.
/// This allows enabling multi-model via config without CLI flags.
fn load_multi_model_from_config(project_path: &std::path::Path) {
    // Search for cto-config.json in project path and its parents
    let config_paths = [
        project_path.join("cto-config.json"),
        project_path.join(".tasks/cto-config.json"),
    ];

    for config_path in &config_paths {
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(config_path) {
                if let Ok(config) = CtoConfig::from_json(&content) {
                    let mm = &config.defaults.intake.multi_model;
                    if mm.enabled {
                        std::env::set_var("TASKS_MULTI_MODEL", "true");
                        std::env::set_var("MULTI_MODEL_GENERATOR", &mm.generator);
                        std::env::set_var("MULTI_MODEL_CRITIC", &mm.critic);
                        std::env::set_var(
                            "MULTI_MODEL_MAX_REFINEMENTS",
                            mm.max_refinements.to_string(),
                        );
                        std::env::set_var(
                            "MULTI_MODEL_CRITIC_THRESHOLD",
                            mm.critic_threshold.to_string(),
                        );
                        tracing::info!(
                            generator = %mm.generator,
                            critic = %mm.critic,
                            max_refinements = mm.max_refinements,
                            critic_threshold = mm.critic_threshold,
                            "Multi-model collaboration enabled (cto-config.json)"
                        );
                        return;
                    }
                }
            }
        }
    }
}

async fn run(cli: Cli) -> Result<(), TasksError> {
    // Apply CLI flags to environment variables so they're picked up by the provider registry
    // The registry checks TASKS_USE_CLI and TASKS_CLI when creating default providers
    if cli.use_external_provider {
        std::env::set_var("TASKS_USE_CLI", "true");
    }
    std::env::set_var("TASKS_CLI", &cli.provider);

    let project_path = get_project_path(cli.project.clone());

    // Apply multi-model configuration
    // Priority: CLI flags > cto-config.json
    if cli.multi_model {
        std::env::set_var("TASKS_MULTI_MODEL", "true");
        std::env::set_var("MULTI_MODEL_GENERATOR", &cli.generator);
        std::env::set_var("MULTI_MODEL_CRITIC", &cli.critic);
        tracing::info!(
            generator = %cli.generator,
            critic = %cli.critic,
            "Multi-model collaboration enabled (CLI)"
        );
    } else {
        // Try loading from cto-config.json
        load_multi_model_from_config(&project_path);
    }

    // Load cto-config.json to get CLI-specific model configuration
    let config_model = load_cli_model_from_config(&project_path, &cli.provider);
    if let Some(model) = &config_model {
        std::env::set_var("TASKS_MODEL", model);
        tracing::info!(cli = %cli.provider, model = %model, "Using model from cto-config.json");
    }

    let storage = Arc::new(FileStorage::new(&project_path));
    let tasks_domain = TasksDomain::new(Arc::clone(&storage) as Arc<dyn intake::storage::Storage>);
    let deps_domain =
        DependencyDomain::new(Arc::clone(&storage) as Arc<dyn intake::storage::Storage>);

    match cli.command {
        Commands::List {
            status,
            with_subtasks,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let status_filter = if let Some(s) = status {
                Some(s.parse::<TaskStatus>()?)
            } else {
                None
            };

            let tasks = tasks_domain
                .list_tasks(tag.as_deref(), status_filter)
                .await?;

            if tasks.is_empty() {
                ui::print_info("No tasks found");
            } else {
                let table = ui::task_table(&tasks, with_subtasks);
                println!("{table}");
                println!();
                ui::print_info(&format!("{} task(s) total", tasks.len()));
            }
        }

        Commands::Show { id, tag } => {
            check_initialized(&tasks_domain).await?;

            let ids: Vec<&str> = id.split(',').map(str::trim).collect();

            for task_id in ids {
                let task = tasks_domain.get_task(task_id, tag.as_deref()).await?;
                ui::display_task_details(&task);
            }
        }

        Commands::Next { tag } => {
            check_initialized(&tasks_domain).await?;

            if let Some(task) = tasks_domain.next_task(tag.as_deref()).await? {
                ui::print_success(&format!("Next task: {} - {}", task.id, task.title));
                println!();
                ui::display_task_details(&task);
            } else {
                ui::print_info("No pending tasks available");
            }
        }

        Commands::Deps(deps_cmd) => {
            check_initialized(&tasks_domain).await?;

            match deps_cmd {
                DepsCommands::Add {
                    id,
                    depends_on,
                    tag,
                } => {
                    deps_domain
                        .add_dependency(&id, &depends_on, tag.as_deref())
                        .await?;
                    ui::print_success(&format!("Added dependency: {} -> {}", id, depends_on));
                }

                DepsCommands::Remove {
                    id,
                    depends_on,
                    tag,
                } => {
                    deps_domain
                        .remove_dependency(&id, &depends_on, tag.as_deref())
                        .await?;
                    ui::print_success(&format!("Removed dependency: {} -> {}", id, depends_on));
                }

                DepsCommands::Validate { tag } => {
                    let result = deps_domain.validate(tag.as_deref()).await?;

                    if result.is_valid {
                        ui::print_success("All dependencies are valid");
                    } else {
                        ui::print_error("Dependency issues found:");

                        for invalid in &result.invalid_deps {
                            println!(
                                "  {} Task {} depends on missing task {}",
                                "•".red(),
                                invalid.task_id,
                                invalid.dep_id
                            );
                        }

                        for cycle in &result.cycles {
                            println!("  {} Cycle detected: {}", "•".red(), cycle.join(" -> "));
                        }

                        println!();
                        ui::print_info("Run 'tasks deps fix' to auto-fix invalid dependencies");
                    }
                }

                DepsCommands::Fix { tag } => {
                    let fixed = deps_domain.fix(tag.as_deref()).await?;
                    ui::print_success(&format!("Fixed {} invalid dependency(ies)", fixed));
                }
            }
        }

        // =========== AI-Powered Command Handlers ===========
        Commands::ParsePrd {
            file,
            num_tasks,
            research,
            model,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let ai_domain =
                AIDomain::new(Arc::clone(&storage) as Arc<dyn intake::storage::Storage>);

            // Read PRD file
            let prd_content =
                tokio::fs::read_to_string(&file)
                    .await
                    .map_err(|e| TasksError::FileReadError {
                        path: file.display().to_string(),
                        reason: e.to_string(),
                    })?;

            ui::print_info(&format!("Parsing PRD: {}", file.display()));
            ui::print_info("This may take a moment...");

            let (tasks, usage) = ai_domain
                .parse_prd(
                    &prd_content,
                    file.to_str().unwrap_or(""),
                    Some(num_tasks),
                    research,
                    model.as_deref(),
                )
                .await?;

            // Save tasks
            for task in &tasks {
                tasks_domain
                    .add_task_full(task.clone(), tag.as_deref())
                    .await?;
            }

            ui::print_success(&format!("Generated {} tasks from PRD", tasks.len()));
            ui::print_info(&format!(
                "Tokens used: {} in, {} out",
                usage.input_tokens, usage.output_tokens
            ));

            // Display tasks
            let table = ui::task_table(&tasks, false);
            println!("{table}");
        }

        Commands::Expand {
            id,
            num,
            research,
            force,
            model,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let ai_domain =
                AIDomain::new(Arc::clone(&storage) as Arc<dyn intake::storage::Storage>);

            let mut task = tasks_domain.get_task(&id, tag.as_deref()).await?;

            if !task.subtasks.is_empty() && !force {
                ui::print_warning(&format!(
                    "Task {} already has {} subtask(s). Use --force to replace.",
                    id,
                    task.subtasks.len()
                ));
                return Ok(());
            }

            if force {
                task.subtasks.clear();
            }

            ui::print_info(&format!("Expanding task: {} - {}", task.id, task.title));
            ui::print_info("This may take a moment...");

            // Try to load complexity report
            let report_path = project_path.join(".tasks").join("complexity-report.json");
            let complexity_report = if report_path.exists() {
                let content = tokio::fs::read_to_string(&report_path).await.ok();
                content.and_then(|c| serde_json::from_str::<ComplexityReport>(&c).ok())
            } else {
                None
            };

            let (subtasks, usage) = ai_domain
                .expand_task(
                    &task,
                    num,
                    research,
                    None,
                    complexity_report.as_ref(),
                    model.as_deref(),
                )
                .await?;

            task.subtasks = subtasks;
            tasks_domain.update_task(&task, tag.as_deref()).await?;

            ui::print_success(&format!(
                "Added {} subtasks to task {}",
                task.subtasks.len(),
                id
            ));
            ui::print_info(&format!(
                "Tokens used: {} in, {} out",
                usage.input_tokens, usage.output_tokens
            ));
        }

        Commands::ExpandAll {
            num,
            research,
            force,
            model,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let ai_domain =
                AIDomain::new(Arc::clone(&storage) as Arc<dyn intake::storage::Storage>);

            let tasks = tasks_domain
                .list_tasks(tag.as_deref(), Some(TaskStatus::Pending))
                .await?;

            if tasks.is_empty() {
                ui::print_info("No pending tasks to expand");
                return Ok(());
            }

            // Load complexity report
            let report_path = project_path.join(".tasks").join("complexity-report.json");
            let complexity_report = if report_path.exists() {
                let content = tokio::fs::read_to_string(&report_path).await.ok();
                content.and_then(|c| serde_json::from_str::<ComplexityReport>(&c).ok())
            } else {
                None
            };

            let mut expanded_count = 0;
            let mut total_input = 0u32;
            let mut total_output = 0u32;

            for mut task in tasks {
                if !task.subtasks.is_empty() && !force {
                    ui::print_info(&format!("Skipping task {} (already has subtasks)", task.id));
                    continue;
                }

                if force {
                    task.subtasks.clear();
                }

                ui::print_info(&format!("Expanding: {} - {}", task.id, task.title));

                match ai_domain
                    .expand_task(
                        &task,
                        num,
                        research,
                        None,
                        complexity_report.as_ref(),
                        model.as_deref(),
                    )
                    .await
                {
                    Ok((subtasks, usage)) => {
                        task.subtasks = subtasks;
                        tasks_domain.update_task(&task, tag.as_deref()).await?;
                        expanded_count += 1;
                        total_input += usage.input_tokens;
                        total_output += usage.output_tokens;
                    }
                    Err(e) => {
                        ui::print_error(&format!("Failed to expand task {}: {}", task.id, e));
                    }
                }
            }

            ui::print_success(&format!("Expanded {} task(s)", expanded_count));
            ui::print_info(&format!(
                "Total tokens: {} in, {} out",
                total_input, total_output
            ));
        }

        Commands::AnalyzeComplexity {
            threshold,
            research,
            output,
            model,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let ai_domain =
                AIDomain::new(Arc::clone(&storage) as Arc<dyn intake::storage::Storage>);

            let tasks = tasks_domain.list_tasks(tag.as_deref(), None).await?;

            if tasks.is_empty() {
                ui::print_info("No tasks to analyze");
                return Ok(());
            }

            ui::print_info(&format!("Analyzing complexity of {} tasks...", tasks.len()));
            ui::print_info("This may take a moment...");

            let (report, usage) = ai_domain
                .analyze_complexity(&tasks, threshold, research, model.as_deref())
                .await?;

            // Save report
            let output_path = output
                .unwrap_or_else(|| project_path.join(".tasks").join("complexity-report.json"));

            let report_json = serde_json::to_string_pretty(&report)?;
            tokio::fs::write(&output_path, &report_json)
                .await
                .map_err(|e| TasksError::FileWriteError {
                    path: output_path.display().to_string(),
                    reason: e.to_string(),
                })?;

            ui::print_success(&format!(
                "Saved complexity report to: {}",
                output_path.display()
            ));
            ui::print_info(&format!(
                "Tokens used: {} in, {} out",
                usage.input_tokens, usage.output_tokens
            ));

            // Display summary
            let needing_expansion = report.tasks_needing_expansion();
            if needing_expansion.is_empty() {
                ui::print_info("No tasks require expansion");
            } else {
                println!();
                println!("{}", "Tasks needing expansion:".bold());
                for analysis in needing_expansion {
                    println!(
                        "  {} Task {}: {} (score: {}, suggested: {} subtasks)",
                        "•".cyan(),
                        analysis.task_id,
                        analysis.task_title,
                        analysis.complexity_score,
                        analysis.recommended_subtasks
                    );
                }
            }
        }

        Commands::ComplexityReport { file } => {
            check_initialized(&tasks_domain).await?;

            let report_path =
                file.unwrap_or_else(|| project_path.join(".tasks").join("complexity-report.json"));

            if !report_path.exists() {
                ui::print_error(&format!(
                    "Complexity report not found: {}",
                    report_path.display()
                ));
                ui::print_info("Run 'intake analyze-complexity' first to generate a report");
                return Ok(());
            }

            let content = tokio::fs::read_to_string(&report_path).await.map_err(|e| {
                TasksError::FileReadError {
                    path: report_path.display().to_string(),
                    reason: e.to_string(),
                }
            })?;

            let report: ComplexityReport = serde_json::from_str(&content)?;

            println!("{}", "Complexity Report".bold().underline());
            println!();
            println!("Generated: {}", report.meta.generated_at);
            println!("Model: {}", report.meta.model);
            println!("Threshold: {}", report.meta.threshold);
            println!();

            for analysis in &report.complexity_analysis {
                let score_color = if analysis.complexity_score >= report.meta.threshold {
                    "red"
                } else if analysis.complexity_score >= 5 {
                    "yellow"
                } else {
                    "green"
                };

                let score_display = match score_color {
                    "red" => format!("{}", analysis.complexity_score).red(),
                    "yellow" => format!("{}", analysis.complexity_score).yellow(),
                    _ => format!("{}", analysis.complexity_score).green(),
                };

                println!(
                    "Task {}: {} [{}]",
                    analysis.task_id, analysis.task_title, score_display
                );
                println!("  Reasoning: {}", analysis.reasoning);
                if analysis.complexity_score >= report.meta.threshold {
                    println!(
                        "  {} Recommended subtasks: {}",
                        "→".cyan(),
                        analysis.recommended_subtasks
                    );
                }
                println!();
            }
        }

        Commands::Generate { output, tag } => {
            check_initialized(&tasks_domain).await?;

            let tasks = tasks_domain.list_tasks(tag.as_deref(), None).await?;

            if tasks.is_empty() {
                ui::print_info("No tasks to generate files for");
                return Ok(());
            }

            // Determine output directory
            let output_dir = output.unwrap_or_else(|| project_path.join(".tasks").join("tasks"));
            tokio::fs::create_dir_all(&output_dir).await?;

            let mut generated = 0;
            for task in &tasks {
                // Zero-pad task ID to 3 digits for proper sorting
                let padded_id = format!("{:0>3}", task.id);
                let task_file = output_dir.join(format!("task-{padded_id}.md"));

                // Generate markdown content
                let mut content = String::new();
                writeln!(content, "# Task {}: {}\n", task.id, task.title).ok();
                writeln!(content, "**Status:** {}", task.status).ok();
                writeln!(content, "**Priority:** {}", task.priority).ok();

                if !task.dependencies.is_empty() {
                    writeln!(
                        content,
                        "**Dependencies:** {}",
                        task.dependencies.join(", ")
                    )
                    .ok();
                }
                content.push('\n');

                content.push_str("## Description\n\n");
                content.push_str(&task.description);
                content.push_str("\n\n");

                if !task.details.is_empty() {
                    content.push_str("## Implementation Details\n\n");
                    content.push_str(&task.details);
                    content.push_str("\n\n");
                }

                if !task.test_strategy.is_empty() {
                    content.push_str("## Test Strategy\n\n");
                    content.push_str(&task.test_strategy);
                    content.push_str("\n\n");
                }

                // Include subtasks if any
                if !task.subtasks.is_empty() {
                    content.push_str("## Subtasks\n\n");
                    for subtask in &task.subtasks {
                        writeln!(
                            content,
                            "- [ ] **{}.{}** {} ({})",
                            task.id, subtask.id, subtask.title, subtask.status
                        )
                        .ok();
                        if !subtask.description.is_empty() {
                            writeln!(content, "  - {}", subtask.description).ok();
                        }
                    }
                    content.push('\n');
                }

                tokio::fs::write(&task_file, &content).await.map_err(|e| {
                    TasksError::FileWriteError {
                        path: task_file.display().to_string(),
                        reason: e.to_string(),
                    }
                })?;

                generated += 1;
            }

            ui::print_success(&format!(
                "Generated {} task file(s) in {}",
                generated,
                output_dir.display()
            ));
        }

        Commands::Intake {
            prd,
            architecture,
            num_tasks,
            no_expand,
            no_analyze,
            threshold,
            research,
            model,
            output,
            repository,
            service,
            docs_repository,
            docs_project_directory,
        } => {
            // Initialize if not already
            if !tasks_domain.is_initialized().await? {
                tasks_domain.init().await?;
                ui::print_info("Initialized tasks structure");
            }

            // Verify PRD exists
            if !prd.exists() {
                return Err(TasksError::FileReadError {
                    path: prd.display().to_string(),
                    reason: "PRD file not found".to_string(),
                });
            }

            let output_dir = output.unwrap_or_else(|| project_path.join(".tasks"));

            // Check for auto-append deploy task from cto-config.json
            let config_paths = [
                project_path.join("cto-config.json"),
                project_path.join(".tasks/cto-config.json"),
            ];
            let mut auto_append_deploy = false;
            for config_path in &config_paths {
                if config_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(config_path) {
                        if let Ok(cto_config) = CtoConfig::from_json(&content) {
                            auto_append_deploy = cto_config.defaults.intake.auto_append_deploy_task;
                            break;
                        }
                    }
                }
            }

            let config = IntakeConfig {
                prd_path: prd,
                architecture_path: architecture,
                num_tasks,
                expand: !no_expand,
                analyze: !no_analyze,
                complexity_threshold: threshold,
                research,
                model,
                output_dir: output_dir.clone(),
                repository,
                service,
                docs_repository,
                docs_project_directory,
                auto_append_deploy_task: auto_append_deploy,
            };

            ui::print_info("Starting intake workflow...");
            ui::print_info(&format!("  PRD: {}", config.prd_path.display()));
            if let Some(arch) = &config.architecture_path {
                ui::print_info(&format!("  Architecture: {}", arch.display()));
            }
            ui::print_info(&format!("  Target tasks: ~{}", config.num_tasks));
            ui::print_info(&format!("  Expand: {}", config.expand));
            ui::print_info(&format!("  Analyze: {}", config.analyze));

            let intake_domain =
                IntakeDomain::new(Arc::clone(&storage) as Arc<dyn intake::storage::Storage>);
            let result = intake_domain.run(&config).await?;

            println!();
            ui::print_success("Intake complete!");
            println!();
            println!("{}", "Summary:".bold());
            println!("  {} Tasks generated: {}", "•".cyan(), result.tasks_count);
            println!(
                "  {} Subtasks generated: {}",
                "•".cyan(),
                result.subtasks_count
            );
            println!(
                "  {} Documentation dirs: {}",
                "•".cyan(),
                result.docs_result.task_dirs_created
            );
            println!(
                "  {} Tokens used: {} in, {} out",
                "•".cyan(),
                result.total_input_tokens,
                result.total_output_tokens
            );
            println!();
            ui::print_info(&format!("Tasks saved to: {}", result.tasks_file.display()));
            ui::print_info(&format!(
                "Docs saved to: {}",
                output_dir.join("docs").display()
            ));
        }

        Commands::GenerateConfig {
            repository,
            service,
            docs_repository,
            docs_project_directory,
            output,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let mut tasks = tasks_domain.list_tasks(tag.as_deref(), None).await?;

            if tasks.is_empty() {
                ui::print_info("No tasks to generate config for");
                return Ok(());
            }

            // Apply agent hints with validation and override capability
            // ALWAYS re-validate AI hints - they may be wrong
            // Route ALL tasks through content-based inference
            let mut hints_modified = 0;
            let tasks_snapshot = tasks.clone();

            // Apply routing to all tasks - FAIL if any can't be routed
            let mut unroutable: Vec<String> = Vec::new();
            for task in &mut tasks {
                match infer_agent_hint_with_deps_str(task, &tasks_snapshot) {
                    Some(inferred) => {
                        if task.agent_hint.as_deref() != Some(inferred) {
                            task.agent_hint = Some(inferred.to_string());
                            hints_modified += 1;
                        }
                    }
                    None => {
                        unroutable.push(format!(
                            "Task {} '{}': {}",
                            task.id, task.title, task.description
                        ));
                    }
                }
            }

            // Fail if any tasks couldn't be routed
            if !unroutable.is_empty() {
                ui::print_error(&format!(
                    "Cannot determine agent for {} task(s):",
                    unroutable.len()
                ));
                for task_info in &unroutable {
                    println!("  {}", task_info);
                }
                return Err(TasksError::ValidationError {
                    field: "agent_hint".to_string(),
                    reason: "Add routing keywords or explicit agent hints to these tasks"
                        .to_string(),
                });
            }

            if hints_modified > 0 {
                ui::print_info(&format!(
                    "Applied/corrected agent hints for {} tasks",
                    hints_modified
                ));
                // Save the updated tasks back
                storage.save_tasks(&tasks, tag.as_deref()).await?;
            }

            // Check for auto-append deploy task from cto-config.json
            let config_paths = [
                project_path.join("cto-config.json"),
                project_path.join(".tasks/cto-config.json"),
            ];
            let mut auto_append_deploy = false;
            for config_path in &config_paths {
                if config_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(config_path) {
                        if let Ok(config) = CtoConfig::from_json(&content) {
                            auto_append_deploy = config.defaults.intake.auto_append_deploy_task;
                            break;
                        }
                    }
                }
            }

            // Auto-append deploy task if configured
            if auto_append_deploy {
                if intake::domain::has_deploy_task(&tasks) {
                    ui::print_info("Deploy task already exists, skipping auto-append");
                } else {
                    ui::print_info("Auto-appending deploy task (depends on all other tasks)");
                    let deploy_task = intake::domain::create_deploy_task(&tasks);
                    tasks.push(deploy_task);
                    // Save the updated tasks
                    storage.save_tasks(&tasks, tag.as_deref()).await?;
                }
            }

            let repository = repository.unwrap_or_else(|| "unknown/unknown".to_string());
            let service = service.unwrap_or_else(|| "unknown".to_string());
            let docs_repository = docs_repository.unwrap_or_else(|| repository.clone());
            let docs_project_directory = docs_project_directory.unwrap_or_else(|| service.clone());

            ui::print_info(&format!(
                "Generating cto-config.json for {} tasks...",
                tasks.len()
            ));
            ui::print_info(&format!("  Repository: {}", repository));
            ui::print_info(&format!("  Service: {}", service));

            let cto_config = intake::domain::generate_cto_config(
                &tasks,
                &repository,
                &service,
                &docs_repository,
                &docs_project_directory,
            );

            let output_dir = output.unwrap_or_else(|| project_path.clone());

            intake::domain::save_cto_config(&cto_config, &output_dir).await?;

            ui::print_success(&format!(
                "Generated cto-config.json with {} agent configurations",
                cto_config.agents.len()
            ));
            for name in cto_config.agents.keys() {
                println!("  {} Agent: {}", "•".cyan(), name);
            }
            ui::print_info(&format!(
                "Output: {}",
                output_dir.join("cto-config.json").display()
            ));
        }

        Commands::GenerateDocs { output, tag } => {
            check_initialized(&tasks_domain).await?;

            let mut tasks = tasks_domain.list_tasks(tag.as_deref(), None).await?;

            if tasks.is_empty() {
                ui::print_info("No tasks to generate documentation for");
                return Ok(());
            }

            // Apply agent hints with validation and override capability
            // ALWAYS re-validate AI hints - they may be wrong
            // Route ALL tasks through content-based inference
            let mut hints_modified = 0;
            let tasks_snapshot = tasks.clone();

            // Apply routing to all tasks - FAIL if any can't be routed
            let mut unroutable: Vec<String> = Vec::new();
            for task in &mut tasks {
                match infer_agent_hint_with_deps_str(task, &tasks_snapshot) {
                    Some(inferred) => {
                        if task.agent_hint.as_deref() != Some(inferred) {
                            task.agent_hint = Some(inferred.to_string());
                            hints_modified += 1;
                        }
                    }
                    None => {
                        unroutable.push(format!(
                            "Task {} '{}': {}",
                            task.id, task.title, task.description
                        ));
                    }
                }
            }

            // Fail if any tasks couldn't be routed
            if !unroutable.is_empty() {
                ui::print_error(&format!(
                    "Cannot determine agent for {} task(s):",
                    unroutable.len()
                ));
                for task_info in &unroutable {
                    println!("  {}", task_info);
                }
                return Err(TasksError::ValidationError {
                    field: "agent_hint".to_string(),
                    reason: "Add routing keywords or explicit agent hints to these tasks"
                        .to_string(),
                });
            }

            if hints_modified > 0 {
                ui::print_info(&format!(
                    "Applied/corrected agent hints for {} tasks",
                    hints_modified
                ));
                // Save the updated tasks back
                storage.save_tasks(&tasks, tag.as_deref()).await?;
            }

            // Check for auto-append deploy task from cto-config.json
            let config_paths = [
                project_path.join("cto-config.json"),
                project_path.join(".tasks/cto-config.json"),
            ];
            let mut auto_append_deploy = false;
            for config_path in &config_paths {
                if config_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(config_path) {
                        if let Ok(config) = CtoConfig::from_json(&content) {
                            auto_append_deploy = config.defaults.intake.auto_append_deploy_task;
                            break;
                        }
                    }
                }
            }

            // Auto-append deploy task if configured
            if auto_append_deploy {
                if intake::domain::has_deploy_task(&tasks) {
                    ui::print_info("Deploy task already exists, skipping auto-append");
                } else {
                    ui::print_info("Auto-appending deploy task (depends on all other tasks)");
                    let deploy_task = intake::domain::create_deploy_task(&tasks);
                    tasks.push(deploy_task);
                    // Save the updated tasks
                    storage.save_tasks(&tasks, tag.as_deref()).await?;
                }
            }

            let output_dir = output.unwrap_or_else(|| project_path.join(".tasks").join("docs"));

            ui::print_info(&format!(
                "Generating documentation for {} tasks...",
                tasks.len()
            ));

            let result = generate_all_docs(&tasks, &output_dir).await?;

            ui::print_success(&format!(
                "Generated documentation for {} tasks",
                result.task_dirs_created
            ));
            println!("  {} XML files: {}", "•".cyan(), result.xml_files);
            println!("  {} Prompt files: {}", "•".cyan(), result.prompt_files);
            println!(
                "  {} Acceptance criteria files: {}",
                "•".cyan(),
                result.acceptance_files
            );
            ui::print_info(&format!("Output directory: {}", output_dir.display()));
        }

        Commands::SyncTask {
            issue_id,
            project_name,
            task_id,
            linear_token,
            tag,
        } => {
            // Initialize if not already
            if !tasks_domain.is_initialized().await? {
                tasks_domain.init().await?;
                ui::print_info("Initialized tasks structure");
            }

            let config = intake::commands::SyncTaskConfig {
                issue_id: issue_id.clone(),
                project_name,
                task_id,
                linear_token,
                tag,
            };

            ui::print_info(&format!("Syncing task from Linear issue: {}", issue_id));

            let result = intake::commands::sync_task(
                Arc::clone(&storage) as Arc<dyn intake::storage::Storage>,
                config,
            )
            .await?;

            if result.created {
                ui::print_success(&format!(
                    "Created task {} from Linear issue",
                    result.task.id
                ));
            } else {
                ui::print_success(&format!(
                    "Updated task {} from Linear issue",
                    result.task.id
                ));
            }

            println!();
            println!("{}", "Task Details:".bold());
            println!("  {} ID: {}", "•".cyan(), result.task.id);
            println!("  {} Title: {}", "•".cyan(), result.task.title);
            println!("  {} Priority: {}", "•".cyan(), result.task.priority);

            if !result.parsed.acceptance_criteria.is_empty() {
                println!(
                    "  {} Acceptance criteria: {} items",
                    "•".cyan(),
                    result.parsed.acceptance_criteria.len()
                );
            }

            if result.parsed.test_strategy.is_some() {
                println!("  {} Test strategy: present", "•".cyan());
            }

            if let Some(ref hint) = result.task.agent_hint {
                println!("  {} Agent hint: {}", "•".cyan(), hint);
            }

            println!();
            ui::print_info("Changed files:");
            for file in &result.changed_files {
                println!("  {} {}", "•".cyan(), file.display());
            }
        }

        // =========== Session 2: Prompt Generation Command Handlers ===========
        Commands::SplitTasks { input, output } => {
            let tasks_json_path = input
                .unwrap_or_else(|| project_path.join(".tasks").join("tasks").join("tasks.json"));
            let output_dir = output.unwrap_or_else(|| project_path.join(".tasks").join("tasks"));

            if !tasks_json_path.exists() {
                return Err(TasksError::FileReadError {
                    path: tasks_json_path.display().to_string(),
                    reason: "tasks.json not found. Run intake or parse-prd first.".to_string(),
                });
            }

            ui::print_info(&format!(
                "Splitting {} into individual task files...",
                tasks_json_path.display()
            ));

            let result = intake::domain::split_tasks(&tasks_json_path, &output_dir).await?;

            ui::print_success(&format!(
                "Created {} individual task files",
                result.files_created
            ));
            ui::print_info(&format!("Output directory: {}", output_dir.display()));
        }

        Commands::GeneratePrompts {
            task_file,
            all,
            tasks_dir,
            include_prd,
            include_arch,
            with_examples,
            research,
            prd_path,
            arch_path,
            output,
            cli,
            model,
            mcp_config,
            tag: _,
        } => {
            let output_dir = output.unwrap_or_else(|| project_path.join(".tasks").join("docs"));
            let tasks_directory =
                tasks_dir.unwrap_or_else(|| project_path.join(".tasks").join("tasks"));

            // Build config
            let config = intake::domain::GeneratePromptsConfig {
                include_prd,
                include_arch,
                with_examples,
                research,
                prd_path: prd_path.or_else(|| {
                    let default = project_path.join(".tasks").join("docs").join("prd.txt");
                    if default.exists() {
                        Some(default)
                    } else {
                        None
                    }
                }),
                arch_path: arch_path.or_else(|| {
                    let default = project_path
                        .join(".tasks")
                        .join("docs")
                        .join("architecture.md");
                    if default.exists() {
                        Some(default)
                    } else {
                        None
                    }
                }),
                output_dir: output_dir.clone(),
                cli,
                model,
                mcp_config,
            };

            let generator = intake::domain::PromptGenerator::new(config);

            if let Some(task_file_path) = task_file {
                // Process single task file
                ui::print_info(&format!(
                    "Generating prompts for {}...",
                    task_file_path.display()
                ));

                let content = tokio::fs::read_to_string(&task_file_path)
                    .await
                    .map_err(|e| TasksError::FileReadError {
                        path: task_file_path.display().to_string(),
                        reason: e.to_string(),
                    })?;

                let task: intake::entities::Task =
                    serde_json::from_str(&content).map_err(|e| TasksError::JsonParseError {
                        reason: format!("Failed to parse task file: {e}"),
                    })?;

                let files = generator.generate_for_task(&task).await?;

                ui::print_success(&format!("Generated prompts for task {}", task.id));
                if let Some(path) = files.prompt_md {
                    println!("  {} prompt.md: {}", "•".cyan(), path.display());
                }
                if let Some(path) = files.prompt_xml {
                    println!("  {} prompt.xml: {}", "•".cyan(), path.display());
                }
                if let Some(path) = files.acceptance_md {
                    println!("  {} acceptance.md: {}", "•".cyan(), path.display());
                }
                if let Some(path) = files.code_examples_md {
                    println!("  {} code-examples.md: {}", "•".cyan(), path.display());
                }
            } else if all {
                // Process all task files in directory
                ui::print_info(&format!(
                    "Generating prompts for all tasks in {}...",
                    tasks_directory.display()
                ));

                let result = generator.generate_all(&tasks_directory).await?;

                ui::print_success(&format!(
                    "Generated prompts for {}/{} tasks",
                    result.tasks_succeeded, result.tasks_processed
                ));

                if result.tasks_failed > 0 {
                    ui::print_warning(&format!("{} tasks failed", result.tasks_failed));
                }

                ui::print_info(&format!("Output directory: {}", output_dir.display()));
            } else {
                return Err(TasksError::InvalidArgument {
                    reason: "Specify --task-file <path> or --all to process tasks".to_string(),
                });
            }
        }
    }

    Ok(())
}

async fn check_initialized(domain: &TasksDomain) -> Result<(), TasksError> {
    if !domain.is_initialized().await? {
        return Err(TasksError::NotInitialized);
    }
    Ok(())
}
