//! Tasks MCP Server - Exposes task management via Model Context Protocol.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unused_async)]
#![allow(clippy::format_push_string)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::map_unwrap_or)]

use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use tasks::domain::{AIDomain, DependencyDomain, TagsDomain, TasksDomain};
use tasks::entities::{TaskPriority, TaskStatus};
use tasks::errors::TasksError;
use tasks::storage::{FileStorage, Storage};

/// Tool mode for selective tool loading.
/// This helps optimize context window usage in AI IDEs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToolMode {
    /// Only essential task management tools
    Core,
    /// Core + frequently used tools (default)
    #[default]
    Standard,
    /// All available tools
    All,
}

impl std::str::FromStr for ToolMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "core" => Self::Core,
            "all" => Self::All,
            _ => Self::Standard,
        })
    }
}

/// Core tools - minimal set for basic task operations.
const CORE_TOOLS: &[&str] = &[
    "get_tasks",
    "get_task",
    "next_task",
    "set_task_status",
    "parse_prd",
    "expand_task",
];

/// Standard tools - core + frequently used tools.
const STANDARD_TOOLS: &[&str] = &[
    // Core tools
    "get_tasks",
    "get_task",
    "next_task",
    "set_task_status",
    "parse_prd",
    "expand_task",
    // Additional standard tools
    "add_task",
    "analyze_complexity",
    "expand_all",
    "add_subtask",
    "remove_task",
    "add_task_ai",
    "complexity_report",
    "validate_dependencies",
];

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// MCP server state
struct McpServer {
    tasks: TasksDomain,
    tags: TagsDomain,
    deps: DependencyDomain,
    ai: AIDomain,
    project_path: PathBuf,
    tool_mode: ToolMode,
}

impl McpServer {
    fn new(project_path: PathBuf, tool_mode: ToolMode) -> Self {
        let storage = Arc::new(FileStorage::new(&project_path));
        Self {
            tasks: TasksDomain::new(Arc::clone(&storage) as Arc<dyn Storage>),
            tags: TagsDomain::new(Arc::clone(&storage) as Arc<dyn Storage>),
            deps: DependencyDomain::new(Arc::clone(&storage) as Arc<dyn Storage>),
            ai: AIDomain::new(Arc::clone(&storage) as Arc<dyn Storage>),
            project_path,
            tool_mode,
        }
    }

    /// Check if a tool should be included based on the current tool mode.
    fn should_include_tool(&self, tool_name: &str) -> bool {
        match self.tool_mode {
            ToolMode::Core => CORE_TOOLS.contains(&tool_name),
            ToolMode::Standard => STANDARD_TOOLS.contains(&tool_name),
            ToolMode::All => true,
        }
    }

    async fn handle_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone().unwrap_or(Value::Null);

        match request.method.as_str() {
            "initialize" => self.handle_initialize(id).await,
            "tools/list" => self.handle_tools_list(id).await,
            "tools/call" => self.handle_tool_call(id, request.params.as_ref()).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        }
    }

    async fn handle_initialize(&self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "tasks-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        }
    }

    async fn handle_tools_list(&self, id: Value) -> JsonRpcResponse {
        let all_tools = self.get_all_tool_definitions();

        // Filter tools based on tool mode
        let filtered_tools: Vec<Value> = all_tools
            .into_iter()
            .filter(|tool| {
                tool.get("name")
                    .and_then(|v| v.as_str())
                    .map(|name| self.should_include_tool(name))
                    .unwrap_or(false)
            })
            .collect();

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({ "tools": filtered_tools })),
            error: None,
        }
    }

    /// Get all tool definitions (unfiltered).
    #[allow(clippy::unused_self)]
    fn get_all_tool_definitions(&self) -> Vec<Value> {
        vec![
            json!({
                "name": "get_tasks",
                "description": "List all tasks with optional status filter",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "description": "Filter by status (pending, in-progress, done, etc.)"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context (defaults to current tag)"
                        }
                    }
                }
            }),
            json!({
                "name": "get_task",
                "description": "Get details of a specific task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task ID"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["id"]
                }
            }),
            json!({
                "name": "next_task",
                "description": "Get the next task to work on",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    }
                }
            }),
            json!({
                "name": "set_task_status",
                "description": "Update task status",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task ID"
                        },
                        "status": {
                            "type": "string",
                            "description": "New status (pending, in-progress, done, etc.)"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["id", "status"]
                }
            }),
            json!({
                "name": "add_task",
                "description": "Add a new task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Task title"
                        },
                        "description": {
                            "type": "string",
                            "description": "Task description"
                        },
                        "priority": {
                            "type": "string",
                            "description": "Priority (low, medium, high, critical)"
                        },
                        "dependencies": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Task IDs this task depends on"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["title"]
                }
            }),
            json!({
                "name": "list_tags",
                "description": "List all tags with statistics",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            json!({
                "name": "use_tag",
                "description": "Switch to a different tag context",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Tag name to switch to"
                        }
                    },
                    "required": ["name"]
                }
            }),
            json!({
                "name": "validate_dependencies",
                "description": "Validate all task dependencies",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    }
                }
            }),
            // AI-powered tools
            json!({
                "name": "parse_prd",
                "description": "Parse a PRD and generate tasks using AI",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "PRD content to parse"
                        },
                        "numTasks": {
                            "type": "integer",
                            "description": "Number of tasks to generate (default: 10)"
                        },
                        "research": {
                            "type": "boolean",
                            "description": "Use research mode for better results"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["content"]
                }
            }),
            json!({
                "name": "expand_task",
                "description": "Expand a task into subtasks using AI",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task ID to expand"
                        },
                        "num": {
                            "type": "integer",
                            "description": "Number of subtasks to generate"
                        },
                        "research": {
                            "type": "boolean",
                            "description": "Use research mode"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Force replace existing subtasks"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["id"]
                }
            }),
            json!({
                "name": "analyze_complexity",
                "description": "Analyze task complexity using AI",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "threshold": {
                            "type": "integer",
                            "description": "Complexity threshold (1-10, default: 5)"
                        },
                        "research": {
                            "type": "boolean",
                            "description": "Use research mode"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    }
                }
            }),
            json!({
                "name": "add_task_ai",
                "description": "Add a new task using AI to generate details",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "Description of the task to create"
                        },
                        "priority": {
                            "type": "string",
                            "description": "Priority (low, medium, high)"
                        },
                        "dependencies": {
                            "type": "array",
                            "items": { "type": "integer" },
                            "description": "Task IDs this task depends on"
                        },
                        "research": {
                            "type": "boolean",
                            "description": "Use research mode"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["prompt"]
                }
            }),
            json!({
                "name": "update_task_ai",
                "description": "Update a task using AI assistance",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task ID to update"
                        },
                        "prompt": {
                            "type": "string",
                            "description": "Context/changes to apply"
                        },
                        "append": {
                            "type": "boolean",
                            "description": "Append to details instead of replacing"
                        },
                        "research": {
                            "type": "boolean",
                            "description": "Use research mode"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["id", "prompt"]
                }
            }),
            // Dependency management tools
            json!({
                "name": "add_dependency",
                "description": "Add a dependency to a task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "taskId": {
                            "type": "string",
                            "description": "Task ID that will depend on another task"
                        },
                        "dependsOn": {
                            "type": "string",
                            "description": "Task ID to depend on"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["taskId", "dependsOn"]
                }
            }),
            json!({
                "name": "remove_dependency",
                "description": "Remove a dependency from a task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "taskId": {
                            "type": "string",
                            "description": "Task ID to remove dependency from"
                        },
                        "dependsOn": {
                            "type": "string",
                            "description": "Task ID to remove from dependencies"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["taskId", "dependsOn"]
                }
            }),
            json!({
                "name": "fix_dependencies",
                "description": "Fix invalid dependencies by removing references to non-existent tasks",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    }
                }
            }),
            // Task management tools
            json!({
                "name": "move_task",
                "description": "Move a task to a new position in the task list",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task ID to move"
                        },
                        "position": {
                            "type": "integer",
                            "description": "New position (0-indexed)"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["id", "position"]
                }
            }),
            json!({
                "name": "remove_task",
                "description": "Remove a task from the task list",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task ID to remove"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["id"]
                }
            }),
            json!({
                "name": "add_subtask",
                "description": "Add a subtask to a task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "taskId": {
                            "type": "string",
                            "description": "Parent task ID"
                        },
                        "title": {
                            "type": "string",
                            "description": "Subtask title"
                        },
                        "description": {
                            "type": "string",
                            "description": "Subtask description"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["taskId", "title"]
                }
            }),
            json!({
                "name": "remove_subtask",
                "description": "Remove a subtask from a task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "taskId": {
                            "type": "string",
                            "description": "Parent task ID"
                        },
                        "subtaskId": {
                            "type": "integer",
                            "description": "Subtask ID to remove"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["taskId", "subtaskId"]
                }
            }),
            json!({
                "name": "clear_subtasks",
                "description": "Clear all subtasks from a task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "taskId": {
                            "type": "string",
                            "description": "Task ID to clear subtasks from"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["taskId"]
                }
            }),
            // AI batch update tool
            json!({
                "name": "update_tasks_ai",
                "description": "Batch update multiple tasks starting from a specific ID using AI",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "fromId": {
                            "type": "integer",
                            "description": "Starting task ID for batch update"
                        },
                        "prompt": {
                            "type": "string",
                            "description": "Context/changes to apply to all tasks"
                        },
                        "research": {
                            "type": "boolean",
                            "description": "Use research mode"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["fromId", "prompt"]
                }
            }),
            // Scope adjustment tools
            json!({
                "name": "scope_down_task",
                "description": "Decrease the complexity of tasks by splitting them into simpler tasks",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "ids": {
                            "type": "string",
                            "description": "Comma-separated list of task IDs to scope down (e.g., '1,3,5')"
                        },
                        "strength": {
                            "type": "string",
                            "description": "Scoping strength: 'light', 'regular', or 'heavy' (default: regular)"
                        },
                        "prompt": {
                            "type": "string",
                            "description": "Custom prompt for specific scoping adjustments"
                        },
                        "threshold": {
                            "type": "integer",
                            "description": "Complexity threshold - tasks >= this are prioritized (default: 5)"
                        },
                        "research": {
                            "type": "boolean",
                            "description": "Use research mode for better results"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["ids"]
                }
            }),
            json!({
                "name": "scope_up_task",
                "description": "Increase the complexity of tasks by consolidating or expanding them",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "ids": {
                            "type": "string",
                            "description": "Comma-separated list of task IDs to scope up (e.g., '1,3,5')"
                        },
                        "strength": {
                            "type": "string",
                            "description": "Scoping strength: 'light', 'regular', or 'heavy' (default: regular)"
                        },
                        "prompt": {
                            "type": "string",
                            "description": "Custom prompt for specific scoping adjustments"
                        },
                        "threshold": {
                            "type": "integer",
                            "description": "Complexity threshold - tasks <= this are candidates for merging (default: 3)"
                        },
                        "research": {
                            "type": "boolean",
                            "description": "Use research mode for better results"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    },
                    "required": ["ids"]
                }
            }),
            json!({
                "name": "expand_all",
                "description": "Expand all pending tasks into subtasks based on complexity or defaults",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "num": {
                            "type": "integer",
                            "description": "Target number of subtasks per task (uses complexity/defaults otherwise)"
                        },
                        "research": {
                            "type": "boolean",
                            "description": "Enable research-backed subtask generation"
                        },
                        "prompt": {
                            "type": "string",
                            "description": "Additional context to guide subtask generation for all tasks"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Force regeneration of subtasks for tasks that already have them"
                        },
                        "model": {
                            "type": "string",
                            "description": "AI model to use"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    }
                }
            }),
            json!({
                "name": "complexity_report",
                "description": "Read the saved complexity analysis report",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "tag": {
                            "type": "string",
                            "description": "Tag context"
                        }
                    }
                }
            }),
        ]
    }

    async fn handle_tool_call(&self, id: Value, params: Option<&Value>) -> JsonRpcResponse {
        let Some(params) = params else {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            };
        };

        let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let result = match tool_name {
            "get_tasks" => self.tool_get_tasks(&arguments).await,
            "get_task" => self.tool_get_task(&arguments).await,
            "next_task" => self.tool_next_task(&arguments).await,
            "set_task_status" => self.tool_set_task_status(&arguments).await,
            "add_task" => self.tool_add_task(&arguments).await,
            "list_tags" => self.tool_list_tags(&arguments).await,
            "use_tag" => self.tool_use_tag(&arguments).await,
            "validate_dependencies" => self.tool_validate_deps(&arguments).await,
            // Dependency management tools
            "add_dependency" => self.tool_add_dependency(&arguments).await,
            "remove_dependency" => self.tool_remove_dependency(&arguments).await,
            "fix_dependencies" => self.tool_fix_dependencies(&arguments).await,
            // Task management tools
            "move_task" => self.tool_move_task(&arguments).await,
            "remove_task" => self.tool_remove_task(&arguments).await,
            "add_subtask" => self.tool_add_subtask(&arguments).await,
            "remove_subtask" => self.tool_remove_subtask(&arguments).await,
            "clear_subtasks" => self.tool_clear_subtasks(&arguments).await,
            // AI-powered tools
            "parse_prd" => self.tool_parse_prd(&arguments).await,
            "expand_task" => self.tool_expand_task(&arguments).await,
            "analyze_complexity" => self.tool_analyze_complexity(&arguments).await,
            "add_task_ai" => self.tool_add_task_ai(&arguments).await,
            "update_task_ai" => self.tool_update_task_ai(&arguments).await,
            "update_tasks_ai" => self.tool_update_tasks_ai(&arguments).await,
            // Scope adjustment tools
            "scope_down_task" => self.tool_scope_down(&arguments).await,
            "scope_up_task" => self.tool_scope_up(&arguments).await,
            "expand_all" => self.tool_expand_all(&arguments).await,
            "complexity_report" => self.tool_complexity_report(&arguments).await,
            _ => Err(format!("Unknown tool: {tool_name}")),
        };

        match result {
            Ok(content) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "content": [{
                        "type": "text",
                        "text": content
                    }]
                })),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Error: {}", e)
                    }],
                    "isError": true
                })),
                error: None,
            },
        }
    }

    async fn tool_get_tasks(&self, args: &Value) -> Result<String, String> {
        let tag = args.get("tag").and_then(|v| v.as_str());
        let status_filter = if let Some(s) = args.get("status").and_then(|v| v.as_str()) {
            Some(s.parse::<TaskStatus>().map_err(|e| e.to_string())?)
        } else {
            None
        };

        let tasks = self
            .tasks
            .list_tasks(tag, status_filter)
            .await
            .map_err(|e| e.to_string())?;

        let output: Vec<Value> = tasks
            .iter()
            .map(|t| {
                json!({
                    "id": t.id,
                    "title": t.title,
                    "status": t.status.to_string(),
                    "priority": t.priority.to_string(),
                    "dependencies": t.dependencies,
                    "subtasks": t.subtasks.len()
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&output).unwrap_or_else(|_| "[]".to_string()))
    }

    async fn tool_get_task(&self, args: &Value) -> Result<String, String> {
        let id = args
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'id' parameter")?;
        let tag = args.get("tag").and_then(|v| v.as_str());

        let task = self
            .tasks
            .get_task(id, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_string_pretty(&task).unwrap_or_else(|_| "{}".to_string()))
    }

    async fn tool_next_task(&self, args: &Value) -> Result<String, String> {
        let tag = args.get("tag").and_then(|v| v.as_str());

        if let Some(task) = self.tasks.next_task(tag).await.map_err(|e| e.to_string())? {
            Ok(serde_json::to_string_pretty(&task).unwrap_or_else(|_| "{}".to_string()))
        } else {
            Ok("No pending tasks available".to_string())
        }
    }

    async fn tool_set_task_status(&self, args: &Value) -> Result<String, String> {
        let id = args
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'id' parameter")?;
        let status_str = args
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'status' parameter")?;
        let tag = args.get("tag").and_then(|v| v.as_str());

        let status: TaskStatus = status_str.parse().map_err(|e: TasksError| e.to_string())?;

        self.tasks
            .set_status(id, status, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("Updated task {} to status: {}", id, status))
    }

    async fn tool_add_task(&self, args: &Value) -> Result<String, String> {
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'title' parameter")?;
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tag = args.get("tag").and_then(|v| v.as_str());

        let task = self
            .tasks
            .add_task(title, description, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("Created task {} - {}", task.id, task.title))
    }

    async fn tool_list_tags(&self, _args: &Value) -> Result<String, String> {
        let stats = self
            .tags
            .list_tags_with_stats()
            .await
            .map_err(|e| e.to_string())?;

        let output: Vec<Value> = stats
            .iter()
            .map(|s| {
                json!({
                    "name": s.name,
                    "taskCount": s.task_count,
                    "completedTasks": s.completed_tasks,
                    "isCurrent": s.is_current
                })
            })
            .collect();

        Ok(serde_json::to_string_pretty(&output).unwrap_or_else(|_| "[]".to_string()))
    }

    async fn tool_use_tag(&self, args: &Value) -> Result<String, String> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' parameter")?;

        self.tags.use_tag(name).await.map_err(|e| e.to_string())?;

        Ok(format!("Switched to tag: {}", name))
    }

    async fn tool_validate_deps(&self, args: &Value) -> Result<String, String> {
        let tag = args.get("tag").and_then(|v| v.as_str());

        let result = self.deps.validate(tag).await.map_err(|e| e.to_string())?;

        if result.is_valid {
            Ok("All dependencies are valid".to_string())
        } else {
            let mut output = String::from("Dependency issues found:\n");

            for invalid in &result.invalid_deps {
                output.push_str(&format!(
                    "  - Task {} depends on missing task {}\n",
                    invalid.task_id, invalid.dep_id
                ));
            }

            for cycle in &result.cycles {
                output.push_str(&format!("  - Cycle detected: {}\n", cycle.join(" -> ")));
            }

            Ok(output)
        }
    }

    // AI-powered tool implementations

    async fn tool_parse_prd(&self, args: &Value) -> Result<String, String> {
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' parameter")?;
        let num_tasks = args
            .get("numTasks")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32);
        let research = args
            .get("research")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let model = args.get("model").and_then(|v| v.as_str());
        let tag = args.get("tag").and_then(|v| v.as_str());

        let (tasks, usage) = self
            .ai
            .parse_prd(content, "prd", num_tasks, research, model)
            .await
            .map_err(|e| e.to_string())?;

        // Save tasks
        for task in &tasks {
            self.tasks
                .add_task_full(task.clone(), tag)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(format!(
            "Generated {} tasks. Tokens used: {} in, {} out.\n\n{}",
            tasks.len(),
            usage.input_tokens,
            usage.output_tokens,
            serde_json::to_string_pretty(&tasks).unwrap_or_else(|_| "[]".to_string())
        ))
    }

    async fn tool_expand_task(&self, args: &Value) -> Result<String, String> {
        let id = args
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'id' parameter")?;
        let num = args.get("num").and_then(|v| v.as_i64()).map(|n| n as i32);
        let research = args
            .get("research")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        let model = args.get("model").and_then(|v| v.as_str());
        let tag = args.get("tag").and_then(|v| v.as_str());

        let mut task = self
            .tasks
            .get_task(id, tag)
            .await
            .map_err(|e| e.to_string())?;

        if !task.subtasks.is_empty() && !force {
            return Err(format!(
                "Task {} already has {} subtask(s). Use force=true to replace.",
                id,
                task.subtasks.len()
            ));
        }

        if force {
            task.subtasks.clear();
        }

        let (subtasks, usage) = self
            .ai
            .expand_task(&task, num, research, None, None, model)
            .await
            .map_err(|e| e.to_string())?;

        task.subtasks = subtasks;
        self.tasks
            .update_task(&task, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!(
            "Added {} subtasks to task {}. Tokens: {} in, {} out.",
            task.subtasks.len(),
            id,
            usage.input_tokens,
            usage.output_tokens
        ))
    }

    async fn tool_analyze_complexity(&self, args: &Value) -> Result<String, String> {
        let threshold = args
            .get("threshold")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32)
            .unwrap_or(5);
        let research = args
            .get("research")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let model = args.get("model").and_then(|v| v.as_str());
        let tag = args.get("tag").and_then(|v| v.as_str());

        let tasks = self
            .tasks
            .list_tasks(tag, None)
            .await
            .map_err(|e| e.to_string())?;

        if tasks.is_empty() {
            return Ok("No tasks to analyze".to_string());
        }

        let (report, usage) = self
            .ai
            .analyze_complexity(&tasks, threshold, research, model)
            .await
            .map_err(|e| e.to_string())?;

        let needing_expansion: Vec<_> = report
            .tasks_needing_expansion()
            .iter()
            .map(|a| {
                json!({
                    "taskId": a.task_id,
                    "title": a.task_title,
                    "score": a.complexity_score,
                    "recommendedSubtasks": a.recommended_subtasks
                })
            })
            .collect();

        Ok(format!(
            "Analyzed {} tasks. {} need expansion (threshold: {}). Tokens: {} in, {} out.\n\nTasks needing expansion:\n{}",
            report.complexity_analysis.len(),
            needing_expansion.len(),
            threshold,
            usage.input_tokens,
            usage.output_tokens,
            serde_json::to_string_pretty(&needing_expansion).unwrap_or_else(|_| "[]".to_string())
        ))
    }

    async fn tool_add_task_ai(&self, args: &Value) -> Result<String, String> {
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'prompt' parameter")?;
        let priority = args
            .get("priority")
            .and_then(|v| v.as_str())
            .map(|p| p.parse::<TaskPriority>().unwrap_or(TaskPriority::Medium));
        let dependencies = args
            .get("dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_i64().map(|n| n as i32))
                    .collect()
            });
        let research = args
            .get("research")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let model = args.get("model").and_then(|v| v.as_str());
        let tag = args.get("tag").and_then(|v| v.as_str());

        let (task, usage) = self
            .ai
            .add_task(prompt, priority, dependencies, research, model)
            .await
            .map_err(|e| e.to_string())?;

        self.tasks
            .add_task_full(task.clone(), tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!(
            "Created task {} - {}. Tokens: {} in, {} out.\n\n{}",
            task.id,
            task.title,
            usage.input_tokens,
            usage.output_tokens,
            serde_json::to_string_pretty(&task).unwrap_or_else(|_| "{}".to_string())
        ))
    }

    async fn tool_update_task_ai(&self, args: &Value) -> Result<String, String> {
        let id = args
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'id' parameter")?;
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'prompt' parameter")?;
        let append = args
            .get("append")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let research = args
            .get("research")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let model = args.get("model").and_then(|v| v.as_str());
        let tag = args.get("tag").and_then(|v| v.as_str());

        let task = self
            .tasks
            .get_task(id, tag)
            .await
            .map_err(|e| e.to_string())?;

        let (updated_task, usage) = self
            .ai
            .update_task(&task, prompt, append, research, model)
            .await
            .map_err(|e| e.to_string())?;

        self.tasks
            .update_task(&updated_task, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!(
            "Updated task {}. Tokens: {} in, {} out.\n\n{}",
            id,
            usage.input_tokens,
            usage.output_tokens,
            serde_json::to_string_pretty(&updated_task).unwrap_or_else(|_| "{}".to_string())
        ))
    }

    // Dependency management tool implementations

    async fn tool_add_dependency(&self, args: &Value) -> Result<String, String> {
        let task_id = args
            .get("taskId")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'taskId' parameter")?;
        let depends_on = args
            .get("dependsOn")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'dependsOn' parameter")?;
        let tag = args.get("tag").and_then(|v| v.as_str());

        self.deps
            .add_dependency(task_id, depends_on, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!(
            "Added dependency: task {} now depends on task {}",
            task_id, depends_on
        ))
    }

    async fn tool_remove_dependency(&self, args: &Value) -> Result<String, String> {
        let task_id = args
            .get("taskId")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'taskId' parameter")?;
        let depends_on = args
            .get("dependsOn")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'dependsOn' parameter")?;
        let tag = args.get("tag").and_then(|v| v.as_str());

        self.deps
            .remove_dependency(task_id, depends_on, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!(
            "Removed dependency: task {} no longer depends on task {}",
            task_id, depends_on
        ))
    }

    async fn tool_fix_dependencies(&self, args: &Value) -> Result<String, String> {
        let tag = args.get("tag").and_then(|v| v.as_str());

        let fixed_count = self.deps.fix(tag).await.map_err(|e| e.to_string())?;

        if fixed_count == 0 {
            Ok("No invalid dependencies found".to_string())
        } else {
            Ok(format!(
                "Fixed {} invalid dependencies (removed references to non-existent tasks)",
                fixed_count
            ))
        }
    }

    // Task management tool implementations

    async fn tool_move_task(&self, args: &Value) -> Result<String, String> {
        let id = args
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'id' parameter")?;
        let position = args
            .get("position")
            .and_then(|v| v.as_u64())
            .ok_or("Missing 'position' parameter")? as usize;
        let tag = args.get("tag").and_then(|v| v.as_str());

        self.tasks
            .move_task(id, position, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("Moved task {} to position {}", id, position))
    }

    async fn tool_remove_task(&self, args: &Value) -> Result<String, String> {
        let id = args
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'id' parameter")?;
        let tag = args.get("tag").and_then(|v| v.as_str());

        self.tasks
            .remove_task(id, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("Removed task {}", id))
    }

    async fn tool_add_subtask(&self, args: &Value) -> Result<String, String> {
        let task_id = args
            .get("taskId")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'taskId' parameter")?;
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'title' parameter")?;
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tag = args.get("tag").and_then(|v| v.as_str());

        let subtask = self
            .tasks
            .add_subtask(task_id, title, description, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!(
            "Added subtask {} to task {}: {}",
            subtask.id, task_id, subtask.title
        ))
    }

    async fn tool_remove_subtask(&self, args: &Value) -> Result<String, String> {
        let task_id = args
            .get("taskId")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'taskId' parameter")?;
        let subtask_id = args
            .get("subtaskId")
            .and_then(|v| v.as_u64())
            .ok_or("Missing 'subtaskId' parameter")? as u32;
        let tag = args.get("tag").and_then(|v| v.as_str());

        let mut task = self
            .tasks
            .get_task(task_id, tag)
            .await
            .map_err(|e| e.to_string())?;

        let removed = task
            .remove_subtask(subtask_id)
            .ok_or_else(|| format!("Subtask {} not found in task {}", subtask_id, task_id))?;

        self.tasks
            .update_task(&task, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!(
            "Removed subtask {} from task {}: {}",
            subtask_id, task_id, removed.title
        ))
    }

    async fn tool_clear_subtasks(&self, args: &Value) -> Result<String, String> {
        let task_id = args
            .get("taskId")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'taskId' parameter")?;
        let tag = args.get("tag").and_then(|v| v.as_str());

        let count = self
            .tasks
            .clear_subtasks(task_id, tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("Cleared {} subtasks from task {}", count, task_id))
    }

    // AI batch update tool implementation

    async fn tool_update_tasks_ai(&self, args: &Value) -> Result<String, String> {
        let from_id = args
            .get("fromId")
            .and_then(|v| v.as_i64())
            .ok_or("Missing 'fromId' parameter")? as i32;
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'prompt' parameter")?;
        let research = args
            .get("research")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let model = args.get("model").and_then(|v| v.as_str());
        let tag = args.get("tag").and_then(|v| v.as_str());

        let (updated_tasks, usage) = self
            .ai
            .update_tasks(from_id, prompt, research, model)
            .await
            .map_err(|e| e.to_string())?;

        // Save updated tasks
        for task in &updated_tasks {
            self.tasks
                .update_task(task, tag)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(format!(
            "Updated {} tasks starting from ID {}. Tokens: {} in, {} out.\n\n{}",
            updated_tasks.len(),
            from_id,
            usage.input_tokens,
            usage.output_tokens,
            serde_json::to_string_pretty(&updated_tasks).unwrap_or_else(|_| "[]".to_string())
        ))
    }

    // Scope adjustment tool implementations

    async fn tool_scope_down(&self, args: &Value) -> Result<String, String> {
        let ids_str = args
            .get("ids")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'ids' parameter")?;
        let strength = args
            .get("strength")
            .and_then(|v| v.as_str())
            .unwrap_or("regular");
        let prompt = args.get("prompt").and_then(|v| v.as_str());
        let threshold = args
            .get("threshold")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32)
            .unwrap_or(5);
        let research = args
            .get("research")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let model = args.get("model").and_then(|v| v.as_str());
        let tag = args.get("tag").and_then(|v| v.as_str());

        // Parse task IDs
        let task_ids: Vec<&str> = ids_str.split(',').map(str::trim).collect();

        // Get the tasks
        let tasks = self
            .tasks
            .get_tasks(&task_ids, tag)
            .await
            .map_err(|e| e.to_string())?;

        if tasks.is_empty() {
            return Err("No valid tasks found for the provided IDs".to_string());
        }

        let (scoped_tasks, usage) = self
            .ai
            .scope_down(&tasks, strength, prompt, threshold, research, model)
            .await
            .map_err(|e| e.to_string())?;

        // Save the scoped tasks (this may include new tasks from splits)
        for task in &scoped_tasks {
            // Check if task exists to decide add vs update
            if self.tasks.get_task(&task.id, tag).await.is_ok() {
                self.tasks
                    .update_task(task, tag)
                    .await
                    .map_err(|e| e.to_string())?;
            } else {
                self.tasks
                    .add_task_full(task.clone(), tag)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(format!(
            "Scoped down {} tasks into {} tasks (strength: {}). Tokens: {} in, {} out.\n\n{}",
            tasks.len(),
            scoped_tasks.len(),
            strength,
            usage.input_tokens,
            usage.output_tokens,
            serde_json::to_string_pretty(&scoped_tasks).unwrap_or_else(|_| "[]".to_string())
        ))
    }

    async fn tool_scope_up(&self, args: &Value) -> Result<String, String> {
        let ids_str = args
            .get("ids")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'ids' parameter")?;
        let strength = args
            .get("strength")
            .and_then(|v| v.as_str())
            .unwrap_or("regular");
        let prompt = args.get("prompt").and_then(|v| v.as_str());
        let threshold = args
            .get("threshold")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32)
            .unwrap_or(3);
        let research = args
            .get("research")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let model = args.get("model").and_then(|v| v.as_str());
        let tag = args.get("tag").and_then(|v| v.as_str());

        // Parse task IDs
        let task_ids: Vec<&str> = ids_str.split(',').map(str::trim).collect();

        // Get the tasks
        let tasks = self
            .tasks
            .get_tasks(&task_ids, tag)
            .await
            .map_err(|e| e.to_string())?;

        if tasks.is_empty() {
            return Err("No valid tasks found for the provided IDs".to_string());
        }

        let (scoped_tasks, usage) = self
            .ai
            .scope_up(&tasks, strength, prompt, threshold, research, model)
            .await
            .map_err(|e| e.to_string())?;

        // For scope up, we may need to remove merged tasks and update remaining
        let original_ids: HashSet<_> = tasks.iter().map(|t| t.id.as_str()).collect();
        let new_ids: HashSet<_> = scoped_tasks.iter().map(|t| t.id.as_str()).collect();

        // Remove tasks that were merged (exist in original but not in new)
        for id in original_ids.difference(&new_ids) {
            self.tasks.remove_task(id, tag).await.ok(); // Ignore errors for missing tasks
        }

        // Save the consolidated tasks
        for task in &scoped_tasks {
            if self.tasks.get_task(&task.id, tag).await.is_ok() {
                self.tasks
                    .update_task(task, tag)
                    .await
                    .map_err(|e| e.to_string())?;
            } else {
                self.tasks
                    .add_task_full(task.clone(), tag)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(format!(
            "Scoped up {} tasks into {} tasks (strength: {}). Tokens: {} in, {} out.\n\n{}",
            tasks.len(),
            scoped_tasks.len(),
            strength,
            usage.input_tokens,
            usage.output_tokens,
            serde_json::to_string_pretty(&scoped_tasks).unwrap_or_else(|_| "[]".to_string())
        ))
    }

    async fn tool_expand_all(&self, args: &Value) -> Result<String, String> {
        let num = args.get("num").and_then(|v| v.as_i64()).map(|n| n as i32);
        let research = args
            .get("research")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let prompt = args.get("prompt").and_then(|v| v.as_str());
        let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        let model = args.get("model").and_then(|v| v.as_str());
        let tag = args.get("tag").and_then(|v| v.as_str());

        // Try to load complexity report
        let complexity_report = self.load_complexity_report(tag).await.ok();

        let (expanded_tasks, usage) = self
            .ai
            .expand_all(
                num,
                force,
                research,
                prompt,
                complexity_report.as_ref(),
                model,
            )
            .await
            .map_err(|e| e.to_string())?;

        // Count tasks that got subtasks
        let tasks_with_subtasks = expanded_tasks
            .iter()
            .filter(|t| !t.subtasks.is_empty())
            .count();

        // Save expanded tasks
        for task in &expanded_tasks {
            self.tasks
                .update_task(task, tag)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(format!(
            "Expanded {} tasks with subtasks. Tokens: {} in, {} out.",
            tasks_with_subtasks, usage.input_tokens, usage.output_tokens
        ))
    }

    async fn tool_complexity_report(&self, args: &Value) -> Result<String, String> {
        let tag = args.get("tag").and_then(|v| v.as_str());

        let report = self
            .load_complexity_report(tag)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string()))
    }

    // Helper method to load complexity report
    async fn load_complexity_report(
        &self,
        _tag: Option<&str>,
    ) -> Result<tasks::ai::schemas::ComplexityReport, String> {
        // Complexity reports are stored in .tasks/complexity-report.json
        let report_path = self.project_path.join(".tasks/complexity-report.json");

        if !report_path.exists() {
            return Err("No complexity report found. Run analyze_complexity first.".to_string());
        }

        let content = std::fs::read_to_string(&report_path)
            .map_err(|e| format!("Failed to read complexity report: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse complexity report: {}", e))
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    // Get project path from environment or current directory
    let project_path = std::env::var("TASKS_PROJECT_PATH").map_or_else(
        |_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    );

    // Get tool mode from environment variable (core, standard, all)
    let tool_mode = std::env::var("TASKS_TOOL_MODE")
        .map(|s| s.parse::<ToolMode>().unwrap_or_default())
        .unwrap_or_default();

    let server = McpServer::new(project_path, tool_mode);

    // Read from stdin, write to stdout (JSON-RPC over stdio)
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();

    for line in reader.lines() {
        let Ok(line) = line else { break };

        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {e}"),
                        data: None,
                    }),
                };
                let _ = writeln!(
                    stdout,
                    "{}",
                    serde_json::to_string(&error_response).unwrap()
                );
                let _ = stdout.flush();
                continue;
            }
        };

        let response = server.handle_request(&request).await;
        let _ = writeln!(stdout, "{}", serde_json::to_string(&response).unwrap());
        let _ = stdout.flush();
    }
}
