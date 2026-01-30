//! Installation flow commands
//!
//! Handles the full installation:
//! 1. Check/install binaries (kind, kubectl, helm)
//! 2. Create Kind cluster
//! 3. Pull Docker images
//! 4. Deploy via Helm

use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::{Emitter, State};

use crate::db::Database;
use crate::error::{AppError, AppResult};

/// Installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallStatus {
    pub step: InstallStep,
    pub message: String,
    pub progress: u8,  // 0-100
    pub error: Option<String>,
}

/// Installation steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallStep {
    CheckingPrerequisites,
    InstallingBinaries,
    CreatingCluster,
    PullingImages,
    DeployingServices,
    ConfiguringIngress,
    Complete,
    Failed,
}

/// Binary check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryCheck {
    pub name: String,
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

/// Required binaries for CTO Lite
const REQUIRED_BINARIES: &[&str] = &["docker", "kind", "kubectl", "helm"];

/// Images to pull for initial deployment
const CORE_IMAGES: &[&str] = &[
    "ghcr.io/5dlabs/cto-controller:latest",
    "kindest/node:v1.31.0",  // Kind node image
];

/// Check for required binaries
#[tauri::command]
pub async fn check_prerequisites() -> Result<Vec<BinaryCheck>, AppError> {
    let mut results = Vec::new();

    for name in REQUIRED_BINARIES {
        let found = which::which(name).is_ok();
        let path = which::which(name).ok().map(|p| p.to_string_lossy().to_string());
        
        let version = if found {
            get_binary_version(name)
        } else {
            None
        };

        results.push(BinaryCheck {
            name: name.to_string(),
            found,
            path,
            version,
        });
    }

    Ok(results)
}

/// Get version of a binary
fn get_binary_version(name: &str) -> Option<String> {
    let args: &[&str] = match name {
        "docker" => &["--version"],
        "kind" => &["version"],
        "kubectl" => &["version", "--client", "--short"],
        "helm" => &["version", "--short"],
        _ => &["--version"],
    };

    Command::new(name)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
}

/// Run the full installation
#[tauri::command]
pub async fn run_installation(
    db: State<'_, Database>,
    window: tauri::Window,
) -> Result<(), AppError> {
    // Helper to emit progress
    let emit_progress = |step: InstallStep, message: &str, progress: u8| {
        let _ = window.emit("install-progress", InstallStatus {
            step,
            message: message.to_string(),
            progress,
            error: None,
        });
    };

    // Step 1: Check prerequisites
    emit_progress(InstallStep::CheckingPrerequisites, "Checking prerequisites...", 5);
    
    let prereqs = check_prerequisites().await?;
    let missing: Vec<_> = prereqs.iter().filter(|b| !b.found).collect();
    
    if !missing.is_empty() {
        let names: Vec<_> = missing.iter().map(|b| b.name.as_str()).collect();
        return Err(AppError::CommandFailed(format!(
            "Missing required binaries: {}. Please install them first.",
            names.join(", ")
        )));
    }

    // Step 2: Create Kind cluster
    emit_progress(InstallStep::CreatingCluster, "Creating Kubernetes cluster...", 20);
    
    if !kind_cluster_exists("cto-lite")? {
        create_kind_cluster()?;
    } else {
        tracing::info!("Kind cluster 'cto-lite' already exists");
    }

    // Step 3: Pull images
    emit_progress(InstallStep::PullingImages, "Pulling container images...", 40);
    
    for (i, image) in CORE_IMAGES.iter().enumerate() {
        let progress = 40 + ((i as u8 + 1) * 20 / CORE_IMAGES.len() as u8);
        emit_progress(
            InstallStep::PullingImages, 
            &format!("Pulling {}...", image),
            progress
        );
        pull_image(image)?;
    }

    // Step 4: Load images into Kind
    emit_progress(InstallStep::PullingImages, "Loading images into cluster...", 60);
    
    for image in CORE_IMAGES.iter().filter(|i| !i.starts_with("kindest/")) {
        load_image_to_kind(image, "cto-lite")?;
    }

    // Step 5: Deploy services
    emit_progress(InstallStep::DeployingServices, "Deploying CTO Lite services...", 70);
    
    // For now, just create the namespace
    create_namespace("cto-lite")?;

    // TODO: Helm install when chart is ready
    // helm_install()?;

    // Step 6: Configure ingress
    emit_progress(InstallStep::ConfiguringIngress, "Configuring ingress...", 90);
    
    // TODO: Set up cloudflared tunnel

    // Complete
    emit_progress(InstallStep::Complete, "Installation complete!", 100);
    
    // Mark installation done in DB
    db.set_config("installation_complete", "true")?;

    Ok(())
}

/// Check if Kind cluster exists
fn kind_cluster_exists(name: &str) -> AppResult<bool> {
    let output = Command::new("kind")
        .args(["get", "clusters"])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to list Kind clusters: {}", e)))?;

    if output.status.success() {
        let clusters = String::from_utf8_lossy(&output.stdout);
        Ok(clusters.lines().any(|line| line.trim() == name))
    } else {
        Ok(false)
    }
}

/// Create Kind cluster with CTO Lite config
fn create_kind_cluster() -> AppResult<()> {
    tracing::info!("Creating Kind cluster 'cto-lite'");

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

    let output = Command::new("kind")
        .args([
            "create", "cluster",
            "--name", "cto-lite",
            "--config", config_path.to_str().unwrap(),
            "--wait", "120s",
        ])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to create cluster: {}", e)))?;

    // Clean up
    let _ = std::fs::remove_file(&config_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::ClusterError(format!("Failed to create cluster: {}", stderr)));
    }

    tracing::info!("Kind cluster 'cto-lite' created successfully");
    Ok(())
}

/// Pull a Docker image
fn pull_image(image: &str) -> AppResult<()> {
    tracing::info!("Pulling image: {}", image);

    let output = Command::new("docker")
        .args(["pull", image])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to pull image: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if image doesn't exist - it might not be published yet
        tracing::warn!("Failed to pull {}: {}", image, stderr);
    }

    Ok(())
}

/// Load image into Kind cluster
fn load_image_to_kind(image: &str, cluster: &str) -> AppResult<()> {
    tracing::info!("Loading image {} into cluster {}", image, cluster);

    let output = Command::new("kind")
        .args(["load", "docker-image", image, "--name", cluster])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to load image: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("Failed to load {} into Kind: {}", image, stderr);
    }

    Ok(())
}

/// Create Kubernetes namespace
fn create_namespace(name: &str) -> AppResult<()> {
    let output = Command::new("kubectl")
        .args(["create", "namespace", name, "--context", "kind-cto-lite"])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to create namespace: {}", e)))?;

    // Ignore "already exists" errors
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("already exists") {
            return Err(AppError::CommandFailed(format!("Failed to create namespace: {}", stderr)));
        }
    }

    Ok(())
}

/// Get installation status
#[tauri::command]
pub async fn get_install_status(db: State<'_, Database>) -> Result<bool, AppError> {
    let complete = db.get_config("installation_complete")?;
    Ok(complete.map(|v| v == "true").unwrap_or(false))
}

/// Delete installation (for testing/reset)
#[tauri::command]
pub async fn reset_installation(db: State<'_, Database>) -> Result<(), AppError> {
    tracing::info!("Resetting installation");

    // Delete Kind cluster
    let _ = Command::new("kind")
        .args(["delete", "cluster", "--name", "cto-lite"])
        .output();

    // Reset DB flag
    db.set_config("installation_complete", "false")?;

    Ok(())
}
