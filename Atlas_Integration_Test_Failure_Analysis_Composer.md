# Atlas Integration Test Failure Analysis - Composer

**Date:** November 22, 2025  
**Analyst:** Composer (Claude Sonnet 4.5)  
**Context:** Deep analysis of Play workflow regression preventing Atlas integration gate activation

---

## Executive Summary

The multi-agent orchestration pipeline is failing at the Tess→Atlas handoff due to **three interconnected root causes**:

1. **Tess PR Review Submission Failure**: The Tess container script treats PR review failures as non-fatal, allowing CodeRuns to succeed without generating the GitHub approval event that Atlas sensors depend on.

2. **Missing Workflow Stage**: The play workflow template lacks an `atlas-integration` stage between testing completion and PR merge, causing workflows to skip directly to `waiting-pr-merged` without Atlas involvement.

3. **Sensor-Workflow Stage Mismatch**: The Tess approval sensor resumes workflows at `waiting-ready-for-qa`, but workflows transition from `testing-in-progress` directly to `waiting-pr-merged`, creating a stage synchronization gap.

**Impact**: Complete pipeline failure - workflows stuck indefinitely at `waiting-atlas-integration` (a stage that doesn't exist in the workflow definition), preventing all multi-agent coordination.

---

## Detailed Root Cause Analysis

### Root Cause #1: Tess PR Review Failure Treated as Non-Fatal

#### Evidence from Code

**File**: `infra/charts/controller/agent-templates/code/integration/container-tess.sh.hbs` (lines 2417-2437)

```bash
# Try to post APPROVE review (may fail if other agents requested changes)
echo "📝 Posting APPROVE review..."
REVIEW_OUTPUT=$(timeout 30 gh pr review "$PR_NUMBER" -R "$REPO_SLUG" --approve --body "..." 2>&1) || {
  REVIEW_EXIT=$?
  echo "⚠️ PR approval review failed (exit code: $REVIEW_EXIT)"
  echo "   This is expected if other agents have requested changes"
  echo "   Output: $REVIEW_OUTPUT"
  echo "ℹ️  Label added successfully - workflow will proceed"
}
```

**Critical Issues**:

1. **Silent Failure**: The script uses `|| { ... }` which captures the error but **does not exit**. Execution continues even when the PR review command fails.

2. **Misleading Assumption**: The comment suggests failures are "expected if other agents have requested changes", but this is incorrect. GitHub allows multiple approvals even when changes are requested - the failure is likely due to:
   - Missing GitHub App permissions
   - Authentication token issues
   - Repository-specific branch protection rules
   - Network/timeout issues

3. **Label Fallback Logic**: The script adds an `approved` label (line 2411) before attempting the PR review, creating a false sense of success. The workflow may proceed based on the label, but **no GitHub webhook event is generated** because the review was never submitted.

4. **No Error Propagation**: The CodeRun exits with success (`CLAUDE_EXIT_CODE=0`) even when PR review fails, preventing the workflow from detecting the failure.

#### Why This Breaks Atlas Integration

The Atlas integration gate relies on GitHub webhook events:
- **Event**: `pull_request_review`
- **Action**: `submitted`
- **State**: `approved`
- **User**: `["5DLabs-Tess[bot]", "5DLabs-Tess", "tess-5dlabs"]`

**Without a successful PR review submission, no webhook event is generated**, so the `stage-aware-tess-approval-sensor` never fires, and Atlas is never triggered.

#### Potential Failure Modes

1. **GitHub App Permissions**: Tess GitHub App (`5DLabs-Tess`) may not have `PullRequestReviews: Write` permission on `cto-parallel-test` repository
2. **Token Expiration**: GitHub App JWT tokens expire after 10 minutes; long-running CodeRuns may use expired tokens
3. **Branch Protection**: Repository may require specific reviewers or CODEOWNERS approval before allowing bot reviews
4. **Rate Limiting**: GitHub API rate limits may cause transient failures that aren't retried

---

### Root Cause #2: Missing Atlas Integration Stage in Workflow

#### Evidence from Code

**File**: `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml` (lines 646-647, 1305-1308)

```yaml
# After Tess completes testing, agents are done
# Workflow will proceed to wait for PR merge
```

**Stage Transition Logic** (lines 1305-1308):
```yaml
"testing-in-progress")
  # Testing → completion (Tess done)
  [[ $new_stage == "waiting-pr-merged" ]] && return 0
  [[ $new_stage == "completed" ]] && return 0
  ;;
```

**Missing Transitions**:
- No `waiting-atlas-integration` stage
- No `atlas-in-progress` stage
- Direct transition from `testing-in-progress` → `waiting-pr-merged`

#### Architecture Intent vs. Implementation

**Documentation Claims** (from `docs/atlas-bolt-final-implementation-summary.md`):
```
1. REX - Implementation
   ↓ Creates PR with code
2. CLEO - Code Quality
   ↓ Reviews, APPROVES
3. TESS - QA/Testing  
   ↓ Tests in staging, APPROVES
4. ATLAS - Integration/Merge
   ↓ MERGES PR to main
```

**Reality**: The workflow skips step 4 entirely, going directly from Tess approval to waiting for manual PR merge.

#### Why This Breaks Atlas Integration

1. **No Workflow Stage for Atlas**: There's no stage label (`waiting-atlas-integration`) that Atlas sensors can monitor or that workflows can transition to.

2. **No CodeRun Creation**: The workflow never creates an Atlas CodeRun CRD, so Atlas never executes even if sensors fire.

3. **Stage Label Mismatch**: The analysis document mentions workflows stuck at `waiting-atlas-integration`, but this stage **doesn't exist in the workflow template**. This suggests:
   - Manual stage label updates that don't match workflow logic
   - Or a previous version of the workflow that was removed

---

### Root Cause #3: Sensor-Workflow Stage Synchronization Gap

#### Evidence from Code

**File**: `infra/gitops/resources/github-webhooks/stage-aware-tess-approval-sensor.yaml` (lines 36-52, 236-237)

**Sensor Configuration**:
```yaml
target-stage: waiting-ready-for-qa
```

**Sensor Logic** (lines 236-237):
```bash
if [ "$CURRENT_STAGE" = "waiting-ready-for-qa" ] && [ "$SUSPEND_NODE" = "Running" ]; then
  echo "Found workflow at correct stage and suspended, resuming..."
```

**Workflow Stage Transitions** (from `play-workflow-template.yaml`):
- `waiting-ready-for-qa` → `testing-in-progress` (when Tess CodeRun starts)
- `testing-in-progress` → `waiting-pr-merged` (when Tess CodeRun completes)

**The Problem**:

1. **Stage Mismatch**: The sensor expects workflows at `waiting-ready-for-qa`, but when Tess completes, workflows are at `testing-in-progress`.

2. **Resume Timing**: The sensor resumes workflows **after** Tess approval, but the workflow may have already transitioned past `waiting-ready-for-qa` to `testing-in-progress` or beyond.

3. **No Atlas Trigger**: Even if the sensor successfully resumes the workflow, it only calls `resume_workflow()` - **it never creates an Atlas CodeRun**. The workflow resumes and proceeds directly to `waiting-pr-merged` without Atlas involvement.

#### Why This Breaks Atlas Integration

The sensor is designed to resume workflows waiting for Tess approval, but:
- It doesn't create Atlas CodeRuns
- It doesn't transition workflows to an Atlas integration stage
- It assumes the workflow will handle Atlas internally, but the workflow has no Atlas stage

---

## Secondary Issues Discovered

### Issue #1: Workflow Stage Label Drift

The analysis document mentions workflows stuck at `waiting-atlas-integration`, but this stage doesn't exist in the current workflow template. This suggests:

1. **Manual Label Updates**: Someone may have manually updated workflow labels to `waiting-atlas-integration` expecting Atlas to trigger
2. **Version Mismatch**: The workflow template may have been updated to remove Atlas stages, but existing workflows still have old labels
3. **Missing Validation**: The workflow template's stage transition validation (lines 1303-1322) doesn't include `waiting-atlas-integration`, so workflows with this label are in an invalid state

### Issue #2: No Fallback Mechanism

When Tess PR review fails:
- No retry logic
- No alternative trigger mechanism (e.g., label-based trigger)
- No timeout/fallback to proceed without Atlas
- Workflows remain suspended indefinitely

### Issue #3: GitHub App Permission Validation

The Tess container script doesn't validate GitHub App permissions before attempting PR review:
- No pre-flight check for `PullRequestReviews: Write` permission
- No verification that the app is installed on the target repository
- No error message indicating permission issues

---

## Remediation Plan

### Phase 1: Immediate Fixes (Critical Path)

#### Fix #1: Make Tess PR Review Failures Fatal

**File**: `infra/charts/controller/agent-templates/code/integration/container-tess.sh.hbs`

**Changes Required**:

1. **Remove Non-Fatal Error Handling**: Change the PR review command to fail the CodeRun if it fails:
```bash
# Post APPROVE review (CRITICAL for workflow progression)
echo "📝 Posting APPROVE review..."
if ! timeout 30 gh pr review "$PR_NUMBER" -R "$REPO_SLUG" --approve --body "..." 2>&1; then
  REVIEW_EXIT=$?
  echo "❌ PR approval review FAILED (exit code: $REVIEW_EXIT)"
  echo "   This is CRITICAL - workflow cannot proceed without PR review"
  echo "   Checking GitHub App permissions and token validity..."
  
  # Verify GitHub App installation
  if ! gh api "/repos/$REPO_SLUG/installation" >/dev/null 2>&1; then
    echo "❌ GitHub App not installed on repository: $REPO_SLUG"
    exit 1
  fi
  
  # Verify PR exists and is accessible
  if ! gh pr view "$PR_NUMBER" -R "$REPO_SLUG" >/dev/null 2>&1; then
    echo "❌ Cannot access PR #$PR_NUMBER in repository: $REPO_SLUG"
    exit 1
  fi
  
  echo "❌ PR review submission failed - failing CodeRun to prevent workflow deadlock"
  exit 1
fi
echo "✅ PR approval review submitted successfully"
```

2. **Add Pre-Flight Validation**: Before attempting PR review, verify:
   - GitHub App is installed on the repository
   - PR exists and is accessible
   - Authentication token is valid (not expired)

3. **Add Retry Logic**: For transient failures (network, rate limits), implement exponential backoff retry:
```bash
MAX_RETRIES=3
RETRY_DELAY=5
for i in $(seq 1 $MAX_RETRIES); do
  if timeout 30 gh pr review "$PR_NUMBER" -R "$REPO_SLUG" --approve --body "..." 2>&1; then
    echo "✅ PR review submitted successfully (attempt $i)"
    break
  fi
  if [ $i -eq $MAX_RETRIES ]; then
    echo "❌ PR review failed after $MAX_RETRIES attempts"
    exit 1
  fi
  echo "⚠️ PR review failed (attempt $i/$MAX_RETRIES), retrying in ${RETRY_DELAY}s..."
  sleep $RETRY_DELAY
  RETRY_DELAY=$((RETRY_DELAY * 2))
done
```

**Impact**: Tess CodeRuns will fail fast when PR reviews can't be submitted, preventing workflows from proceeding without the required GitHub events.

---

#### Fix #2: Add Atlas Integration Stage to Workflow

**File**: `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`

**Changes Required**:

1. **Add Atlas Integration Stage** after testing completion:
```yaml
# After Tess completes testing, transition to Atlas integration
- name: update-to-atlas-integration
  dependencies: [testing-work]
  template: update-workflow-stage
  arguments:
    parameters:
      - name: new-stage
        value: "waiting-atlas-integration"
      - name: verify-update
        value: "true"
  when: "'{{`{{steps.should-run-testing.outputs.parameters.should-run}}`}}' == 'true'"

# Suspend workflow waiting for Atlas integration
- name: wait-for-atlas-integration
  dependencies: [update-to-atlas-integration]
  template: suspend-for-event
  arguments:
    parameters:
      - name: stage
        value: "waiting-atlas-integration"

# Atlas integration work (triggered by sensor)
- name: atlas-integration-work
  dependencies: [wait-for-atlas-integration]
  template: agent-coderun
  arguments:
    parameters:
      - name: github-app
        value: "5DLabs-Atlas"
      - name: task-id
        value: "{{`{{workflow.parameters.task-id}}`}}"
      - name: stage
        value: "integration"
      - name: cli-type
        value: "{{`{{workflow.parameters.integration-cli}}`}}"
      - name: model
        value: "{{`{{workflow.parameters.integration-model}}`}}"
  when: "'{{`{{steps.should-run-integration.outputs.parameters.should-run}}`}}' == 'true'"

# After Atlas completes, proceed to PR merge
- name: update-to-waiting-merge
  dependencies: [atlas-integration-work]
  template: update-workflow-stage
  arguments:
    parameters:
      - name: new-stage
        value: "waiting-pr-merged"
```

2. **Update Stage Transition Validation** (around line 1305):
```yaml
"testing-in-progress")
  # Testing → Atlas integration (after Tess approval)
  [[ $new_stage == "waiting-atlas-integration" ]] && return 0
  # Testing → completion (skip Atlas if disabled)
  [[ $new_stage == "waiting-pr-merged" ]] && return 0
  [[ $new_stage == "completed" ]] && return 0
  ;;
"waiting-atlas-integration")
  # Atlas integration → PR merge (after Atlas completes)
  [[ $new_stage == "waiting-pr-merged" ]] && return 0
  # Allow retry back to testing if Atlas fails
  [[ $new_stage == "testing-in-progress" ]] && return 0
  ;;
```

3. **Add Workflow Parameters** for Atlas configuration:
```yaml
- name: integration-agent
  value: "5DLabs-Atlas"
- name: integration-cli
  value: "claude"
- name: integration-model
  value: "sonnet-4-20250514"
- name: enable-integration
  value: "true"  # Allow disabling Atlas for simple workflows
```

**Impact**: Workflows will have a proper stage for Atlas integration, allowing sensors to trigger Atlas CodeRuns at the correct workflow point.

---

#### Fix #3: Update Tess Approval Sensor to Create Atlas CodeRun

**File**: `infra/gitops/resources/github-webhooks/stage-aware-tess-approval-sensor.yaml`

**Changes Required**:

1. **Update Target Stage**: Change from `waiting-ready-for-qa` to `waiting-atlas-integration`:
```yaml
target-stage: waiting-atlas-integration
```

2. **Add Atlas CodeRun Creation**: After resuming the workflow, create an Atlas CodeRun:
```yaml
triggers:
  - template:
      name: resume-and-trigger-atlas
      conditions: "tess-approved-event"
      k8s:
        operation: create
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            metadata:
              generateName: stage-resume-tess-approved-
              namespace: agent-platform
              labels:
                type: stage-resume
                target-stage: waiting-atlas-integration
            spec:
              entrypoint: resume-and-create-atlas
              templates:
                - name: resume-and-create-atlas
                  steps:
                    - - name: resume-workflow
                        template: resume-workflow-script
                    - - name: create-atlas-coderun
                        template: create-atlas-coderun-script
```

3. **Create Atlas CodeRun Script**: Add a template that creates the Atlas CodeRun CRD:
```yaml
- name: create-atlas-coderun-script
  script:
    image: alpine/k8s:1.31.0
    command: [bash]
    source: |
      #!/bin/bash
      set -e
      
      # Extract task ID and repository from PR
      TASK_ID="{{ .Input.body.pull_request.labels | ... }}"
      REPO_SLUG="{{ .Input.body.repository.full_name }}"
      PR_NUMBER="{{ .Input.body.pull_request.number }}"
      
      # Create Atlas CodeRun
      cat <<EOF | kubectl apply -f -
      apiVersion: cto.5dlabs.io/v1alpha1
      kind: CodeRun
      metadata:
        generateName: coderun-atlas-integration-
        namespace: agent-platform
        labels:
          task-id: "${TASK_ID}"
          stage: "integration"
          repository: "${REPO_SLUG}"
      spec:
        githubApp: "5DLabs-Atlas"
        repository: "${REPO_SLUG}"
        taskId: "${TASK_ID}"
        stage: "integration"
        prNumber: "${PR_NUMBER}"
      EOF
```

**Alternative Approach**: Instead of creating CodeRun in the sensor, update the workflow to create Atlas CodeRun after resuming. This keeps workflow logic in the workflow template.

**Impact**: Atlas CodeRuns will be created immediately after Tess approval, ensuring Atlas integration happens at the correct workflow stage.

---

### Phase 2: Enhanced Error Handling and Observability

#### Enhancement #1: GitHub App Permission Validation

**Add Pre-Flight Checks** in Tess container script:
```bash
# Validate GitHub App permissions before attempting PR review
validate_github_app_permissions() {
  local repo_slug="$1"
  local app_name="5DLabs-Tess"
  
  echo "🔍 Validating GitHub App permissions for $app_name on $repo_slug..."
  
  # Check if app is installed
  if ! INSTALLATION=$(gh api "/repos/$repo_slug/installation" 2>&1); then
    echo "❌ GitHub App $app_name is not installed on repository: $repo_slug"
    echo "   Installation check output: $INSTALLATION"
    return 1
  fi
  
  # Check permissions (requires GitHub API v4 GraphQL or v3 REST)
  PERMISSIONS=$(echo "$INSTALLATION" | jq -r '.permissions // {}')
  if [ "$PERMISSIONS" = "{}" ] || [ "$PERMISSIONS" = "null" ]; then
    echo "⚠️ Cannot verify permissions - proceeding with caution"
    return 0
  fi
  
  PR_REVIEW_PERM=$(echo "$PERMISSIONS" | jq -r '.pull_requests // "none"')
  if [ "$PR_REVIEW_PERM" != "write" ] && [ "$PR_REVIEW_PERM" != "admin" ]; then
    echo "❌ GitHub App $app_name lacks PullRequestReviews write permission"
    echo "   Current permission: $PR_REVIEW_PERM"
    return 1
  fi
  
  echo "✅ GitHub App permissions validated"
  return 0
}
```

#### Enhancement #2: Add Metrics and Alerting

**Add Prometheus Metrics** for PR review submission:
```bash
# After PR review attempt
if [ $REVIEW_EXIT -eq 0 ]; then
  echo "tess_pr_review_success_total{repo=\"$REPO_SLUG\",task_id=\"$TASK_ID\"} 1" | \
    curl -X POST --data-binary @- http://prometheus-pushgateway:9091/metrics/job/tess
else
  echo "tess_pr_review_failure_total{repo=\"$REPO_SLUG\",task_id=\"$TASK_ID\",reason=\"$REVIEW_EXIT\"} 1" | \
    curl -X POST --data-binary @- http://prometheus-pushgateway:9091/metrics/job/tess
fi
```

**Add Workflow Stuck Detection**:
- Alert when workflows remain at `waiting-atlas-integration` > 30 minutes
- Alert when Tess CodeRuns succeed but no PR review is found
- Track time from Tess completion to Atlas trigger

#### Enhancement #3: Add Fallback Trigger Mechanism

**Option A: Label-Based Trigger** (Simpler):
```yaml
# Add sensor that triggers Atlas on "approved" label
- name: atlas-label-trigger
  eventSourceName: github
  eventName: org
  filters:
    data:
      - path: body.action
        type: string
        value: ["labeled"]
      - path: body.label.name
        type: string
        value: ["approved"]
```

**Option B: Workflow Stage Transition Trigger** (More Robust):
- Add Argo Workflows webhook that fires on stage label changes
- Trigger Atlas when workflow transitions to `waiting-atlas-integration`
- More reliable than GitHub events, but requires additional infrastructure

---

### Phase 3: Testing and Validation

#### Test Case #1: Tess PR Review Failure
1. **Setup**: Create test repository without Tess GitHub App installed
2. **Action**: Run Play workflow with Tess agent
3. **Expected**: Tess CodeRun fails with clear error message about missing GitHub App
4. **Validation**: Workflow does not proceed to `waiting-atlas-integration`

#### Test Case #2: Complete Multi-Agent Flow
1. **Setup**: All agents properly configured with GitHub Apps
2. **Action**: Run Play workflow: Rex → Cleo → Tess → Atlas → Merge
3. **Expected**: 
   - Tess submits PR review successfully
   - GitHub webhook event fires
   - Sensor creates Atlas CodeRun
   - Workflow transitions through all stages
   - Atlas merges PR
4. **Validation**: End-to-end workflow completes successfully

#### Test Case #3: Parallel Execution
1. **Setup**: Run multiple tasks in parallel (tasks 1, 3, 4, 6)
2. **Action**: All tasks complete Tess stage simultaneously
3. **Expected**: 
   - All Tess PR reviews submitted
   - All workflows transition to `waiting-atlas-integration`
   - Atlas handles batch integration correctly
4. **Validation**: No workflows stuck, all PRs processed

#### Test Case #4: Cipher Changes Requested
1. **Setup**: Cipher requests changes on PR before Tess approval
2. **Action**: Tess attempts to approve PR
3. **Expected**: 
   - Tess PR review succeeds (GitHub allows multiple approvals)
   - Workflow proceeds to Atlas integration
   - Atlas handles merge despite Cipher's changes requested
4. **Validation**: Workflow completes successfully

---

## Implementation Priority

### Critical (Immediate - Blocks All Multi-Agent Workflows)
1. ✅ **Fix #1**: Make Tess PR review failures fatal
2. ✅ **Fix #2**: Add Atlas integration stage to workflow
3. ✅ **Fix #3**: Update Tess approval sensor to create Atlas CodeRun

### High (Within 1 Week - Prevents Production Issues)
4. **Enhancement #1**: GitHub App permission validation
5. **Enhancement #2**: Metrics and alerting
6. **Test Case #1-2**: Basic validation tests

### Medium (Within 2 Weeks - Improves Reliability)
7. **Enhancement #3**: Fallback trigger mechanism
8. **Test Case #3-4**: Advanced validation tests

---

## Risk Assessment

### Risks of Immediate Fixes

1. **Breaking Existing Workflows**: 
   - **Risk**: Making Tess PR review failures fatal may cause existing workflows to fail
   - **Mitigation**: Add feature flag to enable/disable strict mode, default to strict for new workflows
   - **Impact**: Medium - may require manual intervention for stuck workflows

2. **Workflow Template Changes**:
   - **Risk**: Adding Atlas stage may break workflows expecting direct transition to merge
   - **Mitigation**: Make Atlas stage optional via workflow parameter (`enable-integration: false`)
   - **Impact**: Low - backward compatible with parameter defaults

3. **Sensor Changes**:
   - **Risk**: Updating sensor target stage may not match existing workflow states
   - **Mitigation**: Support both `waiting-ready-for-qa` and `waiting-atlas-integration` stages during transition
   - **Impact**: Low - can support both stages simultaneously

### Risks of Not Fixing

1. **Complete Pipeline Failure**: All multi-agent workflows will remain stuck indefinitely
2. **Resource Waste**: Stuck workflows consume cluster resources without progress
3. **Manual Intervention Required**: Every workflow needs manual resolution
4. **Loss of Trust**: System appears broken, reducing confidence in automation

---

## Conclusion

The root cause is **three interconnected issues** that compound to create a complete pipeline failure:

1. **Tess silently fails** to submit PR reviews, preventing GitHub webhook events
2. **Workflow lacks Atlas stage**, so even if events fired, Atlas wouldn't be triggered
3. **Sensor doesn't create Atlas CodeRuns**, so even if workflow had a stage, Atlas wouldn't run

**The fix requires addressing all three issues simultaneously**:
- Make Tess failures visible and fatal
- Add Atlas integration stage to workflow
- Update sensor to create Atlas CodeRuns

This is a **critical regression** that blocks all multi-agent orchestration. The fixes are straightforward but require coordinated changes across three components (container script, workflow template, sensor configuration).

**Estimated Fix Time**: 4-6 hours for implementation + 2-4 hours for testing = **6-10 hours total**

---

*Analysis completed: November 22, 2025*  
*Model: Composer (Claude Sonnet 4.5)*

