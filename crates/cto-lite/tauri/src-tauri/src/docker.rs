//! Docker runtime detection and management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Docker runtime information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerInfo {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
    pub runtime: DockerRuntime,
}

/// Detected Docker runtime
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DockerRuntime {
    Docker,
    Podman,
    OrbStack,
    Colima,
    RancherDesktop,
    Unknown,
    NotFound,
}

impl Default for DockerRuntime {
    fn default() -> Self {
        Self::NotFound
    }
}

/// Check Docker installation and status
pub fn check_docker() -> Result<DockerInfo> {
    // First, check if docker CLI exists
    let docker_path = which::which("docker").ok();
    
    if docker_path.is_none() {
        return Ok(DockerInfo {
            installed: false,
            running: false,
            version: None,
            runtime: DockerRuntime::NotFound,
        });
    }

    // Get Docker version
    let version_output = Command::new("docker")
        .args(["version", "--format", "{{.Server.Version}}"])
        .output()
        .context("Failed to run docker version")?;

    let version = if version_output.status.success() {
        Some(String::from_utf8_lossy(&version_output.stdout).trim().to_string())
    } else {
        None
    };

    // Check if Docker daemon is running
    let running = Command::new("docker")
        .args(["info"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Detect runtime type
    let runtime = detect_runtime();

    Ok(DockerInfo {
        installed: true,
        running,
        version,
        runtime,
    })
}

/// Detect which Docker runtime is being used
fn detect_runtime() -> DockerRuntime {
    // Check for OrbStack
    if std::env::var("ORBSTACK").is_ok() || which::which("orbctl").is_ok() {
        return DockerRuntime::OrbStack;
    }

    // Check for Colima
    if which::which("colima").is_ok() {
        if let Ok(output) = Command::new("colima").args(["status"]).output() {
            if output.status.success() {
                return DockerRuntime::Colima;
            }
        }
    }

    // Check for Rancher Desktop
    if which::which("rdctl").is_ok() {
        return DockerRuntime::RancherDesktop;
    }

    // Check for Podman
    if let Ok(output) = Command::new("docker").args(["info", "--format", "{{.Host.RemoteSocket.Path}}"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.contains("podman") {
            return DockerRuntime::Podman;
        }
    }

    // Check Docker context for hints
    if let Ok(output) = Command::new("docker").args(["context", "show"]).output() {
        let context = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        if context.contains("orbstack") {
            return DockerRuntime::OrbStack;
        }
        if context.contains("colima") {
            return DockerRuntime::Colima;
        }
        if context.contains("rancher") {
            return DockerRuntime::RancherDesktop;
        }
    }

    // Default to Docker Desktop or unknown
    DockerRuntime::Docker
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_docker() {
        // This test will pass/fail based on local Docker installation
        let info = check_docker().unwrap();
        println!("Docker info: {:?}", info);
        
        // At minimum, we should get a valid response
        assert!(info.runtime != DockerRuntime::Unknown || !info.installed);
    }
}
