//! Kind cluster management commands

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::process::Command;

const CLUSTER_NAME: &str = "cto-lite";

/// Types of Kubernetes clusters we can detect
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
fn run_kind(args: &[&str]) -> AppResult<String> {
    let output = Command::new("kind")
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
            reason: "CTO Lite cluster already exists".to_string(),
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
                "Found running {} - you can use this or create a dedicated CTO Lite cluster",
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
            reason: "CTO Lite cluster already exists".to_string(),
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
                "Found running {} - you can use this or create a dedicated CTO Lite cluster",
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

/// Create the CTO Lite Kind cluster
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
    let output = Command::new("kind")
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

/// Delete the CTO Lite Kind cluster
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

/// Get the status of the CTO Lite cluster
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
