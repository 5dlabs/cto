# Tasks Crate Design Document

## Executive Summary

This document outlines the design for `tasks`, a Rust-native task management system inspired by [Taskmaster](https://github.com/eyaltoledano/claude-task-master). The goal is to create a fully-featured, license-compatible alternative that can be commercialized without restrictions.

### Motivation

Taskmaster is licensed under **MIT with Commons Clause**, which prohibits:
- Selling the software itself
- Offering it as a hosted service
- Creating competing products based on it

Since our CTO platform may eventually be commercialized, we need a clean-room implementation that provides equivalent functionality without licensing restrictions.

### Approach

This is a **clean-room implementation**—we're studying Taskmaster's architecture and feature set, then building our own Rust implementation from scratch. The TypeScript codebase serves as a reference for understanding:
- Core concepts and data structures
- User-facing features and workflows
- Integration patterns (MCP, CLI)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Entry Points                               │
├─────────────────────────────────────────────────────────────────────┤
│   CLI (tasks-cli)        │   MCP Server (tasks-mcp)                 │
│   - clap-based commands  │   - rmcp-based tools                     │
│   - Terminal UI          │   - JSON-RPC integration                 │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Core Library (tasks-core)                     │
├─────────────────────────────────────────────────────────────────────┤
│  Domains (Facades)                                                   │
│  ├── TasksDomain      - Task CRUD, status, expansion                 │
│  ├── ConfigDomain     - Configuration management                     │
│  ├── WorkflowDomain   - TDD workflow orchestration                   │
│  ├── GitDomain        - Git operations, branch integration           │
│  └── TagsDomain       - Tag management and context switching         │
├─────────────────────────────────────────────────────────────────────┤
│  Services                                                            │
│  ├── TaskService          - Core task operations                     │
│  ├── StorageService       - Storage abstraction                      │
│  ├── ConfigService        - Config loading/saving                    │
│  ├── ComplexityService    - Task complexity analysis                 │
│  └── DependencyService    - Dependency validation/resolution         │
├─────────────────────────────────────────────────────────────────────┤
│  AI Integration (Optional Feature)                                   │
│  ├── AIProvider trait     - Provider abstraction                     │
│  ├── Anthropic, OpenAI, etc. providers                               │
│  └── Prompt templates     - PRD parsing, expansion, research         │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Storage Layer                                │
├─────────────────────────────────────────────────────────────────────┤
│  FileStorage              │   ApiStorage (Future)                    │
│  - tasks.json             │   - Remote API backend                   │
│  - config.json            │   - Supabase integration                 │
│  - state.json             │                                          │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Crate Structure

```
crates/
├── tasks-core/           # Core library - all business logic
│   ├── src/
│   │   ├── lib.rs
│   │   ├── domain/       # Domain facades
│   │   │   ├── mod.rs
│   │   │   ├── tasks.rs
│   │   │   ├── config.rs
│   │   │   ├── workflow.rs
│   │   │   ├── git.rs
│   │   │   └── tags.rs
│   │   ├── entities/     # Core data structures
│   │   │   ├── mod.rs
│   │   │   ├── task.rs
│   │   │   ├── subtask.rs
│   │   │   ├── tag.rs
│   │   │   └── config.rs
│   │   ├── services/     # Business logic services
│   │   │   ├── mod.rs
│   │   │   ├── task_service.rs
│   │   │   ├── storage_service.rs
│   │   │   ├── complexity_service.rs
│   │   │   └── dependency_service.rs
│   │   ├── storage/      # Storage implementations
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs
│   │   │   ├── file_storage.rs
│   │   │   └── api_storage.rs
│   │   ├── ai/           # AI provider integration (feature-gated)
│   │   │   ├── mod.rs
│   │   │   ├── provider.rs
│   │   │   ├── anthropic.rs
│   │   │   ├── openai.rs
│   │   │   └── prompts/
│   │   └── error.rs      # Error types
│   └── Cargo.toml
│
├── tasks-cli/            # CLI binary
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/     # Command implementations
│   │   │   ├── mod.rs
│   │   │   ├── init.rs
│   │   │   ├── list.rs
│   │   │   ├── show.rs
│   │   │   ├── next.rs
│   │   │   ├── status.rs
│   │   │   ├── expand.rs
│   │   │   ├── add.rs
│   │   │   ├── update.rs
│   │   │   ├── tags.rs
│   │   │   ├── models.rs
│   │   │   ├── parse_prd.rs
│   │   │   ├── complexity.rs
│   │   │   ├── dependencies.rs
│   │   │   ├── move_task.rs
│   │   │   └── research.rs
│   │   └── ui/           # Terminal UI components
│   │       ├── mod.rs
│   │       ├── display.rs
│   │       ├── tables.rs
│   │       └── formatters.rs
│   └── Cargo.toml
│
└── tasks-mcp/            # MCP server binary
    ├── src/
    │   ├── main.rs
    │   ├── server.rs
    │   └── tools/        # MCP tool implementations
    │       ├── mod.rs
    │       ├── tasks.rs
    │       ├── tags.rs
    │       ├── workflow.rs
    │       └── utils.rs
    └── Cargo.toml
```

---

## Core Data Structures

### Task Entity

```rust
// tasks-core/src/entities/task.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Task status values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Deferred,
    Cancelled,
    Blocked,
    Review,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Task complexity levels (from complexity analysis)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Core task structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier (string to support alphanumeric IDs like "TAS-123")
    pub id: String,
    
    /// Brief, descriptive title
    pub title: String,
    
    /// Concise description of what the task involves
    pub description: String,
    
    /// Current task status
    #[serde(default)]
    pub status: TaskStatus,
    
    /// Task priority level
    #[serde(default)]
    pub priority: TaskPriority,
    
    /// IDs of prerequisite tasks
    #[serde(default)]
    pub dependencies: Vec<String>,
    
    /// In-depth implementation instructions
    #[serde(default)]
    pub details: String,
    
    /// Verification approach
    #[serde(default, rename = "testStrategy")]
    pub test_strategy: String,
    
    /// List of subtasks
    #[serde(default)]
    pub subtasks: Vec<Subtask>,
    
    // Optional enhanced properties
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    
    /// Estimated effort (hours or story points)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<u32>,
    
    /// Actual effort spent
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "actualEffort")]
    pub actual_effort: Option<u32>,
    
    /// Task tags (not to be confused with task list tags)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    
    /// Assigned user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    
    // Complexity analysis fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complexity: Option<ComplexityInfo>,
}

/// Complexity information from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityInfo {
    /// Complexity score (1-10)
    pub score: u8,
    
    /// Recommended number of subtasks
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "recommendedSubtasks")]
    pub recommended_subtasks: Option<u8>,
    
    /// AI-generated expansion prompt
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "expansionPrompt")]
    pub expansion_prompt: Option<String>,
    
    /// Reasoning for complexity score
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

impl Task {
    /// Create a new task with minimal required fields
    pub fn new(id: impl Into<String>, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            status: TaskStatus::default(),
            priority: TaskPriority::default(),
            dependencies: Vec::new(),
            details: String::new(),
            test_strategy: String::new(),
            subtasks: Vec::new(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            effort: None,
            actual_effort: None,
            tags: Vec::new(),
            assignee: None,
            complexity: None,
        }
    }
    
    /// Check if task can be marked as complete
    pub fn can_complete(&self) -> bool {
        // Cannot complete if already done or cancelled
        if matches!(self.status, TaskStatus::Done | TaskStatus::Cancelled) {
            return false;
        }
        
        // Cannot complete if blocked
        if self.status == TaskStatus::Blocked {
            return false;
        }
        
        // All subtasks must be complete
        self.subtasks.iter().all(|s| {
            matches!(s.status, TaskStatus::Done | TaskStatus::Cancelled)
        })
    }
    
    /// Check if task has unmet dependencies
    pub fn has_blocking_dependencies(&self, completed_tasks: &[&str]) -> bool {
        self.dependencies.iter().any(|dep| !completed_tasks.contains(&dep.as_str()))
    }
    
    /// Mark task as complete (returns error if cannot complete)
    pub fn mark_complete(&mut self) -> Result<(), TaskError> {
        if !self.can_complete() {
            return Err(TaskError::CannotComplete {
                task_id: self.id.clone(),
                reason: if !self.subtasks.iter().all(|s| matches!(s.status, TaskStatus::Done | TaskStatus::Cancelled)) {
                    "incomplete subtasks".to_string()
                } else {
                    format!("current status is {:?}", self.status)
                },
            });
        }
        
        self.status = TaskStatus::Done;
        self.updated_at = Some(Utc::now());
        Ok(())
    }
    
    /// Update task status with validation
    pub fn set_status(&mut self, new_status: TaskStatus) -> Result<(), TaskError> {
        // Business rule: Cannot move from done to pending
        if self.status == TaskStatus::Done && new_status == TaskStatus::Pending {
            return Err(TaskError::InvalidTransition {
                task_id: self.id.clone(),
                from: self.status,
                to: new_status,
            });
        }
        
        self.status = new_status;
        self.updated_at = Some(Utc::now());
        Ok(())
    }
    
    /// Get subtask by ID
    pub fn get_subtask(&self, subtask_id: &str) -> Option<&Subtask> {
        self.subtasks.iter().find(|s| s.id.to_string() == subtask_id)
    }
    
    /// Get mutable subtask by ID
    pub fn get_subtask_mut(&mut self, subtask_id: &str) -> Option<&mut Subtask> {
        self.subtasks.iter_mut().find(|s| s.id.to_string() == subtask_id)
    }
    
    /// Add a subtask
    pub fn add_subtask(&mut self, subtask: Subtask) {
        self.subtasks.push(subtask);
        self.updated_at = Some(Utc::now());
    }
}

/// Task-related errors
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Task {task_id} cannot be completed: {reason}")]
    CannotComplete { task_id: String, reason: String },
    
    #[error("Invalid status transition for task {task_id}: {from:?} -> {to:?}")]
    InvalidTransition {
        task_id: String,
        from: TaskStatus,
        to: TaskStatus,
    },
    
    #[error("Task {task_id} not found")]
    NotFound { task_id: String },
    
    #[error("Subtask {subtask_id} not found in task {task_id}")]
    SubtaskNotFound { task_id: String, subtask_id: String },
    
    #[error("Circular dependency detected: {cycle:?}")]
    CircularDependency { cycle: Vec<String> },
    
    #[error("Invalid dependency: task {task_id} depends on non-existent task {dep_id}")]
    InvalidDependency { task_id: String, dep_id: String },
}
```

### Subtask Entity

```rust
// tasks-core/src/entities/subtask.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::task::{TaskStatus, TaskPriority};

/// Subtask structure (nested within tasks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    /// Numeric ID within parent task
    pub id: u32,
    
    /// Parent task ID
    #[serde(rename = "parentId")]
    pub parent_id: String,
    
    /// Brief, descriptive title
    pub title: String,
    
    /// Concise description
    pub description: String,
    
    /// Current status
    #[serde(default)]
    pub status: TaskStatus,
    
    /// Priority (inherits from parent if not set)
    #[serde(default)]
    pub priority: TaskPriority,
    
    /// Dependencies (can reference other subtasks or tasks)
    #[serde(default)]
    pub dependencies: Vec<String>,
    
    /// Implementation details
    #[serde(default)]
    pub details: String,
    
    /// Test strategy
    #[serde(default, rename = "testStrategy")]
    pub test_strategy: String,
    
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
}

impl Subtask {
    /// Create a new subtask
    pub fn new(
        id: u32,
        parent_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id,
            parent_id: parent_id.into(),
            title: title.into(),
            description: description.into(),
            status: TaskStatus::default(),
            priority: TaskPriority::default(),
            dependencies: Vec::new(),
            details: String::new(),
            test_strategy: String::new(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            assignee: None,
        }
    }
    
    /// Get full ID (parentId.subtaskId format)
    pub fn full_id(&self) -> String {
        format!("{}.{}", self.parent_id, self.id)
    }
}
```

### Tag and Tagged Task List

```rust
// tasks-core/src/entities/tag.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::task::Task;

/// Tagged task collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedTaskList {
    /// Tasks in this tag context
    pub tasks: Vec<Task>,
    
    /// Tag metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TagMetadata>,
}

/// Tag metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagMetadata {
    /// Creation timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    
    /// Last update timestamp
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    
    /// Tag description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Version info
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    /// Project name
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "projectName")]
    pub project_name: Option<String>,
}

impl Default for TaggedTaskList {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            metadata: Some(TagMetadata {
                created: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                description: None,
                version: Some("1.0.0".to_string()),
                project_name: None,
            }),
        }
    }
}

/// Tag statistics
#[derive(Debug, Clone)]
pub struct TagStats {
    /// Tag name
    pub name: String,
    
    /// Whether this is the currently active tag
    pub is_current: bool,
    
    /// Total number of tasks
    pub task_count: usize,
    
    /// Number of completed tasks
    pub completed_tasks: usize,
    
    /// Status breakdown
    pub status_breakdown: std::collections::HashMap<String, usize>,
    
    /// Subtask counts
    pub subtask_counts: Option<SubtaskCounts>,
    
    /// Creation date
    pub created: Option<DateTime<Utc>>,
    
    /// Description
    pub description: Option<String>,
}

/// Subtask statistics
#[derive(Debug, Clone)]
pub struct SubtaskCounts {
    pub total: usize,
    pub by_status: std::collections::HashMap<String, usize>,
}
```

### Configuration

```rust
// tasks-core/src/entities/config.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksConfig {
    /// AI model configurations
    #[serde(default)]
    pub models: ModelConfig,
    
    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,
}

/// Model configuration for AI providers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Main model for generation/updates
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main: Option<ModelSettings>,
    
    /// Research model (typically Perplexity)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research: Option<ModelSettings>,
    
    /// Fallback model
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<ModelSettings>,
}

/// Individual model settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    /// Provider name (e.g., "anthropic", "openai")
    pub provider: String,
    
    /// Model ID
    #[serde(rename = "modelId")]
    pub model_id: String,
    
    /// Maximum tokens
    #[serde(default = "default_max_tokens", rename = "maxTokens")]
    pub max_tokens: u32,
    
    /// Temperature (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    
    /// Optional base URL override
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "baseURL")]
    pub base_url: Option<String>,
}

fn default_max_tokens() -> u32 { 64000 }
fn default_temperature() -> f32 { 0.2 }

/// Global configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Log level
    #[serde(default = "default_log_level", rename = "logLevel")]
    pub log_level: String,
    
    /// Debug mode
    #[serde(default)]
    pub debug: bool,
    
    /// Default number of tasks when parsing PRD
    #[serde(default = "default_num_tasks", rename = "defaultNumTasks")]
    pub default_num_tasks: u8,
    
    /// Default number of subtasks when expanding
    #[serde(default = "default_subtasks", rename = "defaultSubtasks")]
    pub default_subtasks: u8,
    
    /// Default task priority
    #[serde(default = "default_priority", rename = "defaultPriority")]
    pub default_priority: String,
    
    /// Default tag context
    #[serde(default = "default_tag", rename = "defaultTag")]
    pub default_tag: String,
    
    /// Project name
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "projectName")]
    pub project_name: Option<String>,
    
    /// Response language
    #[serde(default = "default_language", rename = "responseLanguage")]
    pub response_language: String,
}

fn default_log_level() -> String { "info".to_string() }
fn default_num_tasks() -> u8 { 10 }
fn default_subtasks() -> u8 { 5 }
fn default_priority() -> String { "medium".to_string() }
fn default_tag() -> String { "master".to_string() }
fn default_language() -> String { "English".to_string() }

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            debug: false,
            default_num_tasks: default_num_tasks(),
            default_subtasks: default_subtasks(),
            default_priority: default_priority(),
            default_tag: default_tag(),
            project_name: None,
            response_language: default_language(),
        }
    }
}

/// Runtime state (stored in state.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    /// Currently active tag
    #[serde(default = "default_tag", rename = "currentTag")]
    pub current_tag: String,
    
    /// Last tag switch timestamp
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastSwitched")]
    pub last_switched: Option<String>,
    
    /// Whether migration notice has been shown
    #[serde(default, rename = "migrationNoticeShown")]
    pub migration_notice_shown: bool,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            current_tag: default_tag(),
            last_switched: None,
            migration_notice_shown: false,
        }
    }
}
```

---

## Storage Layer

### Storage Trait

```rust
// tasks-core/src/storage/traits.rs

use async_trait::async_trait;
use crate::entities::{Task, TaskStatus, TaggedTaskList, TagStats};
use crate::error::TasksError;

/// Storage interface for task persistence
#[async_trait]
pub trait Storage: Send + Sync {
    /// Initialize storage (create directories, etc.)
    async fn initialize(&self) -> Result<(), TasksError>;
    
    /// Close and cleanup resources
    async fn close(&self) -> Result<(), TasksError>;
    
    /// Get storage type identifier
    fn storage_type(&self) -> &'static str;
    
    // === Task Operations ===
    
    /// Load all tasks for a tag
    async fn load_tasks(&self, tag: Option<&str>) -> Result<Vec<Task>, TasksError>;
    
    /// Load a single task by ID
    async fn load_task(&self, task_id: &str, tag: Option<&str>) -> Result<Option<Task>, TasksError>;
    
    /// Save all tasks for a tag
    async fn save_tasks(&self, tasks: &[Task], tag: Option<&str>) -> Result<(), TasksError>;
    
    /// Update a single task
    async fn update_task(&self, task_id: &str, task: &Task, tag: Option<&str>) -> Result<(), TasksError>;
    
    /// Update task status
    async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        tag: Option<&str>,
    ) -> Result<UpdateStatusResult, TasksError>;
    
    /// Delete a task
    async fn delete_task(&self, task_id: &str, tag: Option<&str>) -> Result<(), TasksError>;
    
    // === Tag Operations ===
    
    /// Get all available tags
    async fn get_all_tags(&self) -> Result<Vec<String>, TasksError>;
    
    /// Get tags with detailed statistics
    async fn get_tags_with_stats(&self) -> Result<Vec<TagStats>, TasksError>;
    
    /// Create a new tag
    async fn create_tag(&self, name: &str, copy_from: Option<&str>, description: Option<&str>) -> Result<(), TasksError>;
    
    /// Delete a tag
    async fn delete_tag(&self, name: &str) -> Result<(), TasksError>;
    
    /// Rename a tag
    async fn rename_tag(&self, old_name: &str, new_name: &str) -> Result<(), TasksError>;
    
    /// Copy a tag
    async fn copy_tag(&self, source: &str, target: &str, description: Option<&str>) -> Result<(), TasksError>;
    
    /// Check if tasks file exists
    async fn exists(&self, tag: Option<&str>) -> Result<bool, TasksError>;
}

/// Result of status update operation
#[derive(Debug, Clone)]
pub struct UpdateStatusResult {
    pub success: bool,
    pub task_id: String,
    pub old_status: TaskStatus,
    pub new_status: TaskStatus,
}
```

### File Storage Implementation

```rust
// tasks-core/src/storage/file_storage.rs

use std::path::{Path, PathBuf};
use tokio::fs;
use serde_json::Value;
use crate::entities::{Task, TaskStatus, TaggedTaskList};
use crate::error::TasksError;
use super::traits::{Storage, UpdateStatusResult};

/// File-based storage implementation
pub struct FileStorage {
    /// Project root path
    project_path: PathBuf,
    
    /// Path to tasks directory (.tasks/)
    tasks_dir: PathBuf,
    
    /// Path to tasks.json
    tasks_file: PathBuf,
}

impl FileStorage {
    /// Create a new file storage instance
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        let project_path = project_path.as_ref().to_path_buf();
        let tasks_dir = project_path.join(".tasks");
        let tasks_file = tasks_dir.join("tasks").join("tasks.json");
        
        Self {
            project_path,
            tasks_dir,
            tasks_file,
        }
    }
    
    /// Get the active tag from state.json
    async fn get_active_tag(&self) -> Result<String, TasksError> {
        let state_path = self.tasks_dir.join("state.json");
        
        match fs::read_to_string(&state_path).await {
            Ok(content) => {
                let state: Value = serde_json::from_str(&content)?;
                Ok(state.get("currentTag")
                    .and_then(|v| v.as_str())
                    .unwrap_or("master")
                    .to_string())
            }
            Err(_) => Ok("master".to_string()),
        }
    }
    
    /// Detect format of tasks.json (legacy vs tagged)
    fn detect_format(&self, data: &Value) -> TasksFormat {
        if data.get("tasks").is_some() && data.get("metadata").is_some() {
            TasksFormat::Standard
        } else if data.is_object() {
            // Check if keys look like tag names
            let keys: Vec<&str> = data.as_object()
                .map(|o| o.keys().map(|s| s.as_str()).collect())
                .unwrap_or_default();
            
            if keys.iter().any(|k| *k != "tasks" && *k != "metadata") {
                TasksFormat::Tagged
            } else {
                TasksFormat::Standard
            }
        } else {
            TasksFormat::Standard
        }
    }
    
    /// Extract tasks from JSON data for a specific tag
    fn extract_tasks(&self, data: &Value, tag: &str) -> Vec<Task> {
        match self.detect_format(data) {
            TasksFormat::Standard if tag == "master" => {
                data.get("tasks")
                    .and_then(|v| serde_json::from_value::<Vec<Task>>(v.clone()).ok())
                    .unwrap_or_default()
            }
            TasksFormat::Tagged => {
                data.get(tag)
                    .and_then(|t| t.get("tasks"))
                    .and_then(|v| serde_json::from_value::<Vec<Task>>(v.clone()).ok())
                    .unwrap_or_default()
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TasksFormat {
    Standard,  // { "tasks": [...], "metadata": {...} }
    Tagged,    // { "master": { "tasks": [...] }, "feature": { "tasks": [...] } }
}

#[async_trait::async_trait]
impl Storage for FileStorage {
    async fn initialize(&self) -> Result<(), TasksError> {
        fs::create_dir_all(self.tasks_dir.join("tasks")).await?;
        fs::create_dir_all(self.tasks_dir.join("reports")).await?;
        Ok(())
    }
    
    async fn close(&self) -> Result<(), TasksError> {
        Ok(())
    }
    
    fn storage_type(&self) -> &'static str {
        "file"
    }
    
    async fn load_tasks(&self, tag: Option<&str>) -> Result<Vec<Task>, TasksError> {
        let tag = tag.unwrap_or("master");
        
        let content = match fs::read_to_string(&self.tasks_file).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            }
            Err(e) => return Err(e.into()),
        };
        
        let data: Value = serde_json::from_str(&content)?;
        Ok(self.extract_tasks(&data, tag))
    }
    
    async fn load_task(&self, task_id: &str, tag: Option<&str>) -> Result<Option<Task>, TasksError> {
        let tasks = self.load_tasks(tag).await?;
        
        // Handle subtask notation (e.g., "1.2")
        if task_id.contains('.') {
            let parts: Vec<&str> = task_id.split('.').collect();
            if parts.len() == 2 {
                let parent_id = parts[0];
                let subtask_id = parts[1];
                
                if let Some(parent) = tasks.iter().find(|t| t.id == parent_id) {
                    if let Some(subtask) = parent.subtasks.iter().find(|s| s.id.to_string() == subtask_id) {
                        // Return subtask as a task-like structure
                        return Ok(Some(Task {
                            id: task_id.to_string(),
                            title: subtask.title.clone(),
                            description: subtask.description.clone(),
                            status: subtask.status,
                            priority: subtask.priority,
                            dependencies: subtask.dependencies.clone(),
                            details: subtask.details.clone(),
                            test_strategy: subtask.test_strategy.clone(),
                            subtasks: Vec::new(),
                            created_at: subtask.created_at,
                            updated_at: subtask.updated_at,
                            effort: None,
                            actual_effort: None,
                            tags: Vec::new(),
                            assignee: subtask.assignee.clone(),
                            complexity: None,
                        }));
                    }
                }
                return Ok(None);
            }
        }
        
        Ok(tasks.into_iter().find(|t| t.id == task_id))
    }
    
    async fn save_tasks(&self, tasks: &[Task], tag: Option<&str>) -> Result<(), TasksError> {
        let tag = tag.unwrap_or("master");
        
        // Ensure directory exists
        fs::create_dir_all(self.tasks_file.parent().unwrap()).await?;
        
        // Load existing data
        let mut data: Value = match fs::read_to_string(&self.tasks_file).await {
            Ok(c) => serde_json::from_str(&c)?,
            Err(_) => serde_json::json!({}),
        };
        
        // Create metadata
        let metadata = serde_json::json!({
            "version": "1.0.0",
            "lastModified": chrono::Utc::now().to_rfc3339(),
            "taskCount": tasks.len(),
            "completedCount": tasks.iter().filter(|t| t.status == TaskStatus::Done).count(),
        });
        
        // Update the tag in the data structure
        let format = self.detect_format(&data);
        
        match format {
            TasksFormat::Standard if tag == "master" => {
                data = serde_json::json!({
                    "tasks": tasks,
                    "metadata": metadata,
                });
            }
            _ => {
                // Convert to or update tagged format
                if let Some(obj) = data.as_object_mut() {
                    obj.insert(tag.to_string(), serde_json::json!({
                        "tasks": tasks,
                        "metadata": metadata,
                    }));
                }
            }
        }
        
        // Write file
        let content = serde_json::to_string_pretty(&data)?;
        fs::write(&self.tasks_file, content).await?;
        
        Ok(())
    }
    
    async fn update_task(&self, task_id: &str, task: &Task, tag: Option<&str>) -> Result<(), TasksError> {
        let mut tasks = self.load_tasks(tag).await?;
        
        if let Some(idx) = tasks.iter().position(|t| t.id == task_id) {
            tasks[idx] = task.clone();
            self.save_tasks(&tasks, tag).await?;
            Ok(())
        } else {
            Err(TasksError::TaskNotFound { task_id: task_id.to_string() })
        }
    }
    
    async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        tag: Option<&str>,
    ) -> Result<UpdateStatusResult, TasksError> {
        let mut tasks = self.load_tasks(tag).await?;
        
        // Handle subtask
        if task_id.contains('.') {
            let parts: Vec<&str> = task_id.split('.').collect();
            let parent_id = parts[0];
            let subtask_id: u32 = parts[1].parse().map_err(|_| TasksError::InvalidId { id: task_id.to_string() })?;
            
            if let Some(parent) = tasks.iter_mut().find(|t| t.id == parent_id) {
                if let Some(subtask) = parent.subtasks.iter_mut().find(|s| s.id == subtask_id) {
                    let old_status = subtask.status;
                    subtask.status = status;
                    subtask.updated_at = Some(chrono::Utc::now());
                    
                    // Auto-update parent status based on subtasks
                    let all_done = parent.subtasks.iter().all(|s| matches!(s.status, TaskStatus::Done | TaskStatus::Cancelled));
                    let any_in_progress = parent.subtasks.iter().any(|s| s.status == TaskStatus::InProgress);
                    let any_done = parent.subtasks.iter().any(|s| s.status == TaskStatus::Done);
                    
                    if all_done {
                        parent.status = TaskStatus::Done;
                    } else if any_in_progress || any_done {
                        parent.status = TaskStatus::InProgress;
                    }
                    
                    parent.updated_at = Some(chrono::Utc::now());
                    self.save_tasks(&tasks, tag).await?;
                    
                    return Ok(UpdateStatusResult {
                        success: true,
                        task_id: task_id.to_string(),
                        old_status,
                        new_status: status,
                    });
                }
            }
            return Err(TasksError::SubtaskNotFound { 
                task_id: parent_id.to_string(), 
                subtask_id: subtask_id.to_string() 
            });
        }
        
        // Handle regular task
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            let old_status = task.status;
            task.status = status;
            task.updated_at = Some(chrono::Utc::now());
            
            self.save_tasks(&tasks, tag).await?;
            
            Ok(UpdateStatusResult {
                success: true,
                task_id: task_id.to_string(),
                old_status,
                new_status: status,
            })
        } else {
            Err(TasksError::TaskNotFound { task_id: task_id.to_string() })
        }
    }
    
    async fn delete_task(&self, task_id: &str, tag: Option<&str>) -> Result<(), TasksError> {
        let mut tasks = self.load_tasks(tag).await?;
        let len_before = tasks.len();
        tasks.retain(|t| t.id != task_id);
        
        if tasks.len() == len_before {
            return Err(TasksError::TaskNotFound { task_id: task_id.to_string() });
        }
        
        self.save_tasks(&tasks, tag).await
    }
    
    async fn get_all_tags(&self) -> Result<Vec<String>, TasksError> {
        let content = match fs::read_to_string(&self.tasks_file).await {
            Ok(c) => c,
            Err(_) => return Ok(vec!["master".to_string()]),
        };
        
        let data: Value = serde_json::from_str(&content)?;
        
        match self.detect_format(&data) {
            TasksFormat::Standard => Ok(vec!["master".to_string()]),
            TasksFormat::Tagged => {
                let tags: Vec<String> = data.as_object()
                    .map(|o| o.keys().cloned().collect())
                    .unwrap_or_else(|| vec!["master".to_string()]);
                Ok(tags)
            }
        }
    }
    
    async fn get_tags_with_stats(&self) -> Result<Vec<crate::entities::TagStats>, TasksError> {
        let tags = self.get_all_tags().await?;
        let active_tag = self.get_active_tag().await?;
        
        let mut stats = Vec::new();
        for tag_name in tags {
            let tasks = self.load_tasks(Some(&tag_name)).await?;
            
            let mut status_breakdown = std::collections::HashMap::new();
            let mut completed = 0;
            let mut total_subtasks = 0;
            let mut subtasks_by_status: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            
            for task in &tasks {
                let status_key = format!("{:?}", task.status).to_lowercase();
                *status_breakdown.entry(status_key).or_insert(0) += 1;
                
                if task.status == TaskStatus::Done {
                    completed += 1;
                }
                
                for subtask in &task.subtasks {
                    total_subtasks += 1;
                    let sub_status = format!("{:?}", subtask.status).to_lowercase();
                    *subtasks_by_status.entry(sub_status).or_insert(0) += 1;
                }
            }
            
            stats.push(crate::entities::TagStats {
                name: tag_name.clone(),
                is_current: tag_name == active_tag,
                task_count: tasks.len(),
                completed_tasks: completed,
                status_breakdown,
                subtask_counts: if total_subtasks > 0 {
                    Some(crate::entities::SubtaskCounts {
                        total: total_subtasks,
                        by_status: subtasks_by_status,
                    })
                } else {
                    None
                },
                created: None,
                description: None,
            });
        }
        
        Ok(stats)
    }
    
    async fn create_tag(&self, name: &str, copy_from: Option<&str>, description: Option<&str>) -> Result<(), TasksError> {
        let tasks_to_copy = if let Some(source) = copy_from {
            self.load_tasks(Some(source)).await?
        } else {
            Vec::new()
        };
        
        // Check if tag already exists
        let existing_tags = self.get_all_tags().await?;
        if existing_tags.contains(&name.to_string()) {
            return Err(TasksError::TagAlreadyExists { name: name.to_string() });
        }
        
        self.save_tasks(&tasks_to_copy, Some(name)).await
    }
    
    async fn delete_tag(&self, name: &str) -> Result<(), TasksError> {
        if name == "master" {
            return Err(TasksError::CannotDeleteMasterTag);
        }
        
        let content = fs::read_to_string(&self.tasks_file).await?;
        let mut data: Value = serde_json::from_str(&content)?;
        
        if let Some(obj) = data.as_object_mut() {
            if obj.remove(name).is_none() {
                return Err(TasksError::TagNotFound { name: name.to_string() });
            }
        }
        
        let content = serde_json::to_string_pretty(&data)?;
        fs::write(&self.tasks_file, content).await?;
        
        Ok(())
    }
    
    async fn rename_tag(&self, old_name: &str, new_name: &str) -> Result<(), TasksError> {
        if old_name == "master" {
            return Err(TasksError::CannotRenameMasterTag);
        }
        
        let content = fs::read_to_string(&self.tasks_file).await?;
        let mut data: Value = serde_json::from_str(&content)?;
        
        if let Some(obj) = data.as_object_mut() {
            if let Some(tag_data) = obj.remove(old_name) {
                obj.insert(new_name.to_string(), tag_data);
            } else {
                return Err(TasksError::TagNotFound { name: old_name.to_string() });
            }
        }
        
        let content = serde_json::to_string_pretty(&data)?;
        fs::write(&self.tasks_file, content).await?;
        
        Ok(())
    }
    
    async fn copy_tag(&self, source: &str, target: &str, _description: Option<&str>) -> Result<(), TasksError> {
        let tasks = self.load_tasks(Some(source)).await?;
        self.save_tasks(&tasks, Some(target)).await
    }
    
    async fn exists(&self, _tag: Option<&str>) -> Result<bool, TasksError> {
        Ok(self.tasks_file.exists())
    }
}
```

---

## CLI Commands

### Command Structure

```rust
// tasks-cli/src/main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tasks")]
#[command(about = "Task management for AI-driven development", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Project root directory
    #[arg(long, global = true)]
    project: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project with tasks structure
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
        
        /// Project description
        #[arg(short, long)]
        description: Option<String>,
        
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
        /// Task description prompt
        #[arg(short, long)]
        prompt: String,
        
        /// Dependencies
        #[arg(short, long)]
        dependencies: Option<String>,
        
        /// Priority
        #[arg(long)]
        priority: Option<String>,
        
        /// Use research model
        #[arg(short, long)]
        research: bool,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// Expand a task into subtasks
    Expand {
        /// Task ID
        #[arg(short, long)]
        id: Option<String>,
        
        /// Expand all pending tasks
        #[arg(long)]
        all: bool,
        
        /// Number of subtasks
        #[arg(short, long)]
        num: Option<u8>,
        
        /// Additional context
        #[arg(short, long)]
        prompt: Option<String>,
        
        /// Force regeneration
        #[arg(short, long)]
        force: bool,
        
        /// Use research model
        #[arg(short, long)]
        research: bool,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// Update tasks from a starting ID
    Update {
        /// Starting task ID
        #[arg(long)]
        from: String,
        
        /// Update context/prompt
        #[arg(short, long)]
        prompt: String,
        
        /// Use research model
        #[arg(short, long)]
        research: bool,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// Update a specific task
    UpdateTask {
        /// Task ID
        #[arg(short, long)]
        id: String,
        
        /// Update prompt
        #[arg(short, long)]
        prompt: String,
        
        /// Append mode
        #[arg(long)]
        append: bool,
        
        /// Use research model
        #[arg(short, long)]
        research: bool,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// Update a subtask
    UpdateSubtask {
        /// Subtask ID (parentId.subtaskId)
        #[arg(short, long)]
        id: String,
        
        /// Update prompt
        #[arg(short, long)]
        prompt: String,
        
        /// Use research model
        #[arg(short, long)]
        research: bool,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// Parse a PRD file and generate tasks
    ParsePrd {
        /// Path to PRD file
        file: String,
        
        /// Number of tasks to generate
        #[arg(short, long)]
        num_tasks: Option<u8>,
        
        /// Force overwrite
        #[arg(short, long)]
        force: bool,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// Analyze task complexity
    AnalyzeComplexity {
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
        
        /// Complexity threshold
        #[arg(short, long)]
        threshold: Option<u8>,
        
        /// Use research model
        #[arg(short, long)]
        research: bool,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// View complexity report
    ComplexityReport {
        /// Report file
        #[arg(short, long)]
        file: Option<String>,
        
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
    
    /// Move tasks
    Move {
        /// Source task ID(s)
        #[arg(long)]
        from: String,
        
        /// Target position
        #[arg(long)]
        to: String,
        
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
        /// Task ID(s)
        #[arg(short, long)]
        id: Option<String>,
        
        /// Clear all
        #[arg(long)]
        all: bool,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// Generate task files
    Generate {
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// Configure AI models
    Models {
        /// Set main model
        #[arg(long)]
        set_main: Option<String>,
        
        /// Set research model
        #[arg(long)]
        set_research: Option<String>,
        
        /// Set fallback model
        #[arg(long)]
        set_fallback: Option<String>,
        
        /// Mark as Ollama model
        #[arg(long)]
        ollama: bool,
        
        /// Mark as OpenRouter model
        #[arg(long)]
        openrouter: bool,
        
        /// Interactive setup
        #[arg(long)]
        setup: bool,
    },
    
    /// Perform AI-powered research
    Research {
        /// Research query
        query: String,
        
        /// Task IDs for context
        #[arg(short, long)]
        id: Option<String>,
        
        /// Files for context
        #[arg(short, long)]
        files: Option<String>,
        
        /// Additional context
        #[arg(short, long)]
        context: Option<String>,
        
        /// Include project tree
        #[arg(long)]
        tree: bool,
        
        /// Detail level
        #[arg(long)]
        detail: Option<String>,
        
        /// Save to task
        #[arg(long)]
        save_to: Option<String>,
        
        /// Save to file
        #[arg(long)]
        save_file: bool,
        
        /// Tag context
        #[arg(long)]
        tag: Option<String>,
    },
}

#[derive(Subcommand)]
enum TagsCommands {
    /// List all tags
    List {
        /// Show metadata
        #[arg(long)]
        show_metadata: bool,
    },
    
    /// Add a new tag
    Add {
        /// Tag name
        name: Option<String>,
        
        /// Create from current branch
        #[arg(long)]
        from_branch: bool,
        
        /// Copy from current tag
        #[arg(long)]
        copy_from_current: bool,
        
        /// Copy from specific tag
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
```

---

## MCP Server Design

### MCP Tools

The MCP server will expose the following tools (matching Taskmaster functionality):

| Tool Name | Description |
|-----------|-------------|
| `initialize_project` | Initialize a new project |
| `get_tasks` | List tasks with optional filtering |
| `get_task` | Get details of specific task(s) |
| `next_task` | Get the next available task |
| `set_task_status` | Update task status |
| `add_task` | Add a new task using AI |
| `add_subtask` | Add a subtask to a parent task |
| `expand_task` | Break down a task into subtasks |
| `expand_all` | Expand all pending tasks |
| `update` | Update multiple tasks from a starting ID |
| `update_task` | Update a specific task |
| `update_subtask` | Append to a subtask |
| `parse_prd` | Parse PRD and generate tasks |
| `analyze_project_complexity` | Analyze task complexity |
| `complexity_report` | View complexity report |
| `add_dependency` | Add task dependency |
| `remove_dependency` | Remove task dependency |
| `validate_dependencies` | Check for dependency issues |
| `fix_dependencies` | Auto-fix dependency issues |
| `move_task` | Move task to new position |
| `remove_task` | Delete a task |
| `clear_subtasks` | Remove subtasks from a task |
| `generate` | Generate task files |
| `models` | View/configure AI models |
| `research` | AI-powered research |
| `list_tags` | List all tags |
| `add_tag` | Create a new tag |
| `delete_tag` | Delete a tag |
| `use_tag` | Switch tag context |
| `rename_tag` | Rename a tag |
| `copy_tag` | Copy a tag |

---

## AI Integration

### Provider Trait

```rust
// tasks-core/src/ai/provider.rs

use async_trait::async_trait;
use serde_json::Value;

/// AI provider interface
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &'static str;
    
    /// Generate text completion
    async fn generate_text(&self, params: GenerateTextParams) -> Result<String, AIError>;
    
    /// Generate structured object
    async fn generate_object(&self, params: GenerateObjectParams) -> Result<Value, AIError>;
    
    /// Check if provider supports streaming
    fn supports_streaming(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct GenerateTextParams {
    pub model_id: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GenerateObjectParams {
    pub model_id: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub system_prompt: Option<String>,
    pub schema: Value,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, thiserror::Error)]
pub enum AIError {
    #[error("API error: {message}")]
    ApiError { message: String },
    
    #[error("Invalid response: {message}")]
    InvalidResponse { message: String },
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Authentication failed")]
    AuthFailed,
    
    #[error("Model not found: {model_id}")]
    ModelNotFound { model_id: String },
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
```

---

## Implementation Phases

### Phase 1: Core Foundation (2-3 weeks)
- [ ] Set up crate structure
- [ ] Implement core entities (Task, Subtask, Tag, Config)
- [ ] Implement file storage layer
- [ ] Implement basic error handling
- [ ] Create CLI scaffolding with clap

### Phase 2: Basic CLI Commands (2-3 weeks)
- [ ] `init` - Project initialization
- [ ] `list` - List tasks
- [ ] `show` - Show task details
- [ ] `next` - Get next task
- [ ] `set-status` - Update status
- [ ] `tags` - Tag management

### Phase 3: Task Operations (2-3 weeks)
- [ ] `add` - Add tasks (without AI initially)
- [ ] `update` - Update tasks
- [ ] `remove` - Remove tasks
- [ ] `move` - Move tasks
- [ ] Dependency management
- [ ] `generate` - Task file generation

### Phase 4: AI Integration (2-3 weeks)
- [ ] AIProvider trait and implementations
- [ ] `parse-prd` - PRD parsing
- [ ] `expand` - Task expansion
- [ ] `analyze-complexity` - Complexity analysis
- [ ] `research` - AI research

### Phase 5: MCP Server (2-3 weeks)
- [ ] MCP server setup with rmcp
- [ ] Tool implementations
- [ ] Session handling
- [ ] Integration testing

### Phase 6: Polish & Documentation (1-2 weeks)
- [ ] Terminal UI improvements
- [ ] Comprehensive tests
- [ ] Documentation
- [ ] Examples and templates

---

## Key Differences from Taskmaster

1. **Language**: Rust instead of TypeScript/JavaScript
2. **License**: MIT or Apache 2.0 (fully permissive)
3. **Directory Structure**: `.tasks/` instead of `.taskmaster/`
4. **Binary Names**: `tasks` (CLI), `tasks-mcp` (MCP server)
5. **No Legacy Support**: Fresh start without backward compatibility burden
6. **API Storage**: Designed for future CTO platform integration

---

## Dependencies

### tasks-core
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
async-trait = "0.1"
tokio = { version = "1.0", features = ["fs", "macros", "rt-multi-thread"] }
tracing = "0.1"

# Optional AI features
reqwest = { version = "0.11", features = ["json"], optional = true }
```

### tasks-cli
```toml
[dependencies]
tasks-core = { path = "../tasks-core" }
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
colored = "2.0"
comfy-table = "7.0"
indicatif = "0.17"
dialoguer = "0.11"
```

### tasks-mcp
```toml
[dependencies]
tasks-core = { path = "../tasks-core" }
rmcp = { version = "0.1", features = ["server"] }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

---

## Next Steps

1. Review this design document with the team
2. Create the crate structure
3. Start with Phase 1 implementation
4. Engage the planning agent to generate TaskMaster tasks for the implementation

---

*Document created: 2025-11-28*
*Author: CTO Agent*

