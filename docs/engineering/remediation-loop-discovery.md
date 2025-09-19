# Remediation Loop Discovery: Controller + Argo Workflows

This document captures an initial discovery of the remediation loop that should kick in when a Play workflow doesn’t achieve full acceptance criteria. It covers both the Rust controller and the Argo Workflow templates, outlines how things appear to work today, and highlights incomplete or missing parts for making the loop operational.

## Summary
- A substantial remediation framework exists in the controller (parsing/remediation state/labels/orchestrator), and Play Workflow templates support suspend–resume coordination and multi-agent sequencing.
- However, key integration points are incomplete: webhook handling is a placeholder; label‑based orchestration isn’t wired up; “no‑PR” resume is stubbed; and suspend nodes for external events (e.g., needs-tess) don’t have a working resume path.
- Net effect: the conceptual loop is present but not end‑to‑end runnable without additional glue and resumption logic.

## Intended Flow (High Level)
- Run Play workflow with agents:
  - Implementation agent (Rex/Blaze) creates code + PR
  - Quality agent (Cleo) performs QA and transitions PR label to needs-tess
  - Testing agent (Tess) validates acceptance criteria
- If Tess finds issues, she writes a structured “Required Changes” comment with “Acceptance Criteria Not Met” items; this should:
  - Parse feedback and update remediation state
  - Flip PR labels/state (needs-fixes, fixing-in-progress, needs-cleo, needs-tess, etc.)
  - Trigger a new implementation iteration
  - Repeat until criteria met or max iterations reached

## Controller: What’s Implemented

### Feedback Parsing and Types
- Structured feedback parsing for “🔴 Required Changes” comments with “Acceptance Criteria Not Met” checkboxes.
  - Criteria section parsing: `controller/src/remediation/markdown.rs:19`
  - Parser orchestration with metadata and criteria: `controller/src/remediation/parser.rs`
  - Error taxonomy includes “NoCriteriaFound” and “AllCriteriaMet”.
  - References:
    - controller/src/remediation/markdown.rs:19
    - controller/src/remediation/parser.rs:56
    - controller/src/remediation/error.rs

### Remediation State Management
- ConfigMap-backed state for iterations, feedback history, and active runs:
  - Initialize/load/save state, increment iteration on new feedback, set/clear active runs.
  - Completion/termination/failure ops; cleanup and statistics helpers.
  - References:
    - controller/src/remediation/state.rs:177
    - controller/src/remediation/state.rs:313
    - controller/src/remediation/state.rs:386
    - controller/src/remediation/state.rs:434
    - controller/src/remediation/state.rs:514

### Label Schema + Orchestrator (GitHub)
- Label schema models workflow states and transitions (needs-fixes → fixing-in-progress → needs-cleo → needs-tess → approved), plus override labels; includes state machine and label client with retry/ETag handling.
- Orchestrator validates transitions/conditions and batches label operations atomically.
- Iteration-based conditions and “increment_iteration” exist but return placeholders; not integrated with state manager.
  - References:
    - controller/src/tasks/label/schema.rs
    - controller/src/tasks/label/client.rs
    - controller/src/tasks/label/orchestrator.rs:219
    - controller/src/tasks/label/orchestrator.rs:371

### State-Aware Cancellation
- Cancel running CodeRuns with distributed lock and basic state checks; notes intended integration with remediation state manager for richer semantics.
  - References:
    - controller/src/tasks/cancel/aware.rs:52
    - controller/src/tasks/cancel/aware.rs:275

### Workflow Resumption Utilities
- Utilities to “force re-evaluate” Argo Workflow nodes waiting on CodeRun completion by patching a retry annotation via raw HTTP to the Kubernetes API.
- Support for resuming “PR found” and “failure” cases; “no PR” is stubbed.
  - References:
    - controller/src/tasks/workflow.rs:37
    - controller/src/tasks/workflow.rs:86
    - controller/src/tasks/workflow.rs:107

### CodeRun Controller Hooks
- On CodeRun Succeeded/Failed:
  - Update CR status safely (TTL‑friendly), then attempt workflow resumption.
  - For Succeeded: try to detect PR (status or fallback GitHub query), then resume workflow with PR context; otherwise call “no PR” resume (currently stubbed).
  - References:
    - controller/src/tasks/code/controller.rs (completion/failure paths)

## Argo Workflows: What’s Implemented

### Play Workflow Template
- Multi-agent orchestration with clear stage updates and suspend points:
  - Implementation cycle (create CodeRun, wait for PR)
  - Quality (Cleo) hands off by tagging the PR with `needs-tess`; the workflow continues without an external suspend gate
  - Testing (Tess) then suspend until PR merged
  - Final completion step
- References:
  - infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml:198
  - infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml:887

Key details:
- “check-or-wait-for-pr” polls GitHub for PR; if not found in implementation stage it exits success with empty outputs, leaving downstream steps to decide how to proceed.
- “wait-coderun-completion” is a resource get with success/failure conditions; controller’s resume helper can unstick these by retry annotation.
- “suspend-for-event” templates use an indefinite suspend awaiting external resume via webhooks or similar; there’s no built-in resume path.

### Stage Transitions Template
- Robust label update with optimistic locking, validation of legal stage transitions, and verification.
  - References:
    - infra/charts/controller/templates/stage-transitions-template.yaml
    - infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml:703

### Agent Templates
- Cleo template always attempts to apply the `needs-tess` label if a PR is known.
  - References:
    - infra/charts/controller/claude-templates/code/container-cleo.sh.hbs:1060

## Gaps / Incomplete Work
These items prevent the remediation loop from running end-to-end:

1) Webhook ingestion is a placeholder
- `agent_controller` exposes `/webhook` but doesn’t process GitHub events or PR comments.
  - controller/src/bin/agent_controller.rs:146

2) Comment parsing → orchestration is not wired
- The parser/state/orchestrator exist but are not invoked from webhook handlers or any reconciliation path to:
  - Parse Tess’s “Required Changes” comments
  - Update remediation state & iteration
- Apply PR labels (needs-fixes, fixing-in-progress)
  - Trigger next iteration’s CodeRun (via Argo resume or new Workflow/CRD)

3) Suspend nodes have no resume implementation
- Play workflow historically used `suspend: {}` for “needs-tess” and “pr-merged” waits; the remediation loop now evaluates labels inline (still relying on an event-driven resume for PR merges).
- Current controller resume helper targets “wait-coderun-completion”, not suspended nodes.

4) “No PR” resume is stubbed
- `resume_workflow_for_no_pr` logs only; it never resumes the workflow or re-runs implementation.
  - controller/src/tasks/workflow.rs:86

5) Label orchestrator not used in the control-path
- The orchestrator and label client exist with state machine, but no code constructs an instance and drives transitions on events (e.g., `tess_feedback_received`).

6) Iteration and bypass/override persistence are placeholders
- Orchestrator iteration reads/updates return constants.
  - controller/src/tasks/label/orchestrator.rs:219, 371
- OverrideDetector’s bypass storage is a stub.

7) Implementation-cycle “looping” is ambiguous
- Template name suggests a loop until PR exists, but “check-or-wait-for-pr” may succeed with empty outputs, enabling downstream steps to run without a PR.
- Cleo cannot apply `needs-tess` when no PR exists, so the loop has to detect that condition and retry implementation.

## Risks and Edge Cases
- Suspends will accumulate stalled workflows if no external resume occurs.
- If implementation produces no PR, the quality step runs with empty PR context; Cleo cannot add `needs-tess` and the pipeline must re-run implementation.
- Without label orchestration, remediation signals (needs-fixes/fixing-in-progress) do not guide the workflow.
- Max iteration handling and state cleanup need to be enforced in the orchestrated loop (schema exists but not enforced by control-path).

## Suggested Next Steps

1) Implement webhook → remediation pipeline
- In `webhook_handler`:
  - Validate event and actor
  - On PR comment with “🔴 Required Changes”: call `remediation::parse_feedback_comment`, update `RemediationStateManager`, and invoke `LabelOrchestrator` to set `needs-fixes` and increment iteration.
  - Optionally persist structured feedback to state history.

2) Add resume handling for suspended nodes
- Implement (or retain) an Argo resume capability for event-driven waits (e.g., PR merged), even though the needs-tess gate is now handled inline.
  - Option A: `argo workflow resume <name>` equivalence via API (HTTP patch to remove Suspension / set node to proceed).
  - Option B: tailor event-driven Sensor/Sink that targets the workflow by labels and resumes it.

3) Complete “no PR” resume path
- Decide policy: re‑run implementation or fail-fast.
- If re‑run: use resume to re-enter implementation path or create a new CodeRun iteration.
  - Implement `resume_workflow_for_no_pr` accordingly.

4) Wire LabelOrchestrator into webhook and/or controllers
- Instantiate orchestrator with GitHub auth context and RemediationStateManager.
- Drive transitions on events: `tess_feedback_received`, `rex_remediation_started`, `rex_remediation_completed`, etc.
- Enforce `max_iterations` by transitioning to `failed-remediation`.

5) Harden iteration/state integration
- Replace orchestrator placeholders for iteration with reads/writes to `RemediationStateManager`.
- Store bypass/override requests (ConfigMap or DB) and enforce in orchestrator.

6) Tighten PR detection and gating
- In `implementation-cycle`, consider failing the step if PR is not found to enforce a clean repeat loop (or explicitly loop) rather than allowing downstream steps to proceed with empty PR context.

7) Minimal E2E test plan
- Unit:
  - Parser: acceptance criteria extraction and error taxonomy.
  - State manager: add_feedback increments iterations and persists history.
  - Orchestrator: transition decisions and atomic label operations (mock HTTP).
- Integration:
  - Simulate PR comment event → webhook → parser → orchestrator → state.
  - Confirm inline remediation status checks react to `needs-fixes`, `needs-cleo`, `needs-tess`, and `approved` labels.
  - Loop through one remediation iteration (issue → fix → needs-cleo → needs-tess → approved/failed).

## Key File References
- controller/src/tasks/workflow.rs:86
- controller/src/bin/agent_controller.rs:146
- controller/src/remediation/markdown.rs:19
- controller/src/remediation/state.rs:177
- controller/src/remediation/state.rs:313
- controller/src/tasks/label/orchestrator.rs:219
- controller/src/tasks/label/orchestrator.rs:371
- infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml:198
- infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml:887
- infra/charts/controller/claude-templates/code/container-cleo.sh.hbs:1060
