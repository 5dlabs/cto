//! Container runtime detection and management
//!
//! Detects and manages container runtimes compatible with Kind (Kubernetes in Docker):
//!
//! ## Docker-compatible runtimes (provide Docker socket)
//! - Docker Desktop (macOS, Windows, Linux) - reference implementation
//! - OrbStack (macOS) - fast, lightweight, uses Apple Virtualization
//! - Colima (macOS, Linux) - Lima-based, provides Docker socket
//! - Rancher Desktop (macOS, Windows, Linux) - can use dockerd or containerd
//!
//! ## Alternative runtimes (require KIND_EXPERIMENTAL_PROVIDER)
//! - Podman (macOS, Windows, Linux) - daemonless, rootless containers
//! - nerdctl + containerd - Docker-compatible CLI for containerd
//! - Finch (macOS, Linux) - AWS's minimal container tool (uses Lima + nerdctl)
//!
//! ## Low-level (not directly Kind-compatible)
//! - Lima (macOS, Linux) - lightweight VMs, used by Colima/Finch under the hood
//!
//! ## Note on Apple Virtualization Framework
//! Not a runtime itself - it's the macOS 13+ hypervisor that powers OrbStack,
//! Colima, Lima, and Finch on Apple Silicon for near-native performance.

use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::error::{AppError, AppResult};

/// Supported container runtimes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerRuntime {
    /// Docker Desktop - reference Docker implementation
    Docker,
    /// OrbStack - fast macOS-native Docker alternative (uses Apple Virtualization)
    OrbStack,
    /// Colima - lightweight Docker on Lima VMs
    Colima,
    /// Podman - daemonless container engine (KIND_EXPERIMENTAL_PROVIDER=podman)
    Podman,
    /// nerdctl - Docker-compatible CLI for containerd (KIND_EXPERIMENTAL_PROVIDER=nerdctl)  
    Nerdctl,
    /// Finch - AWS's minimal container tool (Lima + nerdctl)
    Finch,
    /// Rancher Desktop - includes K8s, can use dockerd or containerd
    RancherDesktop,
    /// Lima - low-level VM manager (used by Colima/Finch, not directly Kind-compatible)
    Lima,
}

impl std::fmt::Display for ContainerRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Docker => write!(f, "Docker Desktop"),
            Self::OrbStack => write!(f, "OrbStack"),
            Self::Colima => write!(f, "Colima"),
            Self::Podman => write!(f, "Podman"),
            Self::Nerdctl => write!(f, "nerdctl (containerd)"),
            Self::Finch => write!(f, "Finch"),
            Self::RancherDesktop => write!(f, "Rancher Desktop"),
            Self::Lima => write!(f, "Lima"),
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
    /// Provides Docker socket at /var/run/docker.sock
    pub docker_compatible: bool,
    /// Includes built-in Kubernetes
    pub kubernetes_included: bool,
    /// Compatible with Kind (Kubernetes in Docker)
    pub kind_compatible: bool,
    /// Environment variable needed for Kind (if not using Docker socket)
    pub kind_provider: Option<String>,
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
/// Ordered by recommendation: Docker-socket providers first, then experimental providers
pub fn get_preferred_runtimes() -> Vec<ContainerRuntime> {
    #[cfg(target_os = "macos")]
    {
        vec![
            // Docker-socket compatible (work with Kind out of the box)
            ContainerRuntime::OrbStack, // Fastest, uses Apple Virtualization
            ContainerRuntime::Docker,   // Reference implementation
            ContainerRuntime::Colima,   // Lightweight Lima-based
            ContainerRuntime::RancherDesktop, // Includes K8s
            // Experimental Kind providers
            ContainerRuntime::Finch,   // AWS's tool (Lima + nerdctl)
            ContainerRuntime::Nerdctl, // containerd CLI
            ContainerRuntime::Podman,  // Daemonless/rootless
            // Low-level (not directly Kind-compatible)
            ContainerRuntime::Lima, // VM manager (used by Colima/Finch)
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec![
            ContainerRuntime::Docker,
            ContainerRuntime::Podman,
            ContainerRuntime::Nerdctl,
            ContainerRuntime::Colima,
            ContainerRuntime::RancherDesktop,
            ContainerRuntime::Finch,
        ]
    }

    #[cfg(target_os = "windows")]
    {
        vec![
            ContainerRuntime::Docker,
            ContainerRuntime::RancherDesktop,
            ContainerRuntime::Podman,
            // nerdctl/Finch not commonly used on Windows
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
        ContainerRuntime::Nerdctl => "nerdctl",
        ContainerRuntime::Finch => "finch",
        ContainerRuntime::RancherDesktop => "rdctl",
        ContainerRuntime::Lima => "limactl",
    }
}

/// Common paths where binaries might be installed (not in default app PATH)
fn get_common_binary_paths() -> Vec<std::path::PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

    vec![
        // System paths
        "/opt/homebrew/bin".into(), // macOS ARM Homebrew
        "/usr/local/bin".into(),    // macOS Intel Homebrew / Linux
        "/usr/bin".into(),          // System binaries
        // Application bundles
        "/Applications/Docker.app/Contents/Resources/bin".into(),
        "/Applications/OrbStack.app/Contents/MacOS".into(),
        "/Applications/Rancher Desktop.app/Contents/Resources/resources/darwin/bin".into(),
        "/Applications/Finch/bin".into(),
        // User-local installs
        format!("{}/.local/bin", home).into(), // nerdctl, containerd tools
        format!("{}/.finch/bin", home).into(), // Finch alternate location
        format!("{}/.rd/bin", home).into(),    // Rancher Desktop user bin
    ]
}

/// Find a binary by checking common paths (since app PATH is limited)
fn find_binary(name: &str) -> Option<std::path::PathBuf> {
    // First try which (works if PATH is set correctly)
    if let Ok(path) = which::which(name) {
        return Some(path);
    }

    // Check common paths
    for dir in get_common_binary_paths() {
        let path = dir.join(name);
        if path.exists() && path.is_file() {
            return Some(path);
        }
    }

    None
}

/// Check if a runtime is installed
pub fn is_runtime_installed(runtime: ContainerRuntime) -> bool {
    let cmd = get_runtime_binary(runtime);
    find_binary(cmd).is_some()
}

/// Get the path to the runtime binary
pub fn get_runtime_path(runtime: ContainerRuntime) -> Option<String> {
    let cmd = get_runtime_binary(runtime);
    find_binary(cmd).map(|p| p.to_string_lossy().to_string())
}

/// Get the version of a runtime
pub fn get_runtime_version(runtime: ContainerRuntime) -> Option<String> {
    let binary = get_runtime_binary(runtime);
    let cmd_path = find_binary(binary)?;

    let args: &[&str] = match runtime {
        ContainerRuntime::Docker => &["--version"],
        ContainerRuntime::OrbStack => &["version"],
        ContainerRuntime::Colima => &["version"],
        ContainerRuntime::Podman => &["--version"],
        ContainerRuntime::Nerdctl => &["--version"],
        ContainerRuntime::Finch => &["version"],
        ContainerRuntime::RancherDesktop => &["version"],
        ContainerRuntime::Lima => &["--version"],
    };

    Command::new(cmd_path)
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
    let binary = get_runtime_binary(runtime);
    let cmd_path = match find_binary(binary) {
        Some(p) => p,
        None => return false,
    };

    match runtime {
        ContainerRuntime::Docker => {
            // Try docker info - returns 0 if daemon is running
            Command::new(&cmd_path)
                .args(["info"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        ContainerRuntime::OrbStack => {
            // OrbStack status check
            Command::new(&cmd_path)
                .args(["status"])
                .output()
                .ok()
                .map(|o| {
                    o.status.success() && String::from_utf8_lossy(&o.stdout).contains("running")
                })
                .unwrap_or(false)
        }
        ContainerRuntime::Colima => {
            // Check colima status
            Command::new(&cmd_path)
                .args(["status"])
                .output()
                .ok()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        ContainerRuntime::Podman => {
            // Try podman info
            Command::new(&cmd_path)
                .args(["info"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        ContainerRuntime::Nerdctl => {
            // nerdctl requires containerd to be running
            // Try nerdctl info to check
            Command::new(&cmd_path)
                .args(["info"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        ContainerRuntime::Finch => {
            // Finch uses Lima under the hood
            // Check if finch vm is running
            Command::new(&cmd_path)
                .args(["vm", "status"])
                .output()
                .ok()
                .map(|o| {
                    o.status.success() && String::from_utf8_lossy(&o.stdout).contains("Running")
                })
                .unwrap_or(false)
        }
        ContainerRuntime::RancherDesktop => {
            // Check if Rancher Desktop is running
            Command::new(&cmd_path)
                .args(["list-settings"])
                .output()
                .ok()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        ContainerRuntime::Lima => {
            // Check if default lima instance is running
            Command::new(&cmd_path)
                .args(["list", "--format", "{{.Status}}"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains("Running"))
                .unwrap_or(false)
        }
    }
}

/// Check if runtime provides Docker socket compatibility
/// This means Kind can use it without KIND_EXPERIMENTAL_PROVIDER
fn has_docker_compatibility(runtime: ContainerRuntime) -> bool {
    match runtime {
        ContainerRuntime::Docker => true,
        ContainerRuntime::OrbStack => true, // Full Docker compatibility
        ContainerRuntime::Colima => true,   // Provides Docker socket
        ContainerRuntime::RancherDesktop => true, // Can use dockerd
        ContainerRuntime::Podman => {
            // Podman can provide Docker compatibility via socket
            std::path::Path::new("/var/run/docker.sock").exists()
                || std::env::var("DOCKER_HOST").is_ok()
        }
        ContainerRuntime::Nerdctl => false, // Uses containerd, needs KIND_EXPERIMENTAL_PROVIDER
        ContainerRuntime::Finch => false,   // Uses nerdctl, needs KIND_EXPERIMENTAL_PROVIDER
        ContainerRuntime::Lima => false,    // Low-level VM, need manual config
    }
}

/// Get the KIND_EXPERIMENTAL_PROVIDER value for non-Docker runtimes
fn get_kind_provider(runtime: ContainerRuntime) -> Option<&'static str> {
    match runtime {
        ContainerRuntime::Podman => Some("podman"),
        ContainerRuntime::Nerdctl => Some("nerdctl"),
        ContainerRuntime::Finch => Some("nerdctl"), // Finch uses nerdctl under the hood
        _ => None,
    }
}

/// Check if runtime is compatible with Kind
fn is_kind_compatible(runtime: ContainerRuntime) -> bool {
    match runtime {
        // Docker-socket providers work out of the box
        ContainerRuntime::Docker => true,
        ContainerRuntime::OrbStack => true,
        ContainerRuntime::Colima => true,
        ContainerRuntime::RancherDesktop => true,
        // These need KIND_EXPERIMENTAL_PROVIDER but work
        ContainerRuntime::Podman => true,
        ContainerRuntime::Nerdctl => true,
        ContainerRuntime::Finch => true,
        // Lima alone isn't directly Kind-compatible
        ContainerRuntime::Lima => false,
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
        ContainerRuntime::OrbStack => true,       // Includes K8s
        ContainerRuntime::Colima => false,        // Need to enable with --kubernetes
        ContainerRuntime::RancherDesktop => true, // Primary feature
        ContainerRuntime::Podman => false,
        ContainerRuntime::Nerdctl => false, // Just container runtime
        ContainerRuntime::Finch => false,   // Just container runtime
        ContainerRuntime::Lima => false,
    }
}

/// Get full status of a runtime
pub fn get_runtime_status(runtime: ContainerRuntime) -> RuntimeStatus {
    let installed = is_runtime_installed(runtime);
    let running = if installed {
        is_runtime_running(runtime)
    } else {
        false
    };
    let version = if installed {
        get_runtime_version(runtime)
    } else {
        None
    };
    let path = if installed {
        get_runtime_path(runtime)
    } else {
        None
    };
    let docker_compatible = if running {
        has_docker_compatibility(runtime)
    } else {
        false
    };
    let kubernetes_included = has_kubernetes_included(runtime);
    let kind_compatible = is_kind_compatible(runtime);
    let kind_provider = get_kind_provider(runtime).map(|s| s.to_string());

    RuntimeStatus {
        runtime,
        installed,
        running,
        version,
        path,
        docker_compatible,
        kubernetes_included,
        kind_compatible,
        kind_provider,
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
        "No container runtime found. Please install Docker Desktop, OrbStack, Colima, or Podman."
            .to_string(),
    ))
}

/// Scan the full runtime environment
pub fn scan_runtime_environment() -> RuntimeEnvironment {
    let runtimes: Vec<RuntimeStatus> = get_preferred_runtimes()
        .into_iter()
        .map(get_runtime_status)
        .collect();

    let docker_available = runtimes.iter().any(|r| r.running && r.docker_compatible);

    let kubernetes_available = runtimes.iter().any(|r| r.running && r.kubernetes_included);

    // Find recommended runtime
    let recommended = runtimes
        .iter()
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
                let _ = Command::new("open").args(["-a", "OrbStack"]).status();
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
                return Err(AppError::CommandFailed(
                    "Failed to start Colima".to_string(),
                ));
            }

            Ok(())
        }
        ContainerRuntime::Docker => {
            tracing::info!("Starting Docker Desktop...");
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("open").args(["-a", "Docker"]).status();
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
                return Err(AppError::CommandFailed(
                    "Failed to start Podman machine".to_string(),
                ));
            }

            Ok(())
        }
        ContainerRuntime::Nerdctl => {
            tracing::info!("nerdctl requires containerd to be running...");
            // nerdctl itself doesn't need starting - it's just a CLI
            // containerd needs to be running (usually via systemd on Linux)
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("sudo")
                    .args(["systemctl", "start", "containerd"])
                    .status();
            }
            // On macOS, containerd typically runs inside Lima/Finch
            Ok(())
        }
        ContainerRuntime::Finch => {
            tracing::info!("Starting Finch VM...");
            if let Some(cmd_path) = find_binary("finch") {
                let status = Command::new(cmd_path)
                    .args(["vm", "start"])
                    .status()
                    .map_err(|e| AppError::CommandFailed(e.to_string()))?;

                if !status.success() {
                    return Err(AppError::CommandFailed(
                        "Failed to start Finch VM".to_string(),
                    ));
                }
            }
            Ok(())
        }
        ContainerRuntime::Lima => {
            tracing::info!("Starting Lima...");
            if let Some(cmd_path) = find_binary("limactl") {
                let status = Command::new(cmd_path)
                    .args(["start"])
                    .status()
                    .map_err(|e| AppError::CommandFailed(e.to_string()))?;

                if !status.success() {
                    return Err(AppError::CommandFailed("Failed to start Lima".to_string()));
                }
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
                let _ = Command::new("rancher-desktop").status();
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
