# MCP Add Server Tool Remediation

## Status: ✅ Both Fixes Complete

### Summary
Both issues have been resolved:
1. **mcp-check partial fix** - Deployed (commit `3dbef85`)
2. **DNS race condition fix** - PR #3595 (adds DNS readiness check before git clone)

### Evidence Fix Works
- ✅ CodeRun created: `coderun-mcp-add-8q4fc`
- ✅ Phase: `Running`  
- ✅ Job created: `mcp-add-nano-banana-mcp-d169b76e-v1`
- ✅ Pod started: Container `claude-claude-sonnet-4-20250514` initialized
- ❌ Failed at git clone: `fatal: unable to access 'https://github.com/5dlabs/cto/': Could not resolve host: github.com`

## Original Issue (RESOLVED)

The `add_mcp_server` tool was failing due to a missing template partial in the controller.

### Problem
When creating a CodeRun via `add_mcp_server`, the controller fails with:
```
Error rendering "cli_execute" line 27, col 1: Partial not found mcp-check
```

### Root Cause
The `mcp-check.sh.hbs` partial exists in `/app/templates/_shared/partials/` but was not registered with the Handlebars template engine in `templates.rs`.

### Fix Applied
Commit `3dbef85` on `develop` branch added:
1. `PARTIAL_MCP_CHECK` constant in `template_paths.rs`
2. Registration of `("mcp-check", PARTIAL_MCP_CHECK)` in `templates.rs`

## Tasks to Complete

### 1. Monitor CI Build
```bash
# Check Controller CI status
gh run list --workflow=controller-ci.yaml --limit=1

# Watch specific run
gh run watch <run-id> --exit-status
```

**Expected**: All jobs pass, `build-and-push` completes successfully.

### 2. Deploy New Controller Image
Once CI passes, restart the controller deployment to pull the new image:
```bash
kubectl rollout restart deployment/cto-controller -n cto
kubectl rollout status deployment/cto-controller -n cto --timeout=120s
```

### 3. Verify Controller Startup
Check that the controller starts without errors:
```bash
kubectl logs -n cto deployment/cto-controller --tail=50 | grep -E "Starting|ERROR|partial"
```

**Expected**: `🚀 Starting 5D Labs Controller Service` with no partial errors.

### 4. Test MCP Add Server Tool
Create a new CodeRun to test the fix:
```bash
# Via MCP tool
add_mcp_server({ github_url: "https://github.com/ConechoAI/Nano-Banana-MCP" })
```

Or via kubectl:
```bash
kubectl get coderuns -n cto | grep mcp
kubectl describe coderun <name> -n cto
```

**Expected**: CodeRun should have a `phase` set and a Job should be created.

### 5. Monitor Agent Execution
```bash
# Check for jobs
kubectl get jobs -n cto | grep mcp

# Check pod logs
kubectl logs -n cto -l coderun=<coderun-name> --all-containers
```

**Expected**: Rex agent should:
1. Clone the `5dlabs/cto` repository
2. Read `infra/charts/cto/values.yaml`
3. Add the `nano-banana-mcp` server configuration
4. Create a PR targeting `develop`
5. Merge the PR after CI passes
6. Trigger ArgoCD sync for `cto-tools`

## Acceptance Criteria

- [ ] Controller CI builds successfully (`build-and-push` job completes)
- [ ] Controller deployment rolled out with new image
- [ ] Controller logs show no `Partial not found` errors
- [ ] `add_mcp_server` tool creates CodeRun with valid phase
- [ ] Job is created for the CodeRun
- [ ] Agent pod starts and runs successfully
- [ ] PR is created with `nano-banana-mcp` configuration
- [ ] PR is merged and `cto-tools` synced

## Previous CodeRuns to Clean Up

These CodeRuns failed due to the missing partial and can be deleted:
```bash
kubectl delete coderun coderun-mcp-add-slbdb -n cto
kubectl delete coderun coderun-mcp-add-rgd6m -n cto
```

## Context Files

- Fix commit: `3dbef85` on `develop`
- Template paths: `crates/controller/src/tasks/template_paths.rs`
- Template registration: `crates/controller/src/tasks/code/templates.rs`
- MCP check partial: `templates/_shared/partials/mcp-check.sh.hbs`
- MCP tool implementation: `crates/mcp/src/main.rs` (handle_add_mcp_server)

## CI Workflow
- Controller CI: `.github/workflows/controller-ci.yaml`
- Triggers on changes to `crates/controller/**`
- Builds and pushes `ghcr.io/5dlabs/controller:develop`

---

## ✅ DNS Race Condition (RESOLVED)

### Symptom
Agent pods were failing with:
```
fatal: unable to access 'https://github.com/5dlabs/cto/': Could not resolve host: github.com
```

### Root Cause
This was NOT a cluster networking issue. DNS works fine in the cluster. The problem was a **race condition** - the agent container started executing the git clone before the pod's network namespace was fully configured.

### Fix Applied
PR #3595 adds a DNS readiness check loop to `templates/_shared/partials/git-setup.sh.hbs`:
- Waits up to 60 seconds for DNS to be ready
- Uses `nslookup github.com` to verify DNS resolution
- Proceeds with git clone only after DNS is confirmed working

### Deployment Steps
1. Merge PR #3595 to `develop`
2. Wait for ArgoCD to sync the `cto-controller` application
3. Templates are stored in ConfigMap `controller-templates-shared`
4. Controller pod will pick up new templates on restart
