use crate::crds::{CodeRun, CodeRunCondition};
use crate::tasks::types::{Context, Result};
use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, Patch, PatchParams};
use kube::ResourceExt;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct CodeStatusManager;

#[allow(dead_code)]
impl CodeStatusManager {
    /// Monitor Job status and update `CodeRun` CRD accordingly
    pub async fn monitor_job_status(
        code_run: &Arc<CodeRun>,
        jobs: &Api<Job>,
        ctx: &Arc<Context>,
    ) -> Result<()> {
        let job_name = Self::get_current_job_name(code_run);

        if let Some(job_name) = job_name {
            // Get the current job
            match jobs.get(&job_name).await {
                Ok(job) => {
                    let (phase, message) = Self::analyze_job_status(&job);
                    Self::update_status(code_run, ctx, &phase, &message, None).await?;

                    // Schedule cleanup if job is complete and cleanup is enabled
                    if ctx.config.cleanup.enabled && (phase == "Succeeded" || phase == "Failed") {
                        Self::schedule_job_cleanup(code_run, ctx, &job_name, &phase).await?;
                    }
                }
                Err(kube::Error::Api(ae)) if ae.code == 404 => {
                    warn!(
                        "Job {} not found for CodeRun {}",
                        job_name,
                        code_run.name_any()
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to get job {} for CodeRun {}: {}",
                        job_name,
                        code_run.name_any(),
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Update the status when a job starts
    pub async fn update_job_started(
        code_run: &Arc<CodeRun>,
        ctx: &Arc<Context>,
        job_name: &str,
        _cm_name: &str,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = code_run.name_any();

        let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

        let current_retry_count = code_run
            .status
            .as_ref()
            .map_or(0, |s| s.retry_count.unwrap_or(0));

        let status_patch = json!({
            "status": {
                "phase": "Running",
                "message": "Code implementation job started",
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "jobName": job_name,
                "retryCount": current_retry_count,
                "conditions": Self::build_conditions("Running", "Code implementation job started", &chrono::Utc::now().to_rfc3339())
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match code_api.patch_status(&name, &pp, &patch).await {
            Ok(_) => {
                info!("Updated CodeRun status: {} -> Running", name);
            }
            Err(e) => {
                error!("Failed to update CodeRun status for {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Increment retry count for failed attempts
    #[allow(dead_code)]
    pub async fn increment_retry_count(code_run: &Arc<CodeRun>, ctx: &Arc<Context>) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = code_run.name_any();

        let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);
        let current_retry_count = code_run
            .status
            .as_ref()
            .map_or(0, |s| s.retry_count.unwrap_or(0));
        let new_retry_count = current_retry_count + 1;

        let status_patch = json!({
            "status": {
                "retryCount": new_retry_count,
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "message": format!("Retry attempt {} scheduled", new_retry_count)
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match code_api.patch_status(&name, &pp, &patch).await {
            Ok(_) => {
                info!(
                    "Updated CodeRun retry count: {} -> {}",
                    name, new_retry_count
                );
            }
            Err(e) => {
                error!("Failed to update CodeRun retry count for {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Update session info for retries
    #[allow(dead_code)]
    pub async fn update_session_info(
        code_run: &Arc<CodeRun>,
        ctx: &Arc<Context>,
        session_id: &str,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = code_run.name_any();

        let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

        let status_patch = json!({
            "status": {
                "sessionId": session_id,
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "message": format!("Session {} started", session_id)
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match code_api.patch_status(&name, &pp, &patch).await {
            Ok(_) => {
                info!("Updated CodeRun session info: {} -> {}", name, session_id);
            }
            Err(e) => {
                error!("Failed to update CodeRun session info for {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Update the `CodeRun` CRD status
    async fn update_status(
        code_run: &Arc<CodeRun>,
        ctx: &Arc<Context>,
        phase: &str,
        message: &str,
        retry_count_override: Option<u32>,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = code_run.name_any();

        let current_time = chrono::Utc::now().to_rfc3339();
        let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

        let current_retry_count = retry_count_override.unwrap_or_else(|| {
            code_run
                .status
                .as_ref()
                .map_or(0, |s| s.retry_count.unwrap_or(0))
        });
        let session_id = code_run
            .status
            .as_ref()
            .and_then(|s| s.session_id.as_deref());

        let mut status_patch = json!({
            "status": {
                "phase": phase,
                "message": message,
                "lastUpdate": current_time,
                "retryCount": current_retry_count,
                "conditions": Self::build_conditions(phase, message, &current_time)
            }
        });

        // Include session ID if present
        if let Some(sid) = session_id {
            status_patch["status"]["sessionId"] = json!(sid);
        }

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match code_api.patch_status(&name, &pp, &patch).await {
            Ok(updated_code_run) => {
                info!(
                    "✅ Successfully updated CodeRun status: {} -> {}",
                    name, phase
                );
                info!(
                    "✅ Updated resource version: {:?}",
                    updated_code_run.metadata.resource_version
                );
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to update CodeRun status for {}: {}", name, e);
                error!("❌ Error type: {}", std::any::type_name_of_val(&e));
                error!("❌ Full error details: {:?}", e);
                Err(e.into())
            }
        }
    }

    /// Get the current job name for a code task
    fn get_current_job_name(code_run: &CodeRun) -> Option<String> {
        code_run.status.as_ref().and_then(|s| s.job_name.clone())
    }

    /// Analyze job status and return (phase, message)
    fn analyze_job_status(job: &Job) -> (String, String) {
        if let Some(status) = &job.status {
            // Check completion time first
            if status.completion_time.is_some() {
                if let Some(conditions) = &status.conditions {
                    for condition in conditions {
                        if condition.type_ == "Complete" && condition.status == "True" {
                            return (
                                "Succeeded".to_string(),
                                "Code implementation completed successfully".to_string(),
                            );
                        } else if condition.type_ == "Failed" && condition.status == "True" {
                            let message = condition
                                .message
                                .as_deref()
                                .unwrap_or("Code implementation failed");
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
                        "Code implementation is running".to_string(),
                    );
                }
            }

            // Check for failure conditions
            if let Some(failed) = status.failed {
                if failed > 0 {
                    return (
                        "Failed".to_string(),
                        "Code implementation failed".to_string(),
                    );
                }
            }
        }

        (
            "Pending".to_string(),
            "Code implementation job pending".to_string(),
        )
    }

    /// Build `CodeRun` conditions
    fn build_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<CodeRunCondition> {
        vec![CodeRunCondition {
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
        code_run: &Arc<CodeRun>,
        ctx: &Arc<Context>,
        job_name: &str,
        phase: &str,
    ) -> Result<()> {
        info!(
            "Scheduling cleanup for CodeRun {} job {} (phase: {})",
            code_run.name_any(),
            job_name,
            phase
        );

        // For code jobs, we might want to keep them longer for debugging
        // or implement different cleanup policies based on success/failure
        let cleanup_delay_minutes = if phase == "Succeeded" {
            ctx.config.cleanup.completed_job_delay_minutes
        } else {
            ctx.config.cleanup.failed_job_delay_minutes
        };

        if cleanup_delay_minutes > 0 {
            info!(
                "Delaying cleanup for {} minutes for CodeRun job {}",
                cleanup_delay_minutes, job_name
            );
            // In a real implementation, you might schedule this with a timer or job queue
            // For now, just log the intent
        } else {
            // Clean up immediately
            let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

            if let Err(e) = jobs
                .delete(job_name, &kube::api::DeleteParams::default())
                .await
            {
                warn!("Failed to delete completed code job {}: {}", job_name, e);
            } else {
                info!("Successfully deleted completed code job: {}", job_name);
            }
        }

        Ok(())
    }
}
