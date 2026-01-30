//! Container runtime detection and management
//!
//! Detects and manages container runtimes:
//! - Docker Desktop (macOS, Windows, Linux)
//! - OrbStack (macOS) - lightweight Docker/Linux alternative
//! - Colima (macOS, Linux) - Lima-based Docker runtime
//! - Podman (macOS, Windows, Linux)
//! - Lima (macOS, Linux) - lightweight VMs
//! - Apple Virtualization (macOS 13+) - native container support

use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::error::{AppError, AppResult};

/// Supported container runtimes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerRuntime {
    Docker,
    OrbStack,
    Colima,
    Podman,
    Lima,
    RancherDesktop,
}

impl std::fmt::Display for ContainerRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Docker => write!(f, "Docker Desktop"),
            Self::OrbStack => write!(f, "OrbStack"),
            Self::Colima => write!(f, "Colima"),
            Self::Podman => write!(f, "Podman"),
            Self::Lima => write!(f, "Lima"),
            Self::RancherDesktop => write!(f, "Rancher Desktop"),
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
    pub docker_compatible: bool,
    pub kubernetes_included: bool,
}

/// Full runtime environment scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEnvironment {
    pub runtimes: Vec<RuntimeStatus>,
    pub docker_available: bool,
    pub kubernetes_available: bool,
    pub recommended: Option<ContainerRuntime>,
    pub macos_version: Option<String>,
    pub can_use_apple_virtualization: bool,
}

/// Get macOS version
#[cfg(target_os = "macos")]
fn get_macos_version() -> Option<String> {
    Command::new("sw_vers")
        .args(["-productVersion"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

#[cfg(not(target_os = "macos"))]
fn get_macos_version() -> Option<String> {
    None
}

/// Check if macOS version supports Apple Virtualization framework well
/// (macOS 13 Ventura+ has good container support)
#[cfg(target_os = "macos")]
fn supports_apple_virtualization() -> bool {
    get_macos_version()
        .and_then(|v| {
            let parts: Vec<&str> = v.split('.').collect();
            parts.first()?.parse::<u32>().ok()
        })
        .map(|major| major >= 13)
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
fn supports_apple_virtualization() -> bool {
    false
}

/// Detect the preferred container runtime based on OS
pub fn get_preferred_runtimes() -> Vec<ContainerRuntime> {
    #[cfg(target_os = "macos")]
    {
        vec![
            ContainerRuntime::OrbStack,      // Fastest, most macOS-native
            ContainerRuntime::Docker,         // Most common
            ContainerRuntime::Colima,         // Good lightweight option
            ContainerRuntime::RancherDesktop, // Includes K8s
            ContainerRuntime::Podman,         // Rootless option
            ContainerRuntime::Lima,           // Low-level VM option
        ]
    }
    
    #[cfg(target_os = "linux")]
    {
        vec![
            ContainerRuntime::Docker,
            ContainerRuntime::Podman,
            ContainerRuntime::Colima,
            ContainerRuntime::RancherDesktop,
        ]
    }
    
    #[cfg(target_os = "windows")]
    {
        vec![
            ContainerRuntime::Docker,
            ContainerRuntime::RancherDesktop,
            ContainerRuntime::Podman,
        ]
    }
}

/// Get the binary name for a runtime
fn get_runtime_binary(runtime: ContainerRuntime) -> &'static str {
    match runtime {
        ContainerRuntime::Docker => "docker",
        ContainerRuntime::OrbStack => "orb",
        ContainerRuntime::Colima => "colima",
        ContainerRuntime::Podman => "podman",
        ContainerRuntime::Lima => "limactl",
        ContainerRuntime::RancherDesktop => "rdctl",
    }
}

/// Check if a runtime is installed
pub fn is_runtime_installed(runtime: ContainerRuntime) -> bool {
    let cmd = get_runtime_binary(runtime);
    which::which(cmd).is_ok()
}

/// Get the path to the runtime binary
pub fn get_runtime_path(runtime: ContainerRuntime) -> Option<String> {
    let cmd = get_runtime_binary(runtime);
    which::which(cmd)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// Get the version of a runtime
pub fn get_runtime_version(runtime: ContainerRuntime) -> Option<String> {
    let (cmd, args): (&str, &[&str]) = match runtime {
        ContainerRuntime::Docker => ("docker", &["--version"]),
        ContainerRuntime::OrbStack => ("orb", &["version"]),
        ContainerRuntime::Colima => ("colima", &["version"]),
        ContainerRuntime::Podman => ("podman", &["--version"]),
        ContainerRuntime::Lima => ("limactl", &["--version"]),
        ContainerRuntime::RancherDesktop => ("rdctl", &["version"]),
    };
    
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            let version = String::from_utf8_lossy(&o.stdout);
            // Take first line, trim whitespace
            version.lines().next().unwrap_or("").trim().to_string()
        })
        .filter(|s| !s.is_empty())
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
        ContainerRuntime::OrbStack => {
            // OrbStack status check
            Command::new("orb")
                .args(["status"])
                .output()
                .ok()
                .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains("running"))
                .unwrap_or(false)
        }
        ContainerRuntime::Colima => {
            // Check colima status
            Command::new("colima")
                .args(["status"])
                .output()
                .ok()
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
        ContainerRuntime::Lima => {
            // Check if default lima instance is running
            Command::new("limactl")
                .args(["list", "--format", "{{.Status}}"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains("Running"))
                .unwrap_or(false)
        }
        ContainerRuntime::RancherDesktop => {
            // Check if Rancher Desktop is running
            Command::new("rdctl")
                .args(["list-settings"])
                .output()
                .ok()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }
}

/// Check if runtime provides Docker compatibility
fn has_docker_compatibility(runtime: ContainerRuntime) -> bool {
    match runtime {
        ContainerRuntime::Docker => true,
        ContainerRuntime::OrbStack => true,  // Full Docker compatibility
        ContainerRuntime::Colima => true,    // Provides Docker socket
        ContainerRuntime::RancherDesktop => true, // Can use dockerd
        ContainerRuntime::Podman => {
            // Podman can provide Docker compatibility via socket
            // Check if docker socket exists
            std::path::Path::new("/var/run/docker.sock").exists()
                || std::env::var("DOCKER_HOST").is_ok()
        }
        ContainerRuntime::Lima => false, // Need to configure manually
    }
}

/// Check if runtime includes Kubernetes
fn has_kubernetes_included(runtime: ContainerRuntime) -> bool {
    match runtime {
        ContainerRuntime::Docker => {
            // Docker Desktop has optional K8s - check if enabled
            // This would require checking Docker settings
            false // Conservative - user needs to enable it
        }
        ContainerRuntime::OrbStack => true,  // Includes K8s
        ContainerRuntime::Colima => false,   // Need to enable with --kubernetes
        ContainerRuntime::RancherDesktop => true, // Primary feature
        ContainerRuntime::Podman => false,
        ContainerRuntime::Lima => false,
    }
}

/// Get full status of a runtime
pub fn get_runtime_status(runtime: ContainerRuntime) -> RuntimeStatus {
    let installed = is_runtime_installed(runtime);
    let running = if installed { is_runtime_running(runtime) } else { false };
    let version = if installed { get_runtime_version(runtime) } else { None };
    let path = if installed { get_runtime_path(runtime) } else { None };
    let docker_compatible = if running { has_docker_compatibility(runtime) } else { false };
    let kubernetes_included = has_kubernetes_included(runtime);
    
    RuntimeStatus {
        runtime,
        installed,
        running,
        version,
        path,
        docker_compatible,
        kubernetes_included,
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
        "No container runtime found. Please install Docker Desktop, OrbStack, Colima, or Podman.".to_string()
    ))
}

/// Scan the full runtime environment
pub fn scan_runtime_environment() -> RuntimeEnvironment {
    let runtimes: Vec<RuntimeStatus> = get_preferred_runtimes()
        .into_iter()
        .map(get_runtime_status)
        .collect();
    
    let docker_available = runtimes.iter()
        .any(|r| r.running && r.docker_compatible);
    
    let kubernetes_available = runtimes.iter()
        .any(|r| r.running && r.kubernetes_included);
    
    // Find recommended runtime
    let recommended = runtimes.iter()
        .filter(|r| r.running && r.docker_compatible)
        .map(|r| r.runtime)
        .next();
    
    let macos_version = get_macos_version();
    let can_use_apple_virtualization = supports_apple_virtualization();
    
    RuntimeEnvironment {
        runtimes,
        docker_available,
        kubernetes_available,
        recommended,
        macos_version,
        can_use_apple_virtualization,
    }
}

/// Start a container runtime
pub fn start_runtime(runtime: ContainerRuntime) -> AppResult<()> {
    match runtime {
        ContainerRuntime::OrbStack => {
            tracing::info!("Starting OrbStack...");
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("open")
                    .args(["-a", "OrbStack"])
                    .status();
            }
            Ok(())
        }
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
            tracing::info!("Starting Docker Desktop...");
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
            
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("cmd")
                    .args(["/c", "start", "", "Docker Desktop"])
                    .status();
            }
            
            Ok(())
        }
        ContainerRuntime::Podman => {
            tracing::info!("Starting Podman machine...");
            let status = Command::new("podman")
                .args(["machine", "start"])
                .status()
                .map_err(|e| AppError::CommandFailed(e.to_string()))?;
            
            if !status.success() {
                return Err(AppError::CommandFailed("Failed to start Podman machine".to_string()));
            }
            
            Ok(())
        }
        ContainerRuntime::Lima => {
            tracing::info!("Starting Lima...");
            let status = Command::new("limactl")
                .args(["start"])
                .status()
                .map_err(|e| AppError::CommandFailed(e.to_string()))?;
            
            if !status.success() {
                return Err(AppError::CommandFailed("Failed to start Lima".to_string()));
            }
            
            Ok(())
        }
        ContainerRuntime::RancherDesktop => {
            tracing::info!("Starting Rancher Desktop...");
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("open")
                    .args(["-a", "Rancher Desktop"])
                    .status();
            }
            
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("rancher-desktop")
                    .status();
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
