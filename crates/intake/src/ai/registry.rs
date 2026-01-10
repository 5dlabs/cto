//! Provider Registry - Manages AI provider instances.
//!
//! Provides a singleton registry for dynamically registering and
//! retrieving AI providers.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::errors::{TasksError, TasksResult};

use super::anthropic::AnthropicProvider;
use super::cli_provider::CLITextGenerator;
use super::openai::OpenAIProvider;
use super::provider::AIProvider;

/// Singleton registry for AI providers.
pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn AIProvider>>>,
}

impl ProviderRegistry {
    /// Create a new provider registry.
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    /// Create a registry with default providers registered.
    ///
    /// Provider selection is controlled by `TASKS_USE_CLI` env var:
    /// - `TASKS_USE_CLI=true` or `TASKS_USE_CLI=1`: Use CLI provider (claude, codex, etc.)
    /// - Otherwise: Use API providers (Anthropic, OpenAI)
    ///
    /// When CLI mode is enabled, the specific CLI is selected via `TASKS_CLI` env var
    /// (defaults to "claude"). Supported values: claude, codex, cursor, factory, opencode, gemini, dexter
    pub fn with_defaults() -> Self {
        let registry = Self::new();

        // Check if CLI mode is enabled
        let use_cli = std::env::var("TASKS_USE_CLI")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let mut cli_provider_registered = false;

        if use_cli {
            // Register CLI provider (uses TASKS_CLI env var for CLI type selection)
            match CLITextGenerator::from_env() {
                Ok(provider) => {
                    tracing::info!(
                        cli_type = %provider.cli_type(),
                        "Using CLI provider for AI operations"
                    );
                    registry.register(Arc::new(provider));
                    cli_provider_registered = true;
                }
                Err(e) => {
                    tracing::warn!("Failed to create CLI provider: {e}, falling back to API providers");
                }
            }
        }

        // Only register API providers if CLI mode is not enabled OR CLI provider failed
        if !cli_provider_registered {
            if let Ok(provider) = AnthropicProvider::from_env() {
                registry.register(Arc::new(provider));
            }

            if let Ok(provider) = OpenAIProvider::from_env() {
                registry.register(Arc::new(provider));
            }
        }

        registry
    }

    /// Register a provider.
    pub fn register(&self, provider: Arc<dyn AIProvider>) {
        let mut providers = self.providers.write().unwrap();
        providers.insert(provider.name().to_string(), provider);
    }

    /// Get a provider by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn AIProvider>> {
        let providers = self.providers.read().unwrap();
        providers.get(name).cloned()
    }

    /// Get the first configured provider.
    pub fn get_configured(&self) -> Option<Arc<dyn AIProvider>> {
        let providers = self.providers.read().unwrap();
        providers.values().find(|p| p.is_configured()).cloned()
    }

    /// Get a provider that supports a specific model.
    pub fn get_for_model(&self, model: &str) -> Option<Arc<dyn AIProvider>> {
        let providers = self.providers.read().unwrap();
        providers
            .values()
            .find(|p| p.supports_model(model))
            .cloned()
    }

    /// Check if a provider is registered.
    pub fn has_provider(&self, name: &str) -> bool {
        let providers = self.providers.read().unwrap();
        providers.contains_key(name)
    }

    /// Get all registered provider names.
    pub fn provider_names(&self) -> Vec<String> {
        let providers = self.providers.read().unwrap();
        providers.keys().cloned().collect()
    }

    /// Get all configured providers.
    pub fn configured_providers(&self) -> Vec<Arc<dyn AIProvider>> {
        let providers = self.providers.read().unwrap();
        providers
            .values()
            .filter(|p| p.is_configured())
            .cloned()
            .collect()
    }

    /// Remove a provider.
    pub fn unregister(&self, name: &str) -> bool {
        let mut providers = self.providers.write().unwrap();
        providers.remove(name).is_some()
    }

    /// Clear all providers.
    pub fn clear(&self) {
        let mut providers = self.providers.write().unwrap();
        providers.clear();
    }

    /// Get a provider, returning an error if not found.
    pub fn require(&self, name: &str) -> TasksResult<Arc<dyn AIProvider>> {
        self.get(name)
            .ok_or_else(|| TasksError::Ai(format!("Provider '{name}' not found")))
    }

    /// Get any configured provider, returning an error if none available.
    pub fn require_any(&self) -> TasksResult<Arc<dyn AIProvider>> {
        self.get_configured().ok_or_else(|| {
            TasksError::Ai(
                "No AI provider is configured. Set TASKS_USE_CLI=true to use CLI mode, \
                 or set ANTHROPIC_API_KEY/OPENAI_API_KEY for API mode"
                    .to_string(),
            )
        })
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Global provider registry instance.
static GLOBAL_REGISTRY: std::sync::OnceLock<ProviderRegistry> = std::sync::OnceLock::new();

/// Get the global provider registry.
pub fn global_registry() -> &'static ProviderRegistry {
    GLOBAL_REGISTRY.get_or_init(ProviderRegistry::with_defaults)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ProviderRegistry::new();
        assert!(registry.provider_names().is_empty());
    }

    #[test]
    fn test_provider_registration() {
        let registry = ProviderRegistry::new();

        // Create a mock provider (in real tests, we'd use a mock)
        // For now, just test the registry structure works
        assert!(!registry.has_provider("test"));
    }
}
