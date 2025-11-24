use crate::crds::{DocsRun, DocsRunCondition};
use crate::tasks::types::{Context, Result};
use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, Patch, PatchParams};
use kube::ResourceExt;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct DocsStatusManager;

#[allow(dead_code)]
impl DocsStatusManager {
    /// Monitor Job status and update `DocsRun` CRD accordingly
    pub async fn monitor_job_status(
        docs_run: &Arc<DocsRun>,
        jobs: &Api<Job>,
        ctx: &Arc<Context>,
    ) -> Result<()> {
        error!(
            "🔍 STATUS_MANAGER: Starting monitor_job_status for DocsRun: {}",
            docs_run.name_any()
        );
        let job_name = Self::get_current_job_name(docs_run);

        if let Some(job_name) = job_name {
            error!("✅ STATUS_MANAGER: Found job_name to monitor: {}", job_name);
            // Get the current job
            match jobs.get(&job_name).await {
                Ok(job) => {
                    let (phase, message) = Self::analyze_job_status(&job);
                    Self::update_status(docs_run, ctx, &phase, &message).await?;

                    // Schedule cleanup if job is complete and cleanup is enabled
                    if ctx.config.cleanup.enabled && (phase == "Succeeded" || phase == "Failed") {
                        Self::schedule_job_cleanup(docs_run, ctx, &job_name, &phase).await?;
                    }
                }
                Err(kube::Error::Api(ae)) if ae.code == 404 => {
                    warn!(
                        "Job {} not found for DocsRun {}",
                        job_name,
                        docs_run.name_any()
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to get job {} for DocsRun {}: {}",
                        job_name,
                        docs_run.name_any(),
                        e
                    );
                }
            }
        } else {
            error!("❌ STATUS_MANAGER: No job_name found in DocsRun status - cannot monitor job!");
            error!(
                "❌ STATUS_MANAGER: This means the initial status update failed or was overwritten"
            );
        }

        Ok(())
    }

    /// Update the status when a job starts
    pub async fn update_job_started(
        docs_run: &Arc<DocsRun>,
        ctx: &Arc<Context>,
        job_name: &str,
        _cm_name: &str,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = docs_run.name_any();

        let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

        let status_patch = json!({
            "status": {
                "phase": "Running",
                "message": "Documentation generation job started",
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "jobName": job_name,
                "conditions": Self::build_conditions("Running", "Documentation generation job started", &chrono::Utc::now().to_rfc3339())
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        error!(
            "🔄 STATUS_MANAGER: Attempting to update DocsRun status with job_name: {}",
            job_name
        );
        error!(
            "🔄 STATUS_MANAGER: Status patch: {}",
            serde_json::to_string_pretty(&status_patch)
                .unwrap_or_else(|e| format!("Failed to serialize patch: {e}"))
        );

        match docs_api.patch_status(&name, &pp, &patch).await {
            Ok(updated_docs_run) => {
                error!(
                    "✅ STATUS_MANAGER: Successfully updated DocsRun status: {} -> Running",
                    name
                );
                error!(
                    "✅ STATUS_MANAGER: Updated resource version: {:?}",
                    updated_docs_run.metadata.resource_version
                );
                error!(
                    "✅ STATUS_MANAGER: Updated job_name in status: {:?}",
                    updated_docs_run
                        .status
                        .as_ref()
                        .and_then(|s| s.job_name.as_ref())
                );
            }
            Err(e) => {
                error!(
                    "❌ STATUS_MANAGER: Failed to update DocsRun status for {}: {}",
                    name, e
                );
                error!(
                    "❌ STATUS_MANAGER: Error type: {}",
                    std::any::type_name_of_val(&e)
                );
                error!("❌ STATUS_MANAGER: Full error details: {:?}", e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// Update the `DocsRun` CRD status
    async fn update_status(
        docs_run: &Arc<DocsRun>,
        ctx: &Arc<Context>,
        phase: &str,
        message: &str,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = docs_run.name_any();

        let current_time = chrono::Utc::now().to_rfc3339();
        let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

        let status_patch = json!({
            "status": {
                "phase": phase,
                "message": message,
                "lastUpdate": current_time,
                "conditions": Self::build_conditions(phase, message, &current_time)
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match docs_api.patch_status(&name, &pp, &patch).await {
            Ok(updated_docs_run) => {
                info!(
                    "✅ Successfully updated DocsRun status: {} -> {}",
                    name, phase
                );
                info!(
                    "✅ Updated resource version: {:?}",
                    updated_docs_run.metadata.resource_version
                );
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to update DocsRun status for {}: {}", name, e);
                error!("❌ Error type: {}", std::any::type_name_of_val(&e));
                error!("❌ Full error details: {:?}", e);
                Err(e.into())
            }
        }
    }

    /// Get the current job name for a docs task
    fn get_current_job_name(docs_run: &DocsRun) -> Option<String> {
        let job_name = docs_run.status.as_ref().and_then(|s| s.job_name.clone());
        error!(
            "🔍 STATUS_MANAGER: get_current_job_name for {}: {:?}",
            docs_run.name_any(),
            job_name
        );
        error!("🔍 STATUS_MANAGER: DocsRun status: {:?}", docs_run.status);
        job_name
    }

    /// Analyze job status and return (phase, message)
    fn analyze_job_status(job: &Job) -> (String, String) {
        let job_name = job.metadata.name.as_deref().unwrap_or("unknown");
        error!(
            "🔍 STATUS_MANAGER: analyze_job_status for job: {}",
            job_name
        );

        if let Some(status) = &job.status {
            error!("📊 STATUS_MANAGER: Job status - active: {:?}, succeeded: {:?}, failed: {:?}, completion_time: {:?}",
                status.active, status.succeeded, status.failed, status.completion_time);
            // Check completion time first
            if status.completion_time.is_some() {
                if let Some(conditions) = &status.conditions {
                    for condition in conditions {
                        error!(
                            "🏷️ STATUS_MANAGER: Job condition - type: {}, status: {}",
                            condition.type_, condition.status
                        );
                        if condition.type_ == "Complete" && condition.status == "True" {
                            error!("🎉 STATUS_MANAGER: Job COMPLETED successfully! Setting phase to Succeeded");
                            return (
                                "Succeeded".to_string(),
                                "Documentation generation completed successfully".to_string(),
                            );
                        } else if condition.type_ == "Failed" && condition.status == "True" {
                            let message = condition
                                .message
                                .as_deref()
                                .unwrap_or("Documentation generation failed");
                            error!(
                                "💥 STATUS_MANAGER: Job FAILED! Setting phase to Failed: {}",
                                message
                            );
                            return ("Failed".to_string(), message.to_string());
                        }
                    }
                }
            }

            // Check if job is running
            if let Some(active) = status.active {
                if active > 0 {
                    return (
                        "Running".to_string(),
                        "Documentation generation is running".to_string(),
                    );
                }
            }

            // Check for failure conditions
            if let Some(failed) = status.failed {
                if failed > 0 {
                    return (
                        "Failed".to_string(),
                        "Documentation generation failed".to_string(),
                    );
                }
            }
        }

        (
            "Pending".to_string(),
            "Documentation generation job pending".to_string(),
        )
    }

    /// Build `DocsRun` conditions
    fn build_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<DocsRunCondition> {
        vec![DocsRunCondition {
            condition_type: phase.to_string(),
            status: "True".to_string(),
            last_transition_time: Some(timestamp.to_string()),
            reason: Some(match phase {
                "Running" => "JobStarted".to_string(),
                "Succeeded" => "JobCompleted".to_string(),
                "Failed" => "JobFailed".to_string(),
                _ => "Unknown".to_string(),
            }),
            message: Some(message.to_string()),
        }]
    }

    /// Schedule cleanup of completed job
    async fn schedule_job_cleanup(
        docs_run: &Arc<DocsRun>,
        ctx: &Arc<Context>,
        job_name: &str,
        phase: &str,
    ) -> Result<()> {
        info!(
            "Scheduling cleanup for DocsRun {} job {} (phase: {})",
            docs_run.name_any(),
            job_name,
            phase
        );

        // For docs jobs, we can clean up immediately since they don't need session persistence
        let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

        if let Err(e) = jobs
            .delete(job_name, &kube::api::DeleteParams::default())
            .await
        {
            warn!("Failed to delete completed docs job {}: {}", job_name, e);
        } else {
            info!("Successfully deleted completed docs job: {}", job_name);
        }

        Ok(())
    }
}
