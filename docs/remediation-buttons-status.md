# Remediation Buttons - Implementation Status

**Date:** 2026-01-30
**Branch:** `feat/remediation-buttons-phase-a`

## Overview

This work implements the foundation for CI remediation buttons - allowing users to click a button on a failed GitHub check run to trigger an AI agent to fix the issue.

## ✅ Completed Work

### 1. Detection Module (`crates/pm/src/detection/`)

A comprehensive detection system for identifying language, framework, and appropriate agent from PR file paths.

**Files Created:**
- `crates/pm/src/detection/mod.rs` - Main detection orchestration (275 lines)
- `crates/pm/src/detection/language.rs` - Language detection from extensions (150 lines)
- `crates/pm/src/detection/framework.rs` - Framework detection from package files (375 lines)
- `crates/pm/src/detection/agent.rs` - Agent selection logic (264 lines)

**Features:**
- Language detection: Rust, Go, TypeScript, C# (Unity), C++ (Unreal)
- Framework detection: Axum, Tokio, Chi, Effect-TS, Elysia, Express, Next.js, React Native, Expo, Electron
- TypeScript disambiguation:
  - `expo-router` → Tap (mobile)
  - `electron` → Spark (desktop)
  - `elysia`/`express` → Nova (backend)
  - `next`/`react` → Blaze (web)
- 26 unit tests passing

### 2. Agent-Mention Sensor Fix

Fixed the Argo Events sensor to properly trigger CodeRuns on @mentions.

**File Modified:**
- `infra/gitops/manifests/argo-workflows/sensors/agent-mention-sensor.yaml`

**Changes:**
- Simplified trigger template for Argo Events compatibility
- Removed overly complex resource templates
- Now successfully creates CodeRuns on @mention comments

### 3. Controller Configuration Fixes

**Files Modified:**
- `infra/charts/cto/values.yaml` - Changed default agent image from `ghcr.io/5dlabs/agent:latest` (doesn't exist) to `ghcr.io/5dlabs/claude:dev`
- `infra/charts/cto/templates/controller/task-controller-config.yaml` - Added `stitch` agent with GitHub App config

### 4. Template Fixes

**Files Modified:**
- `templates/_shared/partials/github-auth.sh.hbs` - Fixed `CURL_MAX_RETRIES: unbound variable` error by defining variable before conditional blocks
- `templates/agents/stitch/review.md.hbs` - Enhanced with:
  - Detection-first workflow (STEP 1)
  - Context7 integration guidance (STEP 2)
  - Must use `gh` CLI for posting reviews (identity fix)
  - Detection summary in review output

### 5. Test Fixtures

Created 8 test PRs for each agent type:

| PR | Agent | Language/Framework | Status |
|----|-------|-------------------|--------|
| #4113 | Rex | Rust/Axum | Open |
| #4114 | Blaze | TypeScript/Next.js | Open |
| #4115 | Nova | TypeScript/Elysia | Open |
| #4116 | Grizz | Go/Chi | Open |
| #4117 | Tap | TypeScript/Expo | Open |
| #4118 | Spark | TypeScript/Electron | Open |
| #4119 | Vex | C#/Unity | Open |
| #4120 | Forge | C++/Unreal | Open |

## ⚠️ Known Issues

### 1. GitHub App Identity for Reviews

**Problem:** Stitch reviews are still posted as `kaseonedge` instead of `5DLabs-Stitch[bot]`

**Root Cause:** MCP tools server uses a separate `GITHUB_TOKEN` from the agent container. Even though the agent generates a token from the Stitch GitHub App, the MCP `github_create_pull_request_review` tool uses the tools server's credentials.

**Mitigation Applied:** Updated Stitch template to use `gh` CLI instead of MCP tools for posting reviews. The `gh` CLI uses the container's `GITHUB_TOKEN` which is correctly set from the Stitch GitHub App.

**Status:** Template updated, but not yet verified working due to cluster resource constraints.

### 2. Cluster Resource Constraints

**Problem:** Pods frequently stuck in `Pending` state with "Insufficient cpu" errors.

**Root Cause:** 
- Worker node at 97-99% CPU requests
- GitHub Actions runners consume 4 CPU (8 runners × 500m each)
- Agent pods require 500m CPU each

**Workaround:** Manually delete GitHub runners to free resources: `kubectl delete pods -n arc-runners --all`

### 3. Workspace PVC Corruption

**Problem:** Some runs fail with `fatal: .git/index: index file smaller than expected`

**Root Cause:** Previous failed runs leave workspace in corrupted state.

**Workaround:** Delete the workspace PVC: `kubectl delete pvc workspace-cto-stitch -n cto`

## 🚧 Outstanding Work

### Phase B: Button Rendering

When a GitHub check run fails, render a "Fix with @Agent" button in the check run summary.

**Tasks:**
1. Implement `check_run` webhook handler in pm-server
2. On failure, detect language/framework from PR files
3. Generate GitHub Check Run annotation with remediation button
4. Button click triggers new comment with @mention

### Phase C: Click Handler

Handle button clicks to trigger remediation.

**Tasks:**
1. Implement check run "requested_action" webhook handler
2. Extract PR number and agent from action identifier
3. Create CodeRun for the appropriate agent
4. Agent fixes issue and pushes commit

### Phase D: Detection Integration

Integrate detection into Stitch review initialization.

**Tasks:**
1. Pass detection results to Stitch via template variables
2. Stitch queries Context7 for detected frameworks
3. Review output includes detection summary
4. Remediation suggestion uses detected agent name

## File Summary

### New Files
```
crates/pm/src/detection/mod.rs
crates/pm/src/detection/language.rs
crates/pm/src/detection/framework.rs
crates/pm/src/detection/agent.rs
docs/remediation-buttons-status.md (this file)
tests/stitch-agents/rust-rex/
tests/stitch-agents/web-blaze/
tests/stitch-agents/node-nova/
tests/stitch-agents/go-grizz/
tests/stitch-agents/mobile-tap/
tests/stitch-agents/desktop-spark/
tests/stitch-agents/unity-vex/
tests/stitch-agents/unreal-forge/
```

### Modified Files
```
crates/pm/src/lib.rs (added detection module export)
infra/charts/cto/values.yaml (agent image fix)
infra/charts/cto/templates/controller/task-controller-config.yaml (stitch agent)
infra/gitops/manifests/argo-workflows/sensors/agent-mention-sensor.yaml
templates/_shared/partials/github-auth.sh.hbs (CURL_MAX_RETRIES fix)
templates/agents/stitch/review.md.hbs (detection + gh CLI)
```

## Testing Commands

```bash
# Trigger Stitch review on a PR
gh pr comment 4113 --body "@5DLabs-Stitch review this code"

# Watch CodeRun status
kubectl get coderun -n cto -w

# Check pod status
kubectl get pods -n cto | grep stitch

# View agent logs
kubectl logs -n cto <pod-name> -f

# Free up cluster resources
kubectl delete pods -n arc-runners --all

# Delete corrupted workspace
kubectl delete pvc workspace-cto-stitch -n cto
```

## Next Steps for Handoff

1. **Verify Stitch identity fix** - Post a review and confirm it's from `5DLabs-Stitch[bot]`
2. **Implement Phase B** - Check run failure → button rendering
3. **Add check_run webhook handler** to pm-server
4. **Test full flow** - CI failure → button → click → agent fixes → PR updated
