//! Update management commands
//!
//! CTO components are distributed as Docker images, making updates
//! simple: just pull new images and restart the cluster.

use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::State;

use crate::db::Database;
use crate::error::AppResult;

/// Image version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVersion {
    pub image: String,
    pub current: Option<String>,
    pub latest: Option<String>,
    pub has_update: bool,
}

/// Update status for all components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub images: Vec<ImageVersion>,
    pub has_updates: bool,
    pub last_checked: Option<String>,
}

/// Core images that CTO uses
const CORE_IMAGES: &[&str] = &[
    "ghcr.io/5dlabs/controller",
    "ghcr.io/5dlabs/pm-lite",
    "ghcr.io/5dlabs/tool-server",
    "ghcr.io/5dlabs/intake",
];

/// Agent images
const AGENT_IMAGES: &[&str] = &[
    "ghcr.io/5dlabs/agent-morgan",
    "ghcr.io/5dlabs/agent-grizz",
    "ghcr.io/5dlabs/agent-nova",
    "ghcr.io/5dlabs/agent-blaze",
    "ghcr.io/5dlabs/agent-cleo",
    "ghcr.io/5dlabs/agent-cipher",
    "ghcr.io/5dlabs/agent-tess",
    "ghcr.io/5dlabs/agent-bolt",
];

/// Check for available updates
#[tauri::command]
pub async fn check_updates(db: State<'_, Database>) -> AppResult<UpdateStatus> {
    let mut images = Vec::new();
    let mut has_updates = false;

    // Check core images
    for image in CORE_IMAGES.iter().chain(AGENT_IMAGES.iter()) {
        let version = check_image_version(image).await?;
        if version.has_update {
            has_updates = true;
        }
        images.push(version);
    }

    // Save last checked time
    let now = chrono::Utc::now().to_rfc3339();
    db.set_config("last_update_check", &now)?;

    Ok(UpdateStatus {
        images,
        has_updates,
        last_checked: Some(now),
    })
}

/// Check a single image for updates
async fn check_image_version(image: &str) -> AppResult<ImageVersion> {
    // Get current local digest
    let current = get_local_digest(image);

    // Get latest remote digest
    let latest = get_remote_digest(image).await;

    let has_update = match (&current, &latest) {
        (Some(c), Some(l)) => c != l,
        (None, Some(_)) => true, // Not pulled yet
        _ => false,
    };

    Ok(ImageVersion {
        image: image.to_string(),
        current,
        latest,
        has_update,
    })
}

/// Get the digest of a locally pulled image
fn get_local_digest(image: &str) -> Option<String> {
    let output = Command::new("docker")
        .args([
            "inspect",
            "--format",
            "{{.Id}}",
            &format!("{}:latest", image),
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let digest = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !digest.is_empty() {
            return Some(digest);
        }
    }
    None
}

/// Get the digest of the latest remote image
async fn get_remote_digest(image: &str) -> Option<String> {
    // Use docker manifest inspect to get remote digest without pulling
    let output = tokio::process::Command::new("docker")
        .args([
            "manifest",
            "inspect",
            &format!("{}:latest", image),
            "--verbose",
        ])
        .output()
        .await
        .ok()?;

    if output.status.success() {
        // Parse the digest from manifest
        let manifest = String::from_utf8_lossy(&output.stdout);
        // Look for the digest in the manifest JSON
        if let Some(start) = manifest.find("\"digest\":") {
            let rest = &manifest[start + 10..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

/// Pull updates for all images
#[tauri::command]
pub async fn pull_updates(images: Option<Vec<String>>) -> AppResult<Vec<PullResult>> {
    let images_to_pull: Vec<&str> = match &images {
        Some(list) => list.iter().map(|s| s.as_str()).collect(),
        None => CORE_IMAGES
            .iter()
            .chain(AGENT_IMAGES.iter())
            .copied()
            .collect(),
    };

    let mut results = Vec::new();

    for image in images_to_pull {
        let result = pull_image(image).await;
        results.push(result);
    }

    Ok(results)
}

/// Result of pulling an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResult {
    pub image: String,
    pub success: bool,
    pub message: String,
}

/// Pull a single image
async fn pull_image(image: &str) -> PullResult {
    let image_tag = format!("{}:latest", image);

    let output = tokio::process::Command::new("docker")
        .args(["pull", &image_tag])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => PullResult {
            image: image.to_string(),
            success: true,
            message: "Updated successfully".to_string(),
        },
        Ok(o) => PullResult {
            image: image.to_string(),
            success: false,
            message: String::from_utf8_lossy(&o.stderr).to_string(),
        },
        Err(e) => PullResult {
            image: image.to_string(),
            success: false,
            message: e.to_string(),
        },
    }
}

/// Apply updates to the running cluster
#[tauri::command]
pub async fn apply_updates() -> AppResult<String> {
    // After pulling new images, we need to restart the deployments
    // to pick up the new versions

    let deployments = ["controller", "pm-lite", "tool-server", "intake"];

    let mut messages = Vec::new();

    for deployment in deployments {
        let output = tokio::process::Command::new("kubectl")
            .args([
                "rollout",
                "restart",
                "deployment",
                deployment,
                "-n",
                "cto-lite",
            ])
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => {
                messages.push(format!("✓ Restarted {}", deployment));
            }
            Ok(o) => {
                messages.push(format!(
                    "✗ Failed to restart {}: {}",
                    deployment,
                    String::from_utf8_lossy(&o.stderr)
                ));
            }
            Err(e) => {
                messages.push(format!("✗ Failed to restart {}: {}", deployment, e));
            }
        }
    }

    Ok(messages.join("\n"))
}

/// Get the current version of CTO components
#[tauri::command]
pub async fn get_component_versions() -> AppResult<Vec<ComponentVersion>> {
    let mut versions = Vec::new();

    // Get versions from running pods
    let output = tokio::process::Command::new("kubectl")
        .args([
            "get",
            "pods",
            "-n",
            "cto-lite",
            "-o",
            "jsonpath={range .items[*]}{.metadata.name}{\"\\t\"}{.spec.containers[0].image}{\"\\n\"}{end}",
        ])
        .output()
        .await;

    if let Ok(o) = output {
        if o.status.success() {
            let output_str = String::from_utf8_lossy(&o.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() == 2 {
                    versions.push(ComponentVersion {
                        name: parts[0].to_string(),
                        image: parts[1].to_string(),
                    });
                }
            }
        }
    }

    Ok(versions)
}

/// Version info for a running component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVersion {
    pub name: String,
    pub image: String,
}
