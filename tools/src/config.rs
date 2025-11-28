use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

fn default_transport() -> String {
    "stdio".to_string()
}

/// New simplified client-side configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Tools to request from the remote proxy server
    #[serde(rename = "remoteTools")]
    pub remote_tools: Vec<String>,
    /// Local servers to spawn in client context
    #[serde(rename = "localServers")]
    pub local_servers: HashMap<String, LocalServerConfig>,
}

/// Configuration for a local MCP server to be spawned by the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalServerConfig {
    /// Command to execute (e.g., "npx")
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Tools to expose from this local server
    pub tools: Vec<String>,
    /// Working directory for the server process (optional, defaults to project directory)
    #[serde(rename = "workingDirectory", default)]
    pub working_directory: Option<String>,
    /// Environment variables for the server process
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Configuration for a single MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    /// Transport type: "stdio" (default) or "http"
    #[serde(default = "default_transport")]
    pub transport: String,
    /// For stdio: command to execute
    #[serde(default)]
    pub command: String,
    /// For stdio: command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// For http: URL to connect to
    pub url: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Working directory for the server process (optional, defaults to project directory)
    /// Supports: "project_root", absolute paths like "/usr/local/bin", or relative paths
    #[serde(rename = "workingDirectory", default)]
    pub working_directory: Option<String>,
}

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServersConfig {
    pub servers: HashMap<String, ServerConfig>,
}

/// Session-based configuration sent during MCP initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Client information
    pub client_info: ClientInfo,
    /// Server configurations for this session
    pub servers: HashMap<String, ServerConfig>,
    /// Session-specific settings
    pub session_settings: SessionSettings,
}

/// Client information for session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
    pub working_directory: Option<String>,
    pub session_id: Option<String>,
}

/// Session-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    /// Request timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Maximum concurrent requests
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
    /// Whether to auto-start servers
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
}

fn default_timeout() -> u64 {
    30000
}

fn default_max_concurrent() -> u32 {
    10
}

fn default_auto_start() -> bool {
    true
}

/// Configuration manager for loading and managing server configs
#[derive(Debug, Clone)]
pub struct SystemConfigManager {
    config_path: PathBuf,
    config: ServersConfig,
}

impl SystemConfigManager {
    pub fn new(project_dir: Option<PathBuf>) -> Result<Self> {
        let config_path = if let Some(dir) = project_dir {
            dir.join("mcp-servers.json")
        } else {
            PathBuf::from("mcp-servers.json")
        };

        let config = if config_path.exists() {
            let config_content = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&config_content)?
        } else {
            ServersConfig {
                servers: HashMap::new(),
            }
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn get_servers(&self) -> &HashMap<String, ServerConfig> {
        &self.config.servers
    }

    pub fn get_server(&self, name: &str) -> Option<&ServerConfig> {
        self.config.servers.get(name)
    }

    /// Save the configuration to file
    pub fn save(&self) -> Result<()> {
        let config_json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, config_json)?;
        Ok(())
    }

    /// Get a mutable reference to the config for bulk updates
    pub fn get_config_mut(&mut self) -> &mut ServersConfig {
        &mut self.config
    }

    /// Get the path to the configuration file (for debugging)
    pub fn get_config_path(&self) -> &std::path::Path {
        &self.config_path
    }

    /// Enhanced atomic save with backup and recovery
    pub fn save_atomic(&self) -> Result<()> {
        let backup_path = self.create_backup()?;

        match self.save_atomic_impl() {
            Ok(()) => {
                self.cleanup_old_backups()?;
                Ok(())
            }
            Err(e) => {
                // Attempt to restore from backup on failure
                if let Err(restore_err) = self.restore_from_backup(&backup_path) {
                    tracing::debug!(
                        "Failed to restore from backup after save failure: {restore_err}"
                    );
                }
                Err(e)
            }
        }
    }

    /// Core atomic write implementation using temp file + rename
    fn save_atomic_impl(&self) -> Result<()> {
        use std::process;

        // 1. Generate unique temporary filename
        let timestamp = chrono::Utc::now().timestamp();
        let pid = process::id();
        let temp_filename = format!(
            "{}.tmp.{}.{}",
            self.config_path.file_name().unwrap().to_string_lossy(),
            pid,
            timestamp
        );
        let temp_path = self.config_path.parent().unwrap().join(temp_filename);

        // 2. Write configuration to temporary file
        let config_json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(&temp_path, &config_json)?;

        // 3. Validate temporary file
        self.validate_config_file(&temp_path)?;

        // 4. Sync to disk for durability
        self.sync_file(&temp_path)?;

        // 5. Atomic rename
        std::fs::rename(&temp_path, &self.config_path)?;

        Ok(())
    }

    /// Create timestamped backup of current config
    fn create_backup(&self) -> Result<PathBuf> {
        if !self.config_path.exists() {
            // No existing config to backup
            return Ok(PathBuf::new());
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let backup_filename = format!(
            "{}.backup.{}",
            self.config_path.file_name().unwrap().to_string_lossy(),
            timestamp
        );
        let backup_path = self.config_path.parent().unwrap().join(backup_filename);

        std::fs::copy(&self.config_path, &backup_path)?;
        Ok(backup_path)
    }

    /// Restore configuration from backup file
    fn restore_from_backup(&self, backup_path: &PathBuf) -> Result<()> {
        if backup_path.exists() && !backup_path.as_os_str().is_empty() {
            std::fs::copy(backup_path, &self.config_path)?;
        }
        Ok(())
    }

    /// Validate configuration file for JSON syntax and basic structure
    fn validate_config_file(&self, file_path: &PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(file_path)?;

        // Parse JSON to validate syntax
        let parsed_config: ServersConfig = serde_json::from_str(&content)?;

        // Basic structure validation
        if parsed_config.servers.is_empty() {
            return Err(anyhow::anyhow!(
                "Configuration validation failed: no servers defined"
            ));
        }

        // Validate each server configuration
        for (server_name, server_config) in &parsed_config.servers {
            if server_config.command.is_empty() {
                return Err(anyhow::anyhow!(
                    "Configuration validation failed: server '{server_name}' has empty command"
                ));
            }
        }

        Ok(())
    }

    /// Sync file to disk for durability
    fn sync_file(&self, file_path: &PathBuf) -> Result<()> {
        use std::fs::File;

        let file = File::open(file_path)?;

        // Use fsync on Unix-like systems for durability
        #[cfg(unix)]
        {
            unsafe {
                if libc::fsync(file.as_raw_fd()) != 0 {
                    return Err(anyhow::anyhow!("Failed to sync file to disk"));
                }
            }
        }

        // On Windows, file is automatically synced on close
        #[cfg(windows)]
        {
            // File will be synced when dropped
        }

        Ok(())
    }

    /// Cleanup old backup files, keeping only the 5 most recent
    fn cleanup_old_backups(&self) -> Result<()> {
        let Some(parent_dir) = self.config_path.parent() else {
            return Ok(()); // Can't clean up if no parent directory
        };

        let config_filename = self.config_path.file_name().unwrap().to_string_lossy();
        let backup_pattern = format!("{config_filename}.backup.");

        // Find all backup files
        let mut backup_files = Vec::new();
        for entry in std::fs::read_dir(parent_dir)? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();

            if filename.starts_with(&backup_pattern) {
                if let Ok(metadata) = entry.metadata() {
                    backup_files.push((
                        entry.path(),
                        metadata
                            .modified()
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                    ));
                }
            }
        }

        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove files beyond the 5 most recent
        for (path, _) in backup_files.iter().skip(5) {
            if let Err(e) = std::fs::remove_file(path) {
                tracing::debug!("Warning: Failed to remove old backup file {path:?}: {e}");
            }
        }

        Ok(())
    }

    /// Cleanup orphaned temporary files on startup
    pub fn cleanup_temp_files(&self) -> Result<()> {
        let Some(parent_dir) = self.config_path.parent() else {
            return Ok(());
        };

        let config_filename = self.config_path.file_name().unwrap().to_string_lossy();
        let temp_pattern = format!("{config_filename}.tmp.");

        for entry in std::fs::read_dir(parent_dir)? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();

            if filename.starts_with(&temp_pattern) {
                // Remove temp files older than 1 hour
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let age = std::time::SystemTime::now()
                            .duration_since(modified)
                            .unwrap_or_default();
                        if age > std::time::Duration::from_secs(3600) {
                            // 1 hour
                            if let Err(e) = std::fs::remove_file(entry.path()) {
                                tracing::debug!(
                                    "Warning: Failed to remove temp file {:?}: {}",
                                    entry.path(),
                                    e
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

/// Helper function to resolve working directory patterns
pub fn resolve_working_directory(working_dir: &str, project_dir: &std::path::Path) -> PathBuf {
    match working_dir {
        "project_root" | "project" => project_dir.to_path_buf(),
        path if path.starts_with('/') => PathBuf::from(path), // Absolute path
        path => project_dir.join(path),                       // Relative to project directory
    }
}

/// Template context for variable substitution
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub project_dir: PathBuf,
    pub working_dir: PathBuf,
    pub server_name: String,
}

impl TemplateContext {
    pub fn new(project_dir: PathBuf, working_dir: PathBuf, server_name: String) -> Self {
        Self {
            project_dir,
            working_dir,
            server_name,
        }
    }
}

/// Substitute template variables in a string
/// Supports: {{project_dir}}, {{working_dir}}, {{server_name}}
pub fn substitute_template_variables(template: &str, context: &TemplateContext) -> String {
    let mut result = template.to_string();

    // Replace template variables
    result = result.replace("{{project_dir}}", &context.project_dir.to_string_lossy());
    result = result.replace("{{working_dir}}", &context.working_dir.to_string_lossy());
    result = result.replace("{{server_name}}", &context.server_name);

    result
}

/// Process environment variables with template substitution
pub fn process_env_templates(
    env: &HashMap<String, String>,
    context: &TemplateContext,
) -> HashMap<String, String> {
    env.iter()
        .map(|(key, value)| {
            let processed_key = substitute_template_variables(key, context);
            let processed_value = substitute_template_variables(value, context);
            (processed_key, processed_value)
        })
        .collect()
}
