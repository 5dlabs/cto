//! Container runtime detection and management
//!
//! Detects and manages container runtimes:
//! - Docker Desktop (macOS, Windows, Linux)
//! - Colima (macOS, Linux)
//! - Podman (macOS, Windows, Linux)

use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::error::{AppError, AppResult};

/// Supported container runtimes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerRuntime {
    Docker,
    Colima,
    Podman,
}

impl std::fmt::Display for ContainerRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Docker => write!(f, "Docker"),
            Self::Colima => write!(f, "Colima"),
            Self::Podman => write!(f, "Podman"),
        }
    }
}

/// Runtime status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub runtime: ContainerRuntime,
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

/// Detect the preferred container runtime based on OS
pub fn get_preferred_runtimes() -> Vec<ContainerRuntime> {
    #[cfg(target_os = "macos")]
    {
        vec![ContainerRuntime::Colima, ContainerRuntime::Docker, ContainerRuntime::Podman]
    }
    
    #[cfg(target_os = "linux")]
    {
        vec![ContainerRuntime::Docker, ContainerRuntime::Podman, ContainerRuntime::Colima]
    }
    
    #[cfg(target_os = "windows")]
    {
        vec![ContainerRuntime::Docker, ContainerRuntime::Podman]
    }
}

/// Check if a runtime is installed
pub fn is_runtime_installed(runtime: ContainerRuntime) -> bool {
    let cmd = match runtime {
        ContainerRuntime::Docker => "docker",
        ContainerRuntime::Colima => "colima",
        ContainerRuntime::Podman => "podman",
    };
    
    which::which(cmd).is_ok()
}

/// Get the path to the runtime binary
pub fn get_runtime_path(runtime: ContainerRuntime) -> Option<String> {
    let cmd = match runtime {
        ContainerRuntime::Docker => "docker",
        ContainerRuntime::Colima => "colima",
        ContainerRuntime::Podman => "podman",
    };
    
    which::which(cmd)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// Get the version of a runtime
pub fn get_runtime_version(runtime: ContainerRuntime) -> Option<String> {
    let cmd = match runtime {
        ContainerRuntime::Docker => "docker",
        ContainerRuntime::Colima => "colima",
        ContainerRuntime::Podman => "podman",
    };
    
    let output = Command::new(cmd)
        .arg("--version")
        .output()
        .ok()?;
    
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        // Extract version number from output like "Docker version 24.0.7, build afdd53b"
        Some(version.trim().to_string())
    } else {
        None
    }
}

/// Check if a runtime is currently running
pub fn is_runtime_running(runtime: ContainerRuntime) -> bool {
    match runtime {
        ContainerRuntime::Docker => {
            // Try docker info - returns 0 if daemon is running
            Command::new("docker")
                .args(["info"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        ContainerRuntime::Colima => {
            // Check colima status
            let output = Command::new("colima")
                .args(["status"])
                .output()
                .ok();
            
            output
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        ContainerRuntime::Podman => {
            // Try podman info
            Command::new("podman")
                .args(["info"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    }
}

/// Get full status of a runtime
pub fn get_runtime_status(runtime: ContainerRuntime) -> RuntimeStatus {
    let installed = is_runtime_installed(runtime);
    let running = if installed { is_runtime_running(runtime) } else { false };
    let version = if installed { get_runtime_version(runtime) } else { None };
    let path = if installed { get_runtime_path(runtime) } else { None };
    
    RuntimeStatus {
        runtime,
        installed,
        running,
        version,
        path,
    }
}

/// Detect the best available container runtime
/// Returns the first runtime that is both installed and running
pub fn detect_running_runtime() -> AppResult<ContainerRuntime> {
    for runtime in get_preferred_runtimes() {
        let status = get_runtime_status(runtime);
        if status.installed && status.running {
            tracing::info!("Detected running container runtime: {}", runtime);
            return Ok(runtime);
        }
    }
    
    // Check if any runtime is installed but not running
    for runtime in get_preferred_runtimes() {
        if is_runtime_installed(runtime) {
            return Err(AppError::RuntimeNotRunning(format!(
                "{} is installed but not running. Please start it and try again.",
                runtime
            )));
        }
    }
    
    Err(AppError::RuntimeNotFound(
        "No container runtime found. Please install Docker Desktop, Colima, or Podman.".to_string()
    ))
}

/// Start a container runtime
pub fn start_runtime(runtime: ContainerRuntime) -> AppResult<()> {
    match runtime {
        ContainerRuntime::Colima => {
            tracing::info!("Starting Colima...");
            let status = Command::new("colima")
                .args(["start"])
                .status()
                .map_err(|e| AppError::CommandFailed(e.to_string()))?;
            
            if !status.success() {
                return Err(AppError::CommandFailed("Failed to start Colima".to_string()));
            }
            
            Ok(())
        }
        ContainerRuntime::Docker => {
            // Docker Desktop needs to be started via GUI on most platforms
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("open")
                    .args(["-a", "Docker"])
                    .status();
            }
            
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("systemctl")
                    .args(["--user", "start", "docker-desktop"])
                    .status();
            }
            
            Ok(())
        }
        ContainerRuntime::Podman => {
            // Podman machine needs to be started
            let status = Command::new("podman")
                .args(["machine", "start"])
                .status()
                .map_err(|e| AppError::CommandFailed(e.to_string()))?;
            
            if !status.success() {
                return Err(AppError::CommandFailed("Failed to start Podman machine".to_string()));
            }
            
            Ok(())
        }
    }
}

/// Get all runtimes and their status
pub fn get_all_runtime_status() -> Vec<RuntimeStatus> {
    get_preferred_runtimes()
        .into_iter()
        .map(get_runtime_status)
        .collect()
}
