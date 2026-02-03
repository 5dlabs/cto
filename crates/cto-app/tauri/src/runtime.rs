// Container runtime detection module
// Provides functions to detect and manage container runtimes (Docker, Colima, Podman)
// and Kind clusters for local Kubernetes development

use serde::Serialize;
use std::process::Command;

/// Represents a detected container runtime
#[derive(Debug, Serialize, Clone)]
pub struct ContainerRuntime {
    pub name: String, // "docker", "colima", "podman"
    pub version: String,
    pub available: bool,
    pub path: Option<String>,
}

/// Detailed runtime information
#[derive(Debug, Serialize, Clone)]
pub struct RuntimeInfo {
    pub runtime: String,
    pub version: String,
    pub api_version: String,
    pub containers: usize,
    pub images: usize,
}

/// Status of a Kind cluster
#[derive(Debug, Serialize, Clone)]
pub struct ClusterStatus {
    pub name: String,
    pub status: String, // "running", "stopped", "unknown"
    pub kubernetes_version: Option<String>,
    pub nodes: usize,
}

/// Detects which container runtime is available by checking in order of preference:
/// Docker Desktop -> Docker CLI -> Colima -> Podman
/// Returns the first available runtime with version info
pub fn get_container_runtime() -> Result<ContainerRuntime, String> {
    // Check for Docker Desktop (macOS/Windows)
    if let Some(runtime) = check_docker_desktop() {
        return Ok(runtime);
    }

    // Check for Docker CLI
    if let Some(runtime) = check_docker_cli() {
        return Ok(runtime);
    }

    // Check for Colima
    if let Some(runtime) = check_colima() {
        return Ok(runtime);
    }

    // Check for Podman
    if let Some(runtime) = check_podman() {
        return Ok(runtime);
    }

    Ok(ContainerRuntime {
        name: "none".to_string(),
        version: String::new(),
        available: false,
        path: None,
    })
}

/// Check if Docker Desktop is available (macOS/Windows)
fn check_docker_desktop() -> Option<ContainerRuntime> {
    // Check common Docker Desktop installation paths
    let docker_paths = [
        "/Applications/Docker.app/Contents/Resources/bin/docker",
        "/usr/local/bin/docker",
        "/usr/bin/docker",
    ];

    for path in &docker_paths {
        if Command::new("sh")
            .arg("-c")
            .arg(format!("test -x {} 2>/dev/null", path))
            .output()
            .ok()?
            .status
            .success()
        {
            // Check if it's Docker Desktop by looking for docker-desktop context
            let context_output = Command::new("docker")
                .arg("context")
                .arg("ls")
                .output()
                .ok()?;

            let context_str = String::from_utf8_lossy(&context_output.stdout);
            if context_str.contains("desktop-linux") || context_str.contains("docker-desktop") {
                let version = get_version("docker", "--version").unwrap_or_default();
                tracing::info!("Docker Desktop detected at {}", path);
                return Some(ContainerRuntime {
                    name: "docker".to_string(),
                    version,
                    available: true,
                    path: Some(path.to_string()),
                });
            }
        }
    }

    None
}

/// Check if Docker CLI is available
fn check_docker_cli() -> Option<ContainerRuntime> {
    if !check_command_exists("docker") {
        return None;
    }

    let version = get_version("docker", "--version")?;
    let path = Command::new("sh")
        .arg("-c")
        .arg("which docker")
        .output()
        .ok()
        .and_then(|o| Some(String::from_utf8_lossy(&o.stdout).trim().to_string()));

    tracing::info!("Docker CLI detected at {:?}", path);
    Some(ContainerRuntime {
        name: "docker".to_string(),
        version,
        available: true,
        path,
    })
}

/// Check if Colima is available
fn check_colima() -> Option<ContainerRuntime> {
    if !check_command_exists("colima") {
        return None;
    }

    let version = get_version("colima", "--version")?;
    let path = Command::new("sh")
        .arg("-c")
        .arg("which colima")
        .output()
        .ok()
        .and_then(|o| Some(String::from_utf8_lossy(&o.stdout).trim().to_string()));

    tracing::info!("Colima detected at {:?}", path);
    Some(ContainerRuntime {
        name: "colima".to_string(),
        version,
        available: true,
        path,
    })
}

/// Check if Podman is available
fn check_podman() -> Option<ContainerRuntime> {
    if !check_command_exists("podman") {
        return None;
    }

    let version = get_version("podman", "--version")?;
    let path = Command::new("sh")
        .arg("-c")
        .arg("which podman")
        .output()
        .ok()
        .and_then(|o| Some(String::from_utf8_lossy(&o.stdout).trim().to_string()));

    tracing::info!("Podman detected at {:?}", path);
    Some(ContainerRuntime {
        name: "podman".to_string(),
        version,
        available: true,
        path,
    })
}

/// Get detailed information about a specific container runtime
pub fn get_runtime_info(runtime: &str) -> Result<RuntimeInfo, String> {
    match runtime {
        "docker" => get_docker_info(),
        "colima" => get_colima_info(),
        "podman" => get_podman_info(),
        _ => Err(format!("Unknown container runtime: {}", runtime)),
    }
}

/// Get Docker-specific runtime information
fn get_docker_info() -> Result<RuntimeInfo, String> {
    if !check_command_exists("docker") {
        return Err("Docker is not installed or not in PATH".to_string());
    }

    let version = get_version("docker", "--version")
        .ok_or_else(|| "Failed to get Docker version".to_string())?;

    let api_version = get_version("docker", "version", "--format", "{{.Server.APIVersion}}")
        .unwrap_or_else(|| "unknown".to_string());

    let containers = parse_container_count("docker", "ps", "-aq")?;
    let images = parse_image_count("docker", "images", "-q")?;

    Ok(RuntimeInfo {
        runtime: "docker".to_string(),
        version,
        api_version,
        containers,
        images,
    })
}

/// Get Colima-specific runtime information
fn get_colima_info() -> Result<RuntimeInfo, String> {
    if !check_command_exists("colima") {
        return Err("Colima is not installed or not in PATH".to_string());
    }

    let version = get_version("colima", "--version")
        .ok_or_else(|| "Failed to get Colima version".to_string())?;

    // Colima uses Docker under the hood, so we query docker with COLIMA=1 env
    let containers = parse_container_count_with_env("COLIMA=1", "docker", "ps", "-aq")?;
    let images = parse_image_count_with_env("COLIMA=1", "docker", "images", "-q")?;

    Ok(RuntimeInfo {
        runtime: "colima".to_string(),
        version,
        api_version: "docker".to_string(), // Colima exposes Docker API
        containers,
        images,
    })
}

/// Get Podman-specific runtime information
fn get_podman_info() -> Result<RuntimeInfo, String> {
    if !check_command_exists("podman") {
        return Err("Podman is not installed or not in PATH".to_string());
    }

    let version = get_version("podman", "--version")
        .ok_or_else(|| "Failed to get Podman version".to_string())?;

    let api_version = get_version("podman", "info", "--format", "{{.Host.ServerVersion}}")
        .unwrap_or_else(|| "unknown".to_string());

    let containers = parse_podman_container_count()?;
    let images = parse_podman_image_count()?;

    Ok(RuntimeInfo {
        runtime: "podman".to_string(),
        version,
        api_version,
        containers,
        images,
    })
}

/// Ensures Kind CLI is installed, downloads if not present
/// Supports: Linux x64/ARM64, macOS Intel/Apple Silicon
pub fn ensure_kind_installed() -> Result<bool, String> {
    if check_command_exists("kind") {
        tracing::info!("Kind is already installed");
        return Ok(true);
    }

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let kind_version = get_latest_kind_version()?;
    let download_url = format!("https://kind.sigs.k8s.io/dl/v}/kind-{}-{}", os, arch);

    // Adjust arch naming for URL
    let arch_str = match arch {
        "x86_64" => "amd64",
        "aarch64" | "arm64" => "arm64",
        _ => arch,
    };

    let download_url = format!(
        "https://kind.sigs.k8s.io/dl/v{}/kind-{}-{}",
        kind_version, os, arch_str
    );

    tracing::info!("Downloading Kind from {}", download_url);

    // Create temp directory
    let temp_dir = std::env::temp_dir();
    let kind_path = temp_dir.join(format!("kind-{}", std::process::id()));

    // Download Kind binary
    let download_output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "curl -Lo {} {} && chmod +x {}",
            kind_path.display(),
            download_url,
            kind_path.display()
        ))
        .output()
        .map_err(|e| format!("Failed to download Kind: {}", e))?;

    if !download_output.status.success() {
        return Err(String::from_utf8_lossy(&download_output.stderr).to_string());
    }

    // Move to /usr/local/bin or ~/.local/bin
    let target_path = if Command::new("sh")
        .arg("-c")
        .arg("test -w /usr/local/bin")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "/usr/local/bin/kind"
    } else {
        format!("{}/bin/kind", std::env::home_dir()?.display())
    };

    let move_output = Command::new("sh")
        .arg("-c")
        .arg(format!("mv {} {}", kind_path.display(), target_path))
        .output()
        .map_err(|e| format!("Failed to move Kind: {}", e))?;

    if !move_output.status.success() {
        return Err(String::from_utf8_lossy(&move_output.stderr).to_string());
    }

    tracing::info!("Kind installed successfully to {}", target_path);
    Ok(true)
}

/// Get the latest Kind release version
fn get_latest_kind_version() -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("curl -s https://api.github.com/repos/kubernetes-sigs/kind/releases/latest | grep -oP '\"tag_name\": \"\\K[^\"]+'")
        .output()
        .map_err(|e| format!("Failed to fetch Kind version: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version.trim_start_matches('v').to_string())
}

/// Check if a specific Kind cluster is running
pub fn is_kind_cluster_running(name: &str) -> Result<bool, String> {
    if !check_command_exists("kind") {
        return Err("Kind is not installed".to_string());
    }

    let output = Command::new("kind")
        .arg("get")
        .arg("clusters")
        .output()
        .map_err(|e| format!("Failed to get clusters: {}", e))?;

    let clusters = String::from_utf8_lossy(&output.stdout);
    let cluster_list: Vec<&str> = clusters.lines().collect();

    Ok(cluster_list.contains(&name))
}

/// Get status of all Kind clusters
pub fn get_all_cluster_status() -> Result<Vec<ClusterStatus>, String> {
    if !check_command_exists("kind") {
        return Err("Kind is not installed".to_string());
    }

    // Get list of clusters
    let output = Command::new("kind")
        .arg("get")
        .arg("clusters")
        .output()
        .map_err(|e| format!("Failed to get clusters: {}", e))?;

    let clusters_output = String::from_utf8_lossy(&output.stdout);
    let cluster_names: Vec<&str> = clusters_output.lines().filter(|s| !s.is_empty()).collect();

    let mut statuses = Vec::new();

    for name in cluster_names {
        let status = get_cluster_status(name)?;
        statuses.push(status);
    }

    Ok(statuses)
}

/// Get detailed status of a specific Kind cluster
fn get_cluster_status(name: &str) -> Result<ClusterStatus, String> {
    // Get cluster nodes
    let node_output = Command::new("kind")
        .arg("get")
        .arg("nodes")
        .arg("--name")
        .arg(name)
        .output()
        .map_err(|e| format!("Failed to get nodes: {}", e))?;

    let nodes_str = String::from_utf8_lossy(&node_output.stdout);
    let node_names: Vec<&str> = nodes_str.lines().filter(|s| !s.is_empty()).collect();
    let node_count = node_names.len();

    // Check if cluster is running by trying to get kubeconfig
    let cluster_info = Command::new("kind")
        .arg("get")
        .arg("kubeconfig")
        .arg("--name")
        .arg(name)
        .output()
        .ok();

    let status = match cluster_info {
        Some(ref output) if output.status.success() => "running",
        _ => "stopped",
    };

    // Try to get Kubernetes version
    let k8s_version = get_kubernetes_version(name).ok();

    Ok(ClusterStatus {
        name: name.to_string(),
        status: status.to_string(),
        kubernetes_version: k8s_version,
        nodes: node_count,
    })
}

/// Get the Kubernetes version of a Kind cluster
fn get_kubernetes_version(cluster_name: &str) -> Result<String, String> {
    let output = Command::new("kubectl")
        .arg("--context")
        .arg(format!("kind-{}", cluster_name))
        .arg("version")
        .arg("--short")
        .output()
        .ok()
        .filter(|o| o.status.success());

    match output {
        Some(o) => {
            let version_str = String::from_utf8_lossy(&o.stdout);
            // Parse version like "Client Version: v1.28.0" or "Server Version: v1.28.0"
            if let Some(version) = version_str.lines().find(|l| l.contains("Server Version")) {
                if let Some(v) = version.split("Server Version: ").nth(1) {
                    return Ok(v.trim().to_string());
                }
            }
            Err("Could not parse Kubernetes version".to_string())
        }
        None => Err("kubectl not available or cluster not accessible".to_string()),
    }
}

// Helper functions

/// Check if a command exists in PATH
fn check_command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("which {} 2>/dev/null", cmd))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get version output from a command
fn get_version(cmd: &str, version_flag: &str) -> Option<String> {
    get_version_with_args(cmd, &[version_flag])
}

/// Get version output with custom arguments
fn get_version_with_args(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd).args(args).output().ok().and_then(|o| {
        if o.status.success() {
            Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
        } else {
            None
        }
    })
}

/// Parse container count from docker ps output
fn parse_container_count(cmd: &str, subcmd: &str, flag: &str) -> Result<usize, String> {
    let output = Command::new(cmd)
        .arg(subcmd)
        .arg(flag)
        .output()
        .map_err(|e| format!("Failed to execute {} {}: {}", cmd, subcmd, e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().filter(|s| !s.is_empty()).count();
        Ok(count)
    } else {
        Ok(0)
    }
}

/// Parse container count with environment variable
fn parse_container_count_with_env(
    env: &str,
    cmd: &str,
    subcmd: &str,
    flag: &str,
) -> Result<usize, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("{} {} {} {}", env, cmd, subcmd, flag))
        .output()
        .map_err(|e| format!("Failed to execute {} {}: {}", cmd, subcmd, e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().filter(|s| !s.is_empty()).count();
        Ok(count)
    } else {
        Ok(0)
    }
}

/// Parse image count from docker images output
fn parse_image_count(cmd: &str, subcmd: &str, flag: &str) -> Result<usize, String> {
    parse_container_count(cmd, subcmd, flag)
}

/// Parse image count with environment variable
fn parse_image_count_with_env(
    env: &str,
    cmd: &str,
    subcmd: &str,
    flag: &str,
) -> Result<usize, String> {
    parse_container_count_with_env(env, cmd, subcmd, flag)
}

/// Parse Podman container count
fn parse_podman_container_count() -> Result<usize, String> {
    let output = Command::new("podman")
        .arg("ps")
        .arg("-aq")
        .output()
        .map_err(|e| format!("Failed to execute podman ps: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().filter(|s| !s.is_empty()).count();
        Ok(count)
    } else {
        Ok(0)
    }
}

/// Parse Podman image count
fn parse_podman_image_count() -> Result<usize, String> {
    let output = Command::new("podman")
        .arg("images")
        .arg("-q")
        .output()
        .map_err(|e| format!("Failed to execute podman images: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().filter(|s| !s.is_empty()).count();
        Ok(count)
    } else {
        Ok(0)
    }
}

// Re-export for use as tauri commands
#[cfg(feature = "shell-open")]
mod shell_commands {
    use super::*;

    #[tauri::command]
    pub fn get_container_runtime_command() -> Result<ContainerRuntime, String> {
        get_container_runtime()
    }

    #[tauri::command]
    pub fn get_runtime_info_command(runtime: String) -> Result<RuntimeInfo, String> {
        get_runtime_info(&runtime)
    }

    #[tauri::command]
    pub fn ensure_kind_installed_command() -> Result<bool, String> {
        ensure_kind_installed()
    }

    #[tauri::command]
    pub fn is_kind_cluster_running_command(name: String) -> Result<bool, String> {
        is_kind_cluster_running(&name)
    }

    #[tauri::command]
    pub fn get_all_cluster_status_command() -> Result<Vec<ClusterStatus>, String> {
        get_all_cluster_status()
    }

    pub use ensure_kind_installed_command as ensure_kind_installed;
    pub use get_all_cluster_status_command as get_all_cluster_status;
    pub use get_container_runtime_command as get_container_runtime;
    pub use get_runtime_info_command as get_runtime_info;
    pub use is_kind_cluster_running_command as is_kind_cluster_running;
}

#[cfg(feature = "shell-open")]
pub use shell_commands::*;
