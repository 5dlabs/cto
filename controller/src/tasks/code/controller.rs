use super::naming::ResourceNaming;
use super::resources::CodeResourceManager;
use crate::crds::CodeRun;
use crate::tasks::types::{Context, Result, CODE_FINALIZER_NAME};
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use kube::api::{DeleteParams, Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::finalizer::{finalizer, Event as FinalizerEvent};
use kube::{Api, ResourceExt};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, instrument, warn};

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
pub async fn reconcile_code_run(code_run: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action> {
    info!("🎯 Starting reconcile for CodeRun: {}", code_run.name_any());

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = code_run.name_any();

    info!("🔄 Reconciling CodeRun: {}", name);

    // Create APIs
    info!("🔗 Creating Kubernetes API clients...");
    let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);
    info!("✅ API clients created successfully");

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

    info!("🏁 Reconcile completed with result: {:?}", result);

    Ok(result)
}

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
async fn reconcile_code_create_or_update(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    let code_run_name = code_run.name_any();
    info!(
        "Starting status-first idempotent reconcile for CodeRun: {}",
        code_run_name
    );

    // STEP 1: Check CodeRun status first (status-first idempotency)
    if let Some(status) = &code_run.status {
        // Check for completion based on work_completed field (TTL-safe)
        if status.work_completed == Some(true) {
            // Double-check with GitHub to ensure status hasn't changed
            if let Some(pr_url) = &status.pull_request_url {
                if let Ok(is_still_complete) = verify_github_completion_status(pr_url).await {
                    if !is_still_complete {
                        warn!("Local work_completed=true but GitHub shows incomplete - clearing stale status");
                        clear_work_completed_status(&code_run, ctx).await?;
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
                update_code_status_with_completion(
                    &code_run,
                    ctx,
                    "Succeeded",
                    "Code implementation completed successfully",
                    true,
                )
                .await?;

                // Handle workflow resumption for already succeeded CodeRuns
                handle_workflow_resumption_on_completion(&code_run, ctx).await?;

                return Ok(Action::await_change());
            }
            "Failed" => {
                info!("Already failed, no retry logic");
                return Ok(Action::await_change());
            }
            "Running" => {
                info!("Status shows running, checking actual job state");
                // Continue to job state check below
            }
            _ => {
                info!("Status is '{}', proceeding with job creation", status.phase);
                // Continue to job creation below
            }
        }
    } else {
        info!("No status found, initializing");
    }

    // STEP 2: Check job state for running jobs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let job_name = get_job_name(&code_run);
    info!("Generated job name: {}", job_name);

    let job_state = check_code_job_state(&jobs, &job_name).await?;
    info!("Current job state: {:?}", job_state);

    match job_state {
        CodeJobState::NotFound => {
            info!("No existing job found, using optimistic job creation");

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
            )
            .await?;

            // Requeue to check job progress
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }

        CodeJobState::Running => {
            info!("Job is still running, monitoring progress");

            // Update status to Running with workCompleted=false
            update_code_status_with_completion(
                &code_run,
                ctx,
                "Running",
                "Code task in progress",
                false,
            )
            .await?;

            // Continue monitoring
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }

        CodeJobState::Completed => {
            info!("Job completed successfully - marking work as completed");

            // CRITICAL: Update with work_completed=true for TTL safety
            update_code_status_with_completion(
                &code_run,
                ctx,
                "Succeeded",
                "Code implementation completed successfully",
                true,
            )
            .await?;

            // Handle workflow resumption after successful completion
            handle_workflow_resumption_on_completion(&code_run, ctx).await?;

            // Cleanup per controller configuration
            if ctx.config.cleanup.enabled {
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

            // Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }

        CodeJobState::Failed => {
            info!("Job failed - marking as failed");

            // Update to failed status (no work_completed=true for failures)
            update_code_status_with_completion(
                &code_run,
                ctx,
                "Failed",
                "Code implementation failed",
                false,
            )
            .await?;

            // Handle workflow resumption for failed jobs
            handle_workflow_resumption_on_failure(&code_run, ctx).await?;

            // Cleanup per controller configuration (failed jobs)
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

            // Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }
    }
}

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
async fn cleanup_code_resources(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    info!("🧹 Cleaning up resources for CodeRun");

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

/// Get job name for CodeRun - prefer stored name, fallback to generation
/// This fixes the job name mismatch that was causing status update failures
fn get_job_name(code_run: &CodeRun) -> String {
    // First try to get the job name from CodeRun status (set during creation)
    if let Some(status) = &code_run.status {
        if let Some(job_name) = &status.job_name {
            info!("Using stored job name from status: {}", job_name);
            return job_name.clone();
        }
    }

    // Fallback to unified generation
    let generated_name = ResourceNaming::job_name(code_run);
    info!("Generated job name: {}", generated_name);
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

async fn update_code_status_with_completion(
    code_run: &CodeRun,
    ctx: &Context,
    new_phase: &str,
    new_message: &str,
    work_completed: bool,
) -> Result<()> {
    // Only update if status actually changed or work_completed changed
    let current_phase = code_run
        .status
        .as_ref()
        .map(|s| s.phase.as_str())
        .unwrap_or("");
    let current_work_completed = code_run
        .status
        .as_ref()
        .and_then(|s| s.work_completed)
        .unwrap_or(false);

    if current_phase == new_phase && current_work_completed == work_completed {
        info!(
            "Status already '{}' with work_completed={}, skipping update to prevent reconciliation",
            new_phase, work_completed
        );
        return Ok(());
    }

    info!(
        "Updating status from '{}' (work_completed={}) to '{}' (work_completed={})",
        current_phase, current_work_completed, new_phase, work_completed
    );

    let coderuns: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    let status_patch = json!({
        "status": {
            "phase": new_phase,
            "message": new_message,
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
            "workCompleted": work_completed,
        }
    });

    // Use status subresource to avoid triggering spec reconciliation
    coderuns
        .patch_status(
            &code_run.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&status_patch),
        )
        .await?;

    info!(
        "Status updated successfully to '{}' with work_completed={}",
        new_phase, work_completed
    );
    Ok(())
}

/// Handle workflow resumption when CodeRun completes successfully
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

/// Handle workflow resumption when CodeRun fails
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
                if let Err(e) = crate::tasks::workflow::resume_workflow_for_pr(
                    &ctx.client,
                    &ctx.namespace,
                    workflow_name,
                    pr_url,
                    pr_number,
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
        if let Err(e) = crate::tasks::workflow::resume_workflow_for_pr(
            &ctx.client,
            &ctx.namespace,
            workflow_name,
            &pr_url,
            pr_number,
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
        .map(|s| s.phase.as_str())
        .unwrap_or("Succeeded");

    if let Err(e) =
        resume_workflow_for_no_pr(&ctx.client, &ctx.namespace, workflow_name, coderun_status).await
    {
        warn!("Failed to resume workflow with no-PR status: {}", e);
    }
    Ok(())
}

/// Verify completion status with GitHub to prevent stale local state
async fn verify_github_completion_status(_pr_url: &str) -> Result<bool> {
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
    Ok(true) // Placeholder - assume complete for now
}

/// Clear stale work_completed status
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
