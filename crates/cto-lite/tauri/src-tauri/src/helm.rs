//! Helm chart management for CTO Lite
//!
//! Handles deployment of the cto-lite Helm chart to the local Kind cluster.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Helm release status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
    pub revision: u32,
    pub status: String,
    pub chart: String,
    pub app_version: String,
}

/// Values to pass to helm install/upgrade
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelmValues {
    /// Anthropic API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anthropic_api_key: Option<String>,

    /// OpenAI API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openai_api_key: Option<String>,

    /// GitHub token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_token: Option<String>,

    /// Cloudflare tunnel token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloudflare_tunnel_token: Option<String>,

    /// Stack selection (grizz or nova)
    #[serde(default = "default_stack")]
    pub stack: String,
}

fn default_stack() -> String {
    "grizz".to_string()
}

/// Check if helm is installed
pub async fn check_helm() -> Result<Option<String>> {
    let output = Command::new("helm")
        .arg("version")
        .arg("--short")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            debug!("Helm version: {}", version);
            Ok(Some(version))
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            warn!("Helm check failed: {}", stderr);
            Ok(None)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("Helm not found");
            Ok(None)
        }
        Err(e) => Err(e).context("Failed to check helm"),
    }
}

/// Install or upgrade the CTO Lite helm chart
pub async fn deploy_chart(values: &HelmValues) -> Result<()> {
    // Check helm is available
    check_helm()
        .await?
        .ok_or_else(|| anyhow::anyhow!("Helm is not installed"))?;

    // Get the chart path (bundled with app)
    let chart_path = get_chart_path()?;

    // Build helm command
    let mut cmd = Command::new("helm");
    cmd.arg("upgrade")
        .arg("--install")
        .arg("cto-lite")
        .arg(&chart_path)
        .arg("--namespace")
        .arg("cto-lite")
        .arg("--create-namespace")
        .arg("--wait")
        .arg("--timeout")
        .arg("5m");

    // Add values as --set flags
    if let Some(ref key) = values.anthropic_api_key {
        cmd.arg("--set")
            .arg("secrets.anthropic.enabled=true");
        cmd.arg("--set")
            .arg(format!("secrets.anthropic.apiKey={}", key));
    }

    if let Some(ref key) = values.openai_api_key {
        cmd.arg("--set").arg("secrets.openai.enabled=true");
        cmd.arg("--set")
            .arg(format!("secrets.openai.apiKey={}", key));
    }

    if let Some(ref token) = values.github_token {
        cmd.arg("--set").arg("secrets.github.enabled=true");
        cmd.arg("--set")
            .arg(format!("secrets.github.token={}", token));
    }

    if let Some(ref token) = values.cloudflare_tunnel_token {
        cmd.arg("--set")
            .arg(format!("cloudflared.tunnelToken={}", token));
    }

    cmd.arg("--set").arg(format!("stack={}", values.stack));

    info!("Deploying CTO Lite chart...");

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to run helm upgrade")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Helm deploy failed: {}", stderr);
    }

    info!("CTO Lite deployed successfully");
    Ok(())
}

/// Get the status of the CTO Lite release
pub async fn get_release_status() -> Result<Option<HelmRelease>> {
    let output = Command::new("helm")
        .args(["status", "cto-lite", "-n", "cto-lite", "-o", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to check helm status")?;

    if !output.status.success() {
        // Release not found is not an error
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") {
            return Ok(None);
        }
        anyhow::bail!("Helm status failed: {}", stderr);
    }

    // Parse the JSON output
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse helm status output")?;

    let release = HelmRelease {
        name: json["name"].as_str().unwrap_or("cto-lite").to_string(),
        namespace: json["namespace"].as_str().unwrap_or("cto-lite").to_string(),
        revision: json["version"].as_u64().unwrap_or(1) as u32,
        status: json["info"]["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        chart: json["chart"]["metadata"]["name"]
            .as_str()
            .unwrap_or("cto-lite")
            .to_string(),
        app_version: json["chart"]["metadata"]["appVersion"]
            .as_str()
            .unwrap_or("0.1.0")
            .to_string(),
    };

    Ok(Some(release))
}

/// Uninstall the CTO Lite release
pub async fn uninstall_chart() -> Result<()> {
    let output = Command::new("helm")
        .args(["uninstall", "cto-lite", "-n", "cto-lite"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to run helm uninstall")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("not found") {
            anyhow::bail!("Helm uninstall failed: {}", stderr);
        }
    }

    info!("CTO Lite uninstalled");
    Ok(())
}

/// Get the path to the bundled helm chart
fn get_chart_path() -> Result<String> {
    // In development, use the relative path
    let dev_path = std::path::PathBuf::from("../../../infra/charts/cto-lite");
    if dev_path.exists() {
        return Ok(dev_path.to_string_lossy().to_string());
    }

    // In production, the chart is bundled with the app
    // For now, we'll need to handle this differently
    // Option 1: Download from GHCR
    // Option 2: Bundle as a resource
    // Option 3: Use embedded tarball

    // Fallback to OCI registry
    Ok("oci://ghcr.io/5dlabs/charts/cto-lite".to_string())
}

/// Update Argo Workflows dependency
pub async fn update_dependencies() -> Result<()> {
    let chart_path = get_chart_path()?;

    let output = Command::new("helm")
        .args(["dependency", "update", &chart_path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to update helm dependencies")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Helm dependency update failed: {}", stderr);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_helm() {
        // This test will pass if helm is installed, skip otherwise
        let result = check_helm().await;
        assert!(result.is_ok());
    }
}
