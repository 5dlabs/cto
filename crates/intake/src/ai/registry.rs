//! Provider Registry - Manages AI provider instances.
//!
//! Simplified to use ONLY the Anthropic provider - no fallbacks.
//! Fallbacks mask issues and make debugging harder.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::errors::{TasksError, TasksResult};

use super::provider::AIProvider;
use super::sdk_provider::AgentSdkProvider;

/// Singleton registry for AI providers.
pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn AIProvider>>>,
}

impl ProviderRegistry {
    /// Create a new provider registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    /// Create a registry with the Claude Agent SDK provider registered.
    ///
    /// No fallbacks - fail fast to surface issues immediately.
    #[must_use]
    pub fn with_defaults() -> Self {
        let registry = Self::new();

        // Register Claude Agent SDK provider ONLY - no fallbacks
        match AgentSdkProvider::from_env() {
            Ok(provider) => {
                tracing::info!(
                    "Using Claude Agent SDK provider (TypeScript binary with MCP support)"
                );
                registry.register(Arc::new(provider));
            }
            Err(e) => {
                // Don't silently fallback - log the error clearly
                tracing::error!(
                    error = %e,
                    "Claude Agent SDK provider initialization failed. Build intake-agent: cd apps/intake-agent && bun run build"
                );
            }
        }

        registry
    }

    /// Register a provider.
    pub fn register(&self, provider: Arc<dyn AIProvider>) {
        let mut providers = self.providers.write().expect("lock poisoned");
        providers.insert(provider.name().to_string(), provider);
    }

    /// Get a provider by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Arc<dyn AIProvider>> {
        let providers = self.providers.read().expect("lock poisoned");
        providers.get(name).cloned()
    }

    /// Get the first configured provider.
    #[must_use]
    pub fn get_configured(&self) -> Option<Arc<dyn AIProvider>> {
        let providers = self.providers.read().expect("lock poisoned");
        providers.values().find(|p| p.is_configured()).cloned()
    }

    /// Get a provider that supports a specific model.
    #[must_use]
    pub fn get_for_model(&self, model: &str) -> Option<Arc<dyn AIProvider>> {
        let providers = self.providers.read().expect("lock poisoned");
        providers
            .values()
            .find(|p| p.supports_model(model))
            .cloned()
    }

    /// Check if a provider is registered.
    #[must_use]
    pub fn has_provider(&self, name: &str) -> bool {
        let providers = self.providers.read().expect("lock poisoned");
        providers.contains_key(name)
    }

    /// Get all registered provider names.
    #[must_use]
    pub fn provider_names(&self) -> Vec<String> {
        let providers = self.providers.read().expect("lock poisoned");
        providers.keys().cloned().collect()
    }

    /// Get all configured providers.
    #[must_use]
    pub fn configured_providers(&self) -> Vec<Arc<dyn AIProvider>> {
        let providers = self.providers.read().expect("lock poisoned");
        providers
            .values()
            .filter(|p| p.is_configured())
            .cloned()
            .collect()
    }

    /// Remove a provider.
    pub fn unregister(&self, name: &str) -> bool {
        let mut providers = self.providers.write().expect("lock poisoned");
        providers.remove(name).is_some()
    }

    /// Clear all providers.
    pub fn clear(&self) {
        let mut providers = self.providers.write().expect("lock poisoned");
        providers.clear();
    }

    /// Get a provider, returning an error if not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider is not registered.
    pub fn require(&self, name: &str) -> TasksResult<Arc<dyn AIProvider>> {
        self.get(name)
            .ok_or_else(|| TasksError::Ai(format!("Provider '{name}' not found")))
    }

    /// Get any configured provider, returning an error if none available.
    ///
    /// # Errors
    ///
    /// Returns an error if no provider is configured.
    pub fn require_any(&self) -> TasksResult<Arc<dyn AIProvider>> {
        self.get_configured().ok_or_else(|| {
            TasksError::Ai(
                "No AI provider is configured. Ensure:\n\
                 1. ANTHROPIC_API_KEY is set\n\
                 2. intake-agent binary is built: cd apps/intake-agent && bun run build"
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
        assert!(!registry.has_provider("test"));
    }
}
