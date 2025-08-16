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
  - Different `github-app` parameter selects different agent profiles/personas:
    - **Morgan**: PM agent with awareness of all other agents
    - **Rex**: Primary implementation agent
    - **Clippy Agent**: Formatting (`cargo fmt`) and pedantic warnings
    - **QA Agent**: Tests only (cannot modify implementation, leaves comments); MUST observe feature working in Kubernetes with proof (logs, request/response); can approve PRs (but not merge)
    - **Triage Agent**: CI failure remediation
    - **Security Agent**: Reads vulnerability reports and fixes issues
    - **PR Comment Agent**: Addresses review comments
    - **Issue Agent**: Converts issues to implementations
  - Reads `task/prompt.md` as user prompt input; uses MCP tool config when available.
  - Project-wide MCP tool configuration (not per-task) for simpler management.
- CI/CD
  - GitHub Actions performs build/test/deploy; Argo CD syncs deployments from `main`.
- Secrets and config
  - GitHub App secrets via External Secrets; Helm values define agents and system prompts.
- Telemetry
  - OTEL collector exports traces/metrics; Grafana dashboards surface step status and throughput.

## Control flow
1) Event arrives (e.g., PR opened). Sensor submits an orchestrator Workflow with params such as repo, branch, prNumber, taskId.
2) Orchestrator DAG runs the appropriate path:
   - **Pull Request**: Clippy Agent → QA Agent (test in real environment)
   - **Issue/Task**: Rex (implement) → Clippy → QA → deploy → acceptance
   - **PR/Issue comment**: Rex re-invoked with downloaded comments
   - **CI failure**: Triage Agent attempts fixes
   - **Security scan complete**: Security Agent remediates vulnerabilities
3) Each CodeRun step:
   - Workflow creates CodeRun CR with appropriate `github-app` parameter (selects agent persona)
   - Controller reconciles CR, creates Job with mounted system prompt from agents ConfigMap
   - Job container reads `task/prompt.md` and executes with selected agent profile
   - QA Agent can only add tests; MUST verify in Kubernetes environment with proof (logs/responses); can approve PR if criteria met
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
  - Parallel execution: Each agent gets a git worktree as working directory (or separate PVC for complete isolation)
  - Semaphore-based rate limiting (per repo/org/global)
  - Dependency analysis from TaskMaster to determine parallelizable work

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
1. **Simplify API**: Auto-detect parameters to reduce to 1-2 required arguments for better agent success.
2. **Standardize structure**: Docs in same project (no separate docs repo).
3. **Project-wide MCP tools**: Move tool configuration from per-task to project-wide in requirements.
4. **Define agent personas**: Create GitHub Apps and system prompts for each (Morgan, Rex, Clippy, QA, etc.).
5. **Comment retrieval**: Add MCP tool or API for efficient PR comment downloading.
6. **PR flow first**: Implement Clippy → QA flow for pull requests.
7. **Orchestrator DAG**: Create WorkflowTemplate chaining `coderun-template` with different agents.
8. **Event sensors**: Configure for PR, issue, comment, and CI failure events.
9. **Parallel execution**: Git worktrees as working directories (or separate PVCs for isolation).
10. **Security remediation**: Integrate vulnerability report reading and fixing.

## Open items
- Git worktree implementation: worktree root as working directory vs separate PVCs
- Comment retrieval: Extract from webhook payload vs GitHub MCP tool (document in prompt)
- Security reports: Use GitHub CLI with CodeQL (`gh api` commands)
- Morgan PM agent: Out of scope for current sprint
- Preview environments: Use existing namespaces to avoid secret duplication
- Event storm guardrails: Address if/when it becomes a problem
