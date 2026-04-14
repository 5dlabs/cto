//! Kind cluster management commands

use crate::error::{AppError, AppResult};
use crate::runtime::{self as runtime, ContainerRuntime};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing;

const CLUSTER_NAME: &str = "cto-lite";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClusterType {
    /// Kind cluster (what we create)
    Kind,
    /// Docker Desktop's built-in Kubernetes
    DockerDesktop,
    /// Rancher Desktop
    RancherDesktop,
    /// Minikube
    Minikube,
    /// K3d (k3s in docker)
    K3d,
    /// OrbStack
    OrbStack,
    /// Unknown/other cluster
    Other,
}

/// Detected existing cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedCluster {
    pub name: String,
    pub context: String,
    pub cluster_type: ClusterType,
    pub server: Option<String>,
    pub is_running: bool,
    pub is_current: bool,
    pub kubernetes_version: Option<String>,
    pub kubeconfig_path: Option<String>,
}

/// Installed Kubernetes tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledTool {
    pub name: String,
    pub path: String,
    pub version: Option<String>,
}

/// Full environment scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentScan {
    /// Installed K8s management tools
    pub installed_tools: Vec<InstalledTool>,
    /// Detected kubeconfig files
    pub kubeconfig_files: Vec<KubeconfigFile>,
    /// Detected clusters from all configs
    pub clusters: Vec<DetectedCluster>,
    /// Whether any usable cluster exists
    pub has_existing: bool,
    /// Recommendation
    pub recommendation: ClusterRecommendation,
}

/// Kubeconfig file info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubeconfigFile {
    pub path: String,
    pub exists: bool,
    pub contexts: Vec<String>,
}

/// Result of cluster detection (kept for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterDetectionResult {
    pub clusters: Vec<DetectedCluster>,
    pub has_existing: bool,
    pub recommendation: ClusterRecommendation,
}

/// Recommendation for which cluster to use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterRecommendation {
    /// Use an existing cluster
    UseExisting { context: String, reason: String },
    /// Create a new Kind cluster
    CreateKind { reason: String },
}

/// Cluster status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub name: String,
    pub exists: bool,
    pub running: bool,
    pub nodes: Vec<NodeStatus>,
    pub kubeconfig_path: Option<String>,
}

/// Node status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub name: String,
    pub role: String,
    pub status: String,
}

/// Run a kind command and return stdout
fn kind_command() -> Command {
    let mut command = Command::new("kind");

    if let Some(docker_path) = runtime::get_runtime_path(ContainerRuntime::Docker) {
        if let Some(parent) = std::path::Path::new(&docker_path).parent() {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let parent_path = parent.to_string_lossy();
            if !current_path.split(':').any(|entry| entry == parent_path) {
                command.env("PATH", format!("{}:{}", parent_path, current_path));
            }
        }
    }

    command
}

fn run_kind(args: &[&str]) -> AppResult<String> {
    let output = kind_command()
        .args(args)
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run kind: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::ClusterError(stderr.to_string()))
    }
}

/// Run kubectl command
fn run_kubectl(args: &[&str]) -> AppResult<String> {
    let output = Command::new("kubectl")
        .args(args)
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to run kubectl: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::ClusterError(stderr.to_string()))
    }
}

/// Check if kind is installed
fn is_kind_installed() -> bool {
    which::which("kind").is_ok()
}

/// Check if kubectl is installed
fn is_kubectl_installed() -> bool {
    which::which("kubectl").is_ok()
}

/// Get list of kind clusters
fn list_kind_clusters() -> AppResult<Vec<String>> {
    let output = run_kind(&["get", "clusters"])?;
    Ok(output
        .lines()
        .map(String::from)
        .filter(|s| !s.is_empty())
        .collect())
}

/// Check if our cluster exists
fn cluster_exists() -> bool {
    list_kind_clusters()
        .map(|clusters| clusters.contains(&CLUSTER_NAME.to_string()))
        .unwrap_or(false)
}

/// Get version from a command
fn get_tool_version(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            // Extract version number - usually first line, may have prefix
            out.lines().next().unwrap_or("").trim().to_string()
        })
        .filter(|s| !s.is_empty())
}

/// Scan for installed Kubernetes tools
fn scan_installed_tools() -> Vec<InstalledTool> {
    let tools = [
        ("kubectl", &["version", "--client", "--short"][..]),
        ("kind", &["version"][..]),
        ("minikube", &["version", "--short"][..]),
        ("k3d", &["version"][..]),
        ("docker", &["--version"][..]),
        ("colima", &["version"][..]),
        ("podman", &["--version"][..]),
        ("orbctl", &["version"][..]),
        ("helm", &["version", "--short"][..]),
    ];

    tools
        .iter()
        .filter_map(|(name, version_args)| {
            which::which(name).ok().map(|path| InstalledTool {
                name: name.to_string(),
                path: path.to_string_lossy().to_string(),
                version: get_tool_version(name, version_args),
            })
        })
        .collect()
}

/// Get all possible kubeconfig file paths
#[allow(dead_code)]
fn get_kubeconfig_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    // Default location
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".kube").join("config"));
    }

    // KUBECONFIG environment variable (can be multiple paths separated by :)
    if let Ok(kubeconfig_env) = std::env::var("KUBECONFIG") {
        for path in kubeconfig_env.split(':') {
            if !path.is_empty() {
                paths.push(std::path::PathBuf::from(path));
            }
        }
    }

    // Tool-specific locations
    if let Some(home) = dirs::home_dir() {
        // Minikube
        paths.push(home.join(".minikube").join("profiles"));
        // K3d
        paths.push(home.join(".k3d"));
        // Rancher Desktop
        paths.push(home.join(".rd").join("kube").join("config"));
        // Docker Desktop (macOS)
        #[cfg(target_os = "macos")]
        paths.push(
            home.join("Library")
                .join("Group Containers")
                .join("group.com.docker")
                .join("settings.json"),
        );
    }

    // Remove duplicates
    paths.sort();
    paths.dedup();
    paths
}

/// Scan kubeconfig files and extract contexts
fn scan_kubeconfig_files() -> Vec<KubeconfigFile> {
    let mut results = Vec::new();

    // Primary kubeconfig
    let primary = dirs::home_dir().map(|h| h.join(".kube").join("config"));

    if let Some(path) = primary {
        let exists = path.exists();
        let contexts = if exists {
            Command::new("kubectl")
                .args([
                    "config",
                    "get-contexts",
                    "-o",
                    "name",
                    "--kubeconfig",
                    path.to_str().unwrap_or(""),
                ])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| {
                    String::from_utf8_lossy(&o.stdout)
                        .lines()
                        .map(String::from)
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };

        results.push(KubeconfigFile {
            path: path.to_string_lossy().to_string(),
            exists,
            contexts,
        });
    }

    // Check KUBECONFIG env
    if let Ok(kubeconfig_env) = std::env::var("KUBECONFIG") {
        for path_str in kubeconfig_env.split(':') {
            if path_str.is_empty() {
                continue;
            }
            let path = std::path::PathBuf::from(path_str);
            if results.iter().any(|k| k.path == path.to_string_lossy()) {
                continue; // Skip duplicates
            }

            let exists = path.exists();
            let contexts = if exists {
                Command::new("kubectl")
                    .args([
                        "config",
                        "get-contexts",
                        "-o",
                        "name",
                        "--kubeconfig",
                        path_str,
                    ])
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout)
                            .lines()
                            .map(String::from)
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                vec![]
            };

            results.push(KubeconfigFile {
                path: path.to_string_lossy().to_string(),
                exists,
                contexts,
            });
        }
    }

    results
}

/// Get Kubernetes server version for a context
fn get_cluster_version(context: &str) -> Option<String> {
    Command::new("kubectl")
        .args(["--context", context, "version", "--short"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            // Look for "Server Version:" line
            out.lines()
                .find(|l| l.contains("Server"))
                .map(|l| l.replace("Server Version:", "").trim().to_string())
        })
}

/// Get node status from kubectl
fn get_node_status() -> AppResult<Vec<NodeStatus>> {
    let output = run_kubectl(&[
        "get", "nodes",
        "--context", &format!("kind-{}", CLUSTER_NAME),
        "-o", "custom-columns=NAME:.metadata.name,ROLE:.metadata.labels.node-role\\.kubernetes\\.io/control-plane,STATUS:.status.conditions[-1].type",
        "--no-headers",
    ])?;

    let nodes = output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            NodeStatus {
                name: parts.first().unwrap_or(&"").to_string(),
                role: if parts.get(1).map(|s| !s.is_empty()).unwrap_or(false) {
                    "control-plane".to_string()
                } else {
                    "worker".to_string()
                },
                status: parts.get(2).unwrap_or(&"Unknown").to_string(),
            }
        })
        .collect();

    Ok(nodes)
}

/// Detect the type of cluster from context name and server
fn detect_cluster_type(context: &str, server: &str) -> ClusterType {
    let ctx_lower = context.to_lowercase();
    let server_lower = server.to_lowercase();

    if ctx_lower.starts_with("kind-") || ctx_lower.contains("kind") {
        ClusterType::Kind
    } else if ctx_lower == "docker-desktop" || server_lower.contains("docker.internal") {
        ClusterType::DockerDesktop
    } else if ctx_lower.contains("rancher") || server_lower.contains("rancher") {
        ClusterType::RancherDesktop
    } else if ctx_lower.contains("minikube") {
        ClusterType::Minikube
    } else if ctx_lower.starts_with("k3d-") {
        ClusterType::K3d
    } else if ctx_lower.contains("orbstack")
        || server_lower.contains("orbstack")
        || server_lower.contains("orb.local")
    {
        ClusterType::OrbStack
    } else {
        ClusterType::Other
    }
}

/// Check if a cluster context is reachable
fn is_cluster_running(context: &str) -> bool {
    let output = Command::new("kubectl")
        .args(["--context", context, "cluster-info"])
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

/// Detect existing Kubernetes clusters on the system
#[tauri::command]
pub async fn detect_existing_clusters() -> Result<ClusterDetectionResult, AppError> {
    if !is_kubectl_installed() {
        return Ok(ClusterDetectionResult {
            clusters: vec![],
            has_existing: false,
            recommendation: ClusterRecommendation::CreateKind {
                reason: "No kubectl found - will install Kind with kubectl".to_string(),
            },
        });
    }

    // Get all contexts from kubeconfig
    let contexts_output = Command::new("kubectl")
        .args(["config", "get-contexts", "-o", "name"])
        .output();

    let contexts: Vec<String> = contexts_output
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(String::from)
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // Get current context
    let current_context = Command::new("kubectl")
        .args(["config", "current-context"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let mut clusters = Vec::new();
    let mut has_cto_lite = false;
    let mut has_running_local = false;
    let mut best_existing: Option<String> = None;

    for context in contexts {
        // Get cluster info for this context
        let cluster_info = Command::new("kubectl")
            .args([
                "config",
                "view",
                "-o",
                &format!(
                    "jsonpath={{.contexts[?(@.name==\"{}\")].context.cluster}}",
                    context
                ),
            ])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        // Get server URL
        let server = Command::new("kubectl")
            .args([
                "config",
                "view",
                "-o",
                &format!(
                    "jsonpath={{.clusters[?(@.name==\"{}\")].cluster.server}}",
                    cluster_info
                ),
            ])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let cluster_type = detect_cluster_type(&context, &server);
        let is_running = is_cluster_running(&context);
        let is_current = current_context.as_ref() == Some(&context);

        // Track if we already have cto-lite
        if context == format!("kind-{}", CLUSTER_NAME) {
            has_cto_lite = true;
        }

        // Track if there's a running local cluster we could use
        if is_running && cluster_type != ClusterType::Other {
            has_running_local = true;
            if best_existing.is_none() {
                best_existing = Some(context.clone());
            }
        }

        // Get K8s version if running
        let kubernetes_version = if is_running {
            get_cluster_version(&context)
        } else {
            None
        };

        clusters.push(DetectedCluster {
            name: cluster_info.clone(),
            context: context.clone(),
            cluster_type,
            server: if server.is_empty() {
                None
            } else {
                Some(server)
            },
            is_running,
            is_current,
            kubernetes_version,
            kubeconfig_path: None, // Using default kubeconfig
        });
    }

    // Determine recommendation
    let recommendation = if has_cto_lite {
        ClusterRecommendation::UseExisting {
            context: format!("kind-{}", CLUSTER_NAME),
            reason: "CTO cluster already exists".to_string(),
        }
    } else if let Some(ctx) = best_existing {
        // Find the cluster type for better messaging
        let cluster_type = clusters
            .iter()
            .find(|c| c.context == ctx)
            .map(|c| &c.cluster_type);

        let type_name = match cluster_type {
            Some(ClusterType::DockerDesktop) => "Docker Desktop Kubernetes",
            Some(ClusterType::RancherDesktop) => "Rancher Desktop",
            Some(ClusterType::Minikube) => "Minikube",
            Some(ClusterType::K3d) => "K3d",
            Some(ClusterType::OrbStack) => "OrbStack",
            Some(ClusterType::Kind) => "Kind cluster",
            _ => "existing cluster",
        };

        ClusterRecommendation::UseExisting {
            context: ctx,
            reason: format!(
                "Found running {} - you can use this or create a dedicated CTO cluster",
                type_name
            ),
        }
    } else {
        ClusterRecommendation::CreateKind {
            reason: "No running local Kubernetes found - will create a Kind cluster".to_string(),
        }
    };

    Ok(ClusterDetectionResult {
        clusters,
        has_existing: has_running_local,
        recommendation,
    })
}

/// Full environment scan - checks installed tools, kubeconfigs, and clusters
#[tauri::command]
pub async fn scan_environment() -> Result<EnvironmentScan, AppError> {
    tracing::info!("Scanning environment for Kubernetes tools and clusters");

    // Scan for installed tools
    let installed_tools = scan_installed_tools();
    tracing::info!("Found {} installed tools", installed_tools.len());

    // Scan kubeconfig files
    let kubeconfig_files = scan_kubeconfig_files();
    tracing::info!("Found {} kubeconfig files", kubeconfig_files.len());

    // Collect all unique contexts across all kubeconfigs
    let all_contexts: Vec<String> = kubeconfig_files
        .iter()
        .flat_map(|k| k.contexts.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Get current context
    let current_context = Command::new("kubectl")
        .args(["config", "current-context"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let mut clusters = Vec::new();
    let mut has_cto_lite = false;
    let mut has_running_local = false;
    let mut best_existing: Option<String> = None;

    for context in all_contexts {
        // Get cluster info for this context
        let cluster_info = Command::new("kubectl")
            .args([
                "config",
                "view",
                "-o",
                &format!(
                    "jsonpath={{.contexts[?(@.name==\"{}\")].context.cluster}}",
                    context
                ),
            ])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        // Get server URL
        let server = Command::new("kubectl")
            .args([
                "config",
                "view",
                "-o",
                &format!(
                    "jsonpath={{.clusters[?(@.name==\"{}\")].cluster.server}}",
                    cluster_info
                ),
            ])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let cluster_type = detect_cluster_type(&context, &server);
        let is_running = is_cluster_running(&context);
        let is_current = current_context.as_ref() == Some(&context);

        // Get K8s version if running
        let kubernetes_version = if is_running {
            get_cluster_version(&context)
        } else {
            None
        };

        // Find which kubeconfig this context came from
        let kubeconfig_path = kubeconfig_files
            .iter()
            .find(|k| k.contexts.contains(&context))
            .map(|k| k.path.clone());

        // Track if we already have cto-lite
        if context == format!("kind-{}", CLUSTER_NAME) {
            has_cto_lite = true;
        }

        // Track if there's a running local cluster we could use
        if is_running && cluster_type != ClusterType::Other {
            has_running_local = true;
            if best_existing.is_none() {
                best_existing = Some(context.clone());
            }
        }

        clusters.push(DetectedCluster {
            name: cluster_info,
            context: context.clone(),
            cluster_type,
            server: if server.is_empty() {
                None
            } else {
                Some(server)
            },
            is_running,
            is_current,
            kubernetes_version,
            kubeconfig_path,
        });
    }

    // Sort clusters: running first, then by type
    clusters.sort_by(|a, b| match (a.is_running, b.is_running) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.context.cmp(&b.context),
    });

    // Determine recommendation
    let recommendation = if has_cto_lite {
        ClusterRecommendation::UseExisting {
            context: format!("kind-{}", CLUSTER_NAME),
            reason: "CTO cluster already exists".to_string(),
        }
    } else if let Some(ctx) = best_existing {
        let cluster_type = clusters
            .iter()
            .find(|c| c.context == ctx)
            .map(|c| &c.cluster_type);

        let type_name = match cluster_type {
            Some(ClusterType::DockerDesktop) => "Docker Desktop Kubernetes",
            Some(ClusterType::RancherDesktop) => "Rancher Desktop",
            Some(ClusterType::Minikube) => "Minikube",
            Some(ClusterType::K3d) => "K3d",
            Some(ClusterType::OrbStack) => "OrbStack",
            Some(ClusterType::Kind) => "Kind cluster",
            _ => "existing cluster",
        };

        ClusterRecommendation::UseExisting {
            context: ctx,
            reason: format!(
                "Found running {} - you can use this or create a dedicated CTO cluster",
                type_name
            ),
        }
    } else if installed_tools.iter().any(|t| t.name == "docker") {
        ClusterRecommendation::CreateKind {
            reason: "Docker is available - will create a Kind cluster".to_string(),
        }
    } else {
        ClusterRecommendation::CreateKind {
            reason: "No container runtime found - please install Docker first".to_string(),
        }
    };

    tracing::info!(
        "Environment scan complete: {} tools, {} configs, {} clusters ({} running)",
        installed_tools.len(),
        kubeconfig_files.len(),
        clusters.len(),
        clusters.iter().filter(|c| c.is_running).count()
    );

    Ok(EnvironmentScan {
        installed_tools,
        kubeconfig_files,
        clusters,
        has_existing: has_running_local,
        recommendation,
    })
}

/// Create the CTO Kind cluster
#[tauri::command]
pub async fn create_cluster() -> Result<ClusterStatus, AppError> {
    // Check prerequisites
    if !is_kind_installed() {
        return Err(AppError::CommandFailed(
            "kind is not installed. Please install it first.".to_string(),
        ));
    }

    if !is_kubectl_installed() {
        return Err(AppError::CommandFailed(
            "kubectl is not installed. Please install it first.".to_string(),
        ));
    }

    // Check if cluster already exists
    if cluster_exists() {
        tracing::info!("Cluster {} already exists", CLUSTER_NAME);
        return get_cluster_status().await;
    }

    tracing::info!("Creating Kind cluster: {}", CLUSTER_NAME);

    // Create cluster with a configuration that supports ingress
    let config = r#"
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
  - containerPort: 8080
    hostPort: 8080
    protocol: TCP
"#;

    // Write config to temp file
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("cto-lite-kind-config.yaml");
    std::fs::write(&config_path, config)?;

    // Create cluster
    let output = kind_command()
        .args([
            "create",
            "cluster",
            "--name",
            CLUSTER_NAME,
            "--config",
            config_path.to_str().unwrap(),
            "--wait",
            "300s",
        ])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to create cluster: {}", e)))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&config_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::ClusterError(format!(
            "Failed to create cluster: {}",
            stderr
        )));
    }

    tracing::info!("Cluster {} created successfully", CLUSTER_NAME);

    get_cluster_status().await
}

/// Delete the CTO Kind cluster
#[tauri::command]
pub async fn delete_cluster() -> Result<(), AppError> {
    if !cluster_exists() {
        tracing::info!("Cluster {} does not exist", CLUSTER_NAME);
        return Ok(());
    }

    tracing::info!("Deleting Kind cluster: {}", CLUSTER_NAME);

    run_kind(&["delete", "cluster", "--name", CLUSTER_NAME])?;

    tracing::info!("Cluster {} deleted", CLUSTER_NAME);
    Ok(())
}

/// Get the status of the CTO cluster
#[tauri::command]
pub async fn get_cluster_status() -> Result<ClusterStatus, AppError> {
    let exists = cluster_exists();

    if !exists {
        return Ok(ClusterStatus {
            name: CLUSTER_NAME.to_string(),
            exists: false,
            running: false,
            nodes: vec![],
            kubeconfig_path: None,
        });
    }

    // Get kubeconfig path
    let kubeconfig = dirs::home_dir()
        .map(|h: std::path::PathBuf| h.join(".kube").join("config"))
        .and_then(|p: std::path::PathBuf| p.to_str().map(String::from));

    // Try to get node status - if this fails, cluster exists but isn't running
    let (running, nodes) = match get_node_status() {
        Ok(nodes) => {
            let all_ready = nodes.iter().all(|n| n.status == "Ready");
            (all_ready, nodes)
        }
        Err(_) => (false, vec![]),
    };

    Ok(ClusterStatus {
        name: CLUSTER_NAME.to_string(),
        exists,
        running,
        nodes,
        kubeconfig_path: kubeconfig,
    })
}

/// List all Kind clusters
#[tauri::command]
pub async fn list_clusters() -> Result<Vec<String>, AppError> {
    list_kind_clusters()
}

/// Use an existing cluster instead of creating a new Kind cluster
#[tauri::command]
pub async fn use_existing_cluster(
    context: String,
    db: tauri::State<'_, crate::db::Database>,
) -> Result<ClusterStatus, AppError> {
    // Verify the cluster is reachable
    if !is_cluster_running(&context) {
        return Err(AppError::ClusterError(format!(
            "Cluster context '{}' is not reachable. Make sure it's running.",
            context
        )));
    }

    // Set as current context
    let output = Command::new("kubectl")
        .args(["config", "use-context", &context])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to switch context: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::ClusterError(format!(
            "Failed to switch context: {}",
            stderr
        )));
    }

    // Save the chosen context to config
    db.set_config("cluster_context", &context)?;
    db.set_config("cluster_type", "existing")?;

    tracing::info!("Using existing cluster context: {}", context);

    // Get node status for this cluster
    let nodes_output = Command::new("kubectl")
        .args([
            "--context", &context,
            "get", "nodes",
            "-o", "custom-columns=NAME:.metadata.name,ROLE:.metadata.labels.node-role\\.kubernetes\\.io/control-plane,STATUS:.status.conditions[-1].type",
            "--no-headers",
        ])
        .output();

    let nodes = nodes_output
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    NodeStatus {
                        name: parts.first().unwrap_or(&"").to_string(),
                        role: if parts.get(1).map(|s| !s.is_empty()).unwrap_or(false) {
                            "control-plane".to_string()
                        } else {
                            "worker".to_string()
                        },
                        status: parts.get(2).unwrap_or(&"Unknown").to_string(),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let kubeconfig = dirs::home_dir()
        .map(|h| h.join(".kube").join("config"))
        .and_then(|p| p.to_str().map(String::from));

    Ok(ClusterStatus {
        name: context,
        exists: true,
        running: true,
        nodes,
        kubeconfig_path: kubeconfig,
    })
}

// ============================================================================
// Smart Initialization - Zero Friction Setup
// ============================================================================

/// Result of smart initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartInitResult {
    /// Whether Docker was started
    pub docker_started: bool,
    /// Whether a cluster is ready
    pub cluster_ready: bool,
    /// The context being used
    pub context: String,
    /// What actions were taken
    pub actions: Vec<String>,
    /// Any errors that occurred
    pub errors: Vec<String>,
    /// Whether user intervention is required
    pub needs_user_action: bool,
    /// Message for user if intervention needed
    pub user_message: Option<String>,
}

/// Smart initialization - fully automated setup
/// This is the main entry point for zero-touch initialization
#[tauri::command]
pub async fn smart_init(
    db: tauri::State<'_, crate::db::Database>,
) -> Result<SmartInitResult, AppError> {
    tracing::info!("Starting smart initialization...");

    let mut result = SmartInitResult {
        docker_started: false,
        cluster_ready: false,
        context: String::new(),
        actions: Vec::new(),
        errors: Vec::new(),
        needs_user_action: false,
        user_message: None,
    };

    // Step 1: Check and start a Docker-compatible runtime
    tracing::info!("Step 1: Checking container runtime...");
    match check_and_start_runtime().await {
        Ok(started) => {
            if started {
                result.docker_started = true;
                result
                    .actions
                    .push("Started a container runtime".to_string());
            } else {
                result
                    .actions
                    .push("A compatible container runtime was already running".to_string());
            }
        }
        Err(e) => {
            tracing::error!("Failed to start container runtime: {}", e);
            result.errors.push(format!("Container runtime: {}", e));
        }
    }

    // Step 2: Wait for Docker specifically if Docker Desktop was the runtime that came up.
    if result.docker_started
        && crate::runtime::is_runtime_running(crate::runtime::ContainerRuntime::Docker)
    {
        tracing::info!("Waiting for Docker daemon to be ready...");
        if let Err(e) = wait_for_docker_ready(30) {
            tracing::warn!("Docker ready check timed out: {}", e);
            result
                .actions
                .push("Waiting for Docker (may still be starting)".to_string());
        }
    }

    // Step 3: Check/install Kind
    tracing::info!("Step 2: Checking Kind installation...");
    match ensure_kind_installed().await {
        Ok(true) => {
            result
                .actions
                .push("Kind installed (or already present)".to_string());
        }
        Ok(false) => {
            result.actions.push("Kind binary found".to_string());
        }
        Err(e) => {
            tracing::error!("Failed to install Kind: {}", e);
            result.errors.push(format!("Kind installation: {}", e));
            result.needs_user_action = true;
            result.user_message =
                Some("Failed to install Kind. Please run: brew install kind".to_string());
        }
    }

    // Step 4: Check for existing clusters
    tracing::info!("Step 3: Checking for existing clusters...");
    let existing = detect_existing_clusters().await?;

    // Check if CTO cluster exists
    let cto_context = format!("kind-{}", CLUSTER_NAME);
    let cto_cluster = existing.clusters.iter().find(|c| c.context == cto_context);

    if let Some(cluster) = cto_cluster {
        if cluster.is_running {
            tracing::info!("Found existing CTO cluster running");
            result.actions.push(format!(
                "Using existing cluster '{}' (v{})",
                cluster.context,
                cluster.kubernetes_version.as_deref().unwrap_or("unknown")
            ));
            result.cluster_ready = true;
            result.context = cluster.context.clone();

            // Switch to it
            let _ = Command::new("kubectl")
                .args(["config", "use-context", &cluster.context])
                .output();
        } else {
            tracing::info!("Found existing CTO cluster (not running)");
            result
                .actions
                .push("Found CTO cluster (not running)".to_string());
            // For now, we'll still try to create a new one since this one isn't running
        }
    }

    // Step 5: If no CTO cluster, check for other usable clusters
    if !result.cluster_ready {
        tracing::info!("No CTO cluster found, checking for alternatives...");

        // Look for running Docker Desktop K8s, OrbStack, etc.
        let usable = existing
            .clusters
            .iter()
            .filter(|c| c.is_running && c.cluster_type != ClusterType::Other)
            .collect::<Vec<_>>();

        if let Some(best) = usable.first() {
            tracing::info!("Found alternative cluster: {}", best.context);
            result.actions.push(format!(
                "Using existing {} cluster",
                match best.cluster_type {
                    ClusterType::DockerDesktop => "Docker Desktop",
                    ClusterType::OrbStack => "OrbStack",
                    ClusterType::RancherDesktop => "Rancher Desktop",
                    _ => "Kubernetes",
                }
            ));

            // Switch to it
            let output = Command::new("kubectl")
                .args(["config", "use-context", &best.context])
                .output();

            if output.map(|o| o.status.success()).unwrap_or(false) {
                result.cluster_ready = true;
                result.context = best.context.clone();

                // Save preference
                let _ = db.set_config("prefer_existing_cluster", "true");
                let _ = db.set_config("cluster_context", &best.context);
            }
        }
    }

    // Step 6: If still no cluster, create one
    if !result.cluster_ready {
        tracing::info!("No usable cluster found, creating new CTO cluster...");
        result.actions.push("Creating new CTO cluster".to_string());

        // Check prerequisites first
        if !is_kind_installed() {
            result.errors.push("Kind is not installed".to_string());
            result.needs_user_action = true;
            result.user_message = Some("Please install Kind: brew install kind".to_string());
            return Ok(result);
        }

        // Create the cluster
        match create_cluster().await {
            Ok(status) => {
                if status.running {
                    result.cluster_ready = true;
                    result.context = cto_context.clone();
                    result
                        .actions
                        .push("CTO cluster created and running".to_string());

                    // Save preference
                    let _ = db.set_config("prefer_cto_cluster", "true");
                    let _ = db.set_config("cluster_context", &cto_context);
                } else {
                    result
                        .errors
                        .push("Cluster created but not running".to_string());
                    result.needs_user_action = true;
                    result.user_message = Some(
                        "Cluster was created but is not yet ready. Please wait...".to_string(),
                    );
                }
            }
            Err(e) => {
                tracing::error!("Failed to create cluster: {}", e);
                result.errors.push(format!("Cluster creation: {}", e));
                result.needs_user_action = true;
                result.user_message =
                    Some("Failed to create cluster. Check logs for details.".to_string());
            }
        }
    }

    // Final status
    tracing::info!(
        "Smart init complete: docker_started={}, cluster_ready={}, context={}",
        result.docker_started,
        result.cluster_ready,
        result.context
    );

    Ok(result)
}

/// Check and start container runtime if needed
async fn check_and_start_runtime() -> AppResult<bool> {
    use crate::runtime as rt;

    match rt::auto_start_runtime()? {
        Some(runtime) => {
            tracing::info!("Started container runtime: {}", runtime);
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Wait for Docker daemon to be ready
fn wait_for_docker_ready(timeout_secs: u64) -> AppResult<()> {
    use crate::runtime as rt;
    rt::wait_for_docker_ready(timeout_secs)
}

/// Ensure Kind is installed, download if missing
async fn ensure_kind_installed() -> AppResult<bool> {
    use std::os::unix::fs::PermissionsExt;

    if is_kind_installed() {
        return Ok(false);
    }

    tracing::info!("Kind not found, downloading...");

    // Determine platform
    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        return Err(AppError::Other("Unsupported platform".to_string()));
    };

    // Determine architecture
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "amd64"
    };

    // Download URL for Kind
    let url = format!(
        "https://kind.sigs.k8s.io/dl/v0.25.0/kind-{}-{}",
        platform, arch
    );

    // Download to temp location
    let temp_dir = std::env::temp_dir();
    let kind_path = temp_dir.join("kind");

    let response = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(AppError::HttpError)?;

    if !response.status().is_success() {
        return Err(AppError::HttpError(
            response.error_for_status().unwrap_err(),
        ));
    }

    let bytes = response.bytes().await.map_err(AppError::HttpError)?;

    // Write to temp file
    std::fs::write(&kind_path, &bytes)?;

    // Make executable
    let mut perms = std::fs::metadata(&kind_path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&kind_path, perms)?;

    // Move to ~/.local/bin (create if needed)
    let local_bin = dirs::home_dir()
        .ok_or_else(|| AppError::CommandFailed("Cannot find home directory".to_string()))?
        .join(".local/bin");

    std::fs::create_dir_all(&local_bin)?;

    let final_path = local_bin.join("kind");
    std::fs::rename(&kind_path, &final_path)?;

    // Ensure ~/.local/bin is in PATH
    let path_var = std::env::var("PATH").unwrap_or_default();
    if !path_var.contains(".local/bin") {
        tracing::warn!("~/.local/bin is not in PATH. Consider adding it to your shell profile.");
    }

    tracing::info!("Kind installed to: {:?}", final_path);
    Ok(true)
}

/// Quick health check - returns what's ready
#[tauri::command]
pub async fn quick_health_check() -> Result<serde_json::Value, AppError> {
    use crate::runtime as rt;

    let docker_ready = rt::is_runtime_running(rt::ContainerRuntime::Docker);
    let kind_ready = is_kind_installed();
    let cluster = get_cluster_status().await.ok();
    let cluster_ready = cluster.as_ref().map(|c| c.running).unwrap_or(false);

    Ok(serde_json::json!({
        "docker": {
            "available": rt::is_docker_available(),
            "ready": docker_ready,
        },
        "kind": {
            "installed": kind_ready,
        },
        "cluster": {
            "exists": cluster.as_ref().map(|c| c.exists).unwrap_or(false),
            "ready": cluster_ready,
            "name": cluster.map(|c| c.name).unwrap_or_else(|| "".to_string()),
        },
        "overall": docker_ready && cluster_ready,
        "message": if docker_ready && cluster_ready {
            "Everything is ready!"
        } else if !docker_ready {
            "Docker is not running"
        } else if !cluster_ready {
            "Kubernetes cluster is not ready"
        } else {
            "Some dependencies are missing"
        }
    }))
}
