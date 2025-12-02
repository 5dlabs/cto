use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;

/// User context configuration stored per context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub context_id: String,
    pub project_path: String,
    pub user_id: Option<String>,
    pub client_type: Option<String>,
    #[serde(default)]
    pub enabled_tools: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub disabled_tools: HashMap<String, Vec<String>>,
    pub last_updated: String,
}

impl ContextConfig {
    /// Create a new context config
    pub fn new(project_path: String, user_id: Option<String>, client_type: Option<String>) -> Self {
        let context_key = if let Some(ref uid) = user_id {
            format!("{project_path}+{uid}")
        } else {
            project_path.clone()
        };

        let context_id = Self::hash_context(&context_key);

        Self {
            context_id,
            project_path,
            user_id,
            client_type,
            enabled_tools: HashMap::new(),
            disabled_tools: HashMap::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Generate a hash for the context key to create a safe filename
    fn hash_context(context_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(context_key.as_bytes());
        let result = hasher.finalize();
        format!("{result:x}")[..16].to_string() // Use first 16 chars of hash
    }

    /// Update the last_updated timestamp
    pub fn touch(&mut self) {
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// Check if a tool is enabled in this context
    pub fn is_tool_enabled(&self, server_name: &str, tool_name: &str) -> Option<bool> {
        // Check disabled first (takes precedence)
        if let Some(disabled_tools) = self.disabled_tools.get(server_name) {
            if disabled_tools.contains(&tool_name.to_string()) {
                return Some(false);
            }
        }

        // Check enabled
        if let Some(enabled_tools) = self.enabled_tools.get(server_name) {
            if enabled_tools.contains(&tool_name.to_string()) {
                return Some(true);
            }
        }

        // No preference set in this context
        None
    }
}

/// Context manager for handling user-specific tool configurations
#[derive(Debug, Clone)]
pub struct ContextManager {
    contexts_dir: PathBuf,
    current_context: Option<ContextConfig>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Unable to determine home directory"))?;

        let contexts_dir = home_dir.join(".mcp-bridge-proxy").join("contexts");

        // Create contexts directory if it doesn't exist
        std::fs::create_dir_all(&contexts_dir)?;

        Ok(Self {
            contexts_dir,
            current_context: None,
        })
    }

    /// Load or create a context for the given project and user
    pub fn load_context(
        &mut self,
        project_path: String,
        user_id: Option<String>,
        client_type: Option<String>,
    ) -> Result<()> {
        let context_key = if let Some(ref uid) = user_id {
            format!("{project_path}+{uid}")
        } else {
            project_path.clone()
        };

        let context_id = ContextConfig::hash_context(&context_key);
        let context_file = self.contexts_dir.join(format!("{context_id}.json"));

        let context = if context_file.exists() {
            // Load existing context
            let content = std::fs::read_to_string(&context_file)?;
            let mut context: ContextConfig = serde_json::from_str(&content)?;

            // Update client type if provided and different
            if let Some(ref client) = client_type {
                if context.client_type.as_ref() != Some(client) {
                    context.client_type = Some(client.clone());
                    context.touch();
                }
            }

            context
        } else {
            // Create new context
            ContextConfig::new(project_path, user_id, client_type)
        };

        self.current_context = Some(context);
        Ok(())
    }

    /// Get the current context (must call load_context first)
    pub fn get_context(&self) -> Option<&ContextConfig> {
        self.current_context.as_ref()
    }

    /// Get mutable reference to current context
    pub fn get_context_mut(&mut self) -> Option<&mut ContextConfig> {
        self.current_context.as_mut()
    }

    /// Save the current context to file
    pub fn save_context(&self) -> Result<()> {
        if let Some(ref context) = self.current_context {
            let context_file = self
                .contexts_dir
                .join(format!("{}.json", context.context_id));
            let content = serde_json::to_string_pretty(context)?;
            std::fs::write(&context_file, content)?;
        }
        Ok(())
    }

    /// Check if a tool should be enabled based on context and default config
    /// Returns: Some(true) = force enabled, Some(false) = force disabled, None = use default
    pub fn should_tool_be_enabled(&self, server_name: &str, tool_name: &str) -> Option<bool> {
        self.current_context
            .as_ref()
            .and_then(|ctx| ctx.is_tool_enabled(server_name, tool_name))
    }

    /// List all available contexts
    pub fn list_contexts(&self) -> Result<Vec<ContextConfig>> {
        let mut contexts = Vec::new();

        for entry in std::fs::read_dir(&self.contexts_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(context) = serde_json::from_str::<ContextConfig>(&content) {
                    contexts.push(context);
                }
            }
        }

        Ok(contexts)
    }

    /// Clean up old context files (older than 30 days with no activity)
    pub fn cleanup_old_contexts(&self) -> Result<()> {
        let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);

        for entry in std::fs::read_dir(&self.contexts_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(context) = serde_json::from_str::<ContextConfig>(&content) {
                    if let Ok(last_updated) =
                        chrono::DateTime::parse_from_rfc3339(&context.last_updated)
                    {
                        if last_updated.with_timezone(&chrono::Utc) < thirty_days_ago {
                            if let Err(e) = std::fs::remove_file(&path) {
                                tracing::debug!(
                                    "Warning: Failed to remove old context file {path:?}: {e}"
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
