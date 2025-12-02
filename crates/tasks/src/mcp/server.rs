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

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use tasks::domain::{AIDomain, DependencyDomain, TagsDomain, TasksDomain};
use tasks::entities::{TaskPriority, TaskStatus};
use tasks::errors::TasksError;
use tasks::storage::{FileStorage, Storage};

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
    #[allow(dead_code)]
    project_path: PathBuf,
}

impl McpServer {
    fn new(project_path: PathBuf) -> Self {
        let storage = Arc::new(FileStorage::new(&project_path));
        Self {
            tasks: TasksDomain::new(Arc::clone(&storage) as Arc<dyn Storage>),
            tags: TagsDomain::new(Arc::clone(&storage) as Arc<dyn Storage>),
            deps: DependencyDomain::new(Arc::clone(&storage) as Arc<dyn Storage>),
            ai: AIDomain::new(Arc::clone(&storage) as Arc<dyn Storage>),
            project_path,
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
        let tools = json!({
            "tools": [
                {
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
                },
                {
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
                },
                {
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
                },
                {
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
                },
                {
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
                },
                {
                    "name": "list_tags",
                    "description": "List all tags with statistics",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
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
                },
                {
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
                },
                // AI-powered tools
                {
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
                },
                {
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
                },
                {
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
                },
                {
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
                },
                {
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
                }
            ]
        });

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(tools),
            error: None,
        }
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
            // AI-powered tools
            "parse_prd" => self.tool_parse_prd(&arguments).await,
            "expand_task" => self.tool_expand_task(&arguments).await,
            "analyze_complexity" => self.tool_analyze_complexity(&arguments).await,
            "add_task_ai" => self.tool_add_task_ai(&arguments).await,
            "update_task_ai" => self.tool_update_task_ai(&arguments).await,
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

    let server = McpServer::new(project_path);

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
