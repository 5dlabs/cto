## Multi‑Agent, Event‑Driven Orchestration — Brainstorm

This is a working ideation page for evolving the platform from single‑job execution to a coordinated, event‑driven, multi‑agent pipeline that can “press play” on a task backlog and autonomously progress tasks to Done (merged), reacting to GitHub events along the way.

### Current primitives (baseline)
- **CRDs**: `CodeRun` and `DocsRun` under `agents.platform/v1`.
  - CRD specs define inputs (repo, working directory, model, etc.) and expose `status.phase`, `pullRequestUrl`, etc.
  - Paths: `infra/charts/controller/crds/coderun-crd.yaml`, `infra/charts/controller/crds/docsrun-crd.yaml`.
- **WorkflowTemplates**: Argo templates that create CRDs and wait for completion.
  - Paths: `infra/charts/controller/templates/coderun-template.yaml`, `infra/charts/controller/templates/docsrun-template.yaml`.
- **Controller**: Reconciles the CRDs and runs the corresponding job containers.

These will remain the atomic execution units. The “orchestrator” composes them into higher‑level flows.

### Final goal
- A single action (“press play” or an event) triggers an orchestrated pipeline that:
  - Plans, sequences, and batches tasks (parallel where safe, serialized where required).
  - Advances each task until it is merged to the target branch.
  - Reacts to GitHub events (comments, issues, CI results, merges) to continue/resume work.
  - Applies quality gates with dedicated agents, not the initial authoring agent.

### Proposed high‑level architecture
- **Orchestrator Workflow (Argo)**
  - A DAG that composes multiple `CodeRun` invocations and waits for their statuses.
  - Uses existing CodeRun/DocsRun CRDs (no new orchestration CRDs).
  - Uses Argo Events to start/resume based on GitHub events.

- **Agent Personas & GitHub Apps**
  - **Morgan (PM Agent)**: Product Manager, orchestrates and has awareness of all other agents
  - **Rex (Implementation Agent)**: Primary code author, writes initial implementation
  - **Clippy Agent**: Fixes clippy pedantic warnings + handles formatting (`cargo fmt`)
  - **QA Agent**: Adds tests only (cannot modify implementation), leaves comments if implementation needs changes
  - **Triage Agent**: Responds to CI failures (clippy/format/test failures) and attempts fixes
  - **Security Agent**: Reads existing vulnerability reports and remediates the identified security issues in code
  - **PR Comment Agent**: Downloads and addresses PR review comments (with easy MCP/API for comment retrieval)
  - **Issue Agent**: Converts issues to implementations through the same test pipeline

Each agent is a different GitHub App with its own character/persona and specialized system prompt.

### Event‑driven orchestration
- **Event sources (Argo Events)**
  - GitHub webhooks: `pull_request`, `issue_comment`, `pull_request_review_comment`, `check_run`, `workflow_run`, `issues`, `push`.
  - Security scan completion events (vulnerability reports already generated).
- **Event → Agent mapping**
  - `pull_request` opened/updated → Clippy Agent → QA Agent (test in real environment)
  - `issue_comment` or `pull_request_review_comment` → Rex (Implementation Agent) re-invoked with comments
  - `workflow_run` failure → Triage Agent (attempts to fix failures)
  - `issues` opened → Issue Agent → same testing pipeline
  - Security scan complete → Security Agent
- **Sensors**
  - Map events to DAG entrypoints or `resume` paths via labels/parameters.
  - Correlate using keys such as `(repo, PR, task-id)` stored as Workflow labels.

### Configuration improvements
- **Simplified arguments**: Reduce CodeRun parameters through auto-detection:
  - Auto-detect repository from git context
  - Infer working directory from task context
  - Default model selection based on task type
  - Goal: 1-2 required arguments maximum
- **Project-wide settings**:
  - MCP tool configuration at project level (not per-task)
  - Shared requirements.yaml with tool allowlist
  - Common environment variables and secrets
- **Workspace management**:
  - Persistent PVCs per service/project
  - Git worktrees for parallel execution
  - Shared cache directories for dependencies

### DAG flows

**PR Flow:**
```yaml
# Triggered on pull_request event
- name: pr-validation
  dag:
    tasks:
      - name: clippy-format
        templateRef: { name: coderun-template }
        arguments: { parameters: [{ name: github-app, value: "clippy-agent" }] }
        
      - name: qa-testing
        dependencies: [clippy-format]
        templateRef: { name: coderun-template }
        arguments: { parameters: [{ name: github-app, value: "qa-agent" }] }
```

**Issue/Task Flow:**
```yaml
# Triggered on issue or task creation
- name: implementation
  dag:
    tasks:
      - name: implement
        templateRef: { name: coderun-template }
        arguments: { parameters: [{ name: github-app, value: "rex" }] }
        
      - name: clippy-format
        dependencies: [implement]
        templateRef: { name: coderun-template }
        arguments: { parameters: [{ name: github-app, value: "clippy-agent" }] }
        
      - name: qa-testing
        dependencies: [clippy-format]
        templateRef: { name: coderun-template }
        arguments: { parameters: [{ name: github-app, value: "qa-agent" }] }
```

### Batch and parallel processing
- **Dependency analysis**: Parse TaskMaster output to identify task dependencies and determine parallelizable work.
- **Parallel execution**: Independent tasks run their full chains in parallel using DAG fan‑out.
- **Workspace isolation**: Use git worktrees or separate directories on same PVC for parallel work (separate runs, shared PVC).
- **Resource management**: Argo semaphores to rate‑limit shared resources (runners, API calls).
- **Batch windows**: Group related changes to reduce PR churn.

### Quality gates (Rust example)
- Editor agent should not attempt to be perfect; downstream agents own gates:
  - Clippy: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`.
  - Format: `cargo fmt --all -- --check` then remediate.
  - Tests: `cargo test --all --all-features` and fix.
  - Lints for YAML/Helm/K8s manifests (kubeconform, yamllint, helm lint) as applicable.

### Deployment and acceptance
- Build and push images via GitHub Actions; Argo CD syncs from `main`.
- For PR validation, spin up ephemeral preview envs (namespaced App‑of‑Apps) keyed to PR.
- Acceptance Agent validates:
  - Health checks (readiness, rollout complete), synthetic probes, and dashboards in `infra/telemetry`.
  - Business acceptance criteria from task requirements (e.g., endpoint returns 200 with shape X).
  - If acceptance passes → “ready to merge”; if not → loop back with a remediation agent.

### GitHub‑driven remediation
- **PR comments**: Launch PR Comment Remediator Agent on `issue_comment`/`pull_request_review_comment` events; it patches code and pushes to the same branch.
- **Issues**: For `issues:opened`, either create a new TaskSequence item or trigger an Authoring Agent.
- **CI failures**: For `check_run` failure, trigger CI Failure Remediator Agent, scoped to failing job.

### State, idempotency, and resume
- Store correlation keys and progress in labels/annotations and status fields of CRDs/Workflows.
- Use `status.phase` and a resumable cursor to ensure replays are safe.
- TTL strategies are already in templates; ensure orchestrator cleans up or archives state after merge.

### Security and credentials
- Use GitHub App auth (preferred) wired via External Secrets (`infra/secret-store/`).
- Minimal scopes per agent task. Inject runtime‑only tokens; avoid long‑lived PATs.

### Observability
- Emit OTEL spans/events around each agent step with correlation IDs.
- Surface metrics to Grafana dashboards (`infra/telemetry`).
- Log links to PRs, Actions runs, and Argo Workflows in CRD status.

### Implementation priorities
1) **Simplify API**: Auto-detect more parameters, reduce to 1-2 required arguments for better agent success rate.
2) **Unified project structure**: Docs in same project (no separate docs repo) is working well - standardize this.
3) **Project-wide MCP tools**: Move tool configuration from per-task to project-wide in requirements (less tedious).
4) **Agent profiles**: Create GitHub Apps and system prompts for each persona (Morgan, Rex, Clippy, QA, etc.).
5) **Comment retrieval**: Add MCP tool or simple API for downloading PR comments efficiently.
6) **PR flow first**: Implement Clippy → QA flow for PRs.
7) **Parallel execution**: Implement worktree/directory isolation for parallel tasks.
8) **Security remediation**: Agent that reads vulnerability reports and fixes identified issues.

### Open questions
- Best approach for git worktrees with agent containers?
- How to handle QA agent comments when implementation changes needed?
- Optimal PR comment retrieval method (MCP vs GitHub API)?
- Security scanning integration points and report formats?
- Morgan (PM agent) coordination patterns with other agents?
- Auto-merge policies after all agents approve?


