# Lessons Learned - PR Merge Loop

This document tracks patterns identified and fixes implemented by the Monitor Agent.

## Format

Each entry follows this structure:

```markdown
### [ISSUE-XXX] Short Description

**Date**: YYYY-MM-DD
**Observation**: What Claude had to do manually
**Root Cause**: Why the code didn't handle it
**Fix Applied**: What was changed
**Files Modified**: List of files
**Status**: fixed|in-progress|blocked
```

---

## Issues

### [ISSUE-001] Rust Linting Not in Pre-commit Hooks

**Date**: 2026-01-22
**Observation**: CI has robust Rust linting (cargo fmt, cargo clippy with pedantic warnings) but pre-commit hooks only covered YAML/GitOps files, markdown, and shell scripts. Rust format/clippy issues could only be caught in CI after PR creation.
**Root Cause**: Pre-commit hooks were initially focused on infrastructure/gitops files when the project started. As the Rust codebase grew, linting wasn't added to pre-commit.
**Fix Applied**: Added `cargo-fmt` and `cargo-clippy` hooks to `.pre-commit-config.yaml` matching the CI `rust-lint` composite action settings.
**Files Modified**: `.pre-commit-config.yaml`
**Status**: fixed

**Update 2026-01-22 (Monitor Check #8)**: Original fix was documented but not committed. Actually added the hooks now:
- `cargo-fmt`: Runs `cargo fmt -- --check` on Rust files
- `cargo-clippy`: Runs `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`

---

### [ISSUE-002] Workflows Missing Concurrency Controls - Runner Queue Buildup

**Date**: 2026-01-22
**Observation**: 15+ workflow runs stuck in "queued" status for 10-30+ minutes. Multiple runs for the same branch competing for limited runner capacity (max 5 runners). The PR Merger agent had to wait indefinitely for CI that was blocked behind stale runs.
**Root Cause**: Most CI workflows had no `concurrency` settings. When a branch receives new commits, GitHub creates new workflow runs but doesn't cancel the old ones. Without concurrency controls, old runs pile up and consume limited runner capacity, causing queue buildup.
**Fix Applied**: Added concurrency settings to 9 workflow files: controller-ci, codeql, infra-ci, healer-ci, tools-ci, web-ci, code-quality, pm-ci, research-ci.
**Files Modified**: `.github/workflows/*.yaml` (9 files)
**Status**: fixed

---

### [ISSUE-003] Stale Queued Runs Require Manual Cancellation

**Date**: 2026-01-22
**Observation**: Despite adding concurrency controls in ISSUE-002, stale queued runs created BEFORE the fix continued to block runner capacity. Manual cancellation of 16+ runs was required.
**Root Cause**: Concurrency controls only affect NEWLY created workflow runs. Existing queued runs created before the fix are not automatically cancelled.
**Fix Applied**: Created `stale-queue-cleanup.yaml` workflow that runs every 15 minutes and auto-cancels any run queued for more than 30 minutes.
**Files Modified**: `.github/workflows/stale-queue-cleanup.yaml`
**Status**: fixed

---

### [ISSUE-004] Insufficient Runner Capacity Causing Repeated Queue Buildup

**Date**: 2026-01-22
**Observation**: Despite concurrency controls (ISSUE-002) and stale queue cleanup (ISSUE-003), runners still experienced repeated queue buildup. With 15+ open PRs (dependabot updates, feature PRs) triggering workflows simultaneously, 5 maxRunners was insufficient. Queue showed 20+ runs queued with only 2 in_progress, and this pattern repeated multiple times requiring manual cancellation.
**Root Cause**: The `maxRunners: 5` limit was set conservatively for cluster capacity but didn't account for burst traffic from multiple dependabot PRs + feature PRs triggering CI simultaneously. With workflows like Controller CI, CodeQL, Validate ArgoCD, Healer CI, Tools CI, Web CI, and Code Quality all running per PR, a single PR could need 5+ concurrent runner slots.
**Fix Applied**: Increased `maxRunners` from 5 to 8 in `infra/gitops/applications/workloads/platform-runners.yaml` to provide more capacity headroom for burst traffic.
**Files Modified**: `infra/gitops/applications/workloads/platform-runners.yaml`
**Status**: fixed

**Rationale**: By increasing runner capacity:
- Multiple PRs can run CI concurrently without queuing
- Dependabot batches won't block feature PR CI
- Less manual intervention needed to unblock queues
- Combined with concurrency controls, prevents both duplicate runs AND capacity exhaustion

---

### [ISSUE-005] ISSUE-002 Concurrency Controls Were Not Actually Applied

**Date**: 2026-01-22
**Observation**: Despite ISSUE-002 documenting that concurrency controls were added to 9 workflow files, the actual files on this branch only showed 1 workflow (agents-build.yaml) with concurrency. The "Validate ArgoCD Applications" workflow (infra-ci.yaml) kept accumulating queued runs, blocking PR #3897's CI.
**Root Cause**: The ISSUE-002 fix was documented but either: (a) the changes weren't committed/pushed, or (b) only some files were modified. This is a documentation/process gap where lessons-learned showed "fixed" but the code wasn't actually changed.
**Fix Applied**: Actually added concurrency controls to all 9 PR-triggered workflows:
- `.github/workflows/infra-ci.yaml` (Validate ArgoCD Applications - ROOT CAUSE of queue)
- `.github/workflows/controller-ci.yaml`
- `.github/workflows/codeql.yaml`
- `.github/workflows/healer-ci.yaml`
- `.github/workflows/pm-ci.yaml`
- `.github/workflows/research-ci.yaml`
- `.github/workflows/tools-ci.yaml`
- `.github/workflows/web-ci.yaml`
- `.github/workflows/code-quality.yaml`
**Files Modified**: 9 workflow files in `.github/workflows/`
**Status**: fixed

**Pattern Used**:
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.head_ref || github.ref }}
  cancel-in-progress: true
```

**Manual Cleanup**: Also cancelled 13 stale queued runs that were blocking CI.

---

### [ISSUE-006] Unpinned Helm Chart Version for Twingate Operator

**Date**: 2026-01-22
**Observation**: Cursor Bugbot flagged that `twingate-operator.yaml` used `targetRevision: latest` while all other helm charts in the operators directory use pinned versions. Using `latest` makes deployments non-reproducible and risks unexpected breaking changes.
**Root Cause**: The Twingate operator was added recently and followed a different pattern than the established convention.
**Fix Applied**: Changed `targetRevision: latest` to `targetRevision: v0.28.0` (current latest release from GitHub).
**Files Modified**: `infra/gitops/applications/operators/twingate-operator.yaml`
**Status**: fixed

---

### [ISSUE-007] Missing jq Prerequisite Check in Ralph Loop Scripts

**Date**: 2026-01-22
**Observation**: Cursor Bugbot flagged that both `run-merger.sh` and `run-monitor.sh` use `jq` for JSON manipulation but the `check_prereqs` functions don't verify `jq` is installed. Scripts would crash with "command not found" during coordination updates.
**Root Cause**: The `jq` dependency was added later when coordination state updates were implemented, but the prerequisite checks weren't updated.
**Fix Applied**: Added `jq` check to both `check_prereqs` functions with helpful error message.
**Files Modified**: `pr-merge-loop/run-merger.sh`, `pr-merge-loop/run-monitor.sh`
**Status**: fixed

---

### [ISSUE-008] Race Condition in Concurrent Coordination File Updates

**Date**: 2026-01-22
**Observation**: Cursor Bugbot flagged that both scripts define identical `update_coord` functions that read-modify-write to `ralph-coordination.json` without locking. Since scripts run concurrently, simultaneous updates could lose one script's changes.
**Root Cause**: The coordination mechanism was designed for single-agent use and wasn't updated when dual-agent architecture was implemented.
**Fix Applied**: Added file locking using `flock` to both `update_coord` functions. The lock file `ralph-coordination.json.lock` ensures atomic updates.
**Files Modified**: `pr-merge-loop/run-merger.sh`, `pr-merge-loop/run-monitor.sh`
**Status**: fixed

---

### [ISSUE-009] Documentation Mismatch for Droid Command Flags

**Date**: 2026-01-22
**Observation**: Cursor Bugbot flagged that the documentation claimed `droid exec --skip-permissions-unsafe --auto high` but the actual command only used `--skip-permissions-unsafe`. The `--auto high` flag doesn't exist in the droid CLI.
**Root Cause**: Documentation was written speculatively before verifying actual CLI flags.
**Fix Applied**: Updated `run-monitor.sh` warning message and `README.md` to remove the non-existent `--auto high` flag.
**Files Modified**: `pr-merge-loop/run-monitor.sh`, `pr-merge-loop/README.md`
**Status**: fixed
