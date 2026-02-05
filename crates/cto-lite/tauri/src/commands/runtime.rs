//! Runtime detection commands

use crate::error::AppError;
use crate::runtime::{self as rt, ContainerRuntime, RuntimeEnvironment, RuntimeStatus};
use serde::{Deserialize, Serialize};

/// Result of runtime detection
#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeDetectionResult {
    /// The detected runtime (if any)
    pub detected: Option<ContainerRuntime>,
    /// All available runtimes and their status
    pub available: Vec<RuntimeStatus>,
    /// Error message if no runtime is available
    pub error: Option<String>,
}

/// Docker installation and daemon status (matches UI DockerInfo interface)
#[derive(Debug, Serialize, Deserialize)]
pub struct DockerInfo {
    /// Whether Docker is installed
    pub installed: bool,
    /// Whether Docker daemon is running
    pub running: bool,
    /// Docker version if running
    pub version: Option<String>,
    /// Runtime type (for UI compatibility)
    #[serde(default)]
    pub runtime: String,
}

/// Scan the complete runtime environment
#[tauri::command]
pub async fn scan_runtime_environment() -> Result<RuntimeEnvironment, AppError> {
    Ok(rt::scan_runtime_environment())
}

/// Start a container runtime
#[tauri::command]
pub async fn start_container_runtime(runtime: ContainerRuntime) -> Result<(), AppError> {
    rt::start_runtime(runtime)
}

/// Detect the container runtime
#[tauri::command]
pub async fn detect_container_runtime() -> Result<RuntimeDetectionResult, AppError> {
    let available = rt::get_all_runtime_status();

    match rt::detect_running_runtime() {
        Ok(runtime) => Ok(RuntimeDetectionResult {
            detected: Some(runtime),
            available,
            error: None,
        }),
        Err(AppError::RuntimeNotRunning(msg)) => Ok(RuntimeDetectionResult {
            detected: None,
            available,
            error: Some(msg),
        }),
        Err(AppError::RuntimeNotFound(msg)) => Ok(RuntimeDetectionResult {
            detected: None,
            available,
            error: Some(msg),
        }),
        Err(e) => Err(e),
    }
}

/// Get status of all runtimes
#[tauri::command]
pub async fn get_runtime_status() -> Result<Vec<RuntimeStatus>, AppError> {
    Ok(rt::get_all_runtime_status())
}

/// Check if Docker daemon is running
#[tauri::command]
pub async fn check_docker_running() -> Result<bool, AppError> {
    Ok(rt::is_runtime_running(ContainerRuntime::Docker))
}

/// Check Docker installation and daemon status (legacy wrapper for UI compatibility)
#[tauri::command]
pub async fn check_docker() -> Result<DockerInfo, AppError> {
    let running = rt::is_runtime_running(ContainerRuntime::Docker);
    Ok(DockerInfo {
        installed: rt::is_docker_available(),
        running,
        version: if running {
            rt::get_runtime_version(ContainerRuntime::Docker)
        } else {
            None
        },
    })
}

/// Fully automated runtime detection and startup
/// Zero-touch: detects, starts if needed, and waits for ready
#[tauri::command]
pub async fn auto_detect_and_start_runtime() -> Result<RuntimeEnvironment, AppError> {
    rt::fully_auto_runtime()
}

/// Auto-detect and start container runtime (legacy, kept for compatibility)
#[tauri::command]
pub async fn auto_start_runtime() -> Result<Option<ContainerRuntime>, AppError> {
    rt::auto_start_runtime()
}
