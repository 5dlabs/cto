//! Provider Registry - Manages AI provider instances.
//!
//! Provides a singleton registry for dynamically registering and
//! retrieving AI providers.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::errors::{TasksError, TasksResult};

use super::anthropic::AnthropicProvider;
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
    pub fn with_defaults() -> Self {
        let registry = Self::new();

        // Register Anthropic provider
        if let Ok(provider) = AnthropicProvider::from_env() {
            registry.register(Arc::new(provider));
        }

        // Register OpenAI provider
        if let Ok(provider) = OpenAIProvider::from_env() {
            registry.register(Arc::new(provider));
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
                "No AI provider is configured. Please set ANTHROPIC_API_KEY or OPENAI_API_KEY"
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
