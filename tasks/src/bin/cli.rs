//! Tasks CLI - Task management for AI-driven development.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::disallowed_macros)]
#![allow(clippy::uninlined_format_args)]

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use colored::Colorize;

use tasks::ai::schemas::ComplexityReport;
use tasks::domain::{AIDomain, ConfigDomain, DependencyDomain, TagsDomain, TasksDomain};
use tasks::entities::{TaskPriority, TaskStatus};
use tasks::errors::TasksError;
use tasks::storage::FileStorage;
use tasks::ui;

#[derive(Parser)]
#[command(name = "tasks")]
#[command(about = "Task management for AI-driven development", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Project root directory
    #[arg(long, global = true)]
    project: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project with tasks structure
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,

        /// Skip interactive prompts
        #[arg(short, long)]
        yes: bool,
    },

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

    /// Set task status
    SetStatus {
        /// Task ID(s), comma-separated
        #[arg(short, long)]
        id: String,

        /// New status
        #[arg(short, long)]
        status: String,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Add a new task
    Add {
        /// Task title
        #[arg(short, long)]
        title: String,

        /// Task description
        #[arg(short, long)]
        description: Option<String>,

        /// Dependencies (comma-separated)
        #[arg(long)]
        deps: Option<String>,

        /// Priority (low, medium, high, critical)
        #[arg(short, long)]
        priority: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Add a subtask to a task
    AddSubtask {
        /// Parent task ID
        #[arg(short, long)]
        id: String,

        /// Subtask title
        #[arg(short, long)]
        title: String,

        /// Subtask description
        #[arg(short, long)]
        description: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Remove a task
    Remove {
        /// Task ID
        #[arg(short, long)]
        id: String,

        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Clear subtasks from a task
    ClearSubtasks {
        /// Task ID
        #[arg(short, long)]
        id: String,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Move a task to a new position
    Move {
        /// Task ID
        #[arg(long)]
        id: String,

        /// New position (1-based)
        #[arg(long)]
        to: usize,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Manage tags
    #[command(subcommand)]
    Tags(TagsCommands),

    /// Manage dependencies
    #[command(subcommand)]
    Deps(DepsCommands),

    /// Configure AI models
    Models {
        /// Set main model (`provider:model_id`)
        #[arg(long)]
        set_main: Option<String>,

        /// Set research model (`provider:model_id`)
        #[arg(long)]
        set_research: Option<String>,

        /// Set fallback model (`provider:model_id`)
        #[arg(long)]
        set_fallback: Option<String>,
    },

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

    /// Add a task using AI
    AddTask {
        /// Description of the task to create
        #[arg(short, long)]
        prompt: String,

        /// Priority (low, medium, high)
        #[arg(long)]
        priority: Option<String>,

        /// Dependencies (comma-separated task IDs)
        #[arg(short, long)]
        dependencies: Option<String>,

        /// Use research mode
        #[arg(short, long)]
        research: bool,

        /// AI model to use
        #[arg(long)]
        model: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Update multiple tasks starting from an ID
    Update {
        /// Starting task ID
        #[arg(long)]
        from: i32,

        /// Context/changes to apply
        #[arg(short, long)]
        prompt: String,

        /// Use research mode
        #[arg(short, long)]
        research: bool,

        /// AI model to use
        #[arg(long)]
        model: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Update a single task with AI
    UpdateTask {
        /// Task ID
        #[arg(short, long)]
        id: String,

        /// Context/changes to apply
        #[arg(short, long)]
        prompt: String,

        /// Append to details instead of replacing
        #[arg(long)]
        append: bool,

        /// Use research mode
        #[arg(short, long)]
        research: bool,

        /// AI model to use
        #[arg(long)]
        model: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },

    /// Update a subtask by appending information
    UpdateSubtask {
        /// Subtask ID (e.g., "1.2")
        #[arg(short, long)]
        id: String,

        /// Information to append
        #[arg(short, long)]
        prompt: String,

        /// Use research mode
        #[arg(short, long)]
        research: bool,

        /// AI model to use
        #[arg(long)]
        model: Option<String>,

        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
}

#[derive(Subcommand)]
enum TagsCommands {
    /// List all tags
    List,

    /// Create a new tag
    Add {
        /// Tag name
        name: String,

        /// Copy tasks from another tag
        #[arg(long)]
        copy_from: Option<String>,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Delete a tag
    Delete {
        /// Tag name
        name: String,

        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },

    /// Switch to a tag
    Use {
        /// Tag name
        name: String,
    },

    /// Rename a tag
    Rename {
        /// Old name
        old_name: String,

        /// New name
        new_name: String,
    },

    /// Copy a tag
    Copy {
        /// Source tag
        source: String,

        /// Target tag
        target: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        ui::print_error(&e.to_string());
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), TasksError> {
    let project_path = get_project_path(cli.project);
    let storage = Arc::new(FileStorage::new(&project_path));
    let tasks_domain = TasksDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);
    let tags_domain = TagsDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);
    let deps_domain =
        DependencyDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);
    let config_domain = ConfigDomain::new(&project_path);

    match cli.command {
        Commands::Init { name, yes: _ } => {
            if tasks_domain.is_initialized().await? {
                ui::print_warning("Project already initialized");
                return Ok(());
            }

            tasks_domain.init().await?;

            if let Some(project_name) = name {
                config_domain.set_project_name(&project_name).await?;
            }

            ui::print_success("Project initialized successfully!");
            ui::print_info(&format!(
                "Tasks directory created at: {}",
                project_path.join(".tasks").display()
            ));
        }

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

        Commands::SetStatus { id, status, tag } => {
            check_initialized(&tasks_domain).await?;

            let new_status: TaskStatus = status.parse()?;
            let ids: Vec<&str> = id.split(',').map(str::trim).collect();

            for task_id in &ids {
                tasks_domain
                    .set_status(task_id, new_status, tag.as_deref())
                    .await?;
            }

            ui::print_success(&format!(
                "Updated {} task(s) to status: {}",
                ids.len(),
                new_status
            ));
        }

        Commands::Add {
            title,
            description,
            deps,
            priority,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let desc = description.unwrap_or_default();
            let mut task = tasks_domain
                .add_task(&title, &desc, tag.as_deref())
                .await?;

            // Set priority if provided
            if let Some(p) = priority {
                task.priority = p.parse::<TaskPriority>()?;
            }

            // Add dependencies if provided
            if let Some(deps_str) = deps {
                task.dependencies = deps_str.split(',').map(|s| s.trim().to_string()).collect();
            }

            tasks_domain.update_task(&task, tag.as_deref()).await?;

            ui::print_success(&format!("Created task {} - {}", task.id, task.title));
        }

        Commands::AddSubtask {
            id,
            title,
            description,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let desc = description.unwrap_or_default();
            let subtask = tasks_domain
                .add_subtask(&id, &title, &desc, tag.as_deref())
                .await?;

            ui::print_success(&format!(
                "Created subtask {}.{} - {}",
                id, subtask.id, subtask.title
            ));
        }

        Commands::Remove { id, yes, tag } => {
            check_initialized(&tasks_domain).await?;

            if !yes {
                ui::print_warning(&format!("About to delete task {id}. Use --yes to confirm."));
                return Ok(());
            }

            tasks_domain.remove_task(&id, tag.as_deref()).await?;
            ui::print_success(&format!("Removed task {id}"));
        }

        Commands::ClearSubtasks { id, tag } => {
            check_initialized(&tasks_domain).await?;

            let count = tasks_domain.clear_subtasks(&id, tag.as_deref()).await?;
            ui::print_success(&format!("Cleared {count} subtask(s) from task {id}"));
        }

        Commands::Move { id, to, tag } => {
            check_initialized(&tasks_domain).await?;

            // Convert 1-based to 0-based
            let position = to.saturating_sub(1);
            tasks_domain.move_task(&id, position, tag.as_deref()).await?;
            ui::print_success(&format!("Moved task {id} to position {to}"));
        }

        Commands::Tags(tags_cmd) => {
            check_initialized(&tasks_domain).await?;

            match tags_cmd {
                TagsCommands::List => {
                    let stats = tags_domain.list_tags_with_stats().await?;
                    let table = ui::tag_table(&stats);
                    println!("{table}");
                }

                TagsCommands::Add {
                    name,
                    copy_from,
                    description,
                } => {
                    tags_domain
                        .create_tag(&name, copy_from.as_deref(), description.as_deref())
                        .await?;
                    ui::print_success(&format!("Created tag: {}", name));
                }

                TagsCommands::Delete { name, yes } => {
                    if !yes {
                        ui::print_warning(&format!(
                            "About to delete tag '{}'. Use --yes to confirm.",
                            name
                        ));
                        return Ok(());
                    }

                    tags_domain.delete_tag(&name).await?;
                    ui::print_success(&format!("Deleted tag: {}", name));
                }

                TagsCommands::Use { name } => {
                    tags_domain.use_tag(&name).await?;
                    ui::print_success(&format!("Switched to tag: {}", name));
                }

                TagsCommands::Rename { old_name, new_name } => {
                    tags_domain.rename_tag(&old_name, &new_name).await?;
                    ui::print_success(&format!("Renamed tag '{}' to '{}'", old_name, new_name));
                }

                TagsCommands::Copy {
                    source,
                    target,
                    description,
                } => {
                    tags_domain
                        .copy_tag(&source, &target, description.as_deref())
                        .await?;
                    ui::print_success(&format!("Copied tag '{}' to '{}'", source, target));
                }
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

        Commands::Models {
            set_main,
            set_research,
            set_fallback,
        } => {
            check_initialized(&tasks_domain).await?;

            if set_main.is_none() && set_research.is_none() && set_fallback.is_none() {
                // Display current models
                let models = config_domain.get_models().await?;

                println!("{}", "Model Configuration".bold().underline());
                println!();

                if let Some(main) = models.main {
                    println!(
                        "  {}: {}:{}",
                        "Main".cyan(),
                        main.provider,
                        main.model_id
                    );
                } else {
                    println!("  {}: {}", "Main".cyan(), "not set".dimmed());
                }

                if let Some(research) = models.research {
                    println!(
                        "  {}: {}:{}",
                        "Research".cyan(),
                        research.provider,
                        research.model_id
                    );
                } else {
                    println!("  {}: {}", "Research".cyan(), "not set".dimmed());
                }

                if let Some(fallback) = models.fallback {
                    println!(
                        "  {}: {}:{}",
                        "Fallback".cyan(),
                        fallback.provider,
                        fallback.model_id
                    );
                } else {
                    println!("  {}: {}", "Fallback".cyan(), "not set".dimmed());
                }
            } else {
                // Set models
                if let Some(main) = set_main {
                    let settings = parse_model_string(&main)?;
                    config_domain.set_main_model(settings).await?;
                    ui::print_success(&format!("Set main model to: {}", main));
                }

                if let Some(research) = set_research {
                    let settings = parse_model_string(&research)?;
                    config_domain.set_research_model(settings).await?;
                    ui::print_success(&format!("Set research model to: {}", research));
                }

                if let Some(fallback) = set_fallback {
                    let settings = parse_model_string(&fallback)?;
                    config_domain.set_fallback_model(settings).await?;
                    ui::print_success(&format!("Set fallback model to: {}", fallback));
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

            let ai_domain = AIDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);

            // Read PRD file
            let prd_content = tokio::fs::read_to_string(&file)
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
                tasks_domain.add_task_full(task.clone(), tag.as_deref()).await?;
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

            let ai_domain = AIDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);

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
                .expand_task(&task, num, research, None, complexity_report.as_ref(), model.as_deref())
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

            let ai_domain = AIDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);

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
                    .expand_task(&task, num, research, None, complexity_report.as_ref(), model.as_deref())
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
            ui::print_info(&format!("Total tokens: {} in, {} out", total_input, total_output));
        }

        Commands::AnalyzeComplexity {
            threshold,
            research,
            output,
            model,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let ai_domain = AIDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);

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
            let output_path = output.unwrap_or_else(|| {
                project_path.join(".tasks").join("complexity-report.json")
            });

            let report_json = serde_json::to_string_pretty(&report)?;
            tokio::fs::write(&output_path, &report_json).await.map_err(|e| {
                TasksError::FileWriteError {
                    path: output_path.display().to_string(),
                    reason: e.to_string(),
                }
            })?;

            ui::print_success(&format!("Saved complexity report to: {}", output_path.display()));
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

            let report_path = file.unwrap_or_else(|| {
                project_path.join(".tasks").join("complexity-report.json")
            });

            if !report_path.exists() {
                ui::print_error(&format!(
                    "Complexity report not found: {}",
                    report_path.display()
                ));
                ui::print_info("Run 'tasks analyze-complexity' first to generate a report");
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
                    analysis.task_id,
                    analysis.task_title,
                    score_display
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

        Commands::AddTask {
            prompt,
            priority,
            dependencies,
            research,
            model,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let ai_domain = AIDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);

            let priority_enum = if let Some(p) = priority {
                Some(p.parse::<TaskPriority>()?)
            } else {
                None
            };

            let deps = dependencies.map(|d| {
                d.split(',')
                    .filter_map(|s| s.trim().parse::<i32>().ok())
                    .collect()
            });

            ui::print_info("Generating task from prompt...");
            ui::print_info("This may take a moment...");

            let (task, usage) = ai_domain
                .add_task(&prompt, priority_enum, deps, research, model.as_deref())
                .await?;

            tasks_domain.add_task_full(task.clone(), tag.as_deref()).await?;

            ui::print_success(&format!("Created task {} - {}", task.id, task.title));
            ui::print_info(&format!(
                "Tokens used: {} in, {} out",
                usage.input_tokens, usage.output_tokens
            ));

            ui::display_task_details(&task);
        }

        Commands::Update {
            from,
            prompt,
            research,
            model,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let ai_domain = AIDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);

            ui::print_info(&format!("Updating tasks from ID {}...", from));
            ui::print_info("This may take a moment...");

            let (updated_tasks, usage) = ai_domain
                .update_tasks(from, &prompt, research, model.as_deref())
                .await?;

            // Save updated tasks
            for task in &updated_tasks {
                tasks_domain.update_task(task, tag.as_deref()).await?;
            }

            ui::print_success(&format!("Updated {} task(s)", updated_tasks.len()));
            ui::print_info(&format!(
                "Tokens used: {} in, {} out",
                usage.input_tokens, usage.output_tokens
            ));
        }

        Commands::UpdateTask {
            id,
            prompt,
            append,
            research,
            model,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let ai_domain = AIDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);

            let task = tasks_domain.get_task(&id, tag.as_deref()).await?;

            ui::print_info(&format!("Updating task: {} - {}", task.id, task.title));
            ui::print_info("This may take a moment...");

            let (updated_task, usage) = ai_domain
                .update_task(&task, &prompt, append, research, model.as_deref())
                .await?;

            tasks_domain.update_task(&updated_task, tag.as_deref()).await?;

            ui::print_success(&format!("Updated task {}", id));
            ui::print_info(&format!(
                "Tokens used: {} in, {} out",
                usage.input_tokens, usage.output_tokens
            ));

            ui::display_task_details(&updated_task);
        }

        Commands::UpdateSubtask {
            id,
            prompt,
            research,
            model,
            tag,
        } => {
            check_initialized(&tasks_domain).await?;

            let ai_domain = AIDomain::new(Arc::clone(&storage) as Arc<dyn tasks::storage::Storage>);

            // Parse subtask ID (e.g., "1.2")
            let parts: Vec<&str> = id.split('.').collect();
            if parts.len() != 2 {
                return Err(TasksError::InvalidId { id: id.clone() });
            }

            let task_id = parts[0];
            let subtask_id: u32 = parts[1].parse().map_err(|_| TasksError::InvalidId { id: id.clone() })?;

            let mut task = tasks_domain.get_task(task_id, tag.as_deref()).await?;

            let subtask = task
                .subtasks
                .iter()
                .find(|s| s.id == subtask_id)
                .ok_or_else(|| TasksError::SubtaskNotFound {
                    task_id: task_id.to_string(),
                    subtask_id: subtask_id.to_string(),
                })?
                .clone();

            ui::print_info(&format!("Updating subtask: {}.{} - {}", task_id, subtask_id, subtask.title));
            ui::print_info("This may take a moment...");

            let (new_content, usage) = ai_domain
                .update_subtask(&task, &subtask, &prompt, research, model.as_deref())
                .await?;

            // Update the subtask details
            if let Some(s) = task.subtasks.iter_mut().find(|s| s.id == subtask_id) {
                let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
                let separator = if s.details.is_empty() { "" } else { "\n\n---\n\n" };
                s.details = format!("{}{}[{}] {}", s.details, separator, timestamp, new_content);
            }

            tasks_domain.update_task(&task, tag.as_deref()).await?;

            ui::print_success(&format!("Updated subtask {}", id));
            ui::print_info(&format!(
                "Tokens used: {} in, {} out",
                usage.input_tokens, usage.output_tokens
            ));
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

fn parse_model_string(s: &str) -> Result<tasks::entities::ModelSettings, TasksError> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(TasksError::InvalidArgument {
            reason: "Model string must be in format 'provider:model_id'".to_string(),
        });
    }

    Ok(tasks::entities::ModelSettings {
        provider: parts[0].to_string(),
        model_id: parts[1].to_string(),
        max_tokens: 64000,
        temperature: 0.2,
        base_url: None,
    })
}

