//! Talos configuration utilities.

use serde::{Deserialize, Serialize};

/// Default Talos version.
pub const DEFAULT_TALOS_VERSION: &str = "v1.9.0";

/// Default schematic ID (vanilla, no extensions).
pub const DEFAULT_SCHEMATIC_ID: &str =
    "376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba";

/// Talos Image Factory base URL.
const IMAGE_FACTORY_URL: &str = "https://pxe.factory.talos.dev";

/// Talos version configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalosVersion {
    /// Talos version (e.g., "v1.9.0").
    pub version: String,
    /// Schematic ID from Talos Image Factory.
    pub schematic_id: String,
}

impl Default for TalosVersion {
    fn default() -> Self {
        Self {
            version: DEFAULT_TALOS_VERSION.to_string(),
            schematic_id: DEFAULT_SCHEMATIC_ID.to_string(),
        }
    }
}

impl TalosVersion {
    /// Create a new Talos version configuration.
    #[must_use]
    pub fn new(version: impl Into<String>, schematic_id: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            schematic_id: schematic_id.into(),
        }
    }

    /// Generate the iPXE URL for this Talos version.
    ///
    /// # Arguments
    /// * `arch` - Architecture (e.g., "amd64", "arm64")
    #[must_use]
    pub fn ipxe_url(&self, arch: &str) -> String {
        format!(
            "{}/pxe/{}/{}/metal-{}",
            IMAGE_FACTORY_URL, self.schematic_id, self.version, arch
        )
    }

    /// Generate the iPXE URL for AMD64 architecture.
    #[must_use]
    pub fn ipxe_url_amd64(&self) -> String {
        self.ipxe_url("amd64")
    }
}

/// Talos cluster configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalosConfig {
    /// Cluster name.
    pub cluster_name: String,
    /// Talos version to use.
    pub version: TalosVersion,
    /// Kubernetes endpoint (e.g., `https://k8s.example.com:6443`).
    pub endpoint: Option<String>,
}

impl TalosConfig {
    /// Create a new Talos configuration.
    #[must_use]
    pub fn new(cluster_name: impl Into<String>) -> Self {
        Self {
            cluster_name: cluster_name.into(),
            version: TalosVersion::default(),
            endpoint: None,
        }
    }

    /// Set the Talos version.
    #[must_use]
    pub fn with_version(mut self, version: TalosVersion) -> Self {
        self.version = version;
        self
    }

    /// Set the Kubernetes endpoint.
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Get the iPXE URL for this configuration.
    #[must_use]
    pub fn ipxe_url(&self) -> String {
        self.version.ipxe_url_amd64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ipxe_url() {
        let version = TalosVersion::default();
        let url = version.ipxe_url_amd64();

        assert!(url.starts_with("https://pxe.factory.talos.dev/pxe/"));
        assert!(url.contains(DEFAULT_TALOS_VERSION));
        assert!(url.ends_with("/metal-amd64"));
    }

    #[test]
    fn test_custom_version() {
        let version = TalosVersion::new("v1.8.0", "custom-schematic-id");
        let url = version.ipxe_url_amd64();

        assert!(url.contains("v1.8.0"));
        assert!(url.contains("custom-schematic-id"));
    }

    #[test]
    fn test_talos_config() {
        let config = TalosConfig::new("my-cluster").with_endpoint("https://k8s.example.com:6443");

        assert_eq!(config.cluster_name, "my-cluster");
        assert_eq!(
            config.endpoint,
            Some("https://k8s.example.com:6443".to_string())
        );
    }
}
