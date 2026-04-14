//! Tool schemas for CTO Lite MCP Server
//!
//! Simplified tool definitions focused on core functionality:
//! - Task management (read/update task files)
//! - Workflow triggering (Argo workflows)
//! - Status queries

use serde_json::{json, Value};

/// Get all tool schemas for MCP protocol
pub fn get_tool_schemas() -> Value {
    json!({
        "tools": [
            get_play_schema(),
            get_play_status_schema(),
            get_jobs_schema(),
            get_stop_job_schema(),
            get_input_schema(),
            get_check_setup_schema(),
            get_task_schema()
        ]
    })
}

/// Play workflow tool schema
fn get_play_schema() -> Value {
    json!({
        "name": "play",
        "description": "Submit a Play workflow for multi-agent task implementation via Argo Workflows.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "Task ID to implement (optional - auto-detects next available task)"
                },
                "repository": {
                    "type": "string",
                    "description": "Target repository URL in org/repo format"
                },
                "repository_path": {
                    "type": "string",
                    "description": "Absolute path to repository on disk"
                },
                "service": {
                    "type": "string",
                    "description": "Service identifier for persistent workspace"
                },
                "model": {
                    "type": "string",
                    "description": "AI model to use (optional, defaults to configuration)"
                },
                "parallel_execution": {
                    "type": "boolean",
                    "description": "Enable parallel execution of independent tasks",
                    "default": false
                }
            }
        }
    })
}

/// Play status tool schema
fn get_play_status_schema() -> Value {
    json!({
        "name": "play_status",
        "description": "Query current play workflow status and progress. Shows active workflows and next available tasks.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "repository": {
                    "type": "string",
                    "description": "Filter by repository URL"
                }
            }
        }
    })
}

/// Jobs listing tool schema
fn get_jobs_schema() -> Value {
    json!({
        "name": "jobs",
        "description": "List running Argo workflows. Returns simplified status info.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace (default: cto)"
                }
            }
        }
    })
}

/// Stop job tool schema
fn get_stop_job_schema() -> Value {
    json!({
        "name": "stop_job",
        "description": "Stop a running Argo workflow.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Workflow name to stop"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace (default: cto)"
                }
            },
            "required": ["name"]
        }
    })
}

/// Input tool schema
fn get_input_schema() -> Value {
    json!({
        "name": "input",
        "description": "Send a live user message to a running Claude job.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "Message to send"
                },
                "namespace": {
                    "type": "string",
                    "description": "Kubernetes namespace (default: cto)"
                },
                "job_type": {
                    "type": "string",
                    "description": "Optional job type filter"
                }
            },
            "required": ["text"]
        }
    })
}

/// Check setup tool schema
fn get_check_setup_schema() -> Value {
    json!({
        "name": "check_setup",
        "description": "Check MCP server dependencies (kubectl, argo) and cluster connectivity.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "verbose": {
                    "type": "boolean",
                    "description": "Show detailed version information"
                }
            }
        }
    })
}

/// Task management tool schema
fn get_task_schema() -> Value {
    json!({
        "name": "task",
        "description": "Read or update task files in .tasks directory. Supports reading task JSON and updating task status.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "Task ID (e.g., '1' for task-1.json)"
                },
                "operation": {
                    "type": "string",
                    "enum": ["read", "update"],
                    "description": "Operation to perform"
                },
                "content": {
                    "type": "string",
                    "description": "Task JSON content (for update operation)"
                },
                "status": {
                    "type": "string",
                    "description": "Update task status (for update operation)"
                }
            },
            "required": ["task_id", "operation"]
        }
    })
}
