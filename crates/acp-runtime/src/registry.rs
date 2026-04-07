use cto_config::{AcpDefaults, AcpRuntimeConfig, AcpServiceConfig};

/// Resolved ACP runtime selection for a service.
#[derive(Debug, Clone)]
pub struct RuntimeSelection {
    /// Service requesting the runtime.
    pub service: String,
    /// Runtime identifier.
    pub runtime_id: String,
    /// Concrete runtime definition.
    pub runtime: AcpRuntimeConfig,
}

/// Runtime registry backed by CTO config ACP defaults.
#[derive(Debug, Clone, Default)]
pub struct AcpRuntimeRegistry {
    defaults: AcpDefaults,
}

impl AcpRuntimeRegistry {
    /// Create a registry from shared ACP defaults.
    #[must_use]
    pub fn new(defaults: AcpDefaults) -> Self {
        Self { defaults }
    }

    /// Borrow the raw ACP defaults.
    #[must_use]
    pub fn defaults(&self) -> &AcpDefaults {
        &self.defaults
    }

    /// Resolve a runtime by ID.
    #[must_use]
    pub fn resolve(&self, runtime_id: &str) -> Option<&AcpRuntimeConfig> {
        self.defaults
            .runtimes
            .get(runtime_id)
            .filter(|runtime| runtime.enabled)
    }

    /// Resolve the service configuration by canonical service name.
    #[must_use]
    pub fn service(&self, service: &str) -> Option<&AcpServiceConfig> {
        match service {
            "healer" => Some(&self.defaults.services.healer),
            "pm" => Some(&self.defaults.services.pm),
            "controller" => Some(&self.defaults.services.controller),
            "mcp" => Some(&self.defaults.services.mcp),
            "mcp-lite" | "mcp_lite" => Some(&self.defaults.services.mcp_lite),
            _ => None,
        }
    }

    /// Check whether ACP is enabled for the requested service.
    #[must_use]
    pub fn service_enabled(&self, service: &str) -> bool {
        self.defaults.enabled && self.service(service).is_some_and(|service| service.enabled)
    }

    /// Select the runtime for a service, optionally overriding the runtime ID.
    #[must_use]
    pub fn select_runtime_for_service(
        &self,
        service: &str,
        requested_runtime: Option<&str>,
    ) -> Option<RuntimeSelection> {
        let service_config = self.service(service)?;
        if !self.defaults.enabled || !service_config.enabled {
            return None;
        }

        let runtime_id = requested_runtime
            .map(ToString::to_string)
            .or_else(|| service_config.default_runtime.clone())
            .or_else(|| self.defaults.default_runtime.clone())?;

        if !service_config.runtime_ids.is_empty()
            && !service_config
                .runtime_ids
                .iter()
                .any(|allowed| allowed == &runtime_id)
        {
            return None;
        }

        let runtime = self.resolve(&runtime_id)?.clone();
        Some(RuntimeSelection {
            service: service.to_string(),
            runtime_id,
            runtime,
        })
    }
}
