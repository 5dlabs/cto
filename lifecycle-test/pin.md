---
title: CTO Lifecycle Test Pin
description: Stable reference context for Ralph lifecycle loops
---

# CTO Lifecycle Test Pin (AlertHub)

This file is the stable reference for all lifecycle loops. Keep it concise and
update only when the platform changes. The execution prompt should remain small
and refer to this file as needed.

## Project Root and Key Files

- Repo root: `/Users/jonathonfritz/code/work-projects/5dlabs/cto`
- Lifecycle config: `lifecycle-test/ralph-cto.json`
- Lifecycle prompt: `lifecycle-test/prompt.md`
- Pin lookup index: `lifecycle-test/pin.lookup.md`
- Implementation plan: `lifecycle-test/implementation-plan.md`
- Lifecycle objective: `lifecycle-test/current-objective.md`
- PRD: `lifecycle-test/prd.json`
- Test PRD: `tests/intake/alerthub-e2e-test/prd.md`
- Architecture: `tests/intake/alerthub-e2e-test/architecture.md`
- Report: `lifecycle-test/report.json`
- Progress: `lifecycle-test/progress.txt`

## AlertHub Agent Stack

| Agent | Tech | Component |
| --- | --- | --- |
| Rex | Rust/Axum | Notification Router Service |
| Nova | Bun/Elysia+Effect | Integration Service |
| Grizz | Go/gRPC | Admin API |
| Blaze | Next.js/React | Web Console |
| Tap | Expo/React Native | Mobile App |
| Spark | Electron | Desktop Client |

## Lifecycle Gates

Use the authoritative checklist:
- `docs/workflow-lifecycle-checklist.md`
- `docs/heal-play.md`

Do not proceed to the next phase without evidence for all gates.

## Pin Lookup Index

Use `lifecycle-test/pin.lookup.md` to resolve aliases and pointer paths before
searching. This reduces missed context and keeps the loop deterministic.

## Infrastructure Health Check (RUN FIRST)

**Before attempting ANY objective, verify infrastructure is healthy:**

```bash
# Check controller is running (MUST have 1+ pods)
kubectl get pods -n cto -l app.kubernetes.io/name=agent-controller
# If "No resources found", FIX IT immediately

# Check PM server is running (MUST have 1+ pods)
kubectl get pods -n cto -l app=pm-server
# If "No resources found", FIX IT immediately
```

**If controller or PM server not running, FIX IT:**

```bash
# Option 1: Scale via ArgoCD (permanent)
kubectl patch application cto -n argocd --type merge -p '{"spec":{"source":{"helm":{"valuesObject":{"controller":{"replicaCount":1},"pm-server":{"replicaCount":1}}}}}}'

# Option 2: Scale deployments directly (temporary, survives until next ArgoCD sync)
kubectl scale deployment agent-controller -n cto --replicas=1
kubectl scale deployment pm-server -n cto --replicas=1

# Wait for pods to be ready
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=agent-controller -n cto --timeout=60s
kubectl wait --for=condition=ready pod -l app=pm-server -n cto --timeout=60s
```

**DO NOT proceed with any objective until both controller and PM server are running.**

## Cleanup Rules (No Duplicate CodeRuns)

Before retrying any phase:
```bash
kubectl delete coderuns -n cto --all --wait=false
kubectl delete pods -n cto --field-selector=status.phase!=Running --wait=false 2>/dev/null || true
kubectl delete pvc -n cto -l app=alerthub --wait=false 2>/dev/null || true
kubectl delete pvc -n cto workspace-prd-alerthub-e2e-test-morgan --wait=false 2>/dev/null || true
sleep 10
kubectl get coderuns,pods -n cto | rg "intake|alerthub|morgan" || echo "Cleanup complete"
```

## Automatic Pod Monitoring (Ralph Loop)

The Ralph loop now includes automatic pod monitoring that runs in the background
during agent execution. Configured in `ralph-cto.json`:

```json
"monitoring": {
  "enabled": true,
  "intervalSeconds": 30,
  "namespace": "cto",
  "failOnPodError": true,
  "podSelectors": ["type=intake", "type=implementation"]
}
```

**What it monitors:**
- Pod phase (Error, Failed)
- Container exit codes (non-zero = failure)
- Reports failures immediately without waiting for gates

**Manual monitoring still recommended for debugging:**

## Monitoring CodeRuns and Pods (CRITICAL)

**ALWAYS check pod status, not just CodeRun status.** The CodeRun status can be
stale or slow to update. The pod status is the source of truth.

```bash
# Check CodeRun status (can be stale)
kubectl get coderuns -n cto -l type=intake

# Check pod status (source of truth)
kubectl get pods -n cto -l type=intake -o wide

# Get container exit codes (critical for debugging)
kubectl get pod <pod-name> -n cto -o jsonpath='{.status.containerStatuses[*].state}'

# Get detailed container status
kubectl get pod <pod-name> -n cto -o jsonpath='{range .status.containerStatuses[*]}{.name}: {.state}{"\n"}{end}'
```

**Key indicators:**
- Pod status `Error` or `Failed` = container exited with non-zero code
- Container `exitCode: 1` = failure (check logs for error)
- Container `exitCode: 0` = success
- CodeRun `Running` but pod `Error` = status is stale, pod actually failed

**After monitoring, always get logs from the main container:**
```bash
# Get container names first
kubectl get pod <pod-name> -n cto -o jsonpath='{.spec.containers[*].name}'

# Then get logs from main container
kubectl logs <pod-name> -n cto -c <container-name> --tail=100
```

## Log Sources (Always Check After Operations)

Redact PII or secrets in evidence. The runner applies redaction rules before
storing logs and report entries.

Local services:
- `/tmp/cto-launchd/controller.log`
- `/tmp/cto-launchd/pm-server.log`
- `/tmp/cto-launchd/healer.log`
- `/tmp/cto-launchd/healer-sensor.log`

Kubernetes:
```bash
kubectl get pods -n cto
kubectl logs -n cto -l type=intake --tail=100
kubectl logs -n cto <pod-name> -c <container-name>
```

## MCP Tools (CTO Platform)

```text
mcp_cto_intake(project_name="alerthub-e2e-test", ...)
mcp_cto_play()
mcp_cto_play_status()
mcp_cto_jobs()
mcp_cto_stop_job()
```

Never use `local=true` for intake.

## Branch Rules

- Base branch: `develop`
- Feature branches: `feat/*` or `ralph/*`
- Never push to `main` or `develop`

## Verification Rules (When Code Changes)

Run before claiming completion:
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`
- `cargo test`
- `pre-commit run --all-files`

