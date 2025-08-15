# Architecture — Multi-Agent Orchestration (Argo-Only, No CRDs)

This document describes the technical architecture for a multi-agent, event-driven orchestration that runs entirely on Argo Workflows + Argo Events without custom CRDs. It complements the PRD and focuses on components, control flow, prompting, security, and migration.

## Components
- Argo Events
  - GitHub EventSource for `pull_request`, `issue_comment`, `pull_request_review_comment`, `workflow_run`, `check_run`, `issues`, `push`.
  - Sensors map events to Workflow submissions with parameters.
- Argo Workflows
  - WorkflowTemplates: `agent-step` (reusable container step), `orchestrator-dag` (author→clippy→tests→deploy→acceptance), optional remediation subflows.
  - Synchronization (semaphores) for concurrency limits per repo/org.
- Agent runtime
  - Single agent image reused for all steps; mounts `controller-agents` ConfigMap for system prompts.
  - Reads `task/prompt.md` as user prompt input; uses MCP tool config when available.
- CI/CD
  - GitHub Actions performs build/test/deploy; Argo CD syncs deployments from `main`.
- Secrets and config
  - GitHub App secrets via External Secrets; Helm values define agents and system prompts.
- Telemetry
  - OTEL collector exports traces/metrics; Grafana dashboards surface step status and throughput.

## Control flow
1) Event arrives (e.g., PR comment). Sensor submits a Workflow with params such as repo, branch, prNumber, taskId, agent.
2) Orchestrator DAG runs the appropriate path:
   - Fresh task: author → clippy → tests → deploy → acceptance
   - PR comment: remediation subflow on the task branch
   - CI failure: failure remediator agent
3) Each `agent-step` container mounts the agents ConfigMap, selects the system prompt for the `agent` parameter, then streams `task/prompt.md` into the session.
4) Steps emit outputs (PR URL, Actions run URL, deployment status) and labels for correlation.
5) On success, the DAG advances; on failure, retry with backoff or surface actionable error.

## Workflow design
- Parameters (common):
  - `repository`: `org/repo`
  - `branch`: source branch to work against
  - `workingDirectory`: path within repo
  - `agent`: agent profile name (drives system prompt)
  - `taskId`: numeric/task identifier
  - `model`: model selection
  - Optional: `prompt_mod`, `prNumber`, `ciJobName`

- `agent-step` template responsibilities:
  - Authenticate via GitHub App (from secrets)
  - Ensure repository checkout and branch
  - Mount and apply system prompt for `agent`
  - Read `task/prompt.md` and stream to agent
  - Enforce local pre-PR gates (fmt, clippy pedantic, tests) when applicable
  - Interact with GitHub (PR creation/update, comment retrieval)

- Orchestrator DAG patterns:
  - Linear chain for single task
  - Fan-out/fan-in for independent tasks using Argo DAG dependencies
  - Semaphore-based rate limiting (per repo/org/global)

## Prompt management
- System prompts are declared in Helm values under `agents[*].systemPrompt`; rendered to `controller-agents` ConfigMap as `NAME_system-prompt.md`.
- `agent-step` selects the system prompt by `agent` name and passes it to the agent process.
- The functional/user prompt is read from `task/prompt.md` produced by the docs service.
- Optional `prompt_mod` parameter can prepend constraints for a run without modifying templates.

## Security
- GitHub App auth only; tokens minted per run with short TTL; no PATs.
- Secrets pulled via External Secrets and mounted as env vars.
- Minimal RBAC for Workflows to run pods and read ConfigMaps/secrets.

## Observability
- Emit spans per step with labels: repo, pr, taskId, agent, step.
- Export counters: successes/failures, retry counts, durations.
- Log links to PRs and Actions runs as Workflow outputs.

## Migration plan (from CRD-based to Argo-only)
1. Introduce `agent-step` and `orchestrator-dag` WorkflowTemplates; stop using CRD-creating templates.
2. Update Sensors to submit the new Workflows directly (no CR creation).
3. Remove CRD manifests from Helm (`infra/charts/controller/crds/*`) and CRD-based WorkflowTemplates.
4. Remove controller Deployment/RBAC once unused.
5. Clean Helm chart values and GitOps apps accordingly; preserve agents ConfigMap generation.

## Open items
- Decide on preview env strategy (namespaced app vs. shared staging).
- Standardize acceptance criteria schema in `requirements.yaml`.
- Guardrails for event storm control and resume semantics.
