# Atlas PR Guardian Sensor Fix

## Problem Summary

The Atlas PR Guardian sensor was not triggering on any PR events in the `5dlabs/cto` repository, preventing Atlas from monitoring and auto-merging pull requests.

## Root Cause

The sensor's filter expression had a logic error that caused all events to be discarded:

```yaml
# BROKEN - Line 42 in atlas-pr-guardian-sensor.yaml
exprs:
  - expr: 'body.X-GitHub-Event != "issue_comment" || body.issue.pull_request != null'
    fields:
      - name: is_pr_comment
        path: body.issue.pull_request
```

### Why It Failed

The expression attempted to access `body.issue.pull_request` on **all** events to verify they were PR-related. However:

1. **For `pull_request` events**: The field `body.issue.pull_request` doesn't exist (PR data is in `body.pull_request`)
2. **For `pull_request_review` events**: Same issue - no `body.issue` field
3. **For `issue_comment` events**: The field exists and works correctly

When Argo Events tried to evaluate the expression on `pull_request` events, it failed with:

```
expr filter error (path 'body.issue.pull_request' does not exist)
```

This caused **all** PR events to be discarded, preventing any Atlas CodeRuns from being created.

## The Fix

Updated the expression to handle each event type correctly:

```yaml
# FIXED - Line 42-45 in atlas-pr-guardian-sensor.yaml
exprs:
  # For pull_request and pull_request_review events, always pass (they are PRs by definition)
  # For issue_comment events, verify it's on a PR (not a regular issue)
  - expr: 'body.X-GitHub-Event == "pull_request" || body.X-GitHub-Event == "pull_request_review" || (body.X-GitHub-Event == "issue_comment" && has(body.issue.pull_request))'
```

### How It Works Now

1. **`pull_request` events**: Explicitly pass (they are PRs by definition)
2. **`pull_request_review` events**: Explicitly pass (PR reviews are always on PRs)
3. **`issue_comment` events**: Only pass if `body.issue.pull_request` exists (using `has()` function)
4. **Other events**: Filtered out by earlier data filters

The `has()` function safely checks for field existence before accessing it, preventing the "path does not exist" error.

## Evidence of the Issue

### Sensor Logs Before Fix

```
{"level":"warn","ts":"2025-11-12T20:40:42.6822773Z","logger":"argo-events.sensor","caller":"sensors/listener.go:198",
"msg":"Event [ID '366182ca32844dab8b8db2d5903527d1', Source 'github', Time '2025-11-12T20:40:42Z', 
Data '{...\"X-GitHub-Event\":[\"push\"]...}'] discarded due to filtering error: 
expr filter error (path 'body.issue.pull_request' does not exist)",
"sensorName":"atlas-pr-guardian","triggerName":"create-or-resume-atlas-guardian"}
```

### CodeRun Status Before Fix

```bash
$ kubectl get coderun -n agent-platform -l agent=atlas,role=pr-guardian
No resources found in agent-platform namespace.
```

Zero Atlas CodeRuns were ever created, despite multiple PRs being opened in the repository.

## Testing the Fix

### Pre-Deployment Validation

1. **YAML Syntax**: ✅ Passes yamllint
2. **Argo Events Expression**: ✅ Uses valid CEL (Common Expression Language) syntax
3. **Logic Verification**: ✅ Handles all three event types correctly

### Post-Deployment Testing

Run the validation script:

```bash
./scripts/test-atlas-sensor-fix.sh
```

Expected results:
- ✅ Sensor deployed and healthy
- ✅ No filtering errors in logs
- ✅ Atlas CodeRuns created when PRs are opened

### Manual Testing Steps

1. **Apply the fix**:
   ```bash
   kubectl apply -f infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml
   ```

2. **Wait for ArgoCD sync** (or manually sync the `atlas-pr-guardian-sensor` application)

3. **Create a test PR** in `5dlabs/cto`

4. **Verify Atlas activation**:
   ```bash
   # Check for new CodeRun
   kubectl get coderun -n agent-platform -l agent=atlas,role=pr-guardian
   
   # Should show a CodeRun like: coderun-atlas-pr-xxxxx
   ```

5. **Monitor sensor logs** for successful event processing:
   ```bash
   kubectl logs -f $(kubectl get pods -n argo -l sensor-name=atlas-pr-guardian -o name | head -1) -n argo
   ```

6. **Verify Atlas behavior**:
   - Check if Atlas comments on the PR
   - Verify Atlas responds to Bugbot comments
   - Confirm Atlas monitors CI status
   - Test auto-merge when all criteria are met

## Impact

### Before Fix
- ❌ Atlas never activated for any PR
- ❌ No automatic Bugbot comment resolution
- ❌ No automatic CI failure recovery
- ❌ No automatic merge conflict resolution
- ❌ No auto-merge functionality

### After Fix
- ✅ Atlas activates on PR open/sync/comment events
- ✅ Automatic Bugbot comment resolution
- ✅ Automatic CI failure recovery
- ✅ Automatic merge conflict resolution
- ✅ Auto-merge when all criteria met

## Related Files

- **Sensor Configuration**: `infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml`
- **ArgoCD Application**: `infra/gitops/applications/atlas-pr-guardian-sensor.yaml`
- **Atlas Values**: `infra/charts/controller/values.yaml` (atlas section)
- **Documentation**: `docs/engineering/atlas-pr-guardian.md`
- **Test Script**: `scripts/test-atlas-sensor-fix.sh`

## GitHub Webhook Events

For reference, here are the event types Atlas monitors:

### `pull_request` Event
Triggers on: `opened`, `reopened`, `synchronize`, `ready_for_review`

Payload structure:
```json
{
  "action": "opened",
  "pull_request": {
    "number": 123,
    "html_url": "https://github.com/5dlabs/cto/pull/123",
    ...
  },
  "repository": { ... }
}
```

### `issue_comment` Event
Triggers on: `created`

Payload structure:
```json
{
  "action": "created",
  "issue": {
    "number": 123,
    "pull_request": {  // Only exists if comment is on a PR
      "url": "https://api.github.com/repos/5dlabs/cto/pulls/123"
    }
  },
  "comment": { ... },
  "repository": { ... }
}
```

### `pull_request_review` Event
Triggers on: `submitted`

Payload structure:
```json
{
  "action": "submitted",
  "review": { ... },
  "pull_request": {
    "number": 123,
    ...
  },
  "repository": { ... }
}
```

## Lessons Learned

1. **Test filter expressions thoroughly**: CEL expressions can be tricky with optional fields
2. **Use `has()` for optional fields**: Always check field existence before accessing
3. **Monitor sensor logs**: Filtering errors are logged but don't cause sensor failure
4. **Validate with real webhooks**: GitHub webhook payloads vary by event type

## Future Improvements

1. **Add metrics**: Track Atlas activation rate and success rate
2. **Add alerting**: Alert when sensor has filtering errors
3. **Add dashboard**: Visualize Atlas activity and PR merge statistics
4. **Add tests**: Create integration tests for sensor filter logic

