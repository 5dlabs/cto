//! Configuration domain facade.

use std::path::{Path, PathBuf};

use tokio::fs;

use crate::entities::{ModelConfig, ModelSettings, TasksConfig};
use crate::errors::{TasksError, TasksResult};

/// Configuration domain facade
pub struct ConfigDomain {
    config_path: PathBuf,
}

impl ConfigDomain {
    /// Create a new config domain
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        Self {
            config_path: project_path.as_ref().join(".tasks/config.json"),
        }
    }

    /// Load configuration
    pub async fn load(&self) -> TasksResult<TasksConfig> {
        match fs::read_to_string(&self.config_path).await {
            Ok(content) => {
                let config: TasksConfig = serde_json::from_str(&content)?;
                Ok(config)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(TasksConfig::default()),
            Err(e) => Err(TasksError::FileReadError {
                path: self.config_path.display().to_string(),
                reason: e.to_string(),
            }),
        }
    }

    /// Save configuration
    pub async fn save(&self, config: &TasksConfig) -> TasksResult<()> {
        // Ensure directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content).await.map_err(|e| {
            TasksError::FileWriteError {
                path: self.config_path.display().to_string(),
                reason: e.to_string(),
            }
        })
    }

    /// Get model configuration
    pub async fn get_models(&self) -> TasksResult<ModelConfig> {
        let config = self.load().await?;
        Ok(config.models)
    }

    /// Set the main model
    pub async fn set_main_model(&self, settings: ModelSettings) -> TasksResult<()> {
        let mut config = self.load().await?;
        config.models.main = Some(settings);
        self.save(&config).await
    }

    /// Set the research model
    pub async fn set_research_model(&self, settings: ModelSettings) -> TasksResult<()> {
        let mut config = self.load().await?;
        config.models.research = Some(settings);
        self.save(&config).await
    }

    /// Set the fallback model
    pub async fn set_fallback_model(&self, settings: ModelSettings) -> TasksResult<()> {
        let mut config = self.load().await?;
        config.models.fallback = Some(settings);
        self.save(&config).await
    }

    /// Get global settings
    pub async fn get_global_settings(&self) -> TasksResult<crate::entities::GlobalConfig> {
        let config = self.load().await?;
        Ok(config.global)
    }

    /// Set project name
    pub async fn set_project_name(&self, name: impl Into<String>) -> TasksResult<()> {
        let mut config = self.load().await?;
        config.global.project_name = Some(name.into());
        self.save(&config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let domain = ConfigDomain::new(temp_dir.path());

        let config = domain.load().await.unwrap();
        assert_eq!(config.global.default_tag, "master");
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let domain = ConfigDomain::new(temp_dir.path());

        let mut config = TasksConfig::default();
        config.global.project_name = Some("Test Project".to_string());

        domain.save(&config).await.unwrap();

        let loaded = domain.load().await.unwrap();
        assert_eq!(loaded.global.project_name, Some("Test Project".to_string()));
    }

    #[tokio::test]
    async fn test_set_main_model() {
        let temp_dir = TempDir::new().unwrap();
        let domain = ConfigDomain::new(temp_dir.path());

        let settings = ModelSettings {
            provider: "openai".to_string(),
            model_id: "gpt-4".to_string(),
            max_tokens: 8000,
            temperature: 0.5,
            base_url: None,
        };

        domain.set_main_model(settings.clone()).await.unwrap();

        let models = domain.get_models().await.unwrap();
        assert!(models.main.is_some());
        assert_eq!(models.main.unwrap().provider, "openai");
    }
}

