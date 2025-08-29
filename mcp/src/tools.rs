use serde_json::{json, Value};
use std::collections::HashMap;

/// Get tool schemas for MCP protocol with rich descriptions
pub fn get_tool_schemas() -> Value {
    json!({
        "tools": [
            get_docs_schema(),
            get_code_schema(&HashMap::new()),
            get_play_schema(&HashMap::new()),
            get_export_schema(),
            get_intake_schema(),
            get_jobs_schema(),
            get_stop_job_schema(),
            get_input_schema()
        ]
    })
}

/// Get tool schemas with config-based agent descriptions
pub fn get_tool_schemas_with_config(agents: &HashMap<String, String>) -> Value {
    json!({
        "tools": [
            get_docs_schema(),
            get_code_schema(agents),
            get_play_schema(agents),
            get_export_schema(),
            get_intake_schema(),
            get_jobs_schema(),
            get_stop_job_schema(),
            get_input_schema()
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

fn get_code_schema(agents: &HashMap<String, String>) -> Value {
    json!({
        "name": "code",
        "description": "Submit a Task Master task for implementation using Claude with persistent workspace",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "Task ID to implement from task files",
                    "minimum": 1
                },
                "service": {
                    "type": "string",
                    "description": "Target service name (creates workspace-{service} PVC). Optional if defaults.code.service is set in config.",
                    "pattern": "^[a-z0-9-]+$"
                },
                "repository": {
                    "type": "string",
                    "description": "Target repository URL (e.g., https://github.com/5dlabs/cto). Optional if defaults.code.repository is set in config."
                },
                "docs_project_directory": {
                    "type": "string",
                    "description": "Project directory within docs repository (e.g., projects/market-research). Optional if defaults.code.docsProjectDirectory is set in config."
                },
                "docs_repository": {
                    "type": "string",
                    "description": "Documentation repository URL. Optional if defaults.code.docsRepository is set in config."
                },
                "agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent name for task assignment".to_string()
                    } else {
                        let agent_list = agents.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                        format!("Agent name for task assignment. Available agents: {agent_list}")
                    }
                },
                "working_directory": {
                    "type": "string",
                    "description": "Working directory within target repository (optional, defaults to '.')"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to configuration)"
                },
                "continue_session": {
                    "type": "boolean",
                    "description": "Whether to continue a previous session (optional, defaults to false)"
                },
                "overwrite_memory": {
                    "type": "boolean",
                    "description": "Whether to overwrite CLAUDE.md memory file (optional, defaults to false)"
                },
                "env": {
                    "type": "object",
                    "description": "Environment variables to set in the container (optional)",
                    "additionalProperties": {
                        "type": "string"
                    }
                },
                "env_from_secrets": {
                    "type": "array",
                    "description": "Environment variables from secrets (optional)",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Name of the environment variable"
                            },
                            "secretName": {
                                "type": "string",
                                "description": "Name of the secret"
                            },
                            "secretKey": {
                                "type": "string",
                                "description": "Key within the secret"
                            }
                        },
                        "required": ["name", "secretName", "secretKey"]
                    }
                }
            },
            "required": ["task_id"]
        }
    })
}

fn get_play_schema(agents: &HashMap<String, String>) -> Value {
    json!({
        "name": "play",
        "description": "Submit a Play workflow for multi-agent orchestration (Rex/Blaze → Cleo → Tess) with event-driven coordination",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "Task ID to implement from task files",
                    "minimum": 1
                },
                "repository": {
                    "type": "string",
                    "description": "Target repository URL (e.g., 5dlabs/cto). Optional if defaults.play.repository is set in config."
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
                        "Agent for implementation work (e.g., 5DLabs-Rex, 5DLabs-Blaze)".to_string()
                    } else {
                        let agent_list = agents.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                        format!("Agent for implementation work. Available agents: {agent_list}")
                    }
                },
                "quality_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for quality assurance (e.g., 5DLabs-Cleo)".to_string()
                    } else {
                        let agent_list = agents.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                        format!("Agent for quality assurance. Available agents: {agent_list}")
                    }
                },
                "testing_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for testing and validation (e.g., 5DLabs-Tess)".to_string()
                    } else {
                        let agent_list = agents.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                        format!("Agent for testing and validation. Available agents: {agent_list}")
                    }
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use for all agents (optional, defaults to configuration)"
                }
            },
            "required": ["task_id"]
        }
    })
}

fn get_export_schema() -> Value {
    json!({
        "name": "export",
        "description": "Export Rust codebase to markdown for documentation context",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    })
}

fn get_intake_schema() -> Value {
    json!({
        "name": "intake",
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
        "description": "List running jobs across platform CRDs (CodeRun, DocsRun) and Argo Workflows (intake). Returns simplified status info.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "namespace": {"type": "string", "description": "Kubernetes namespace (default: agent-platform)"},
                "include": {"type": "array", "items": {"type": "string", "enum": ["code", "docs", "intake"]}, "description": "Filter which job types to include (default: all)"}
            }
        }
    })
}

fn get_stop_job_schema() -> Value {
    json!({
        "name": "stop_job",
        "description": "Stop a running job: CodeRun (code), DocsRun (docs), Argo intake workflow (intake), or Play workflow (play).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "job_type": {"type": "string", "enum": ["code", "docs", "intake", "play"], "description": "Type of job to stop"},
                "name": {"type": "string", "description": "Resource/workflow name"},
                "namespace": {"type": "string", "description": "Kubernetes namespace (default: agent-platform)"}
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
                "namespace": {"type": "string", "description": "Kubernetes namespace (default: agent-platform)"},
                "fifo_path": {"type": "string", "description": "FIFO path inside container (default: /workspace/agent-input.jsonl)"},
                "job_type": {"type": "string", "enum": ["code", "docs"], "description": "Optional job type filter when routing by user"},
                "name": {"type": "string", "description": "Optional CodeRun/DocsRun resource name when routing by explicit job"},
                "user": {"type": "string", "description": "Optional user label (agents.platform/user) to route to active job"}
            },
            "required": ["text"]
        }
    })
}
