# Architecture — Multi-Agent Orchestration

This document describes the technical architecture for a multi-agent, event-driven orchestration that uses existing CodeRun/DocsRun CRDs as execution primitives, orchestrated via Argo Workflows + Argo Events. No new orchestration CRDs are introduced. It complements the PRD and focuses on components, control flow, prompting, security, and implementation.

## Components
- Argo Events
  - GitHub EventSource for `pull_request`, `issue_comment`, `pull_request_review_comment`, `workflow_run`, `check_run`, `issues`, `push`.
  - Sensors map events to Workflow submissions with parameters.
- Argo Workflows
  - WorkflowTemplates: existing `coderun-template` and `docsrun-template` create CRs; new `orchestrator-dag` chains them.
  - Synchronization (semaphores) for concurrency limits per repo/org.
- CodeRun/DocsRun CRDs and Controller
  - Existing CRDs define job specifications; controller reconciles them to Kubernetes Jobs.
  - No changes to CRD schemas or controller logic.
- Agent runtime
  - Single agent image reused for all CodeRun jobs; mounts `controller-agents` ConfigMap for system prompts.
  - Different `github-app` parameter selects different agent profiles (author, clippy, tests, deploy, acceptance).
  - Reads `task/prompt.md` as user prompt input; uses MCP tool config when available.
- CI/CD
  - GitHub Actions performs build/test/deploy; Argo CD syncs deployments from `main`.
- Secrets and config
  - GitHub App secrets via External Secrets; Helm values define agents and system prompts.
- Telemetry
  - OTEL collector exports traces/metrics; Grafana dashboards surface step status and throughput.

## Control flow
1) Event arrives (e.g., PR comment). Sensor submits an orchestrator Workflow with params such as repo, branch, prNumber, taskId.
2) Orchestrator DAG runs the appropriate path:
   - Fresh task: creates CodeRun CRs in sequence (author → clippy → tests → deploy → acceptance)
   - PR comment: creates remediation CodeRun on the task branch
   - CI failure: creates failure remediator CodeRun
3) Each CodeRun step:
   - Workflow creates CodeRun CR with appropriate `github-app` parameter (selects agent profile)
   - Controller reconciles CR, creates Job with mounted system prompt from agents ConfigMap
   - Job container reads `task/prompt.md` and executes with selected agent profile
4) CodeRun status updates (phase, pullRequestUrl) are monitored by Workflow for progression.
5) On success, the DAG advances to next CodeRun; on failure, retry with backoff or surface actionable error.

## Workflow design
- Parameters (common):
  - `task-id`: numeric task identifier
  - `service-id`: service/component name
  - `repository-url`: `org/repo`
  - `docs-repository-url`: docs repo
  - `working-directory`: path within repo
  - `github-app`: agent profile name (drives system prompt selection)
  - `model`: model selection
  - `continue-session`: whether to continue previous session
  - Optional: `overwrite-memory`, `docs-branch`, `task-requirements`

- CodeRun CR responsibilities (unchanged):
  - Controller creates Job with appropriate mounts and environment
  - Job authenticates via GitHub App
  - Job checks out repository and branch
  - Job applies system prompt based on `github-app` parameter
  - Job reads `task/prompt.md` and executes agent
  - Job enforces local pre-PR gates when configured in prompt
  - Job interacts with GitHub (PR creation/update)

- Orchestrator DAG patterns:
  - Linear chain of CodeRun creations for single task
  - Fan-out/fan-in for independent tasks using Argo DAG dependencies
  - Semaphore-based rate limiting (per repo/org/global)

## Prompt management
- System prompts are declared in Helm values under `agents[*].systemPrompt`; rendered to `controller-agents` ConfigMap as `GITHUB_APP_system-prompt.md`.
- CodeRun Job selects the system prompt by `github-app` parameter and mounts it for the agent.
- The functional/user prompt is read from `task/prompt.md` produced by the docs service.
- Different agent profiles (author, clippy, tests, deploy, acceptance) have different system prompts but share the same task prompt.

## Security
- GitHub App auth only; tokens minted per run with short TTL; no PATs.
- Secrets pulled via External Secrets and mounted as env vars.
- Minimal RBAC for Workflows to run pods and read ConfigMaps/secrets.

## Observability
- Emit spans per step with labels: repo, pr, taskId, agent, step.
- Export counters: successes/failures, retry counts, durations.
- Log links to PRs and Actions runs as Workflow outputs.

## Implementation plan
1. Define agent profiles in Helm values with specialized system prompts (author, clippy, tests, deploy, acceptance).
2. Create `orchestrator-dag` WorkflowTemplate that chains existing `coderun-template` calls with different `github-app` parameters.
3. Configure Argo Events sensors to submit orchestrator Workflows based on GitHub events.
4. Add semaphores for concurrency control at repo/org level.
5. Update dashboards to show orchestration-level metrics alongside existing CodeRun metrics.

## Open items
- Decide on preview env strategy (namespaced app vs. shared staging).
- Standardize acceptance criteria schema in `requirements.yaml`.
- Guardrails for event storm control and resume semantics.
