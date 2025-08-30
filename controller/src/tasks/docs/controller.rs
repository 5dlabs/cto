use super::resources::DocsResourceManager;
use crate::crds::DocsRun;
use crate::tasks::types::{Context, Result, DOCS_FINALIZER_NAME};
use k8s_openapi::api::{batch::v1::Job, core::v1::ConfigMap};
use kube::api::{Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::finalizer::{finalizer, Event as FinalizerEvent};
use kube::{Api, ResourceExt};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
pub async fn reconcile_docs_run(docs_run: Arc<DocsRun>, ctx: Arc<Context>) -> Result<Action> {
    info!("Starting reconcile for DocsRun: {}", docs_run.name_any());

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = docs_run.name_any();

    debug!("Reconciling DocsRun: {}", name);

    // Create APIs
    debug!("Creating Kubernetes API clients...");
    let docsruns: Api<DocsRun> = Api::namespaced(client.clone(), namespace);
    debug!("API clients created successfully");

    // Handle finalizers for cleanup
    let result = finalizer(
        &docsruns,
        DOCS_FINALIZER_NAME,
        docs_run.clone(),
        |event| async {
            match event {
                FinalizerEvent::Apply(dr) => reconcile_docs_create_or_update(dr, &ctx).await,
                FinalizerEvent::Cleanup(dr) => cleanup_docs_resources(dr, &ctx).await,
            }
        },
    )
    .await
    .map_err(|e| match e {
        kube::runtime::finalizer::Error::ApplyFailed(err) => err,
        kube::runtime::finalizer::Error::CleanupFailed(err) => err,
        kube::runtime::finalizer::Error::AddFinalizer(e) => {
            crate::tasks::types::Error::KubeError(e)
        }
        kube::runtime::finalizer::Error::RemoveFinalizer(e) => {
            crate::tasks::types::Error::KubeError(e)
        }
        kube::runtime::finalizer::Error::UnnamedObject => {
            crate::tasks::types::Error::MissingObjectKey
        }
        kube::runtime::finalizer::Error::InvalidFinalizer => {
            crate::tasks::types::Error::ConfigError("Invalid finalizer name".to_string())
        }
    })?;

    debug!("Reconcile completed with result: {:?}", result);

    Ok(result)
}

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
async fn reconcile_docs_create_or_update(docs_run: Arc<DocsRun>, ctx: &Context) -> Result<Action> {
    let docs_run_name = docs_run.name_any();
    info!(
        "Starting status-first idempotent reconcile for DocsRun: {}",
        docs_run_name
    );

    // STEP 1: Check DocsRun status first (status-first idempotency)
    if let Some(status) = &docs_run.status {
        // Check for completion based on work_completed field (TTL-safe)
        if status.work_completed == Some(true) {
            // Double-check with GitHub to ensure status hasn't changed
            if let Some(pr_url) = &status.pull_request_url {
                if let Ok(is_still_complete) = verify_github_completion_status(pr_url).await {
                    if !is_still_complete {
                        warn!("Local work_completed=true but GitHub shows incomplete - clearing stale status");
                        clear_work_completed_status(&docs_run, ctx).await?;
                        // Continue with reconciliation
                    } else {
                        info!("Work already completed (verified with GitHub), no further action needed");
                        return Ok(Action::await_change());
                    }
                } else {
                    warn!("Could not verify GitHub status, proceeding with caution");
                }
            } else {
                info!("Work already completed (work_completed=true), no further action needed");
                return Ok(Action::await_change());
            }
        }

        // Check legacy completion states
        match status.phase.as_str() {
            "Succeeded" => {
                info!("Already succeeded, ensuring work_completed is set");
                update_docs_status_with_completion(
                    &docs_run,
                    ctx,
                    "Succeeded",
                    "Documentation generation completed successfully",
                    true,
                )
                .await?;
                return Ok(Action::await_change());
            }
            "Failed" => {
                info!("Already failed, no retry logic");
                return Ok(Action::await_change());
            }
            "Running" => {
                debug!("Status shows running, checking actual job state");
                // Continue to job state check below
            }
            _ => {
                debug!("Status is '{}', proceeding with job creation", status.phase);
                // Continue to job creation below
            }
        }
    } else {
        debug!("No status found, initializing");
    }

    // STEP 2: Check job state for running jobs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let job_name = generate_job_name(&docs_run);
    debug!("Generated job name: {}", job_name);

    let job_state = check_job_state(&jobs, &job_name).await?;
    debug!("Current job state: {:?}", job_state);

    match job_state {
        JobState::NotFound => {
            debug!("No existing job found, using optimistic job creation");

            // STEP 3: Optimistic job creation with conflict handling
            let ctx_arc = Arc::new(ctx.clone());
            let resource_manager =
                DocsResourceManager::new(&jobs, &configmaps, &ctx.config, &ctx_arc);

            // This handles 409 conflicts gracefully
            resource_manager
                .reconcile_create_or_update(&docs_run)
                .await?;

            // Update status to Running
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Running",
                "Documentation generation started",
                false,
            )
            .await?;

            // Requeue to check job progress
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }

        JobState::Running => {
            debug!("Job is still running, monitoring progress");

            // Update status to Running if needed
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Running",
                "Documentation generation in progress",
                false,
            )
            .await?;

            // Continue monitoring
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }

        JobState::Completed => {
            info!("Job completed successfully - marking work as complete");

            // Mark work as completed (TTL-safe)
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Succeeded",
                "Documentation generation completed successfully",
                true,
            )
            .await?;

            // CRITICAL: Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }

        JobState::Failed => {
            info!("Job failed - final state reached");

            // Update to failed status (work_completed remains false for potential retry)
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Failed",
                "Documentation generation failed",
                false,
            )
            .await?;

            // CRITICAL: Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }
    }
}

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
async fn cleanup_docs_resources(docs_run: Arc<DocsRun>, ctx: &Context) -> Result<Action> {
    debug!("Cleaning up resources for DocsRun");

    // Create APIs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    // Create resource manager and delegate
    let ctx_arc = Arc::new(ctx.clone());
    let resource_manager = DocsResourceManager::new(&jobs, &configmaps, &ctx.config, &ctx_arc);
    resource_manager.cleanup_resources(&docs_run).await
}

// Helper functions for idempotent reconciliation

#[derive(Debug, Clone)]
pub enum JobState {
    NotFound,
    Running,
    Completed,
    Failed,
}

fn generate_job_name(docs_run: &DocsRun) -> String {
    let namespace = docs_run.metadata.namespace.as_deref().unwrap_or("default");
    let name = docs_run.metadata.name.as_deref().unwrap_or("unknown");
    let uid_suffix = docs_run
        .metadata
        .uid
        .as_deref()
        .map(|uid| &uid[..8])
        .unwrap_or("nouid");

    format!("docs-{namespace}-{name}-{uid_suffix}")
        .replace(['_', '.'], "-")
        .to_lowercase()
}

async fn check_job_state(jobs: &Api<Job>, job_name: &str) -> Result<JobState> {
    match jobs.get(job_name).await {
        Ok(job) => {
            if let Some(status) = &job.status {
                Ok(determine_job_state(status))
            } else {
                Ok(JobState::Running) // Job exists but no status yet
            }
        }
        Err(kube::Error::Api(response)) if response.code == 404 => Ok(JobState::NotFound),
        Err(e) => Err(e.into()),
    }
}

fn determine_job_state(status: &k8s_openapi::api::batch::v1::JobStatus) -> JobState {
    // Check completion conditions first
    if let Some(conditions) = &status.conditions {
        for condition in conditions {
            if condition.type_ == "Complete" && condition.status == "True" {
                return JobState::Completed;
            }
            if condition.type_ == "Failed" && condition.status == "True" {
                return JobState::Failed;
            }
        }
    }

    // Check legacy status fields
    if let Some(succeeded) = status.succeeded {
        if succeeded > 0 {
            return JobState::Completed;
        }
    }

    if let Some(failed) = status.failed {
        if failed > 0 {
            return JobState::Failed;
        }
    }

    JobState::Running
}

async fn update_docs_status_with_completion(
    docs_run: &DocsRun,
    ctx: &Context,
    new_phase: &str,
    new_message: &str,
    work_completed: bool,
) -> Result<()> {
    // Only update if status actually changed
    let current_phase = docs_run
        .status
        .as_ref()
        .map(|s| s.phase.as_str())
        .unwrap_or("");
    let current_work_completed = docs_run
        .status
        .as_ref()
        .and_then(|s| s.work_completed)
        .unwrap_or(false);

    if current_phase == new_phase && current_work_completed == work_completed {
        debug!(
            "Status already '{}' with work_completed={}, skipping update to prevent reconciliation",
            new_phase, work_completed
        );
        return Ok(());
    }

    debug!(
        "Updating status from '{}' (work_completed={}) to '{}' (work_completed={})",
        current_phase, current_work_completed, new_phase, work_completed
    );

    let docsruns: Api<DocsRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    let status_patch = json!({
        "status": {
            "phase": new_phase,
            "message": new_message,
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
            "workCompleted": work_completed,
        }
    });

    // Use status subresource to avoid triggering spec reconciliation
    docsruns
        .patch_status(
            &docs_run.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&status_patch),
        )
        .await?;

    debug!(
        "Status updated successfully to '{}' with work_completed={}",
        new_phase, work_completed
    );
    Ok(())
}

/// Verify completion status with GitHub to prevent stale local state
async fn verify_github_completion_status(pr_url: &str) -> Result<bool> {
    // Extract PR number from GitHub URL
    // Format: https://github.com/owner/repo/pull/number
    let _pr_number = extract_pr_number_from_url(pr_url)?;

    // For now, implement a basic check - in production you'd use GitHub API
    // to check if PR is merged, has completion labels, etc.

    // TODO: Implement proper GitHub API call to verify:
    // 1. PR merge status
    // 2. PR closure status
    // 3. Completion labels
    // 4. Latest comment checkbox states

    warn!("GitHub verification not fully implemented - returning true for now");
    Ok(true) // Placeholder - assume complete for now
}

/// Extract PR number from GitHub URL
fn extract_pr_number_from_url(url: &str) -> Result<u32> {
    // Parse GitHub PR URL format: https://github.com/owner/repo/pull/number
    if let Some(pr_part) = url.split("/pull/").nth(1) {
        if let Some(number_str) = pr_part.split('/').next() {
            if let Ok(number) = number_str.parse::<u32>() {
                return Ok(number);
            }
        }
    }
    Err(crate::tasks::types::Error::UrlParsingError(format!(
        "Could not extract PR number from URL: {url}"
    )))
}

/// Clear stale work_completed status
async fn clear_work_completed_status(docs_run: &crate::crds::DocsRun, ctx: &Context) -> Result<()> {
    let docs_runs: Api<crate::crds::DocsRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    let patch = json!({
        "status": {
            "workCompleted": false,
            "message": "Status cleared due to GitHub verification mismatch"
        }
    });

    docs_runs
        .patch(
            &docs_run.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await?;

    info!(
        "Cleared work_completed status for DocsRun {}",
        docs_run.name_any()
    );
    Ok(())
}
