# Intake Job Naming & Repository URL Fix - Implementation Summary

## Issues Fixed

### 1. Intake Job Names Don't Indicate They're Intake Jobs ✅

**Problem**: Intake CodeRuns (e.g., `intake-prd-alerthub-e2e-tes-9tzmk`) created Jobs with generic names like `play-coderun-t0-morgan-claude-20bce7e0-v1`, making it impossible to identify intake jobs at a glance.

**Root Cause**: The `ResourceNaming::job_name()` function in [`crates/controller/src/tasks/code/naming.rs`](../../crates/controller/src/tasks/code/naming.rs) had special prefixes for:
- `review` → `review-` prefix
- `remediate` → `remediate-` prefix
- `heal` → `heal-remediation-` prefix
- `watch` → `monitor-` or `remediation-` prefix

But **no special handling for `runType: "intake"`**, so they fell through to the default `play-coderun-` prefix.

**Solution**: Added intake detection logic in [`crates/controller/src/tasks/code/naming.rs`](../../crates/controller/src/tasks/code/naming.rs):

```rust
// Handle intake tasks (Morgan PRD processing)
// Format: intake-t{task_id}-{agent}-{cli}-{uid}-v{version}
if run_type == "intake" {
    let cli = code_run.spec.cli_config.as_ref().map_or_else(
        || "unknown".to_string(),
        |config| config.cli_type.to_string(),
    );
    let base_name = format!("t{task_id}-{agent}-{cli}-{uid_suffix}-v{context_version}");
    let available = MAX_K8S_NAME_LENGTH.saturating_sub(INTAKE_JOB_PREFIX.len());
    let trimmed = Self::ensure_k8s_name_length(&base_name, available);
    return format!("{INTAKE_JOB_PREFIX}{trimmed}");
}
```

**Impact**: All future intake jobs will now have names like:
- `intake-t0-morgan-claude-20bce7e0-v1`

This makes it immediately clear that these are intake jobs when listing Jobs:
```bash
kubectl get jobs -n cto | grep intake-
```

### 2. Empty Repository URL Causes Immediate Failure ✅

**Problem**: Despite the CodeRun spec having `repositoryUrl: https://github.com/5dlabs/prd-alerthub-e2e-test` and the Pod env var being set correctly, the container script received an empty value, causing this error:

```
📄 Loading configuration...
  ✓ Repository: 
fatal: repository '' does not exist
```

**Root Cause**: Line 29 of [`templates/agents/morgan/intake.sh.hbs`](../../templates/agents/morgan/intake.sh.hbs) was reading `REPOSITORY_URL` from the intake ConfigMap JSON file:

```bash
REPOSITORY_URL=$(jq -r '.repository_url' "$CONFIG_FILE")
```

This **overwrote** the environment variable (which was correctly set by the controller) with an empty value from the ConfigMap.

**Solution**: Modified [`templates/agents/morgan/intake.sh.hbs`](../../templates/agents/morgan/intake.sh.hbs) to prioritize the environment variable:

```bash
# Use REPOSITORY_URL from environment if set, otherwise read from config
# The environment variable takes precedence as it's set by the controller
if [ -z "$REPOSITORY_URL" ]; then
    REPOSITORY_URL=$(jq -r '.repository_url' "$CONFIG_FILE")
fi

# Validate required fields
if [ -z "$REPOSITORY_URL" ] || [ "$REPOSITORY_URL" = "null" ]; then
    echo "❌ REPOSITORY_URL not set in environment or config file"
    exit 1
fi
```

**Impact**:
- Repository URL is now correctly read from environment variable
- Falls back to ConfigMap if env var not set (backward compatible)
- Fails fast with clear error if neither source provides a value
- Eliminates silent failures during git clone

## Files Changed

### Controller Code

1. **[`crates/controller/src/tasks/code/naming.rs`](../../crates/controller/src/tasks/code/naming.rs)**
   - Added `INTAKE_JOB_PREFIX` constant (`"intake-"`)
   - Updated doc comment to include intake format
   - Added intake detection logic (lines ~96-108)
   - Added test case `intake_job_name_has_correct_prefix`

### Templates

2. **[`templates/agents/morgan/intake.sh.hbs`](../../templates/agents/morgan/intake.sh.hbs)**
   - Modified repository URL loading to prioritize env var (lines 28-36)
   - Added validation for empty/null repository URL (lines 38-41)

## Testing

All tests pass successfully:

```bash
$ cargo test --package controller --lib
running 186 tests
...
test tasks::code::naming::tests::intake_job_name_has_correct_prefix ... ok
...
test result: ok. 185 passed; 0 failed; 1 ignored
```

### New Test Coverage

Added comprehensive test for intake naming in [`crates/controller/src/tasks/code/naming.rs`](../../crates/controller/src/tasks/code/naming.rs):

```rust
#[test]
fn intake_job_name_has_correct_prefix() {
    // Creates CodeRun with runType: "intake"
    // Verifies job name starts with "intake-"
    // Verifies name contains: task ID, agent, CLI type
    // Verifies length compliance
}
```

## Deployment Steps

### 1. Build New Controller Image

```bash
cd /Users/jonathonfritz/code/work-projects/5dlabs/cto
cargo build --release --bin agent-controller
docker build -t ghcr.io/5dlabs/controller:intake-fixes -f infra/images/controller/Dockerfile .
docker push ghcr.io/5dlabs/controller:intake-fixes
```

### 2. Deploy to Cluster

```bash
kubectl set image deployment/cto-controller \
  controller=ghcr.io/5dlabs/controller:intake-fixes \
  -n cto
```

### 3. Verify Deployment

```bash
# Watch rollout
kubectl rollout status deployment/cto-controller -n cto

# Check logs for startup diagnostics
kubectl logs deployment/cto-controller -n cto | grep -A 20 "Checking critical template partials"
```

### 4. Test with Failed CodeRun

Retry the failed intake CodeRun:

```bash
# Delete the failed Job/Pod
kubectl delete job play-coderun-t0-morgan-claude-20bce7e0-v1 -n cto

# Trigger reconciliation by annotating the CodeRun
kubectl annotate coderun intake-prd-alerthub-e2e-tes-9tzmk \
  reconcile-trigger="$(date +%s)" -n cto
```

### 5. Verify New Job Name

```bash
# Check that new job uses intake- prefix
kubectl get jobs -n cto | grep intake-

# Expected output:
# intake-t0-morgan-claude-<hash>-v1   ...
```

### 6. Verify Repository Clone

```bash
# Get the new Pod name
POD=$(kubectl get pods -n cto -l task-id=0 --sort-by=.metadata.creationTimestamp | tail -1 | awk '{print $1}')

# Check logs
kubectl logs $POD -n cto | grep "Cloning repository"

# Expected output:
# 📦 Cloning repository...
# ✓ Repository: https://github.com/5dlabs/prd-alerthub-e2e-test
```

## Verification Checklist

- [x] Controller compiles without errors
- [x] All tests pass (186 tests)
- [x] Clippy passes with pedantic warnings enabled
- [x] New intake naming test passes
- [x] Template syntax is valid (bash)
- [ ] Controller deployed to cluster
- [ ] New intake jobs use `intake-` prefix
- [ ] Repository URL correctly read from environment
- [ ] Intake workflow completes successfully

## Rollback Plan

If issues occur after deployment:

```bash
# Rollback controller deployment
kubectl rollout undo deployment/cto-controller -n cto

# Or specify exact revision
kubectl rollout history deployment/cto-controller -n cto
kubectl rollout undo deployment/cto-controller -n cto --to-revision=<prev>
```

## Benefits

### Intake Job Naming
- **Improved observability**: Intake jobs immediately identifiable in kubectl output
- **Better debugging**: Can filter logs/events by intake jobs specifically
- **Consistent naming**: Follows same pattern as review/remediate/heal jobs
- **Backward compatible**: Existing jobs continue to work

### Repository URL Fix
- **Fixes critical bug**: Intake jobs will no longer fail due to empty repository URL
- **Fail-fast validation**: Clear error message if repository URL is missing
- **Environment-first approach**: Respects controller-set environment variables
- **Backward compatible**: Falls back to ConfigMap if env var not set

## Related Issues

- Original incident: `intake-prd-alerthub-e2e-tes-9tzmk` failed with empty repository URL
- Failed Job: `play-coderun-t0-morgan-claude-20bce7e0-v1-qs5t7`
- Related to: [CodeRun Template Loading Fix](./coderun-template-fix-implementation.md)

## Notes

- This fix complements the template loading fix implemented earlier
- Both fixes are required for intake workflows to function properly
- The intake naming change is cosmetic but significantly improves operational visibility
- The repository URL fix addresses a critical runtime bug
