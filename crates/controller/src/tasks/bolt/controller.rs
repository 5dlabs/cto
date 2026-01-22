//! BOLT-002: BoltRun Controller Reconciliation
//!
//! Watches BoltRun resources in cto-admin namespace and spawns Bolt agent pods.

use crate::crds::{BoltRun, BoltRunPhase, BoltRunStatus, BoltRunStep};
use crate::tasks::bolt::resources::{create_bolt_job, create_external_secret};
use crate::tasks::bolt::status::update_bolt_status;
use crate::tasks::types::{Context, Error, Result};

use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, DeleteParams, Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::ResourceExt;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, warn};

/// Finalizer name for BoltRun resources
pub const BOLT_FINALIZER_NAME: &str = "boltrun.cto.5dlabs.ai/finalizer";

/// BOLT-002: Main reconciliation function for BoltRun resources
///
/// Controller watches BoltRun resources in cto-admin namespace
/// On create: spawns Bolt agent pod with installer binary
/// Passes tenant config and credential ref to pod
/// Updates BoltRun status as pod progresses
/// Handles pod completion/failure appropriately
#[instrument(skip(ctx), fields(boltrun_name = %bolt_run.name_any(), namespace = %ctx.namespace))]
pub async fn reconcile_bolt_run(bolt_run: Arc<BoltRun>, ctx: Arc<Context>) -> Result<Action> {
    let name = bolt_run.name_any();
    let namespace = bolt_run
        .metadata
        .namespace
        .as_deref()
        .unwrap_or("cto-admin");

    info!("Reconciling BoltRun: {}/{}", namespace, name);

    let bolt_api: Api<BoltRun> = Api::namespaced(ctx.client.clone(), namespace);
    let jobs_api: Api<Job> = Api::namespaced(ctx.client.clone(), namespace);

    // Check if being deleted
    if bolt_run.metadata.deletion_timestamp.is_some() {
        return handle_deletion(&bolt_run, &bolt_api, &jobs_api, &ctx).await;
    }

    // Add finalizer if not present
    if !has_finalizer(&bolt_run) {
        add_finalizer(&bolt_api, &name).await?;
        return Ok(Action::requeue(Duration::from_secs(1)));
    }

    // Get current status
    let current_phase = bolt_run
        .status
        .as_ref()
        .map_or(&BoltRunPhase::Pending, |s| &s.phase);

    match current_phase {
        BoltRunPhase::Pending => handle_pending(&bolt_run, &bolt_api, &jobs_api, &ctx).await,
        BoltRunPhase::Initializing => {
            handle_initializing(&bolt_run, &bolt_api, &jobs_api, &ctx).await
        }
        BoltRunPhase::Running => handle_running(&bolt_run, &bolt_api, &jobs_api, &ctx).await,
        BoltRunPhase::Succeeded => handle_succeeded(&bolt_run).await,
        BoltRunPhase::Failed => handle_failed(&bolt_run).await,
        BoltRunPhase::Cancelled => handle_cancelled(&bolt_run).await,
    }
}

/// Handle BoltRun in Pending phase - start initialization
async fn handle_pending(
    bolt_run: &BoltRun,
    bolt_api: &Api<BoltRun>,
    _jobs_api: &Api<Job>,
    ctx: &Context,
) -> Result<Action> {
    let name = bolt_run.name_any();
    info!("BoltRun {} is Pending, starting initialization", name);

    // Update status to Initializing
    update_bolt_status(
        bolt_api,
        &name,
        BoltRunStatus {
            phase: BoltRunPhase::Initializing,
            message: Some("Creating credentials and job resources".to_string()),
            start_time: Some(chrono::Utc::now().to_rfc3339()),
            last_update: Some(chrono::Utc::now().to_rfc3339()),
            current_step: Some(BoltRunStep {
                number: 1,
                total: 31,
                name: "Initializing".to_string(),
                started_at: Some(chrono::Utc::now().to_rfc3339()),
                completed_at: None,
            }),
            ..Default::default()
        },
    )
    .await?;

    // BOLT-003: Create ExternalSecret for credential injection
    if let Some(ref provision) = bolt_run.spec.provision {
        let external_secret_name = bolt_run.external_secret_name();
        info!(
            "Creating ExternalSecret {} for credential path {}",
            external_secret_name, provision.credential_ref
        );

        create_external_secret(
            &ctx.client,
            bolt_run
                .metadata
                .namespace
                .as_deref()
                .unwrap_or("cto-admin"),
            &external_secret_name,
            &provision.credential_ref,
            &bolt_run.spec.tenant_ref,
        )
        .await?;

        // Update status with ExternalSecret name
        let status_patch = json!({
            "status": {
                "externalSecretName": external_secret_name
            }
        });
        bolt_api
            .patch_status(
                &name,
                &PatchParams::apply("bolt-controller"),
                &Patch::Merge(&status_patch),
            )
            .await
            .map_err(Error::KubeError)?;
    }

    Ok(Action::requeue(Duration::from_secs(2)))
}

/// Handle BoltRun in Initializing phase - create the Job
async fn handle_initializing(
    bolt_run: &BoltRun,
    bolt_api: &Api<BoltRun>,
    jobs_api: &Api<Job>,
    ctx: &Context,
) -> Result<Action> {
    let name = bolt_run.name_any();
    let job_name = bolt_run.job_name();

    info!(
        "BoltRun {} is Initializing, creating Job {}",
        name, job_name
    );

    // Check if Job already exists
    match jobs_api.get(&job_name).await {
        Ok(existing_job) => {
            debug!("Job {} already exists", job_name);
            // Update status to Running
            update_bolt_status(
                bolt_api,
                &name,
                BoltRunStatus {
                    phase: BoltRunPhase::Running,
                    message: Some("Bolt agent running".to_string()),
                    job_name: Some(job_name.clone()),
                    last_update: Some(chrono::Utc::now().to_rfc3339()),
                    current_step: Some(BoltRunStep {
                        number: 2,
                        total: 31,
                        name: "FetchingCredentials".to_string(),
                        started_at: Some(chrono::Utc::now().to_rfc3339()),
                        completed_at: None,
                    }),
                    ..Default::default()
                },
            )
            .await?;

            // Check if pod name is available
            if let Some(ref status) = existing_job.status {
                if let Some(active) = status.active {
                    if active > 0 {
                        debug!("Job {} has {} active pods", job_name, active);
                    }
                }
            }
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // Create the Job
            info!("Creating Job {} for BoltRun {}", job_name, name);
            create_bolt_job(jobs_api, bolt_run, ctx).await?;

            // Update status
            update_bolt_status(
                bolt_api,
                &name,
                BoltRunStatus {
                    phase: BoltRunPhase::Running,
                    message: Some("Job created, Bolt agent starting".to_string()),
                    job_name: Some(job_name.clone()),
                    last_update: Some(chrono::Utc::now().to_rfc3339()),
                    current_step: Some(BoltRunStep {
                        number: 2,
                        total: 31,
                        name: "FetchingCredentials".to_string(),
                        started_at: Some(chrono::Utc::now().to_rfc3339()),
                        completed_at: None,
                    }),
                    ..Default::default()
                },
            )
            .await?;
        }
        Err(e) => {
            error!("Failed to check Job status: {}", e);
            return Err(Error::KubeError(e));
        }
    }

    Ok(Action::requeue(Duration::from_secs(5)))
}

/// Handle BoltRun in Running phase - monitor Job progress
async fn handle_running(
    bolt_run: &BoltRun,
    bolt_api: &Api<BoltRun>,
    jobs_api: &Api<Job>,
    _ctx: &Context,
) -> Result<Action> {
    let name = bolt_run.name_any();
    let job_name = bolt_run.job_name();

    debug!("BoltRun {} is Running, checking Job status", name);

    // Check Job status
    match jobs_api.get(&job_name).await {
        Ok(job) => {
            if let Some(ref status) = job.status {
                // Check for completion
                if let Some(succeeded) = status.succeeded {
                    if succeeded > 0 {
                        info!("Job {} succeeded", job_name);
                        update_bolt_status(
                            bolt_api,
                            &name,
                            BoltRunStatus {
                                phase: BoltRunPhase::Succeeded,
                                message: Some("Provisioning completed successfully".to_string()),
                                completion_time: Some(chrono::Utc::now().to_rfc3339()),
                                last_update: Some(chrono::Utc::now().to_rfc3339()),
                                cluster_name: Some(bolt_run.cluster_name()),
                                current_step: Some(BoltRunStep {
                                    number: 31,
                                    total: 31,
                                    name: "Complete".to_string(),
                                    started_at: None,
                                    completed_at: Some(chrono::Utc::now().to_rfc3339()),
                                }),
                                ..Default::default()
                            },
                        )
                        .await?;
                        return Ok(Action::await_change());
                    }
                }

                // Check for failure
                if let Some(failed) = status.failed {
                    let retry_limit =
                        i32::try_from(bolt_run.spec.execution.retry_limit).unwrap_or(i32::MAX);
                    if failed >= retry_limit {
                        error!("Job {} failed after {} attempts", job_name, failed);
                        let retry_count = u32::try_from(failed).ok();
                        update_bolt_status(
                            bolt_api,
                            &name,
                            BoltRunStatus {
                                phase: BoltRunPhase::Failed,
                                message: Some(format!(
                                    "Provisioning failed after {failed} attempts"
                                )),
                                error: Some("Job reached retry limit".to_string()),
                                completion_time: Some(chrono::Utc::now().to_rfc3339()),
                                last_update: Some(chrono::Utc::now().to_rfc3339()),
                                retry_count,
                                ..Default::default()
                            },
                        )
                        .await?;
                        return Ok(Action::await_change());
                    }
                }

                // Still running, update progress
                debug!("Job {} still running", job_name);
            }
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            warn!("Job {} not found, recreating", job_name);
            // Job was deleted, go back to Initializing
            update_bolt_status(
                bolt_api,
                &name,
                BoltRunStatus {
                    phase: BoltRunPhase::Initializing,
                    message: Some("Job not found, recreating".to_string()),
                    last_update: Some(chrono::Utc::now().to_rfc3339()),
                    ..Default::default()
                },
            )
            .await?;
        }
        Err(e) => {
            error!("Failed to get Job status: {}", e);
        }
    }

    // Requeue to check status again
    Ok(Action::requeue(Duration::from_secs(30)))
}

/// Handle BoltRun in Succeeded phase - cleanup if needed
#[allow(clippy::unused_async)]
async fn handle_succeeded(bolt_run: &BoltRun) -> Result<Action> {
    let name = bolt_run.name_any();
    debug!("BoltRun {} succeeded, no further action needed", name);
    Ok(Action::await_change())
}

/// Handle BoltRun in Failed phase - no retry
#[allow(clippy::unused_async)]
async fn handle_failed(bolt_run: &BoltRun) -> Result<Action> {
    let name = bolt_run.name_any();
    debug!("BoltRun {} failed, no further action", name);
    Ok(Action::await_change())
}

/// Handle BoltRun in Cancelled phase
#[allow(clippy::unused_async)]
async fn handle_cancelled(bolt_run: &BoltRun) -> Result<Action> {
    let name = bolt_run.name_any();
    debug!("BoltRun {} cancelled, no further action", name);
    Ok(Action::await_change())
}

/// Handle deletion of BoltRun - cleanup resources
async fn handle_deletion(
    bolt_run: &BoltRun,
    bolt_api: &Api<BoltRun>,
    jobs_api: &Api<Job>,
    _ctx: &Context,
) -> Result<Action> {
    let name = bolt_run.name_any();
    info!("BoltRun {} is being deleted, cleaning up", name);

    // Delete the Job if it exists
    let job_name = bolt_run.job_name();
    if let Err(e) = jobs_api.delete(&job_name, &DeleteParams::default()).await {
        // Ignore 404 errors
        if !matches!(e, kube::Error::Api(ref ae) if ae.code == 404) {
            warn!("Failed to delete Job {}: {}", job_name, e);
        }
    }

    // Remove finalizer
    remove_finalizer(bolt_api, &name).await?;

    Ok(Action::await_change())
}

/// Check if the BoltRun has the finalizer
fn has_finalizer(bolt_run: &BoltRun) -> bool {
    bolt_run
        .metadata
        .finalizers
        .as_ref()
        .is_some_and(|f| f.contains(&BOLT_FINALIZER_NAME.to_string()))
}

/// Add finalizer to BoltRun
async fn add_finalizer(bolt_api: &Api<BoltRun>, name: &str) -> Result<()> {
    let patch = json!({
        "metadata": {
            "finalizers": [BOLT_FINALIZER_NAME]
        }
    });
    bolt_api
        .patch(
            name,
            &PatchParams::apply("bolt-controller"),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(Error::KubeError)?;
    Ok(())
}

/// Remove finalizer from BoltRun
async fn remove_finalizer(bolt_api: &Api<BoltRun>, name: &str) -> Result<()> {
    let patch = json!({
        "metadata": {
            "finalizers": []
        }
    });
    bolt_api
        .patch(
            name,
            &PatchParams::apply("bolt-controller"),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(Error::KubeError)?;
    Ok(())
}
