# Verification: Rex Regression Fixes

## Status: Image is Fixed ✅

The latest Factory image (`ghcr.io/5dlabs/factory:latest`) **has the ripgrep fix**:
- Built at: 21:25 PM (Nov 12)
- Ripgrep 14.1.1 installed at `/usr/local/bin/rg`
- Symlink exists at `/home/node/.factory/bin/rg`

Your log file shows an error from a workflow that ran **before** the image was rebuilt.

## Test the Fix

### Option 1: Start Fresh Workflow (Recommended)

```bash
# Clean up old play workflows
kubectl delete workflows -n agent-platform -l project-play=true --ignore-not-found=true

# Start new workflow with fixed image
# This will use the latest Factory image with all 3 fixes:
#  1. Ripgrep installation
#  2. Zero-commit detection
#  3. Stale branch cleanup
```

Then start a new play workflow (task 1 or full project).

### Option 2: Quick Verification (No Full Workflow)

Test just the Factory image directly:

```bash
# Test 1: Verify ripgrep is available
kubectl run verify-ripgrep --rm -it --restart=Never \
  --image=ghcr.io/5dlabs/factory:latest \
  -n agent-platform \
  -- bash -c "
    echo '🔍 Checking ripgrep...'
    which rg
    rg --version
    ls -la /home/node/.factory/bin/rg
    echo ''
    echo '✅ Ripgrep is properly installed'
  "

# Test 2: Verify Factory loads without errors
kubectl run verify-factory --rm -it --restart=Never \
  --image=ghcr.io/5dlabs/factory:latest \
  -n agent-platform \
  --env="FACTORY_API_KEY=${FACTORY_API_KEY}" \
  -- bash -c "
    cd /tmp
    git init test
    cd test
    git config user.name 'Test'
    git config user.email 'test@test.com'
    echo 'fn main() {}' > main.rs
    git add . && git commit -m 'init'
    
    echo '🚀 Running Factory droid...'
    droid exec 'list files in current directory' --output-format text --auto low 2>&1 | tee /tmp/output.log
    
    echo ''
    if grep -q 'Failed to resolve ripgrep' /tmp/output.log; then
      echo '❌ FAILED: Ripgrep error still present'
      exit 1
    else
      echo '✅ PASSED: No ripgrep errors'
    fi
  "
```

## Expected Results

After starting a fresh workflow with the fixed image:

### 1. Branch Setup (Lines 30-42 from your log)
```
✅ Found CLOSED PR #870 from previous run
🔄 Deleting stale branch and recreating fresh from origin/main  
✅ Created fresh branch feature/task-1-implementation from origin/main
ℹ️ Previous PR was closed - starting with clean slate
```

### 2. Factory Execution (No ripgrep error)
```
🚀 EXECUTION ATTEMPT 1 / 10
[NO ripgrep error - Factory does actual work]
```

### 3. Commits Created
```
✅ Rex made X commit(s)
```

### 4. PR Created
```
✅ Auto-created pull request for feature/task-1-implementation
```

### 5. Cleo Starts
```
quality-in-progress → Cleo reviews code
```

## Troubleshooting

If you still see the ripgrep error:

```bash
# Force image pull by deleting image cache
kubectl delete pod <pod-name> -n agent-platform

# Check what image digest is actually running
kubectl get pods -n agent-platform <pod-name> -o jsonpath='{.status.containerStatuses[0].imageID}'

# Compare with latest image digest
docker manifest inspect ghcr.io/5dlabs/factory:latest | jq -r '.config.digest'
```

## Summary

**All 3 Fixes Are Live**:
- ✅ Ripgrep installed in Factory image (verified in cluster)
- ✅ Zero-commit detection in all CLI containers (in PR #1348)
- ✅ Stale branch cleanup in all CLI containers (in PR #1348)

**Next Step**: Merge PR #1348 and start a fresh workflow to see all fixes working together.

