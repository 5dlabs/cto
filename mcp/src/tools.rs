use serde_json::{json, Value};
use std::collections::HashMap;

/// Get tool schemas for MCP protocol with rich descriptions
pub fn get_tool_schemas() -> Value {
    json!({
        "tools": [
            get_docs_schema(),
            get_play_schema(&HashMap::new()),
            get_play_status_schema(),
            get_intake_prd_schema(),
            get_jobs_schema(),
            get_stop_job_schema(),
            get_input_schema(),
            get_add_docs_schema()
        ]
    })
}

/// Get tool schemas with config-based agent descriptions
pub fn get_tool_schemas_with_config(agents: &HashMap<String, crate::AgentConfig>) -> Value {
    json!({
        "tools": [
            get_docs_schema(),
            get_play_schema(agents),
            get_play_status_schema(),
            get_intake_prd_schema(),
            get_jobs_schema(),
            get_stop_job_schema(),
            get_input_schema(),
            get_add_docs_schema()
        ]
    })
}

fn get_docs_schema() -> Value {
    json!({
        "name": "docs",
        "description": "Initialize documentation for Task Master tasks using Claude",
        "inputSchema": {
            "type": "object",
            "properties": {
                "working_directory": {
                    "type": "string",
                    "description": "Working directory containing .taskmaster folder (required). Use relative paths like 'projects/market-research'."
                },
                "agent": {
                    "type": "string",
                    "description": "Agent name for task assignment (optional, uses workflow default if not specified)"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to configuration)"
                },
                "include_codebase": {
                    "type": "boolean",
                    "description": "Include existing codebase as markdown context (optional, defaults to false)"
                }
            },
            "required": ["working_directory"]
        }
    })
}

fn get_play_schema(agents: &HashMap<String, crate::AgentConfig>) -> Value {
    json!({
        "name": "play",
        "description": "Submit a Play workflow for multi-agent orchestration (Rex/Blaze → Cleo → Tess → Atlas → Bolt) with event-driven coordination",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "Task ID to implement from task files. Optional - if not provided, will auto-detect next available task from TaskMaster based on dependencies and priority.",
                    "minimum": 1
                },
                "repository": {
                    "type": "string",
                    "description": "Target repository URL (e.g., 5dlabs/cto). Optional if defaults.play.repository is set in config."
                },
                "repository_path": {
                    "type": "string",
                    "description": "Absolute path to the repository on disk (e.g., /Users/name/code/cto-parallel-test). Use this when the target repository is not in the current workspace. Optional - if not provided, will use workspace detection."
                },
                "service": {
                    "type": "string",
                    "description": "Service identifier for persistent workspace. Optional if defaults.play.service is set in config.",
                    "pattern": "^[a-z0-9-]+$"
                },
                "docs_repository": {
                    "type": "string",
                    "description": "Documentation repository URL. Optional if defaults.play.docsRepository is set in config."
                },
                "docs_project_directory": {
                    "type": "string",
                    "description": "Project directory within docs repository (e.g., docs). Optional if defaults.play.docsProjectDirectory is set in config."
                },
                "implementation_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for backend/general implementation work (e.g., 5DLabs-Rex)".to_string()
                    } else {
                        let agent_list = agents.keys().map(std::string::String::as_str).collect::<Vec<_>>().join(", ");
                        format!("Agent for backend/general implementation work. Available agents: {agent_list}")
                    }
                },
                "frontend_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for frontend tasks (React, UI components) (e.g., 5DLabs-Blaze). Optional if defaults.play.frontendAgent is set in config.".to_string()
                    } else {
                        let agent_list = agents.keys().map(std::string::String::as_str).collect::<Vec<_>>().join(", ");
                        format!("Agent for frontend tasks (React, UI components). Available agents: {agent_list}")
                    }
                },
                "quality_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for quality assurance (e.g., 5DLabs-Cleo)".to_string()
                    } else {
                        let agent_list = agents.keys().map(std::string::String::as_str).collect::<Vec<_>>().join(", ");
                        format!("Agent for quality assurance. Available agents: {agent_list}")
                    }
                },
                "testing_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for testing and validation (e.g., 5DLabs-Tess)".to_string()
                    } else {
                        let agent_list = agents.keys().map(std::string::String::as_str).collect::<Vec<_>>().join(", ");
                        format!("Agent for testing and validation. Available agents: {agent_list}")
                    }
                },
                "parallel_execution": {
                    "type": "boolean",
                    "description": "Enable parallel execution of independent tasks. When true, analyzes TaskMaster dependencies and runs tasks in parallel execution levels. When false (default), runs tasks sequentially one at a time. Requires TaskMaster tasks.json with proper dependencies.",
                    "default": false
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use for all agents (optional, defaults to configuration)"
                },
                "opencode_max_retries": {
                    "type": "integer",
                    "description": "Override maximum retry attempts for OpenCode runs (defaults to configuration)"
                },
                "cli": {
                    "type": "string",
                    "description": "CLI tool to use for all agents (optional, defaults to configuration)"
                }
            },
            "required": []
        }
    })
}

fn get_play_status_schema() -> Value {
    json!({
        "name": "play_status",
        "description": "Query current play workflow status and progress. Shows active workflows, next available tasks, and blocked tasks.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "repository": {
                    "type": "string",
                    "description": "Target repository URL (e.g., 5dlabs/cto). Optional if defaults.play.repository is set in config."
                }
            },
            "required": []
        }
    })
}

fn get_intake_prd_schema() -> Value {
    json!({
        "name": "intake_prd",
        "description": "Process a new project intake. Reads PRD from {project_name}/intake/prd.txt and optional architecture from {project_name}/intake/architecture.md. Auto-detects repository and branch from git. Creates TaskMaster structure in project subdirectory and submits PR.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project_name": {
                    "type": "string",
                    "description": "Name of the project subdirectory containing intake files (required)"
                },
                "github_app": {
                    "type": "string",
                    "description": "GitHub App to use (optional, defaults to configuration)"
                },
                "primary_model": {
                    "type": "string",
                    "description": "Primary model for task generation (optional, defaults to configuration)"
                },
                "primary_provider": {
                    "type": "string",
                    "description": "Provider for primary model (e.g., anthropic, claude-code, openai)"
                },
                "research_model": {
                    "type": "string",
                    "description": "Model for research operations (optional, defaults to configuration)"
                },
                "research_provider": {
                    "type": "string",
                    "description": "Provider for research model (e.g., anthropic, claude-code, openai)"
                },
                "fallback_model": {
                    "type": "string",
                    "description": "Fallback model if primary fails (optional, defaults to configuration)"
                },
                "fallback_provider": {
                    "type": "string",
                    "description": "Provider for fallback model (e.g., anthropic, claude-code, openai)"
                },
                "prd_content": {
                    "type": "string",
                    "description": "PRD content (optional, overrides file reading)"
                },
                "architecture_content": {
                    "type": "string",
                    "description": "Architecture content (optional, overrides file reading)"
                }
            },
            "required": ["project_name"]
        }
    })
}

fn get_jobs_schema() -> Value {
    json!({
        "name": "jobs",
        "description": "List running Argo workflows (play, intake, and other workflows). Returns simplified status info.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "namespace": {"type": "string", "description": "Kubernetes namespace (default: cto)"},
                "include": {"type": "array", "items": {"type": "string", "enum": ["play", "intake", "workflow"]}, "description": "Filter which workflow types to include (default: all)"}
            }
        }
    })
}

fn get_stop_job_schema() -> Value {
    json!({
        "name": "stop_job",
        "description": "Stop a running Argo workflow (intake, play, or generic workflow).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "job_type": {"type": "string", "enum": ["intake", "play", "workflow"], "description": "Type of workflow to stop"},
                "name": {"type": "string", "description": "Workflow name"},
                "namespace": {"type": "string", "description": "Kubernetes namespace (default: cto)"}
            },
            "required": ["job_type", "name"]
        }
    })
}

fn get_input_schema() -> Value {
    json!({
        "name": "input",
        "description": "Send a live user message to a running Claude job via stream-json. Route by explicit job name or by user label.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "text": {"type": "string", "description": "Plain text to send as a user message"},
                "namespace": {"type": "string", "description": "Kubernetes namespace (default: cto)"},
                "fifo_path": {"type": "string", "description": "FIFO path inside container (default: /workspace/agent-input.jsonl)"},
                "job_type": {"type": "string", "enum": ["code", "docs"], "description": "Optional job type filter when routing by user"},
                "name": {"type": "string", "description": "Optional CodeRun/DocsRun resource name when routing by explicit job"},
                "user": {"type": "string", "description": "Optional user label (agents.platform/user) to route to active job"}
            },
            "required": ["text"]
        }
    })
}

fn get_add_docs_schema() -> Value {
    json!({
        "name": "add_docs",
        "description": "Ingest documentation from a URL using Firecrawl. Supports GitHub repositories (type: repo) and websites (type: scrape). Returns crawled/scraped content in markdown format.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The documentation source URL. For repos: GitHub URL (e.g., https://github.com/org/repo). For scrape: any website URL (e.g., https://docs.example.com)."
                },
                "type": {
                    "type": "string",
                    "enum": ["repo", "scrape"],
                    "description": "Type of ingestion: 'repo' for GitHub repositories, 'scrape' for websites."
                },
                "query": {
                    "type": "string",
                    "description": "Optional search query/topic to focus the crawl on specific content."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of pages to crawl (default: 50).",
                    "default": 50
                }
            },
            "required": ["url", "type"]
        }
    })
}

