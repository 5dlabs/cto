use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScalewayConfig {
    pub project_id: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub id: String,
    pub name: Option<String>,
    pub disk: String,  // e.g., "/dev/sda"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TalosConfig {
    pub version: String,
    pub architecture: String,  // "arm64" or "amd64"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterConfig {
    pub name: String,
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub scaleway: ScalewayConfig,
    pub server: ServerConfig,
    pub talos: TalosConfig,
    pub cluster: ClusterConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scaleway: ScalewayConfig {
                project_id: std::env::var("SCALEWAY_PROJECT_ID").unwrap_or_default(),
                access_key: std::env::var("SCALEWAY_ACCESS_KEY").unwrap_or_default(),
                secret_key: std::env::var("SCALEWAY_SECRET_KEY").unwrap_or_default(),
            },
            server: ServerConfig {
                id: String::new(),
                name: None,
                disk: "/dev/sda".to_string(),
            },
            talos: TalosConfig {
                version: "1.8.0".to_string(),
                architecture: "arm64".to_string(),
            },
            cluster: ClusterConfig {
                name: "talos-cluster".to_string(),
                endpoint: "https://talos.example.com:6443".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("SCALEWAY"))
            .build()?;

        cfg.try_deserialize()
    }

    pub fn set_from_args(&mut self, server_id: &str, disk: &str) {
        self.server.id = server_id.to_string();
        self.server.disk = disk.to_string();
    }

    pub fn talos_download_url(&self) -> String {
        format!(
            "https://github.com/siderolabs/talos/releases/download/v{}/talos-{}-raw.img.gz",
            self.talos.version, self.talos.architecture
        )
    }

    pub fn talosctl_download_url(&self) -> String {
        format!(
            "https://github.com/siderolabs/talos/releases/download/v{}/talosctl-{}",
            self.talos.version, self.talos.architecture
        )
    }
}
