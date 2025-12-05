# Remediation Loop Implementation Plan

## Goal
Deliver an automated loop that re-runs the Rex → Cleo → Tess pipeline until Tess approves a pull request, without human intervention, using GitHub labels as the workflow state machine.

## Scope
- Applies to play workflows (e.g., `play-workflow-template`) triggered by repository events.
- Focuses on orchestration, state tracking, and agent context hand-off; does not cover task-specific code fixes.
- Targets GitHub-hosted projects using the existing agent controller and Argo Workflow deployment.

## Current Gaps
- One-pass play workflow stops after Tess finishes; no remediation cycle is triggered when Tess requests changes.
- Tess does not emit structured signals that re-queue Rex with relevant review feedback.
- Sensors and controllers do not react to GitHub label changes beyond the initial QA gate.
- No iteration bookkeeping or safeguards to surface runaway loops to operators.

## Workstream Checklists

### 1. GitHub Label State Machine
- [ ] Finalize canonical label set (e.g., `needs-fixes`, `fixing-in-progress`, `needs-cleo`, `needs-tess`, `approved`).
- [ ] Document transition rules for each agent and human override scenarios.
- [ ] Ensure controller/agents apply and remove labels atomically with status posts.
- [ ] Define failure/escape-hatch label (e.g., `remediation-paused`) and owner playbooks.
- [ ] Backfill documentation for label meaning in `github-guidelines.md`.

### 2. Argo Workflow Template Updates
- [ ] Extend `play-workflow-template` DAG to branch on Tess outcome instead of proceeding directly to merge wait.
- [ ] Add suspend/resume nodes that gate on remediation labels rather than one-time `ready-for-qa` and `pr-merged` events.
- [ ] Emit workflow parameters (iteration counters, last failure reason) for agent pods to consume.
- [ ] Persist iteration context in workflow annotations for auditability.
- [ ] Gate exit conditions on `approved` label detection before invoking merge wait.

### 3. GitHub Sensor & Event Handling
- [ ] Update webhook sensors to trigger workflow resumes on remediation labels (`needs-fixes`, `needs-cleo`, `needs-tess`).
- [ ] Add deduplication/Idempotency safeguards to avoid double-resuming on simultaneous label changes.
- [ ] Ensure sensors surface Tess review payloads (comment bodies, blocking status) to the workflow parameters.
- [ ] Add alerting/metrics for sensors that fail to deliver events within SLA.
- [ ] Verify sensors respect manual label overrides to short-circuit the loop when humans intervene.

### 4. Controller & Agent Enhancements
- [ ] Teach Rex to detect remediation mode and pull latest Tess comments plus original task context.
- [ ] Ensure Cleo re-evaluates updated commits and reapplies `needs-tess` label on success.
- [ ] Update Tess to publish structured verdicts (`approved` vs `changes_requested`) and apply corresponding labels.
- [ ] Make agents resilient to stale local clones by force-syncing PR branch at the start of each iteration.
- [ ] Add tracing identifiers (iteration number, agent run id) to log output for cross-agent correlation.

### 5. Iteration Control & Safeguards
- [ ] Track iteration count in workflow parameters and emit warnings past threshold (e.g., 5 attempts).
- [ ] Implement optional hard-stop configuration with notification hooks (Slack/PagerDuty) for manual review.
- [ ] Surface iteration summary (issues addressed vs outstanding) in PR comments after each cycle.
- [ ] Provide manual command (label or slash command) to break the loop when operators step in.
- [ ] Record completion reason (approved, manual stop, iteration cap) in workflow status fields.

### 7. Validation & Rollout
- [ ] Create integration tests simulating Tess change requests and verifying automated re-entry to Rex.
- [ ] Build staging play that exercises multiple remediation passes before approval.
- [ ] Document verification steps for QA to confirm loop behavior pre-production.
- [ ] Coordinate phased rollout (staging → limited production plays → full rollout) with observability toggles.
- [ ] Capture lessons learned and update plan based on pilot feedback.
