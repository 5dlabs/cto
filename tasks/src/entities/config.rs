//! Configuration entities.

use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TasksConfig {
    /// AI model configurations
    #[serde(default)]
    pub models: ModelConfig,

    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,
}

impl TasksConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }
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

const fn default_max_tokens() -> u32 {
    64000
}

const fn default_temperature() -> f32 {
    0.2
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model_id: "claude-sonnet-4-20250514".to_string(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            base_url: None,
        }
    }
}

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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "projectName"
    )]
    pub project_name: Option<String>,

    /// Response language
    #[serde(default = "default_language", rename = "responseLanguage")]
    pub response_language: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

const fn default_num_tasks() -> u8 {
    10
}

const fn default_subtasks() -> u8 {
    5
}

fn default_priority() -> String {
    "medium".to_string()
}

fn default_tag() -> String {
    "master".to_string()
}

fn default_language() -> String {
    "English".to_string()
}

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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastSwitched"
    )]
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

impl RuntimeState {
    /// Create a new default runtime state
    pub fn new() -> Self {
        Self::default()
    }

    /// Switch to a new tag
    pub fn switch_tag(&mut self, tag: impl Into<String>) {
        self.current_tag = tag.into();
        self.last_switched = Some(chrono::Utc::now().to_rfc3339());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tasks_config_default() {
        let config = TasksConfig::default();
        assert_eq!(config.global.default_tag, "master");
        assert_eq!(config.global.default_num_tasks, 10);
    }

    #[test]
    fn test_model_settings_default() {
        let settings = ModelSettings::default();
        assert_eq!(settings.provider, "anthropic");
        assert_eq!(settings.max_tokens, 64000);
    }

    #[test]
    fn test_runtime_state_switch_tag() {
        let mut state = RuntimeState::default();
        state.switch_tag("feature-1");
        assert_eq!(state.current_tag, "feature-1");
        assert!(state.last_switched.is_some());
    }
}
