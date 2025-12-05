# Play Workflow Flow Analysis

> **Document Version**: 1.0  
> **Date**: November 27, 2025  
> **Status**: Post-Refactor Audit

## Executive Summary

This document provides a comprehensive analysis of the Play workflow system after recent refactoring. The audit identified **3 orphaned sensors** that reference non-existent suspend points, as well as validated the correct functioning of Atlas guardian and integration capabilities.

---

## Table of Contents

1. [Workflow Overview](#workflow-overview)
2. [Agent Flow: Rex → Cleo → Cipher → Tess](#agent-flow-rex--cleo--cipher--tess)
3. [Suspend Points & Event Handling](#suspend-points--event-handling)
4. [Atlas Guardian System](#atlas-guardian-system)
5. [CI Failure Remediation](#ci-failure-remediation)
6. [Identified Issues](#identified-issues)
7. [Sensor Reference](#sensor-reference)

---

## Workflow Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         PLAY WORKFLOW ARCHITECTURE                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌──────────────┐     ┌──────────────────────────────────────────────────────┐  │
│  │   GitHub     │────▶│  EventSource (github)                                │  │
│  │   Webhooks   │     │  namespace: automation                               │  │
│  │              │     │  endpoint: /github/webhook:12000                     │  │
│  └──────────────┘     │  events: ["*"] from 5dlabs org                       │  │
│                       └──────────────────────────────────────────────────────┘  │
│                                          │                                       │
│                                          ▼                                       │
│  ┌───────────────────────────────────────────────────────────────────────────┐  │
│  │                          SENSORS (namespace: automation)                   │  │
│  │                                                                            │  │
│  │  ┌─────────────────────┐  ┌─────────────────────┐  ┌───────────────────┐  │  │
│  │  │ stage-aware-*       │  │ atlas-*-monitor     │  │ ci-remediation    │  │  │
│  │  │ (workflow resume)   │  │ (guardian/conflict) │  │ (failure fixes)   │  │  │
│  │  └─────────────────────┘  └─────────────────────┘  └───────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
│                                          │                                       │
│                                          ▼                                       │
│  ┌───────────────────────────────────────────────────────────────────────────┐  │
│  │                     Argo Workflows (namespace: cto)                        │  │
│  │                                                                            │  │
│  │  ┌────────────────────────────────────────────────────────────────────┐   │  │
│  │  │                    Play Workflow Template                           │   │  │
│  │  │    (main DAG → agent-sequence → CodeRun creation → suspends)       │   │  │
│  │  └────────────────────────────────────────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
│                                          │                                       │
│                                          ▼                                       │
│  ┌───────────────────────────────────────────────────────────────────────────┐  │
│  │                      CodeRun CRD (namespace: cto)                          │  │
│  │         Creates agent pods that interact with GitHub via Apps              │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Agent Flow: Rex → Cleo → Cipher → Tess

### Sequence Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    AGENT SEQUENCE (continuous execution)                         │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌───────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                            │  │
│  │    REX (Implementation)                                                    │  │
│  │    ├── GitHub App: 5DLabs-Rex                                              │  │
│  │    ├── Stage: implementation-in-progress                                   │  │
│  │    ├── Role: Creates PR with implementation                                │  │
│  │    └── Polls for PR existence before proceeding                            │  │
│  │                          │                                                 │  │
│  │                          ▼                                                 │  │
│  │    CLEO (Quality Review)                                                   │  │
│  │    ├── GitHub App: 5DLabs-Cleo                                             │  │
│  │    ├── Stage: quality-in-progress                                          │  │
│  │    ├── Role: Code quality review, suggestions                              │  │
│  │    └── continueOn.failed: true (workflow proceeds even if issues found)    │  │
│  │                          │                                                 │  │
│  │                          ▼                                                 │  │
│  │    CIPHER (Security Review) [OPTIONAL]                                     │  │
│  │    ├── GitHub App: 5DLabs-Cipher                                           │  │
│  │    ├── Stage: security-in-progress                                         │  │
│  │    ├── Role: Security scanning, vulnerability detection                    │  │
│  │    ├── Skipped if: security-agent/cli/model not configured                 │  │
│  │    └── continueOn.failed: true                                             │  │
│  │                          │                                                 │  │
│  │                          ▼                                                 │  │
│  │    TESS (Testing)                                                          │  │
│  │    ├── GitHub App: 5DLabs-Tess                                             │  │
│  │    ├── Stage: testing-in-progress                                          │  │
│  │    ├── Role: E2E testing, validation                                       │  │
│  │    └── continueOn.failed: true                                             │  │
│  │                                                                            │  │
│  │    ⚠️  NO SUSPEND POINTS BETWEEN AGENTS                                    │  │
│  │    Agents run continuously without webhook-based resume                    │  │
│  │                                                                            │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Why No Inter-Agent Suspends?

From the workflow template comments:

```yaml
# Note: No longer suspending for PR created event - implementation-cycle already
# verified PR exists via check-or-wait-for-pr polling. Webhook-based resume was
# creating race conditions where PR webhook fired before suspend point was reached.
```

```yaml
# Proceed directly to Tess testing after Cipher completes
# No suspension needed - Tess runs automatically after security review
# continueOn.failed allows workflow to complete even if Tess finds issues
```

---

## Suspend Points & Event Handling

### Active Suspend Points (2 total)

After the agent sequence completes, the workflow has **only 2 suspend points**:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            POST-AGENT SUSPEND FLOW                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  Agent Sequence Complete                                                         │
│           │                                                                      │
│           ▼                                                                      │
│  ┌────────────────────────────────────────────────────────────────────────┐     │
│  │  SUSPEND #1: wait-for-atlas-integration                                 │     │
│  │  ├── Stage Label: waiting-atlas-integration                             │     │
│  │  ├── Event Type: atlas-integration                                      │     │
│  │  ├── Resumed By: stage-aware-tess-approval sensor                       │     │
│  │  │               (triggers on Tess PR approval)                         │     │
│  │  └── Logic:                                                             │     │
│  │      • If single approved PR with no conflicts → skip Atlas             │     │
│  │      • If multiple approved PRs → launch Atlas batch integration        │     │
│  │      • If merge conflicts → launch Atlas conflict resolution            │     │
│  └────────────────────────────────────────────────────────────────────────┘     │
│           │                                                                      │
│           ▼                                                                      │
│  ┌────────────────────────────────────────────────────────────────────────┐     │
│  │  SUSPEND #2: wait-merge-to-main                                         │     │
│  │  ├── Stage Label: waiting-pr-merged                                     │     │
│  │  ├── Event Type: pr-merged                                              │     │
│  │  ├── Resumed By: stage-aware-pr-merged sensor                           │     │
│  │  │               (triggers on PR merge to main)                         │     │
│  │  └── Logic:                                                             │     │
│  │      • Waits for PR to be merged                                        │     │
│  │      • Atlas may auto-merge if approved                                 │     │
│  │      • Human may merge manually                                         │     │
│  └────────────────────────────────────────────────────────────────────────┘     │
│           │                                                                      │
│           ▼                                                                      │
│      TASK COMPLETE                                                               │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Stage Progression

```
pending
    └── implementation-in-progress
            └── quality-in-progress
                    └── security-in-progress (optional)
                            └── testing-in-progress
                                    └── waiting-atlas-integration
                                            └── waiting-pr-merged
                                                    └── completed
```

---

## Atlas Guardian System

Atlas operates in two modes for the CTO repository:

### 1. Guardian Mode (PR Monitoring)

**Sensors**: `atlas-pr-monitor`, `atlas-conflict-monitor`

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          ATLAS GUARDIAN MODE                                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  Triggers:                                                                       │
│  ├── PR opened/reopened/ready_for_review/synchronize                            │
│  └── Merge conflict detected (mergeable=false or state=dirty/unstable)          │
│                                                                                  │
│  Creates CodeRun with:                                                           │
│  ├── agent: atlas                                                                │
│  ├── role: guardian                                                              │
│  ├── ATLAS_MODE: "guardian"                                                      │
│  ├── ATLAS_POLL_INTERVAL: 60s (PR monitor) / 45s (conflict monitor)             │
│  └── ATLAS_MAX_CYCLES: 0 (unlimited) / 120 (conflict)                           │
│                                                                                  │
│  Responsibilities:                                                               │
│  ├── Monitor PR for merge conflicts                                              │
│  ├── Auto-rebase when main branch updates                                        │
│  ├── Resolve merge conflicts                                                     │
│  └── Keep PR in mergeable state                                                  │
│                                                                                  │
│  Safety:                                                                         │
│  ├── Lock mechanism prevents duplicate guardians per PR                          │
│  └── Skips if Atlas triggered the event (loop prevention)                        │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2. Integration Mode (Batch Coordination)

**Sensors**: `atlas-batch-integration`, `stage-aware-tess-approval`

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        ATLAS INTEGRATION MODE                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  Triggers:                                                                       │
│  ├── Tess approval on PR (via stage-aware-tess-approval)                        │
│  │   └── Only if multiple approved PRs exist OR merge conflicts                 │
│  └── Comment containing "batch" or "play workflow complete"                      │
│                                                                                  │
│  Creates CodeRun with:                                                           │
│  ├── agent: atlas                                                                │
│  ├── role: integration                                                           │
│  ├── ATLAS_MODE: "integration-gate"                                              │
│  ├── ATLAS_POLL_INTERVAL: 30s                                                    │
│  └── ATLAS_MAX_CYCLES: 120-240                                                   │
│                                                                                  │
│  Responsibilities:                                                               │
│  ├── Coordinate batch integration of approved PRs                                │
│  ├── Ensure PRs are merged in correct order                                      │
│  ├── Handle dependencies between PRs                                             │
│  └── Resume play workflows after integration completes                           │
│                                                                                  │
│  Skip Conditions:                                                                │
│  ├── Single approved PR with no conflicts → direct merge path                    │
│  └── Workflow sets stage to "ready-for-merge" and resumes                        │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## CI Failure Remediation

### Architecture

> **Note**: CI failure remediation uses **Rex**, not Atlas.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        CI FAILURE REMEDIATION                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  Sensor: ci-failure-remediation                                                  │
│                                                                                  │
│  Trigger:                                                                        │
│  ├── workflow_run event with conclusion=failure                                  │
│  ├── Repository: 5dlabs/cto only                                                 │
│  └── Excludes: commits with [skip-ci-remediation]                                │
│                                                                                  │
│  Creates CodeRun with:                                                           │
│  ├── agent: rex                                                                  │
│  ├── role: ci-remediation                                                        │
│  ├── REMEDIATION_MODE: "ci-failure"                                              │
│  ├── WORKFLOW_NAME, WORKFLOW_RUN_ID, WORKFLOW_RUN_URL                            │
│  └── FAILURE_BRANCH, FAILURE_COMMIT_SHA, FAILURE_COMMIT_MESSAGE                  │
│                                                                                  │
│  Workflow Coverage:                                                              │
│  ├── Infrastructure Images (Docker builds)                                       │
│  ├── Controller CI (Rust clippy, tests)                                          │
│  ├── Agent Templates Check                                                       │
│  ├── Markdown Lint                                                               │
│  └── Helm Publish                                                                │
│                                                                                  │
│  Process:                                                                        │
│  1. Fetch workflow logs via GitHub CLI                                           │
│  2. Identify failure type (build, test, lint, permission)                        │
│  3. Apply minimal, focused fix                                                   │
│  4. Create fix branch/PR                                                         │
│  5. Document root cause in commit                                                │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Bug Comment / Feedback Remediation

> **Note**: Bug comment remediation also uses **Rex**, not Atlas.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      FEEDBACK REMEDIATION                                        │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  Sensor: remediation-feedback-sensor                                             │
│                                                                                  │
│  Triggers (multiple dependencies):                                               │
│  ├── issue_comment with patterns: 🔴, "fix required", "needs changes"            │
│  ├── pull_request_review with state: changes_requested                           │
│  ├── check_run from Tess with conclusion: action_required/failure                │
│  └── Manual: @5dlabs-rex remediate or /remediate command                         │
│                                                                                  │
│  Creates CodeRun with:                                                           │
│  ├── agent: rex (via 5DLabs-Rex GitHub App)                                      │
│  ├── REMEDIATION_MODE: "true"                                                    │
│  ├── FEEDBACK_TYPE: comment/review/check_run/manual                              │
│  └── continueSession: true (maintains context)                                   │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Identified Issues

### 🚨 Critical: Orphaned Sensors

The following sensors reference suspend nodes that **no longer exist** in the workflow template:

| Sensor | Missing Node | Expected Stage | Impact |
|--------|-------------|----------------|--------|
| `stage-aware-cleo-approval` | `wait-for-cleo-approval` | `waiting-quality-complete` | Never matches |
| `play-workflow-ready-for-security` | `wait-for-cipher-approval` | `security-in-progress` | Never matches |
| `play-workflow-ready-for-qa` | `wait-for-tess-approval` | `waiting-ready-for-qa` | Never matches |

**Root Cause**: The workflow was refactored to remove inter-agent suspend points due to webhook race conditions, but the corresponding sensors were not updated or removed.

**Effect**: These sensors process GitHub events, search for workflows at specific stages with specific suspend nodes, and exit early when they find no matches. They consume resources but perform no useful work.

**Recommendation**: Remove these orphaned sensors or repurpose them if needed for a different flow.

### ⚠️ Potential Issue: Atlas PR Monitor Design

There's a `.disabled` version of `atlas-pr-monitor-sensor.yaml` with this comment:

```
# Atlas PR Monitor Sensor - DISABLED
# Atlas should only trigger after Tess approval, not on every PR event
# This sensor was causing Atlas to run continuously and create Task 0
# New design: Atlas triggers via stage-aware-tess-approval-sensor
```

**However**, the active `atlas-pr-monitor-sensor.yaml` IS included in the kustomization and will trigger on **every PR event** (opened, reopened, synchronize, ready_for_review) on the cto repository.

**Potential Issues**:
1. Creates CodeRuns with `taskId: 0` for non-play-workflow PRs
2. May create unnecessary resource usage
3. Conflicts with stated design (Atlas after Tess approval only)

**Current Mitigations**:
- Lock mechanism prevents duplicate guardians per PR
- Checks for existing active CodeRuns before creating new ones
- Skips events from Atlas itself

**Recommendation**: Review whether `atlas-pr-monitor-sensor.yaml` should be disabled in favor of the Tess-approval-triggered flow, or if it serves a distinct purpose (e.g., monitoring PRs not part of play workflows).

### ⚠️ Potential Overlap: Conflict Monitor

Both `atlas-pr-monitor` and `atlas-conflict-monitor` trigger on the same events:
- `pull_request.opened`
- `pull_request.reopened`
- `pull_request.synchronize`
- `pull_request.ready_for_review`

**Mitigation**: Lock mechanism (`atlas-guardian-lock-$PR_NUMBER`) prevents duplicate CodeRuns.
**Differentiation**: `atlas-conflict-monitor` only proceeds if `mergeable=false` or `mergeable_state=dirty/unstable/behind`.

### ℹ️ Design Clarification: Remediation Responsibilities

Current design:
- **Atlas**: Guardian (conflict resolution), Integration (batch coordination)
- **Rex**: CI failure fixes, bug comment remediation

If Atlas should handle bug comments, the `remediation-feedback-sensor` would need to be updated to create Atlas CodeRuns instead of Rex CodeRuns.

---

## Sensor Reference

### Working Sensors ✅

| Sensor | Trigger | Action | Status |
|--------|---------|--------|--------|
| `stage-aware-tess-approval` | Tess approves PR | Launch Atlas or resume workflow | ✅ Works |
| `stage-aware-pr-merged` | PR merged to main | Resume at `waiting-pr-merged` | ✅ Works |
| `play-workflow-pr-created` | PR opened | Update workflow metadata | ✅ Works |
| `implementation-agent-remediation` | Rex pushes | Cancel outdated CodeRuns | ✅ Works |
| `atlas-pr-monitor` | PR events on cto | Launch Atlas guardian | ✅ Works |
| `atlas-conflict-monitor` | PR with conflicts | Launch Atlas conflict handler | ✅ Works |
| `atlas-batch-integration` | Batch comment | Launch Atlas integration | ✅ Works |
| `ci-failure-remediation` | CI failure | Launch Rex CI fix | ✅ Works |
| `remediation-feedback-sensor` | Bug comments/reviews | Launch Rex remediation | ✅ Works |

### Orphaned Sensors ❌

| Sensor | File | Should Be |
|--------|------|-----------|
| `stage-aware-cleo-approval` | `stage-aware-cleo-approval-sensor.yaml` | Removed |
| `play-workflow-ready-for-security` | `play-workflow-sensors.yaml` | Removed |
| `play-workflow-ready-for-qa` | `play-workflow-sensors.yaml` | Removed |

---

## Files Reference

### Core Workflow
- `infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml`

### Sensors
- `infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml`
- `infra/gitops/resources/github-webhooks/stage-aware-*.yaml`
- `infra/gitops/resources/github-webhooks/atlas-*.yaml`
- `infra/gitops/resources/github-webhooks/remediation-feedback-sensor.yaml`
- `infra/gitops/resources/sensors/ci-failure-remediation-sensor.yaml`

### EventSource
- `infra/gitops/resources/github-webhooks/eventsource.yaml`

---

## Appendix: Stage Transition Matrix

Valid stage transitions as defined in `update-workflow-stage` template:

```
pending                      → implementation-in-progress
implementation-in-progress   → quality-in-progress, waiting-quality-complete, waiting-pr-created
waiting-pr-created           → quality-in-progress, waiting-quality-complete
quality-in-progress          → security-in-progress, waiting-quality-complete
security-in-progress         → testing-in-progress, waiting-ready-for-qa
testing-in-progress          → waiting-atlas-integration, waiting-pr-merged, completed
waiting-quality-complete     → waiting-ready-for-qa
waiting-ready-for-qa         → waiting-atlas-integration, waiting-pr-merged, testing-in-progress
waiting-atlas-integration    → atlas-integration-in-progress, waiting-pr-merged
atlas-integration-in-progress → waiting-pr-merged, waiting-atlas-integration
waiting-pr-merged            → quality-in-progress, security-in-progress, testing-in-progress, completed
```
