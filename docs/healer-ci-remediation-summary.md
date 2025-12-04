# Healer CI Remediation Hub - Current Functionality

## Overview

The Healer CI Remediation Hub is a **unified remediation service** that automatically fixes CI failures by intelligently routing them to specialist agents. When a GitHub Actions workflow fails, Healer receives the webhook, classifies the failure, gathers context, and spawns the appropriate agent to fix it.

```
GitHub CI Fails → Webhook → Argo Events Sensor → Healer API → CodeRun → Agent Fixes
```

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                           Healer                                  │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐   │
│  │  HTTP API   │───▶│  CI Router  │───▶│  CodeRun Spawner    │   │
│  │  /api/...   │    │             │    │  (with dedup)       │   │
│  └─────────────┘    └─────────────┘    └─────────────────────┘   │
│         │                  │                      │               │
│         │                  ▼                      ▼               │
│         │          ┌─────────────┐    ┌─────────────────────┐   │
│         │          │  Context    │    │  Remediation        │   │
│         │          │  Gatherer   │    │  Tracker            │   │
│         │          └─────────────┘    └─────────────────────┘   │
│         │                                                         │
│         ▼                                                         │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                    Prompt Templates                          │ │
│  │  prompts/ci/rust-fix.hbs      → Rex                          │ │
│  │  prompts/ci/frontend-fix.hbs  → Blaze                        │ │
│  │  prompts/ci/infra-fix.hbs     → Bolt                         │ │
│  │  prompts/ci/security-fix.hbs  → Cipher                       │ │
│  │  prompts/ci/github-fix.hbs    → Atlas                        │ │
│  │  prompts/ci/retry.hbs         → Retry attempts               │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

---

## Components

### 1. HTTP Server (`crates/healer/src/ci/server.rs`)

The Healer server runs as a Kubernetes deployment, listening for incoming CI failure events.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check (returns `{"status":"healthy","version":"..."}`) |
| `/api/remediate/ci-failure` | POST | Receive CI failure webhook from sensor |
| `/api/remediate/security-alert` | POST | Receive security alert webhook |
| `/api/status` | GET | Current remediation status overview |
| `/api/status/{task_id}` | GET | Status of a specific remediation task |

### 2. Failure Router (`crates/healer/src/ci/router.rs`)

Classifies CI failures and routes them to the appropriate specialist agent.

| Failure Type | Agent | Triggers |
|--------------|-------|----------|
| `rust_clippy`, `rust_test`, `rust_build`, `rust_deps` | **Rex** | Clippy lints, test failures, build errors |
| `frontend_*` | **Blaze** | npm/pnpm, TypeScript, ESLint, frontend tests |
| `docker_build`, `helm_template`, `k8s_manifest`, `yaml_syntax` | **Bolt** | Docker, Helm, Kubernetes, YAML issues |
| `security_*` | **Cipher** | Dependabot, code scanning, secret scanning |
| `git_merge_conflict`, `github_workflow` | **Atlas** | Git conflicts, GitHub Actions, fallback |

### 3. Context Gatherer (`crates/healer/src/ci/context.rs`)

Enriches the remediation context with:
- Workflow logs from GitHub
- PR information (if applicable)
- Changed files in the commit
- ArgoCD application status
- Recent error logs from Loki
- Kubernetes pod state
- Historical context from OpenMemory

### 4. CodeRun Spawner (`crates/healer/src/ci/spawner.rs`)

Creates Kubernetes `CodeRun` resources that:
- Are labeled for tracking (`healer/task-id`, `healer/agent`, `healer/failure-type`)
- Use the appropriate GitHub App (5DLabs-Rex, 5DLabs-Blaze, etc.)
- Include the enriched prompt with full context
- Target the correct branch (existing PR branch or fix branch)

### 5. Remediation Tracker (`crates/healer/src/ci/tracker.rs`)

Tracks remediation state including:
- Task ID and original failure
- All attempts made (agent, outcome, duration)
- Deduplication (prevents duplicate fixes for the same workflow)
- Status progression (Pending → InProgress → Succeeded/Escalated)

### 6. Human Escalation (`crates/healer/src/ci/escalate.rs`)

After max attempts (default: 3), escalates to humans via:
- Discord notification with failure summary
- PR comment explaining what was tried
- Detailed logs of each agent attempt

### 7. OpenMemory Integration (`crates/healer/src/ci/memory.rs`)

Queries OpenMemory for historical context:
- Similar past failures and their solutions
- Agent success rates by failure type
- Known fix patterns

---

## Specialist Agents

| Agent | Domain | GitHub App | Prompt Template |
|-------|--------|------------|-----------------|
| **Rex** | Rust | 5DLabs-Rex | `rust-fix.hbs` |
| **Blaze** | Frontend | 5DLabs-Blaze | `frontend-fix.hbs` |
| **Bolt** | Infrastructure | 5DLabs-Bolt | `infra-fix.hbs` |
| **Cipher** | Security | 5DLabs-Cipher | `security-fix.hbs` |
| **Atlas** | Git/GitHub | 5DLabs-Atlas | `github-fix.hbs` |

---

## Event Flow

### CI Failure Flow

```
1. GitHub workflow fails (e.g., Clippy lint error)
2. GitHub sends workflow_job webhook to Argo Events
3. ci-failure-remediation-sensor filters for failures
4. Sensor POSTs to http://healer.cto.svc:8080/api/remediate/ci-failure
5. Healer:
   a. Validates event (not already being handled)
   b. Classifies failure type (e.g., rust_clippy)
   c. Gathers context (logs, PR info, changed files)
   d. Routes to specialist agent (Rex for Rust)
   e. Renders prompt template with full context
   f. Creates CodeRun resource in cto namespace
6. CodeRun spawns Rex agent pod
7. Rex analyzes logs, fixes code, pushes to PR branch
8. CI re-runs
9. If passes: Success → record to memory
   If fails: Retry with more context (up to 3 attempts)
   If exhausted: Escalate to human
```

### Security Alert Flow

```
1. GitHub creates security alert (Dependabot, code scan, etc.)
2. Webhook triggers ci-failure-remediation-sensor
3. Sensor POSTs to /api/remediate/security-alert
4. Healer routes to Cipher agent
5. Cipher updates vulnerable dependencies or fixes security issues
```

---

## Configuration

### Server Configuration (`healer-config.json`)

```json
{
  "cli": "Factory",
  "model": "claude-opus-4-5-20250929",
  "max_attempts": 3,
  "time_window_mins": 10,
  "memory_url": "http://openmemory.cto-system.svc:3000",
  "memory_enabled": true
}
```

| Setting | Default | Description |
|---------|---------|-------------|
| `cli` | `Factory` | CLI runtime for agents |
| `model` | `claude-opus-4-5-20250929` | Model for agent reasoning |
| `max_attempts` | `3` | Max remediation attempts before escalation |
| `time_window_mins` | `10` | Deduplication window |
| `memory_enabled` | `true` | Query OpenMemory for historical context |

### Kubernetes Resources

| Resource | Location | Purpose |
|----------|----------|---------|
| Deployment | `infra/gitops/applications/cto/healer.yaml` | Healer server with filebrowser sidecar |
| ConfigMap | `infra/gitops/resources/healer/healer-config.yaml` | Server configuration |
| ServiceAccount | `infra/gitops/resources/healer/rbac.yaml` | RBAC for CodeRun creation |
| Role/RoleBinding | `infra/gitops/resources/healer/rbac.yaml` | Permissions for pods, logs, CodeRuns |
| PVC | `infra/gitops/resources/healer/pvc.yaml` | Workspace storage |
| Sensor | `infra/gitops/resources/sensors/ci-failure-remediation-sensor.yaml` | Routes webhooks to Healer |

---

## Deduplication

Healer prevents duplicate remediation attempts:

1. **By workflow run ID** - Only one fix attempt per workflow run
2. **By branch + time window** - No duplicate fixes within 10 minutes on same branch
3. **By existing CodeRun** - Checks for active remediation before spawning

---

## Retry Loop

When an agent fails to fix the issue:

```
Attempt 1: Initial fix attempt
    ↓ (CI still failing)
Attempt 2: Retry with:
    - Previous attempt's output
    - What changes were made
    - Updated CI logs
    - Possibly different agent
    ↓ (CI still failing)
Attempt 3: Final attempt with all context
    ↓ (still failing)
ESCALATE: Human notification via Discord + PR comment
```

The retry prompt (`retry.hbs`) includes:
- Previous attempt number and outcome
- What the last agent tried
- Why it didn't work
- Current CI status
- Instructions to try a different approach

---

## Prompt Templates

Located in `crates/healer/prompts/ci/`:

| Template | Agent | Key Instructions |
|----------|-------|------------------|
| `rust-fix.hbs` | Rex | Fix Clippy/tests, run `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` |
| `frontend-fix.hbs` | Blaze | Fix npm/TypeScript/ESLint errors |
| `infra-fix.hbs` | Bolt | Fix Docker/Helm/K8s/YAML issues |
| `security-fix.hbs` | Cipher | Update vulnerable deps, rotate secrets |
| `github-fix.hbs` | Atlas | Resolve merge conflicts, fix workflow syntax |
| `retry.hbs` | Any | Augmented prompt for retry attempts |

---

## Observability

### Logs

Healer logs are collected via the platform's log collection system:
- Pod annotation: `logs.platform.5dlabs.io/collect: "true"`
- Service label: `logs.platform.5dlabs.io/service: healer`

### Health Check

```bash
curl http://healer.cto.svc:8080/health
# {"status":"healthy","version":"0.12.35"}
```

### Filebrowser Sidecar

A filebrowser sidecar runs on port 8081 for workspace inspection via `healer-filebrowser` service.

---

## Current Deployment Status

| Component | Status | Notes |
|-----------|--------|-------|
| Healer Server | ✅ Running | 3/3 containers (healer, filebrowser, log-tailer) |
| Health Endpoint | ✅ Working | Returns 200 with healthy status |
| ConfigMap | ✅ Deployed | Correct RemediationConfig schema |
| RBAC | ✅ Deployed | CodeRun create/get/list on `agents.platform` |
| Sensor | ✅ Configured | Routes to `http://healer.cto.svc.cluster.local:8080` |
| Prompt Templates | ✅ Available | 6 templates in `prompts/ci/` |

---

## Outstanding Work

### Ready for Testing
- End-to-end test with intentional CI failure
- Verify sensor → Healer → CodeRun → agent flow

### Post-E2E Cleanup
- Remove legacy `templates/remediate/` directory
- Remove Atlas `guardianMode` from controller values

### Future Enhancements
- CodeRun completion watching for retry loop
- Prometheus metrics for remediation success rates
- Dashboard for remediation status

---

## Related Documentation

- Design Document: `crates/healer/docs/ci-remediation-design.md`
- Completion Plan: `.cursor/plans/healer_ci_remediation_completion_110566d7.plan.md`

---

## Quick Reference

### Start Healer Locally

```bash
cargo run -p healer -- server --addr 0.0.0.0:8080 --config heal-config.json
```

### Check Healer Status

```bash
kubectl get pods -n cto -l app.kubernetes.io/instance=cto-healer
kubectl logs -n cto -l app.kubernetes.io/instance=cto-healer -c cto-healer --tail=50
```

### Verify Health

```bash
kubectl exec -n cto -l app.kubernetes.io/instance=cto-healer -c cto-healer -- curl -s localhost:8080/health
```

### Watch for CodeRuns

```bash
kubectl get coderuns -n cto -l app.kubernetes.io/name=healer --watch
```

