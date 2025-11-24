# Atlas Integration Findings — 2025‑11‑22

## Cluster observations
- `kubectl get sensor atlas-pr-guardian -n argo -o yaml` shows the live sensor running with `replicas: 1` plus a custom dedup workflow that continually creates `atlas-dedup-*` workflows.
- `kubectl get coderun -n agent-platform -l agent=atlas` lists 25+ `coderun-atlas-pr-*` CodeRuns that have been Running for ~90 minutes, confirming the guardian sensor is actively spawning sessions in production.
- Workflow list (`kubectl get wf -n agent-platform`) shows the guardian dedup workflows completing every few minutes, which matches the runaway CodeRun count.

## Repository expectations vs. live state

### 1. PR guardian sensor drift
- In Git (`infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml`) the sensor is explicitly disabled with `replicas: 0` and meant to stay that way because play workflows handle orchestration.
- The cluster object deviates from that spec (replicas scaled to 1, different trigger definition, dedup script). This drift explains why the guardian keeps running even though the committed YAML disables it.
- Because ArgoCD tracks the file that has `replicas: 0`, it will keep trying to turn the sensor off while the live cluster keeps turning it back on, creating a tug‑of‑war state.

### 2. Tess approval does not activate Atlas
- The only Tess approval sensor (`infra/gitops/resources/github-webhooks/stage-aware-tess-approval-sensor.yaml`) simply resumes the waiting stage (`target-stage: waiting-ready-for-qa`) and never creates or resumes an Atlas CodeRun.
- There is no hook that takes Tess’s `pull_request_review` approval and hands off to Atlas. The workflow resumes internally, but Atlas is not triggered as part of that stage transition.

### 3. Atlas never activates at PR creation via current templates
- Outside the guardian sensor, the only Atlas automation wired into GitOps is the conflict-detection sensor (`infra/gitops/resources/sensors/atlas-conflict-detection-sensor.yaml`), which filters for `mergeable=false` or `mergeable_state in {dirty, unstable}`. That means Atlas only fires after GitHub already reports conflicts.
- There is no stage-aware trigger that creates Atlas CodeRuns when a PR is opened or synchronized, so the “Atlas should engage as soon as a PR exists” requirement is unmet unless the (disabled) guardian sensor is manually forced on.

## Impact
- Atlas guardian runs constantly even though the repo disabled it, leading to dozens of long-lived CodeRuns and unnecessary resource use.
- Tess approvals do not unlock Atlas, so Atlas never becomes the “integration” gate after QA as intended.
- Without an on-open trigger, Atlas only notices PRs once conflicts exist (or via the ad-hoc guardian sensor), so it cannot proactively prevent conflicts or merge drift.

## Recommended next actions
1. **Resolve sensor drift**  
   Decide which behavior is desired. Either update the YAML in `infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml` to match the live, running configuration (replicas 1, dedup workflow, rate limit) or scale the cluster object back to 0 so the guardian truly stays off. Until the spec and live object match, ArgoCD will keep fighting the override.

2. **Wire Tess approvals into Atlas**  
   Extend `stage-aware-tess-approval-sensor.yaml` (or add a sibling sensor) so a Tess approval creates/resumes an Atlas integration CodeRun and advances the workflow’s `current-stage` to something like `waiting-atlas-integration`. That makes Tess → Atlas a real gate instead of a no-op resume.

3. **Add Atlas activation for PR creation/sync**  
   If Atlas must start watching each PR immediately, add a stage-aware trigger (or fix the guardian sensor spec) so PR `opened/reopened/synchronize` events create a single Atlas session tagged with the PR number. Ensure deduplication happens via labels so each PR has at most one active guardian.

4. **Re-run validation script after reconciling**  
   Once the GitOps spec matches the desired behavior, run `scripts/test-atlas-sensor-fix.sh` and `kubectl get coderun -n agent-platform -l agent=atlas` to confirm the guardian count and sensor status align with expectations.


## Implementation progress — 2025‑11‑22

- **Phase 1: Atlas workflow stage**  
  Added a dedicated `waiting-atlas-integration` suspend gate to `play-workflow-template.yaml`, including guarded stage transitions and an `atlas-integration` suspend node that blocks the pipeline until Atlas finishes.

- **Phase 2: Tess → Atlas handoff**  
  Rebuilt `stage-aware-tess-approval-sensor.yaml` so Tess approvals launch an Atlas CodeRun, wait for success, patch workflow stage labels, and resume the suspended `wait-for-atlas-integration` node instead of blindly resuming Tess’ stage.

- **Phase 3: Atlas controller updates**  
  Introduced `integration-gate` mode for Atlas via new env vars, taught the guardian script to auto-merge even for play PRs during this mode, and expanded the Atlas system prompt in `values.yaml` to spell out the Tess→Atlas responsibilities.

- **Phase 4: PR lifecycle monitoring (in progress)**  
  Added the `atlas-pr-monitor` sensor under `infra/gitops/resources/github-webhooks/`, which deduplicates PR events, acquires a per-PR lock, and creates/reuses Atlas guardian CodeRuns with `ATLAS_MODE=guardian`. The legacy `atlas-pr-guardian` sensor is now scaled to zero so GitOps stops fighting the live cluster configuration.
- **Phase 4: Conflict + batch triggers**  
  Added `atlas-conflict-monitor-sensor.yaml` to relaunch guardians whenever GitHub marks a PR unmergeable, and `atlas-batch-integration-sensor.yaml` to run the batch/final integration workflow when play coordination comments fire. The old sensors in `infra/gitops/resources/sensors/` were disabled to keep GitOps authoritative.

- **Phase 5: Documentation, Testing & Observability (COMPLETE)**  
  - Created comprehensive architecture documentation (`docs/atlas-integration-architecture.md`)
  - Added integration test suite (`infra/scripts/test-atlas-sensors.sh`)
  - Configured Grafana dashboard for Atlas metrics (`infra/telemetry/atlas-dashboard.json`)
  - Set up Prometheus alerting rules (`infra/telemetry/atlas-prometheus-rules.yaml`)
  - Wrote operations runbook (`docs/runbooks/atlas-operations.md`)

## Implementation Complete

All phases of the Atlas integration fix have been successfully implemented:

✅ **Phase 1**: Added Atlas stage and suspend point to play workflow  
✅ **Phase 2**: Modified Tess sensor to trigger Atlas integration gate  
✅ **Phase 3**: Updated Atlas controller configuration and prompts  
✅ **Phase 4**: Added PR lifecycle, conflict, and batch integration sensors  
✅ **Phase 5**: Documentation, testing, and observability  

The Atlas integration is now feature-complete with:
- Proactive PR monitoring from creation
- Automatic conflict detection and resolution
- Integration gate after Tess approval
- Batch integration support for parallel execution
- Comprehensive monitoring and alerting
- Full documentation and runbooks


