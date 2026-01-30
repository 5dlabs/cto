//! Path resolution for bundled resources and binaries
//!
//! Handles finding binaries, charts, and templates across:
//! - Development (use system PATH or build artifacts)
//! - Production (bundled in app bundle)

use std::path::PathBuf;
use crate::error::{AppError, AppResult};

/// Binary names we bundle
pub const KIND: &str = "kind";
pub const KUBECTL: &str = "kubectl";
pub const HELM: &str = "helm";
pub const CLOUDFLARED: &str = "cloudflared";
pub const MCP_LITE: &str = "mcp-lite";

/// Get the path to a bundled binary
///
/// Resolution order:
/// 1. Production: $APP_BUNDLE/Contents/Resources/bin/
/// 2. Development: Try system PATH
pub fn get_binary_path(name: &str) -> AppResult<PathBuf> {
    // Production: bundled in app
    #[cfg(not(debug_assertions))]
    {
        let path = get_resources_dir()?.join("bin").join(binary_name(name));
        if path.exists() {
            return Ok(path);
        }
    }

    // Development or fallback: use system PATH
    which::which(binary_name(name))
        .map_err(|_| AppError::RuntimeNotFound(format!("Binary not found: {name}")))
}

/// Get platform-specific binary name
fn binary_name(name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{name}.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        name.to_string()
    }
}

/// Get the app's resources directory
pub fn get_resources_dir() -> AppResult<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let exe = std::env::current_exe()
            .map_err(|e| AppError::IoError(e))?;
        // /path/to/CTO Lite.app/Contents/MacOS/cto-lite-tauri
        // -> /path/to/CTO Lite.app/Contents/Resources
        let resources = exe
            .parent() // MacOS/
            .and_then(|p| p.parent()) // Contents/
            .map(|p| p.join("Resources"));
        
        resources.ok_or_else(|| AppError::ConfigError("Could not determine resources path".into()))
    }

    #[cfg(target_os = "windows")]
    {
        let exe = std::env::current_exe()
            .map_err(|e| AppError::IoError(e))?;
        // /path/to/CTO Lite/cto-lite.exe -> /path/to/CTO Lite/resources
        let resources = exe.parent().map(|p| p.join("resources"));
        resources.ok_or_else(|| AppError::ConfigError("Could not determine resources path".into()))
    }

    #[cfg(target_os = "linux")]
    {
        let exe = std::env::current_exe()
            .map_err(|e| AppError::IoError(e))?;
        // AppImage or /opt install
        let resources = exe.parent().map(|p| p.join("resources"));
        resources.ok_or_else(|| AppError::ConfigError("Could not determine resources path".into()))
    }
}

/// Get the bundled Helm chart directory
pub fn get_chart_path() -> AppResult<PathBuf> {
    let chart = get_resources_dir()?.join("charts").join("cto-lite");
    
    if chart.exists() {
        return Ok(chart);
    }
    
    // Development fallback
    #[cfg(debug_assertions)]
    {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let dev_chart = manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("infra/charts/cto-lite");
        
        if dev_chart.exists() {
            return Ok(dev_chart);
        }
    }
    
    Err(AppError::ConfigError("Helm chart not found".into()))
}

/// Get the workflow template path
pub fn get_workflow_template_path() -> AppResult<PathBuf> {
    let template = get_resources_dir()?.join("templates").join("play-workflow-lite.yaml");
    
    if template.exists() {
        return Ok(template);
    }
    
    // Development fallback
    #[cfg(debug_assertions)]
    {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let dev_template = manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("templates/workflows/play-workflow-lite.yaml");
        
        if dev_template.exists() {
            return Ok(dev_template);
        }
    }
    
    Err(AppError::ConfigError("Workflow template not found".into()))
}

/// Get the user data directory
pub fn get_data_dir() -> AppResult<PathBuf> {
    // Check environment override first
    if let Ok(dir) = std::env::var("CTO_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }

    #[cfg(target_os = "macos")]
    {
        dirs::data_dir()
            .map(|p| p.join("ai.5dlabs.cto-lite"))
            .ok_or_else(|| AppError::ConfigError("Could not determine data directory".into()))
    }

    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .map(|p| p.join("ai.5dlabs").join("cto-lite"))
            .ok_or_else(|| AppError::ConfigError("Could not determine data directory".into()))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::data_dir()
            .map(|p| p.join("ai.5dlabs.cto-lite"))
            .ok_or_else(|| AppError::ConfigError("Could not determine data directory".into()))
    }
}

/// Get the kubeconfig path for the Kind cluster
pub fn get_kubeconfig_path() -> AppResult<PathBuf> {
    Ok(get_data_dir()?.join("kubeconfig").join("cto-lite-cluster.yaml"))
}

/// Get the logs directory
pub fn get_logs_dir() -> AppResult<PathBuf> {
    Ok(get_data_dir()?.join("logs"))
}

/// MCP configuration for IDE integration
#[derive(serde::Serialize)]
pub struct McpConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
}

/// Generate MCP config for IDE integration
pub fn get_mcp_config() -> AppResult<McpConfig> {
    let mcp_path = get_binary_path(MCP_LITE)?;
    let data_dir = get_data_dir()?;
    let kubeconfig = get_kubeconfig_path()?;
    
    let mut env = std::collections::HashMap::new();
    env.insert("CTO_DATA_DIR".to_string(), data_dir.to_string_lossy().to_string());
    env.insert("KUBECONFIG".to_string(), kubeconfig.to_string_lossy().to_string());
    env.insert("CTO_NAMESPACE".to_string(), "cto-lite".to_string());
    
    Ok(McpConfig {
        command: mcp_path.to_string_lossy().to_string(),
        args: vec![],
        env,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_name() {
        #[cfg(target_os = "windows")]
        assert_eq!(binary_name("kind"), "kind.exe");
        
        #[cfg(not(target_os = "windows"))]
        assert_eq!(binary_name("kind"), "kind");
    }

    #[test]
    fn test_data_dir() {
        let dir = get_data_dir().unwrap();
        assert!(dir.to_string_lossy().contains("cto-lite"));
    }
}
