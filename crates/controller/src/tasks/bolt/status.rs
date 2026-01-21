//! BOLT-004: Progress Reporting
//!
//! Updates BoltRun status with progress information:
//! - Current step number and name
//! - Step progress (e.g., 'Step 5/31: CreatingVLAN')
//! - Last update timestamp
//! - Error details if failed
//! - Logs URL for detailed output

use crate::crds::{BoltRun, BoltRunStatus};
use crate::tasks::types::{Error, Result};

use kube::api::{Api, Patch, PatchParams};
use serde_json::json;
use tracing::{debug, info};

/// Update the status of a BoltRun resource
///
/// BOLT-004: Progress Reporting
/// - Current step number and name in status
/// - Step progress (e.g., 'Step 5/31: CreatingVLAN')
/// - Last update timestamp
/// - Error details if failed
/// - Logs URL for detailed output
pub async fn update_bolt_status(
    api: &Api<BoltRun>,
    name: &str,
    status: BoltRunStatus,
) -> Result<()> {
    info!(
        "Updating BoltRun {} status: phase={:?}, step={:?}",
        name,
        status.phase,
        status.current_step.as_ref().map(|s| &s.name)
    );

    let status_patch = json!({
        "status": {
            "phase": format!("{:?}", status.phase),
            "message": status.message,
            "startTime": status.start_time,
            "completionTime": status.completion_time,
            "currentStep": status.current_step.map(|s| json!({
                "number": s.number,
                "total": s.total,
                "name": s.name,
                "startedAt": s.started_at,
                "completedAt": s.completed_at
            })),
            "lastUpdate": status.last_update,
            "error": status.error,
            "jobName": status.job_name,
            "podName": status.pod_name,
            "logsUrl": status.logs_url,
            "clusterName": status.cluster_name,
            "kubeconfigPath": status.kubeconfig_path,
            "retryCount": status.retry_count,
            "externalSecretName": status.external_secret_name
        }
    });

    api.patch_status(
        name,
        &PatchParams::apply("bolt-controller"),
        &Patch::Merge(&status_patch),
    )
    .await
    .map_err(Error::KubeError)?;

    debug!("BoltRun {} status updated successfully", name);
    Ok(())
}

/// Update just the current step of a BoltRun
///
/// Used by the installer binary to report progress during execution.
#[allow(dead_code)]
pub async fn update_bolt_step(
    api: &Api<BoltRun>,
    name: &str,
    step_number: u32,
    step_name: &str,
    total_steps: u32,
) -> Result<()> {
    info!(
        "Updating BoltRun {} step: {}/{} - {}",
        name, step_number, total_steps, step_name
    );

    let status_patch = json!({
        "status": {
            "currentStep": {
                "number": step_number,
                "total": total_steps,
                "name": step_name,
                "startedAt": chrono::Utc::now().to_rfc3339()
            },
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
            "message": format!("Step {}/{}: {}", step_number, total_steps, step_name)
        }
    });

    api.patch_status(
        name,
        &PatchParams::apply("bolt-controller"),
        &Patch::Merge(&status_patch),
    )
    .await
    .map_err(Error::KubeError)?;

    debug!("BoltRun {} step updated successfully", name);
    Ok(())
}

/// Mark a step as completed and start the next one
#[allow(dead_code)]
pub async fn complete_bolt_step(
    api: &Api<BoltRun>,
    name: &str,
    completed_step: u32,
    next_step_number: u32,
    next_step_name: &str,
    total_steps: u32,
) -> Result<()> {
    info!(
        "BoltRun {} completed step {}, starting step {}/{} - {}",
        name, completed_step, next_step_number, total_steps, next_step_name
    );

    let status_patch = json!({
        "status": {
            "currentStep": {
                "number": next_step_number,
                "total": total_steps,
                "name": next_step_name,
                "startedAt": chrono::Utc::now().to_rfc3339()
            },
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
            "message": format!("Step {}/{}: {}", next_step_number, total_steps, next_step_name)
        }
    });

    api.patch_status(
        name,
        &PatchParams::apply("bolt-controller"),
        &Patch::Merge(&status_patch),
    )
    .await
    .map_err(Error::KubeError)?;

    Ok(())
}

/// Report an error on a BoltRun
#[allow(dead_code)]
pub async fn report_bolt_error(
    api: &Api<BoltRun>,
    name: &str,
    error_message: &str,
    current_step: Option<(u32, &str)>,
) -> Result<()> {
    info!("BoltRun {} error: {}", name, error_message);

    let status_patch = if let Some((step_num, step_name)) = current_step {
        json!({
            "status": {
                "error": error_message,
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "message": format!("Error at step {}: {} - {}", step_num, step_name, error_message),
                "currentStep": {
                    "number": step_num,
                    "name": step_name
                }
            }
        })
    } else {
        json!({
            "status": {
                "error": error_message,
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "message": format!("Error: {}", error_message)
            }
        })
    };

    api.patch_status(
        name,
        &PatchParams::apply("bolt-controller"),
        &Patch::Merge(&status_patch),
    )
    .await
    .map_err(Error::KubeError)?;

    Ok(())
}
