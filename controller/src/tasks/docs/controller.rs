use super::resources::DocsResourceManager;
use crate::crds::DocsRun;
use crate::tasks::cleanup;
use crate::tasks::types::{Context, Result, DOCS_FINALIZER_NAME};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use k8s_openapi::api::{batch::v1::Job, core::v1::ConfigMap};
use kube::api::{Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::finalizer::{finalizer, Event as FinalizerEvent};
use kube::{Api, ResourceExt};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

enum DocsExpireUpdate {
    Unchanged,
    Set(DateTime<Utc>),
    Clear,
}

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
pub async fn reconcile_docs_run(docs_run: Arc<DocsRun>, ctx: Arc<Context>) -> Result<Action> {
    info!("Starting reconcile for DocsRun: {}", docs_run.name_any());

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = docs_run.name_any();

    debug!("Reconciling DocsRun: {}", name);

    if let Some(action) = try_docs_cleanup_after_ttl(&docs_run, &ctx).await? {
        return Ok(action);
    }

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
                // Preserve existing finishedAt to avoid resetting TTL on every reconciliation
                let finished_at = status
                    .finished_at
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));
                let cleanup_deadline =
                    compute_docs_cleanup_deadline(&docs_run, ctx, "Succeeded", finished_at);

                update_docs_status_with_completion(
                    &docs_run,
                    ctx,
                    "Succeeded",
                    "Documentation generation completed successfully",
                    true,
                    Some(finished_at),
                    cleanup_deadline
                        .map(DocsExpireUpdate::Set)
                        .unwrap_or(DocsExpireUpdate::Unchanged),
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
    info!(
        "🔍 DocsRun {}: Generated job name: {}",
        docs_run_name, job_name
    );
    info!(
        "🔍 DocsRun {}: Current job state: {:?}",
        docs_run_name, job_state
    );

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
                None,
                DocsExpireUpdate::Clear,
            )
            .await?;

            // Requeue to check job progress
            // Using 90s instead of 30s to reduce reconciliation load
            Ok(Action::requeue(std::time::Duration::from_secs(90)))
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
                None,
                DocsExpireUpdate::Clear,
            )
            .await?;

            // Continue monitoring
            // Using 90s instead of 30s to reduce reconciliation load
            Ok(Action::requeue(std::time::Duration::from_secs(90)))
        }

        JobState::Completed => {
            info!("Job completed successfully - marking work as complete");

            // Mark work as completed (TTL-safe)
            let finished_at = Utc::now();
            let cleanup_deadline =
                compute_docs_cleanup_deadline(&docs_run, ctx, "Succeeded", finished_at);
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Succeeded",
                "Documentation generation completed successfully",
                true,
                Some(finished_at),
                cleanup_deadline
                    .map(DocsExpireUpdate::Set)
                    .unwrap_or(DocsExpireUpdate::Unchanged),
            )
            .await?;

            // CRITICAL: Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }

        JobState::Failed => {
            info!("Job failed - final state reached");

            // Update to failed status (work_completed remains false for potential retry)
            let finished_at = Utc::now();
            let cleanup_deadline =
                compute_docs_cleanup_deadline(&docs_run, ctx, "Failed", finished_at);
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Failed",
                "Documentation generation failed",
                false,
                Some(finished_at),
                cleanup_deadline
                    .map(DocsExpireUpdate::Set)
                    .unwrap_or(DocsExpireUpdate::Unchanged),
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
    finished_at: Option<DateTime<Utc>>,
    expire_update: DocsExpireUpdate,
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

    let mut status_patch = json!({
        "status": {
            "phase": new_phase,
            "message": new_message,
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
            "workCompleted": work_completed,
        }
    });

    if let Some(done_at) = finished_at {
        status_patch["status"]["finishedAt"] = json!(done_at.to_rfc3339());
    }

    match expire_update {
        DocsExpireUpdate::Set(deadline) => {
            status_patch["status"]["expireAt"] = json!(deadline.to_rfc3339());
        }
        DocsExpireUpdate::Clear => {
            status_patch["status"]["expireAt"] = serde_json::Value::Null;
        }
        DocsExpireUpdate::Unchanged => {}
    }

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

async fn try_docs_cleanup_after_ttl(
    docs_run: &Arc<DocsRun>,
    ctx: &Arc<Context>,
) -> Result<Option<Action>> {
    if !ctx.config.cleanup.enabled {
        return Ok(None);
    }

    if cleanup::is_preserved(&docs_run.metadata) {
        return Ok(None);
    }

    let status = match &docs_run.status {
        Some(status) => status,
        None => return Ok(None),
    };

    if status.cleanup_completed_at.is_some() {
        return Ok(None);
    }

    if !matches!(status.phase.as_str(), "Succeeded" | "Failed") {
        return Ok(None);
    }

    let expire_at = status
        .expire_at
        .as_deref()
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let Some(expire_at) = expire_at else {
        return Ok(None);
    };

    let now = Utc::now();
    if expire_at > now {
        let delay = (expire_at - now)
            .to_std()
            .unwrap_or_else(|_| std::time::Duration::from_secs(0));
        return Ok(Some(Action::requeue(delay)));
    }

    info!(
        docs_run = %docs_run.name_any(),
        "TTL expired for DocsRun - performing cleanup"
    );
    perform_docs_ttl_cleanup(docs_run, ctx).await?;
    mark_docs_cleanup_complete(docs_run, ctx).await?;
    Ok(Some(Action::await_change()))
}

async fn perform_docs_ttl_cleanup(docs_run: &Arc<DocsRun>, ctx: &Arc<Context>) -> Result<()> {
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let resource_manager = DocsResourceManager::new(&jobs, &configmaps, &ctx.config, ctx);
    let _ = resource_manager.cleanup_resources(docs_run).await?;
    Ok(())
}

async fn mark_docs_cleanup_complete(docs_run: &DocsRun, ctx: &Context) -> Result<()> {
    let docsruns: Api<DocsRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let patch = json!({
        "status": {
            "cleanupCompletedAt": Utc::now().to_rfc3339(),
            "expireAt": serde_json::Value::Null
        }
    });
    docsruns
        .patch_status(
            &docs_run.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await?;
    Ok(())
}

fn compute_docs_cleanup_deadline(
    docs_run: &DocsRun,
    ctx: &Context,
    phase: &str,
    finished_at: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    if !ctx.config.cleanup.enabled {
        return None;
    }

    if !matches!(phase, "Succeeded" | "Failed") {
        return None;
    }

    if cleanup::is_preserved(&docs_run.metadata) {
        return None;
    }

    if docs_run
        .status
        .as_ref()
        .and_then(|status| status.expire_at.as_ref())
        .is_some()
    {
        return None;
    }

    let ttl_seconds = cleanup::ttl_override_seconds(&docs_run.metadata).or_else(|| {
        if phase == "Succeeded" {
            Some(ctx.config.cleanup.success_ttl_seconds)
        } else {
            Some(ctx.config.cleanup.failure_ttl_seconds)
        }
    })?;

    if ttl_seconds == 0 {
        return None;
    }

    Some(finished_at + ChronoDuration::seconds(ttl_seconds as i64))
}
