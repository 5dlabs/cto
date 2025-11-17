use super::naming::ResourceNaming;
use super::resources::CodeResourceManager;
use crate::crds::CodeRun;
use crate::tasks::cleanup;
use crate::tasks::types::{Context, Result, CODE_FINALIZER_NAME};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::finalizer::{finalizer, Event as FinalizerEvent};
use kube::{Api, Error as KubeError, ResourceExt};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

enum ExpireAtUpdate {
    Unchanged,
    Set(DateTime<Utc>),
    Clear,
}

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
pub async fn reconcile_code_run(code_run: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action> {
    debug!("Starting reconcile for CodeRun: {}", code_run.name_any());

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = code_run.name_any();

    debug!("Reconciling CodeRun: {}", name);

    if let Some(action) = try_cleanup_after_ttl(&code_run, &ctx).await? {
        debug!("TTL cleanup handling for {} returned {:?}", name, action);
        return Ok(action);
    }

    // Create APIs
    debug!("Creating Kubernetes API clients...");
    let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);
    debug!("API clients created successfully");

    // Handle finalizers for cleanup
    let result = finalizer(
        &coderuns,
        CODE_FINALIZER_NAME,
        code_run.clone(),
        |event| async {
            match event {
                FinalizerEvent::Apply(cr) => reconcile_code_create_or_update(cr, &ctx).await,
                FinalizerEvent::Cleanup(cr) => cleanup_code_resources(cr, &ctx).await,
            }
        },
    )
    .await
    .map_err(|e| match e {
        kube::runtime::finalizer::Error::ApplyFailed(err)
        | kube::runtime::finalizer::Error::CleanupFailed(err) => err,
        kube::runtime::finalizer::Error::AddFinalizer(e)
        | kube::runtime::finalizer::Error::RemoveFinalizer(e) => {
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

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
#[allow(clippy::too_many_lines)]
async fn reconcile_code_create_or_update(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    let code_run_name = code_run.name_any();
    debug!(
        "Starting status-first idempotent reconcile for CodeRun: {}",
        code_run_name
    );

    // STEP 1: Check CodeRun status first (status-first idempotency)
    if let Some(status) = &code_run.status {
        // Check for completion based on work_completed field (TTL-safe)
        if status.work_completed == Some(true) {
            // Double-check with GitHub to ensure status hasn't changed
            if let Some(pr_url) = &status.pull_request_url {
                if verify_github_completion_status(pr_url) {
                    debug!(
                        "Work already completed (verified with GitHub), no further action needed"
                    );
                    return Ok(Action::await_change());
                }
                warn!(
                    "Local work_completed=true but GitHub shows incomplete - clearing stale status"
                );
                clear_work_completed_status(&code_run, ctx).await?;
                // Continue with reconciliation
            } else {
                debug!("Work already completed (work_completed=true), no further action needed");
                return Ok(Action::await_change());
            }
        }

        // Check legacy completion states
        match status.phase.as_str() {
            "Succeeded" => {
                debug!("Already succeeded, ensuring work_completed is set");
                // Preserve existing finishedAt to avoid resetting TTL on every reconciliation
                let finished_at = status
                    .finished_at
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));
                let cleanup_deadline =
                    compute_cleanup_deadline(&code_run, ctx, "Succeeded", finished_at);

                update_code_status_with_completion(
                    &code_run,
                    ctx,
                    "Succeeded",
                    "Code implementation completed successfully",
                    true,
                    None,
                    Some(finished_at),
                    cleanup_deadline.map_or(ExpireAtUpdate::Unchanged, ExpireAtUpdate::Set),
                )
                .await?;

                // Handle workflow resumption for already succeeded CodeRuns
                handle_workflow_resumption_on_completion(&code_run, ctx).await?;

                return Ok(Action::await_change());
            }
            "Failed" => {
                debug!("Already failed, no retry logic");
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
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let job_name = get_job_name(&code_run);
    debug!("Generated job name: {}", job_name);

    let job_state = check_code_job_state(&jobs, &job_name).await?;
    debug!("Current job state: {:?}", job_state);

    match job_state {
        CodeJobState::NotFound => {
            // Check if work was already completed or CodeRun already succeeded
            // (job might have been deleted after success by old controller)
            let work_completed = code_run
                .status
                .as_ref()
                .and_then(|s| s.work_completed)
                .unwrap_or(false);

            let current_phase = code_run.status.as_ref().map_or("", |s| s.phase.as_str());

            if work_completed || current_phase == "Succeeded" {
                debug!(
                    "Job not found but CodeRun {} already completed (phase={}, work_completed={}) - skipping new job creation",
                    code_run.name_any(),
                    current_phase,
                    work_completed
                );

                // Ensure work_completed flag is set if phase is Succeeded
                if current_phase == "Succeeded" && !work_completed {
                    debug!("Backfilling work_completed=true for succeeded CodeRun");
                    // Preserve existing finishedAt to avoid resetting TTL on every reconciliation
                    let finished_at = code_run
                        .status
                        .as_ref()
                        .and_then(|s| s.finished_at.as_ref())
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc));
                    let cleanup_deadline =
                        compute_cleanup_deadline(&code_run, ctx, "Succeeded", finished_at);

                    update_code_status_with_completion(
                        &code_run,
                        ctx,
                        "Succeeded",
                        "Code implementation completed successfully",
                        true,
                        None,
                        Some(finished_at),
                        cleanup_deadline.map_or(ExpireAtUpdate::Unchanged, ExpireAtUpdate::Set),
                    )
                    .await?;
                }

                return Ok(Action::await_change());
            }

            debug!("No existing job found, using optimistic job creation");

            // STEP 3: Optimistic job creation with conflict handling (copied from working docs controller)
            let ctx_arc = Arc::new(ctx.clone());
            let resource_manager =
                CodeResourceManager::new(&jobs, &configmaps, &pvcs, &ctx.config, &ctx_arc);

            // This handles 409 conflicts gracefully (same as docs controller)
            resource_manager
                .reconcile_create_or_update(&code_run)
                .await?;

            // Update status to Running (same pattern as docs)
            update_code_status_with_completion(
                &code_run,
                ctx,
                "Running",
                "Code implementation started",
                false,
                None,
                None,
                ExpireAtUpdate::Clear,
            )
            .await?;

            // Requeue to check job progress
            // Using 90s instead of 30s to reduce reconciliation load
            Ok(Action::requeue(std::time::Duration::from_secs(90)))
        }

        CodeJobState::Running => {
            debug!("Job is still running, monitoring progress");

            // Update status to Running with workCompleted=false
            update_code_status_with_completion(
                &code_run,
                ctx,
                "Running",
                "Code task in progress",
                false,
                None,
                None,
                ExpireAtUpdate::Clear,
            )
            .await?;

            // Continue monitoring
            // Using 90s instead of 30s to reduce reconciliation load
            Ok(Action::requeue(std::time::Duration::from_secs(90)))
        }

        CodeJobState::Completed => {
            debug!("Job completed - evaluating completion signals");

            let coderuns_api: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
            let latest_code_run = match coderuns_api.get(&code_run.name_any()).await {
                Ok(cr) => cr,
                Err(err) => {
                    warn!(
                        "Unable to fetch latest CodeRun {} after completion: {}. Falling back to cached object.",
                        code_run.name_any(),
                        err
                    );
                    code_run.as_ref().clone()
                }
            };

            let remediation_status = latest_code_run
                .status
                .as_ref()
                .and_then(|s| s.remediation_status.as_deref());

            if matches!(
                remediation_status,
                Some("needs-fixes" | "failed-remediation")
            ) {
                let detail = latest_code_run
                    .status
                    .as_ref()
                    .and_then(|s| s.message.as_deref())
                    .filter(|msg| !msg.is_empty())
                    .map(|msg| format!(": {msg}"))
                    .unwrap_or_default();

                let failure_message = format!(
                    "Implementation agent requested manual intervention ({}){}",
                    remediation_status.unwrap_or("needs-fixes"),
                    detail
                );

                warn!(
                    "{} for CodeRun {}",
                    failure_message,
                    latest_code_run.name_any()
                );

                let finished_at = Utc::now();
                let cleanup_deadline =
                    compute_cleanup_deadline(&latest_code_run, ctx, "Failed", finished_at);

                update_code_status_with_completion(
                    &latest_code_run,
                    ctx,
                    "Failed",
                    &failure_message,
                    false,
                    None,
                    Some(finished_at),
                    cleanup_deadline.map_or(ExpireAtUpdate::Unchanged, ExpireAtUpdate::Set),
                )
                .await?;

                handle_workflow_resumption_on_failure(&latest_code_run, ctx).await?;

                // Skip cleanup for workflow-managed jobs (let workflow handle it)
                let has_workflow_owner = latest_code_run
                    .metadata
                    .owner_references
                    .as_ref()
                    .and_then(|refs| refs.iter().find(|r| r.kind == "Workflow"))
                    .is_some();

                if has_workflow_owner {
                    info!(
                        "Skipping cleanup for workflow-managed job {} requiring manual intervention (workflow will manage lifecycle)",
                        job_name
                    );
                } else if ctx.config.cleanup.enabled {
                    let cleanup_delay_minutes = ctx.config.cleanup.failed_job_delay_minutes;
                    if cleanup_delay_minutes == 0 {
                        if let Err(err) = jobs.delete(&job_name, &DeleteParams::default()).await {
                            match err {
                                KubeError::Api(api_err) if api_err.code == 404 => {}
                                other => {
                                    warn!(
                                        "Failed to delete job {} after manual intervention required for CodeRun {}: {}",
                                        job_name,
                                        latest_code_run.name_any(),
                                        other
                                    );
                                }
                            }
                        } else {
                            info!(
                                "Deleted job {} after manual intervention required for CodeRun {}",
                                job_name,
                                latest_code_run.name_any()
                            );
                        }
                    } else {
                        info!(
                            "Delaying cleanup for {} minutes for CodeRun job {} requiring manual intervention",
                            cleanup_delay_minutes,
                            job_name
                        );
                    }
                }

                return Ok(Action::await_change());
            }

            let stage = get_workflow_stage(&latest_code_run);
            let retry_reason = determine_retry_reason(&latest_code_run, &stage);
            let max_retries = extract_max_retries(&latest_code_run);
            let current_retry_count = latest_code_run
                .status
                .as_ref()
                .and_then(|s| s.retry_count)
                .unwrap_or(0);

            if let Some(reason) = retry_reason {
                if max_retries == 0 || current_retry_count < max_retries {
                    let allowed_display = if max_retries == 0 {
                        "∞".to_string()
                    } else {
                        max_retries.to_string()
                    };

                    info!(
                        "CodeRun {} (stage {:?}) completed without success signal: {}. Scheduling retry attempt {} of {}.",
                        latest_code_run.name_any(),
                        stage,
                        reason,
                        current_retry_count + 1,
                        allowed_display
                    );

                    schedule_retry(
                        &latest_code_run,
                        ctx,
                        &jobs,
                        &job_name,
                        current_retry_count,
                        max_retries,
                        &reason,
                    )
                    .await?;

                    return Ok(Action::requeue(std::time::Duration::from_secs(10)));
                }

                warn!(
                    "Retry limit reached for CodeRun {} ({} attempts). Marking as failed: {}",
                    latest_code_run.name_any(),
                    current_retry_count,
                    reason
                );

                let finished_at = Utc::now();
                let cleanup_deadline =
                    compute_cleanup_deadline(&latest_code_run, ctx, "Failed", finished_at);

                update_code_status_with_completion(
                    &latest_code_run,
                    ctx,
                    "Failed",
                    &format!("Retry limit reached without completion: {reason}"),
                    false,
                    None,
                    Some(finished_at),
                    cleanup_deadline.map_or(ExpireAtUpdate::Unchanged, ExpireAtUpdate::Set),
                )
                .await?;

                handle_workflow_resumption_on_failure(&latest_code_run, ctx).await?;

                // Skip cleanup for workflow-managed jobs (let workflow handle it)
                let has_workflow_owner = latest_code_run
                    .metadata
                    .owner_references
                    .as_ref()
                    .and_then(|refs| refs.iter().find(|r| r.kind == "Workflow"))
                    .is_some();

                if has_workflow_owner {
                    info!(
                        "Skipping cleanup for workflow-managed failed job {} (workflow will manage lifecycle)",
                        job_name
                    );
                } else if ctx.config.cleanup.enabled {
                    let cleanup_delay_minutes = ctx.config.cleanup.failed_job_delay_minutes;
                    if cleanup_delay_minutes == 0 {
                        let _ = jobs.delete(&job_name, &DeleteParams::default()).await;
                        info!(
                            "Deleted job {} after exhausting retries for CodeRun {}",
                            job_name,
                            latest_code_run.name_any()
                        );
                    } else {
                        info!(
                            "Delaying cleanup for {} minutes for failed CodeRun job {}",
                            cleanup_delay_minutes, job_name
                        );
                    }
                }

                return Ok(Action::await_change());
            }

            debug!("Job completed successfully - validating before marking complete");

            // CRITICAL: Implementation agents MUST create a PR
            // Validate PR URL exists for implementation stages
            let stage = latest_code_run
                .metadata
                .labels
                .as_ref()
                .and_then(|labels| labels.get("stage"))
                .map(std::string::String::as_str);

            let is_implementation_stage = matches!(
                stage,
                Some("implementation" | "frontend") | None // None = legacy/default implementation
            );

            let pr_url = latest_code_run
                .status
                .as_ref()
                .and_then(|s| s.pull_request_url.as_ref());

            // Validate PR URL is present, non-empty, and not the "no-pr" placeholder
            let has_valid_pr = pr_url.is_some_and(|url| !url.is_empty() && url != "no-pr");

            if is_implementation_stage && !has_valid_pr {
                warn!(
                    "Implementation agent completed but DID NOT create PR - failing CodeRun {}",
                    code_run_name
                );

                let finished_at = Utc::now();
                let cleanup_deadline =
                    compute_cleanup_deadline(&latest_code_run, ctx, "Failed", finished_at);

                update_code_status_with_completion(
                    &latest_code_run,
                    ctx,
                    "Failed",
                    "Implementation agent must create a pull request (PR URL not found in status)",
                    false,
                    None,
                    Some(finished_at),
                    cleanup_deadline.map_or(ExpireAtUpdate::Unchanged, ExpireAtUpdate::Set),
                )
                .await?;

                handle_workflow_resumption_on_failure(&latest_code_run, ctx).await?;

                // Skip cleanup for workflow-managed jobs (let workflow handle it)
                let has_workflow_owner = latest_code_run
                    .metadata
                    .owner_references
                    .as_ref()
                    .and_then(|refs| refs.iter().find(|r| r.kind == "Workflow"))
                    .is_some();

                if has_workflow_owner {
                    info!(
                        "Skipping cleanup for workflow-managed job {} (PR validation failed, workflow will manage lifecycle)",
                        job_name
                    );
                } else if ctx.config.cleanup.enabled {
                    let cleanup_delay_minutes = ctx.config.cleanup.failed_job_delay_minutes;
                    if cleanup_delay_minutes == 0 {
                        if let Err(err) = jobs.delete(&job_name, &DeleteParams::default()).await {
                            match err {
                                KubeError::Api(api_err) if api_err.code == 404 => {}
                                other => {
                                    warn!(
                                        "Failed to delete job {} after PR validation failure for CodeRun {}: {}",
                                        job_name,
                                        latest_code_run.name_any(),
                                        other
                                    );
                                }
                            }
                        }
                    }
                }

                return Ok(Action::await_change());
            }

            debug!("Validation passed - marking work as completed");

            let finished_at = Utc::now();
            let cleanup_deadline =
                compute_cleanup_deadline(&latest_code_run, ctx, "Succeeded", finished_at);

            update_code_status_with_completion(
                &latest_code_run,
                ctx,
                "Succeeded",
                "Code implementation completed successfully",
                true,
                None,
                Some(finished_at),
                cleanup_deadline.map_or(ExpireAtUpdate::Unchanged, ExpireAtUpdate::Set),
            )
            .await?;

            handle_workflow_resumption_on_completion(&latest_code_run, ctx).await?;

            // Skip cleanup for workflow-managed jobs (let workflow handle it)
            let has_workflow_owner = latest_code_run
                .metadata
                .owner_references
                .as_ref()
                .and_then(|refs| refs.iter().find(|r| r.kind == "Workflow"))
                .is_some();

            if has_workflow_owner {
                info!(
                    "Skipping cleanup for workflow-managed job {} (workflow will manage lifecycle)",
                    job_name
                );
            } else if ctx.config.cleanup.enabled {
                let cleanup_delay_minutes = ctx.config.cleanup.completed_job_delay_minutes;
                if cleanup_delay_minutes == 0 {
                    let _ = jobs.delete(&job_name, &DeleteParams::default()).await;
                    info!("Deleted completed code job: {}", job_name);
                } else {
                    info!(
                        "Delaying cleanup for {} minutes for CodeRun job {}",
                        cleanup_delay_minutes, job_name
                    );
                }
            }

            Ok(Action::await_change())
        }

        CodeJobState::Failed => {
            debug!("Job failed - evaluating retry policy");

            let coderuns_api: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
            let latest_code_run = match coderuns_api.get(&code_run.name_any()).await {
                Ok(cr) => cr,
                Err(err) => {
                    warn!(
                        "Unable to fetch latest CodeRun {} after failure: {}. Falling back to cached object.",
                        code_run.name_any(),
                        err
                    );
                    code_run.as_ref().clone()
                }
            };

            let max_retries = extract_max_retries(&latest_code_run);
            let current_retry_count = latest_code_run
                .status
                .as_ref()
                .and_then(|s| s.retry_count)
                .unwrap_or(0);

            if max_retries == 0 || current_retry_count < max_retries {
                let allowed_display = if max_retries == 0 {
                    "∞".to_string()
                } else {
                    max_retries.to_string()
                };

                info!(
                    "Scheduling retry for failed CodeRun {} (attempt {} of {}).",
                    latest_code_run.name_any(),
                    current_retry_count + 1,
                    allowed_display
                );

                schedule_retry(
                    &latest_code_run,
                    ctx,
                    &jobs,
                    &job_name,
                    current_retry_count,
                    max_retries,
                    "Job failed",
                )
                .await?;

                return Ok(Action::requeue(std::time::Duration::from_secs(10)));
            }

            info!(
                "Retry limit reached for failed CodeRun {} ({} attempts). Marking as failed.",
                latest_code_run.name_any(),
                current_retry_count
            );

            let finished_at = Utc::now();
            let cleanup_deadline =
                compute_cleanup_deadline(&latest_code_run, ctx, "Failed", finished_at);

            update_code_status_with_completion(
                &latest_code_run,
                ctx,
                "Failed",
                "Code implementation failed",
                false,
                None,
                Some(finished_at),
                cleanup_deadline.map_or(ExpireAtUpdate::Unchanged, ExpireAtUpdate::Set),
            )
            .await?;

            handle_workflow_resumption_on_failure(&latest_code_run, ctx).await?;

            if ctx.config.cleanup.enabled {
                let cleanup_delay_minutes = ctx.config.cleanup.failed_job_delay_minutes;
                if cleanup_delay_minutes == 0 {
                    let _ = jobs.delete(&job_name, &DeleteParams::default()).await;
                    info!("Deleted failed code job: {}", job_name);
                } else {
                    info!(
                        "Delaying failed-job cleanup for {} minutes for CodeRun job {}",
                        cleanup_delay_minutes, job_name
                    );
                }
            }

            Ok(Action::await_change())
        }
    }
}

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
async fn cleanup_code_resources(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    debug!("Cleaning up resources for CodeRun");

    // Create APIs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    // Create resource manager and delegate
    let ctx_arc = Arc::new(ctx.clone());
    let resource_manager =
        CodeResourceManager::new(&jobs, &configmaps, &pvcs, &ctx.config, &ctx_arc);
    resource_manager.cleanup_resources(&code_run).await
}

// Helper functions for idempotent reconciliation - CodeRun version

#[derive(Debug, Clone)]
pub enum CodeJobState {
    NotFound,
    Running,
    Completed,
    Failed,
}

/// Get job name for `CodeRun` - prefer stored name, fallback to generation
/// This fixes the job name mismatch that was causing status update failures
fn get_job_name(code_run: &CodeRun) -> String {
    // First try to get the job name from CodeRun status (set during creation)
    if let Some(status) = &code_run.status {
        if let Some(job_name) = &status.job_name {
            debug!("Using stored job name from status: {}", job_name);
            return job_name.clone();
        }
    }

    // Fallback to unified generation
    let generated_name = ResourceNaming::job_name(code_run);
    debug!("Generated job name: {}", generated_name);
    generated_name
}

async fn check_code_job_state(jobs: &Api<Job>, job_name: &str) -> Result<CodeJobState> {
    match jobs.get(job_name).await {
        Ok(job) => {
            if let Some(status) = &job.status {
                Ok(determine_code_job_state(status))
            } else {
                Ok(CodeJobState::Running) // Job exists but no status yet
            }
        }
        Err(kube::Error::Api(response)) if response.code == 404 => Ok(CodeJobState::NotFound),
        Err(e) => Err(e.into()),
    }
}

fn determine_code_job_state(status: &k8s_openapi::api::batch::v1::JobStatus) -> CodeJobState {
    // Check completion conditions first
    if let Some(conditions) = &status.conditions {
        for condition in conditions {
            if condition.type_ == "Complete" && condition.status == "True" {
                return CodeJobState::Completed;
            }
            if condition.type_ == "Failed" && condition.status == "True" {
                return CodeJobState::Failed;
            }
        }
    }

    // Check legacy status fields
    if let Some(succeeded) = status.succeeded {
        if succeeded > 0 {
            return CodeJobState::Completed;
        }
    }

    if let Some(failed) = status.failed {
        if failed > 0 {
            return CodeJobState::Failed;
        }
    }

    CodeJobState::Running
}

#[allow(clippy::too_many_arguments)]
async fn update_code_status_with_completion(
    code_run: &CodeRun,
    ctx: &Context,
    new_phase: &str,
    new_message: &str,
    work_completed: bool,
    retry_count_override: Option<u32>,
    finished_at: Option<DateTime<Utc>>,
    expire_update: ExpireAtUpdate,
) -> Result<()> {
    // Only update if status actually changed or work_completed changed
    let current_phase = code_run.status.as_ref().map_or("", |s| s.phase.as_str());
    let current_work_completed = code_run
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

    let coderuns: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    let retry_count = retry_count_override.unwrap_or_else(|| {
        code_run
            .status
            .as_ref()
            .and_then(|s| s.retry_count)
            .unwrap_or(0)
    });

    let mut status_patch = json!({
        "status": {
            "phase": new_phase,
            "message": new_message,
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
            "workCompleted": work_completed,
            "retryCount": retry_count,
        }
    });

    if let Some(done_at) = finished_at {
        status_patch["status"]["finishedAt"] = json!(done_at.to_rfc3339());
    }

    match expire_update {
        ExpireAtUpdate::Set(deadline) => {
            status_patch["status"]["expireAt"] = json!(deadline.to_rfc3339());
        }
        ExpireAtUpdate::Clear => {
            status_patch["status"]["expireAt"] = serde_json::Value::Null;
        }
        ExpireAtUpdate::Unchanged => {}
    }

    // Use status subresource to avoid triggering spec reconciliation
    coderuns
        .patch_status(
            &code_run.name_any(),
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

/// Handle workflow resumption when `CodeRun` completes successfully
async fn handle_workflow_resumption_on_completion(code_run: &CodeRun, ctx: &Context) -> Result<()> {
    use crate::tasks::workflow::{
        extract_pr_number, extract_workflow_name, resume_workflow_for_pr,
    };

    let workflow_name = match extract_workflow_name(code_run) {
        Ok(name) => name,
        Err(e) => {
            warn!("Could not extract workflow name: {}", e);
            return Ok(()); // Not an error - CodeRun might not be part of a workflow
        }
    };

    let remediation_status = code_run
        .status
        .as_ref()
        .and_then(|s| s.remediation_status.as_deref());

    let qa_status = code_run
        .status
        .as_ref()
        .and_then(|s| s.qa_status.as_deref());

    // Check if PR URL is already available
    if let Some(status) = &code_run.status {
        if let Some(pr_url) = &status.pull_request_url {
            if !pr_url.is_empty() && pr_url != "no-pr" {
                info!("PR URL found in CodeRun status: {}", pr_url);
                let pr_number = match extract_pr_number(pr_url) {
                    Ok(num) => num,
                    Err(e) => {
                        warn!("Failed to extract PR number from {}: {}", pr_url, e);
                        return Ok(()); // Not a critical error
                    }
                };
                if let Err(e) = resume_workflow_for_pr(
                    &ctx.client,
                    &ctx.namespace,
                    &workflow_name,
                    pr_url,
                    pr_number,
                    remediation_status,
                    qa_status,
                )
                .await
                {
                    warn!("Failed to resume workflow: {}", e);
                }
                return Ok(());
            }
        }
    }

    // No PR URL found - start timeout handler
    info!("No PR URL found in CodeRun status, starting timeout handler");
    handle_no_pr_timeout(&workflow_name, code_run, ctx).await
}

/// Handle workflow resumption when `CodeRun` fails
async fn handle_workflow_resumption_on_failure(code_run: &CodeRun, ctx: &Context) -> Result<()> {
    use crate::tasks::workflow::{extract_workflow_name, resume_workflow_for_failure};

    let workflow_name = match extract_workflow_name(code_run) {
        Ok(name) => name,
        Err(e) => {
            warn!("Could not extract workflow name: {}", e);
            return Ok(()); // Not an error - CodeRun might not be part of a workflow
        }
    };

    let error_message = code_run
        .status
        .as_ref()
        .and_then(|s| s.message.as_deref())
        .unwrap_or("Code implementation failed");

    if let Err(e) =
        resume_workflow_for_failure(&ctx.client, &ctx.namespace, &workflow_name, error_message)
            .await
    {
        warn!("Failed to resume workflow for failure: {}", e);
    }
    Ok(())
}

/// Handle timeout when no PR is created
#[allow(clippy::too_many_lines)]
async fn handle_no_pr_timeout(
    workflow_name: &str,
    code_run: &CodeRun,
    ctx: &Context,
) -> Result<()> {
    use crate::tasks::github::{check_github_for_pr_by_branch, update_code_run_pr_url};
    use crate::tasks::workflow::resume_workflow_for_no_pr;
    use tokio::time::{sleep, Duration};

    // TODO: Make timeout configurable
    let timeout_seconds = 60;

    info!(
        "Starting no-PR timeout handler ({}s) for workflow: {}",
        timeout_seconds, workflow_name
    );

    // Strategy 1: Wait and check CodeRun again (maybe PR creation was delayed)
    sleep(Duration::from_secs(30)).await;

    // Re-fetch CodeRun to check for updates
    let coderuns: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let updated_code_run = match coderuns.get(&code_run.name_any()).await {
        Ok(cr) => cr,
        Err(e) => {
            warn!("Failed to re-fetch CodeRun: {}", e);
            code_run.clone() // Use original if refetch fails
        }
    };

    // Check if PR URL appeared
    if let Some(status) = &updated_code_run.status {
        if let Some(pr_url) = &status.pull_request_url {
            if !pr_url.is_empty() && pr_url != "no-pr" {
                info!("PR URL found after delay: {}", pr_url);
                let pr_number = match crate::tasks::workflow::extract_pr_number(pr_url) {
                    Ok(num) => num,
                    Err(e) => {
                        warn!("Failed to extract PR number: {}", e);
                        return Ok(());
                    }
                };
                let remediation_status = updated_code_run
                    .status
                    .as_ref()
                    .and_then(|s| s.remediation_status.as_deref());
                let qa_status = updated_code_run
                    .status
                    .as_ref()
                    .and_then(|s| s.qa_status.as_deref());
                if let Err(e) = crate::tasks::workflow::resume_workflow_for_pr(
                    &ctx.client,
                    &ctx.namespace,
                    workflow_name,
                    pr_url,
                    pr_number,
                    remediation_status,
                    qa_status,
                )
                .await
                {
                    warn!("Failed to resume workflow: {}", e);
                }
                return Ok(());
            }
        }
    }

    // Strategy 2: Check GitHub directly for PR by branch name
    // TODO: Get GitHub token from configuration
    if let Ok(Some(pr_url)) = check_github_for_pr_by_branch(&updated_code_run, None).await {
        info!("Found PR via GitHub API: {}", pr_url);

        // Update CodeRun with found PR URL
        if let Err(e) =
            update_code_run_pr_url(&ctx.client, &ctx.namespace, &code_run.name_any(), &pr_url).await
        {
            warn!("Failed to update CodeRun with PR URL: {}", e);
        }

        let pr_number = match crate::tasks::workflow::extract_pr_number(&pr_url) {
            Ok(num) => num,
            Err(e) => {
                warn!("Failed to extract PR number from GitHub API result: {}", e);
                return Ok(());
            }
        };
        let remediation_status = updated_code_run
            .status
            .as_ref()
            .and_then(|s| s.remediation_status.as_deref());
        let qa_status = updated_code_run
            .status
            .as_ref()
            .and_then(|s| s.qa_status.as_deref());
        if let Err(e) = crate::tasks::workflow::resume_workflow_for_pr(
            &ctx.client,
            &ctx.namespace,
            workflow_name,
            &pr_url,
            pr_number,
            remediation_status,
            qa_status,
        )
        .await
        {
            warn!("Failed to resume workflow from GitHub API result: {}", e);
        }
        return Ok(());
    }

    // Strategy 3: Resume workflow with "no PR" status
    info!("No PR found after timeout, resuming workflow with no-pr status");
    let coderun_status = updated_code_run
        .status
        .as_ref()
        .map_or("Succeeded", |s| s.phase.as_str());

    if let Err(e) =
        resume_workflow_for_no_pr(&ctx.client, &ctx.namespace, workflow_name, coderun_status).await
    {
        warn!("Failed to resume workflow with no-PR status: {}", e);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WorkflowStage {
    Implementation,
    Quality,
    Testing,
    Unknown(String),
}

fn get_workflow_stage(code_run: &CodeRun) -> WorkflowStage {
    if let Some(labels) = &code_run.metadata.labels {
        if let Some(stage) = labels.get("workflow-stage") {
            return match stage.as_str() {
                "implementation" => WorkflowStage::Implementation,
                "quality" => WorkflowStage::Quality,
                "testing" => WorkflowStage::Testing,
                other => WorkflowStage::Unknown(other.to_string()),
            };
        }
    }

    WorkflowStage::Unknown("unspecified".to_string())
}

fn extract_max_retries(code_run: &CodeRun) -> u32 {
    const RETRY_KEYS: [&str; 6] = [
        "EXECUTION_MAX_RETRIES",
        "FACTORY_MAX_RETRIES",
        "CODEX_MAX_RETRIES",
        "CURSOR_MAX_RETRIES",
        "CLAUDE_MAX_RETRIES",
        "OPENCODE_MAX_RETRIES",
    ];

    for key in &RETRY_KEYS {
        if let Some(value) = code_run.spec.env.get(*key) {
            if let Ok(parsed) = value.trim().parse::<u32>() {
                return parsed;
            }
        }
    }

    1
}

fn determine_retry_reason(code_run: &CodeRun, stage: &WorkflowStage) -> Option<String> {
    let status = code_run.status.as_ref()?;

    match stage {
        WorkflowStage::Implementation => {
            if matches!(
                status.remediation_status.as_deref(),
                Some("needs-fixes" | "failed-remediation")
            ) {
                return Some("Implementation agent requested fixes".to_string());
            }

            let has_pr = status.pull_request_url.as_ref().is_some_and(|url| {
                let trimmed = url.trim();
                !trimmed.is_empty() && trimmed != "no-pr"
            });

            if !has_pr {
                return Some("Implementation attempt did not produce a pull request".to_string());
            }

            None
        }
        WorkflowStage::Quality => {
            if matches!(status.qa_status.as_deref(), Some("changes_requested")) {
                return Some("Quality review requested changes".to_string());
            }

            if matches!(status.remediation_status.as_deref(), Some("needs-fixes")) {
                return Some("Quality workflow reported remediation needed".to_string());
            }

            None
        }
        WorkflowStage::Testing => {
            if matches!(status.qa_status.as_deref(), Some("changes_requested")) {
                return Some("Testing agent requested changes".to_string());
            }

            if matches!(
                status.remediation_status.as_deref(),
                Some("needs-fixes" | "failed-remediation")
            ) {
                return Some("Testing workflow reported remediation needed".to_string());
            }

            None
        }
        WorkflowStage::Unknown(_) => None,
    }
}

async fn schedule_retry(
    code_run: &CodeRun,
    ctx: &Context,
    jobs: &Api<Job>,
    job_name: &str,
    current_retry_count: u32,
    max_retries: u32,
    reason: &str,
) -> Result<()> {
    if let Err(err) = jobs.delete(job_name, &DeleteParams::default()).await {
        match err {
            KubeError::Api(api_err) if api_err.code == 404 => {}
            other => {
                warn!(
                    "Failed to delete job {} before scheduling retry for {}: {}",
                    job_name,
                    code_run.name_any(),
                    other
                );
            }
        }
    }

    let coderuns_api: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let new_context_version = code_run.spec.context_version + 1;

    let spec_patch = json!({
        "spec": {
            "contextVersion": new_context_version,
            "continueSession": true
        }
    });

    coderuns_api
        .patch(
            &code_run.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&spec_patch),
        )
        .await?;

    let next_attempt = current_retry_count + 1;
    let allowed_display = if max_retries == 0 {
        "∞".to_string()
    } else {
        max_retries.to_string()
    };

    // Update status with incremented retry count in a single atomic operation
    // This fixes a race condition where increment_retry_count and update_code_status_with_completion
    // would overwrite each other's changes
    update_code_status_with_completion(
        code_run,
        ctx,
        "Running",
        &format!("Retry attempt {next_attempt} scheduled (max {allowed_display}): {reason}"),
        false,
        Some(next_attempt), // Pass the incremented retry count
        None,
        ExpireAtUpdate::Clear,
    )
    .await?;

    Ok(())
}

/// Verify completion status with GitHub to prevent stale local state
fn verify_github_completion_status(_pr_url: &str) -> bool {
    // Extract PR number from GitHub URL
    // Format: https://github.com/owner/repo/pull/number
    // let pr_number = extract_pr_number_from_url(pr_url)?;

    // For now, implement a basic check - in production you'd use GitHub API
    // to check if PR is merged, has completion labels, etc.

    // TODO: Implement proper GitHub API call to verify:
    // 1. PR merge status
    // 2. PR closure status
    // 3. Completion labels
    // 4. Latest comment checkbox states

    warn!("GitHub verification not fully implemented - returning true for now");
    true // Placeholder - assume complete for now
}

/// Clear stale `work_completed` status
async fn clear_work_completed_status(code_run: &CodeRun, ctx: &Context) -> Result<()> {
    let code_runs: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    let patch = json!({
        "status": {
            "workCompleted": false,
            "message": "Status cleared due to GitHub verification mismatch"
        }
    });

    code_runs
        .patch(
            &code_run.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await?;

    info!(
        "Cleared work_completed status for CodeRun {}",
        code_run.name_any()
    );
    Ok(())
}

async fn try_cleanup_after_ttl(
    code_run: &Arc<CodeRun>,
    ctx: &Arc<Context>,
) -> Result<Option<Action>> {
    if !ctx.config.cleanup.enabled {
        return Ok(None);
    }

    if cleanup::is_preserved(&code_run.metadata) {
        return Ok(None);
    }

    let Some(status) = &code_run.status else {
        return Ok(None);
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
        code_run = %code_run.name_any(),
        "TTL expired - cleaning up remaining resources"
    );
    perform_ttl_cleanup(code_run, ctx).await?;
    mark_cleanup_complete(code_run, ctx).await?;
    Ok(Some(Action::await_change()))
}

async fn perform_ttl_cleanup(code_run: &Arc<CodeRun>, ctx: &Arc<Context>) -> Result<()> {
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let resource_manager = CodeResourceManager::new(&jobs, &configmaps, &pvcs, &ctx.config, ctx);
    let _ = resource_manager.cleanup_resources(code_run).await?;
    Ok(())
}

async fn mark_cleanup_complete(code_run: &CodeRun, ctx: &Context) -> Result<()> {
    let coderuns: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let patch = json!({
        "status": {
            "cleanupCompletedAt": Utc::now().to_rfc3339(),
            "expireAt": serde_json::Value::Null
        }
    });
    coderuns
        .patch_status(
            &code_run.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await?;
    Ok(())
}

fn compute_cleanup_deadline(
    code_run: &CodeRun,
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

    if cleanup::is_preserved(&code_run.metadata) {
        return None;
    }

    if code_run
        .status
        .as_ref()
        .and_then(|status| status.expire_at.as_ref())
        .is_some()
    {
        return None;
    }

    let ttl_seconds = cleanup::ttl_override_seconds(&code_run.metadata).or_else(|| {
        if phase == "Succeeded" {
            Some(ctx.config.cleanup.success_ttl_seconds)
        } else {
            Some(ctx.config.cleanup.failure_ttl_seconds)
        }
    })?;

    if ttl_seconds == 0 {
        return None;
    }

    #[allow(clippy::cast_possible_wrap)]
    Some(finished_at + ChronoDuration::seconds(ttl_seconds as i64))
}
