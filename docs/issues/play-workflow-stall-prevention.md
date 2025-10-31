# Play Workflow Stall Prevention Guide

## Issue Summary

Play workflows were stalling at the `wait-for-tess-approval` stage, preventing automated task progression and requiring manual intervention.

## Root Causes Identified

### 1. Missing Sensors (CRITICAL - NOW FIXED ✅)

**Problem:**
- The `github-webhooks` kustomization.yaml referenced a non-existent file `stage-aware-resume-sensor.yaml`
- This caused ArgoCD sync errors, preventing ANY GitHub webhook sensors from deploying
- Without sensors, GitHub approval events couldn't resume suspended workflows

**Fix Applied:**
- Updated `infra/gitops/resources/github-webhooks/kustomization.yaml`
- Replaced non-existent file reference with actual sensor files:
  - `stage-aware-cleo-approval-sensor.yaml`
  - `stage-aware-tess-approval-sensor.yaml`
  - `stage-aware-pr-merged-sensor.yaml`
- Merged in PR #1166

**Prevention:**
- ArgoCD application `github-webhooks` is now healthy
- All three stage-aware sensors deployed and running in `argo` namespace
- Verify sensors before running play workflows:
  ```bash
  kubectl get sensors -n argo | grep stage-aware
  # Should show: cleo-approval, tess-approval, pr-merged
  ```

### 2. Auto-Merge Not Working (REPOSITORY CONFIGURATION ISSUE)

**Problem:**
- Workflow parameter `auto-merge: true` is correctly set and passed to agents
- Tess agent code correctly attempts to enable auto-merge: `gh pr merge --auto --squash`
- **But GitHub auto-merge requires branch protection to be configured**
- The `cto-parallel-test` repository has NO branch protection on `main`
- Auto-merge fails silently with error: "Protected branch rules not configured"

**Evidence:**
```bash
$ gh api repos/5dlabs/cto-parallel-test/branches/main/protection
{"message":"Branch not protected","status":"404"}
```

**Impact:**
- PRs get approved by Tess ✅
- But they DON'T auto-merge ❌
- Workflows suspend at `waiting-pr-merged` indefinitely
- Merge conflicts accumulate as other PRs get merged manually

**Fix Required:**

Enable branch protection on target repositories:

```bash
gh api -X PUT repos/5dlabs/cto-parallel-test/branches/main/protection \
  --field required_status_checks='{"strict":false,"contexts":[]}' \
  --field enforce_admins=false \
  --field required_pull_request_reviews='{"required_approving_review_count":1}' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false
```

Or via GitHub UI:
1. Go to repo Settings → Branches
2. Add branch protection rule for `main`
3. Enable "Require a pull request before merging"
4. Set required approvals to 1
5. (Optional) Add status checks if desired

**Validation:**
```bash
# After enabling protection, verify auto-merge works:
gh pr view <PR_NUMBER> --repo <REPO> --json autoMergeRequest
# Should show: {"autoMergeRequest": {"enabledAt": "...", "enabledBy": "..."}}
```

### 3. Race Condition Between Sensor Deployment and Workflow Execution

**Problem:**
- Workflows can reach suspend points BEFORE sensors are deployed
- GitHub sends webhook for approval events
- But if sensor doesn't exist yet, event is lost forever
- Workflow stuck waiting for event that already occurred

**Prevention:**
- Ensure sensors are deployed BEFORE starting play workflows
- Add pre-flight check to play workflow submission:
  ```bash
  # In controller or MCP tool before submitting workflow
  kubectl get sensors -n argo stage-aware-tess-approval stage-aware-pr-merged
  # Fail if sensors don't exist
  ```

## Pre-Flight Checklist for Play Workflows

Before running any play workflow, verify:

### ✅ 1. Sensors Are Deployed
```bash
kubectl get sensors -n argo | grep -E "stage-aware-(cleo|tess|pr-merged)"
```
Expected output:
```
stage-aware-cleo-approval    <age>
stage-aware-pr-merged        <age>
stage-aware-tess-approval    <age>
```

### ✅ 2. ArgoCD Application Is Healthy
```bash
kubectl get application github-webhooks -n argocd -o jsonpath='{.status.sync.status},{.status.health.status}'
```
Expected: `Synced,Healthy`

### ✅ 3. Branch Protection Is Configured (for auto-merge)
```bash
gh api repos/5dlabs/<repo>/branches/main/protection --jq '.required_pull_request_reviews.required_approving_review_count'
```
Expected: `1` (or any number > 0)

If branch protection is missing, auto-merge will fail silently.

### ✅ 4. Event Source Is Receiving Webhooks
```bash
kubectl get eventsource -n argo github -o jsonpath='{.status.conditions[?(@.type=="DeploymentsRunning")].status}'
```
Expected: `True`

## Recommended Improvements

### 1. Add Sensor Health Check to MCP Play Tool
```rust
// In mcp/src/tools/play.rs
async fn preflight_check(&self) -> Result<()> {
    // Check sensors exist
    let sensors = kubectl::get_sensors("argo", vec![
        "stage-aware-tess-approval",
        "stage-aware-cleo-approval", 
        "stage-aware-pr-merged"
    ]).await?;
    
    if sensors.len() < 3 {
        return Err("Required sensors not deployed. Run: kubectl get sensors -n argo");
    }
    
    Ok(())
}
```

### 2. Add Branch Protection Validation
```rust
// Check target repo has branch protection
let protection = github::get_branch_protection(&repo, "main").await?;
if !protection.enabled {
    warn!("Auto-merge will not work without branch protection on {}/main", repo);
}
```

### 3. Add Workflow Timeout Alert
If a workflow is suspended at `wait-for-tess-approval` or `wait-for-pr-merged` for > 10 minutes, alert that sensors may not be working.

### 4. Make Auto-Merge Failure More Visible
Update Tess template to post a comment when auto-merge fails:
```bash
if ! gh pr merge "$PR_NUM" --auto --squash; then
  gh pr comment "$PR_NUM" --body "⚠️ **Auto-merge failed**
  
Branch protection may not be configured on this repository.
Please enable branch protection or merge manually."
fi
```

## Testing After Fix

1. **Verify sensors are deployed:**
   ```bash
   kubectl get sensors -n argo | grep stage-aware
   ```

2. **Enable branch protection on test repo**

3. **Run a small play workflow (1-2 tasks)**

4. **Verify auto-merge happens:**
   - Check PR has auto-merge enabled after Tess approval
   - Workflow progresses past `wait-for-pr-merged` automatically
   - No manual intervention required

## Impact of Fixes

- ✅ **Sensors deployed** - GitHub events can now resume workflows
- ⚠️ **Branch protection needed** - Must be configured per-repository
- 🔄 **Future workflows will auto-progress** - No more manual PR merging

## Next Steps

1. ✅ Kustomization fix merged (PR #1166)
2. ⏳ Enable branch protection on `cto-parallel-test` repository
3. ⏳ Run new play workflow to validate end-to-end
4. ⏳ Consider adding preflight checks to MCP tool

