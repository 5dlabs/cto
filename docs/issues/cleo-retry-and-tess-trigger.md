# Cleo Retry and Tess Trigger Issues

## Problem Report

**Pod:** `code-cleo-claude-agent-platform-rust-ba-76c0d637-81936ffb-4w5q6`

**Issues:**
1. Cleo only ran once despite `max_retries: 2` configuration
2. Tess never started after Cleo approved the PR

## Investigation Results

### Issue 1: Retries Not Working

**Expected Behavior:**
- Cleo should retry up to `max_retries` times if the task is incomplete
- Each retry should create a new Job with incremented version (t1-v1, t1-v2, etc.)

**Actual Behavior:**
- Only one job ran: `code-cleo-claude-agent-platform-rust-ba-76c0d637-81936ffb-t1-v1`
- No retry jobs were created
- Container exited with code 0 (success)

**Root Cause:**
The retry logic has multiple issues:

1. **No Retry on Success**: The controller only considers retries when a Job fails. But with our recent change to always exit with code 0, the Job always succeeds even when the task is incomplete.

   ```rust
   // controller/src/tasks/code/controller.rs:223-221
   CodeJobState::Failed => {
       info!("Job failed - marking as failed");
       // ... retry logic would go here ...
   }
   
   CodeJobState::Completed => {
       info!("Job completed successfully - marking work as completed");
       // No retry logic here!
   }
   ```

2. **Retry Logic Not Implemented**: The controller has `increment_retry_count()` and retry tracking, but doesn't actually create new CodeRun instances for retries.

   ```rust
   // controller/src/tasks/code/status.rs:100-137
   pub async fn increment_retry_count(...) -> Result<()> {
       // This increments a counter but doesn't create a new job
   }
   ```

3. **Agent-Level Retry Not Wired Up**: The `maxRetries` parameter is passed to CodeRun spec but isn't used by the controller's reconciliation loop.

   ```yaml
   # infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml:723-724
   - name: max-retries
     default: ""
   # Parameter is passed but not used in retry logic
   ```

**Proposed Fix:**

Create a completion probe mechanism:
1. Add a completion marker file that agents create when task is truly complete
2. Check for this marker in the controller
3. If marker exists: mark as complete (no retry)
4. If marker missing but exit code 0: check retry count and create new CodeRun
5. Respect `maxRetries` from CodeRun spec

### Issue 2: Tess Never Started

**Expected Behavior:**
- Cleo completes and sets `remediationStatus: "approved"`, `qaStatus: "approved"`
- Controller detects completion and calls `resume_workflow_for_pr()`
- Workflow resume annotation triggers Argo to continue from suspended state
- Tess job starts

**Actual Behavior:**
- Cleo completed successfully and updated CodeRun status
- No Tess job was created
- Workflow either wasn't resumed or resume didn't trigger Tess

**Root Cause:**

1. **Workflow Already Cleaned Up**: The CodeRun completion triggers cleanup before workflow resume:

   ```rust
   // controller/src/tasks/code/controller.rs:203-220
   handle_workflow_resumption_on_completion(&code_run, ctx).await?;
   
   // Cleanup per controller configuration
   if ctx.config.cleanup.enabled {
       let cleanup_delay_minutes = ctx.config.cleanup.completed_job_delay_minutes;
       if cleanup_delay_minutes == 0 {
           // Job deleted immediately!
       }
   }
   ```

2. **Resume Timing Issue**: The `resume_workflow_for_pr()` is called, but:
   - It patches the workflow with a `force-retry` annotation
   - But Argo might not see this if the workflow is in a suspended state
   - Or the CodeRun is deleted before Argo can react

3. **No Verification**: There's no check to verify the workflow actually resumed:

   ```rust
   // controller/src/tasks/workflow.rs:280-285
   info!("✅ Successfully triggered workflow re-evaluation: {}", workflow_name);
   // No verification that Argo actually resumed!
   ```

**Proposed Fix:**

1. **Delay Cleanup**: Don't clean up CodeRun until workflow advances past the current stage
2. **Verify Resume**: After patching workflow, verify it's no longer suspended
3. **Retry Resume**: If resume fails, retry with backoff
4. **Status Polling**: Have Cleo's completion update trigger a workflow resume event via webhook, not just status patch

## Recommended Fixes

### Short-term (Band-aid):
1. Disable immediate cleanup (`cleanup.completed_job_delay_minutes: 5`)
2. Add retry logic to workflow resume
3. Use completion probe instead of exit code 0

### Long-term (Proper Solution):
1. Implement proper retry orchestration at controller level
2. Use Kubernetes events instead of status polling for workflow transitions
3. Add workflow health checks and automatic recovery
4. Separate concerns: agent completion ≠ job completion ≠ workflow progression

## Testing Plan

1. Create test CodeRun with `maxRetries: 3`
2. Have agent exit code 0 but not create completion marker
3. Verify controller creates retry jobs
4. Verify Tess starts after Cleo approval
5. Monitor workflow state transitions

## Related Files

- `controller/src/tasks/code/controller.rs` - Main reconciliation loop
- `controller/src/tasks/workflow.rs` - Workflow resume logic
- `controller/src/tasks/code/status.rs` - Retry count management
- `infra/charts/controller/agent-templates/code/claude/container-cleo.sh.hbs` - Cleo exit logic
- `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml` - Workflow definition

