//! pm-activity - CLI for emitting Linear Agent Activities from workflow containers.
//!
//! This binary provides a command-line interface for emitting activities to Linear's
//! agent system. It is designed to be called from workflow containers (intake, play, etc.)
//! to report progress back to Linear.
//!
//! # Environment Variables
//!
//! - `LINEAR_API_TOKEN` - OAuth access token or API key (required)
//! - `LINEAR_SESSION_ID` - Agent session ID (required, or pass via --session-id)
//!
//! # Examples
//!
//! ```bash
//! # Emit a thought (ephemeral by default)
//! pm-activity thought "Analyzing the PRD..."
//!
//! # Emit a persistent thought
//! pm-activity thought --persistent "Important finding..."
//!
//! # Emit an action in progress
//! pm-activity action "Parsing" "prd.md"
//!
//! # Emit action completion with result
//! pm-activity action-complete "Parsed" "prd.md" "Found 15 requirements"
//!
//! # Emit final response
//! pm-activity response "Generated 12 tasks from PRD"
//!
//! # Emit an error
//! pm-activity error "Failed to parse PRD: invalid YAML"
//!
//! # Request user input
//! pm-activity elicitation "Which repository should I target?"
//!
//! # Update the plan checklist
//! pm-activity plan "Parse PRD:completed" "Generate tasks:in_progress" "Expand:pending"
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use pm::activities::{PlanStep, PlanStepStatus};
use pm::emitter::LinearAgentEmitter;
use pm::AgentActivityEmitter;

/// CLI for emitting Linear Agent Activities from workflow containers.
#[derive(Parser)]
#[command(name = "pm-activity")]
#[command(about = "Emit Linear Agent Activities from workflow containers")]
#[command(version)]
struct Cli {
    /// Linear API token (or set `LINEAR_API_TOKEN` env var)
    #[arg(long, env = "LINEAR_API_TOKEN", hide_env_values = true)]
    token: String,

    /// Agent session ID (or set `LINEAR_SESSION_ID` env var)
    #[arg(long, env = "LINEAR_SESSION_ID")]
    session_id: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Emit a thought activity (internal reasoning/status)
    Thought {
        /// The thought content (markdown supported)
        body: String,

        /// Make the thought persistent (not replaced by next activity)
        #[arg(long, short)]
        persistent: bool,
    },

    /// Emit an action activity (tool/step in progress)
    Action {
        /// Action name (e.g., "Parsing", "Running tests")
        action: String,

        /// Action parameter (e.g., file path, query)
        parameter: String,
    },

    /// Emit an action completion with result
    ActionComplete {
        /// Action name (past tense, e.g., "Parsed", "Tests passed")
        action: String,

        /// Action parameter
        parameter: String,

        /// Action result (markdown supported)
        result: String,
    },

    /// Emit a response activity (work completed)
    Response {
        /// The response content (markdown supported)
        body: String,
    },

    /// Emit an error activity
    Error {
        /// The error message (markdown supported)
        body: String,
    },

    /// Emit an elicitation activity (request user input)
    Elicitation {
        /// The prompt for the user (markdown supported)
        body: String,
    },

    /// Update the session plan (visual checklist)
    ///
    /// Each step is specified as "content:status" where status is one of:
    /// pending, `in_progress`, completed, canceled
    Plan {
        /// Plan steps in format "Step description:status"
        #[arg(required = true)]
        steps: Vec<String>,
    },
}

fn parse_plan_step(s: &str) -> Result<PlanStep> {
    let parts: Vec<&str> = s.rsplitn(2, ':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid plan step format: '{s}'. Expected 'content:status'");
    }

    let status_str = parts[0].trim().to_lowercase();
    let content = parts[1].trim();

    let status = match status_str.as_str() {
        "pending" => PlanStepStatus::Pending,
        "in_progress" | "inprogress" | "in-progress" => PlanStepStatus::InProgress,
        "completed" | "complete" | "done" => PlanStepStatus::Completed,
        "canceled" | "cancelled" | "cancel" => PlanStepStatus::Canceled,
        _ => anyhow::bail!(
            "Invalid status '{status_str}'. Expected: pending, in_progress, completed, or canceled"
        ),
    };

    Ok(PlanStep {
        content: content.to_string(),
        status,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let cli = Cli::parse();

    // Create emitter with provided credentials
    let emitter = LinearAgentEmitter::with_credentials(&cli.token, &cli.session_id)
        .context("Failed to create Linear emitter")?;

    match cli.command {
        Commands::Thought { body, persistent } => {
            let ephemeral = !persistent;
            let id = emitter
                .emit_thought(&body, ephemeral)
                .await
                .context("Failed to emit thought")?;
            tracing::info!(activity_id = %id, "Emitted thought");
        }

        Commands::Action { action, parameter } => {
            let id = emitter
                .emit_action(&action, &parameter)
                .await
                .context("Failed to emit action")?;
            tracing::info!(activity_id = %id, "Emitted action");
        }

        Commands::ActionComplete {
            action,
            parameter,
            result,
        } => {
            let id = emitter
                .emit_action_complete(&action, &parameter, &result)
                .await
                .context("Failed to emit action completion")?;
            tracing::info!(activity_id = %id, "Emitted action completion");
        }

        Commands::Response { body } => {
            let id = emitter
                .emit_response(&body)
                .await
                .context("Failed to emit response")?;
            tracing::info!(activity_id = %id, "Emitted response");
        }

        Commands::Error { body } => {
            let id = emitter
                .emit_error(&body)
                .await
                .context("Failed to emit error")?;
            tracing::info!(activity_id = %id, "Emitted error");
        }

        Commands::Elicitation { body } => {
            let id = emitter
                .emit_elicitation(&body)
                .await
                .context("Failed to emit elicitation")?;
            tracing::info!(activity_id = %id, "Emitted elicitation");
        }

        Commands::Plan { steps } => {
            let plan_steps: Vec<PlanStep> = steps
                .iter()
                .map(|s| parse_plan_step(s))
                .collect::<Result<Vec<_>>>()
                .context("Failed to parse plan steps")?;

            let success = emitter
                .update_plan(&plan_steps)
                .await
                .context("Failed to update plan")?;

            if success {
                tracing::info!(step_count = plan_steps.len(), "Updated plan");
            } else {
                anyhow::bail!("Plan update returned unsuccessful");
            }
        }
    }

    Ok(())
}
