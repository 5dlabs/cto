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
  - A DAG that composes multiple `CodeRun`/`DocsRun` invocations and waits for their statuses.
  - Stores orchestration state in Workflow and/or a new CRD (see “Orchestration CRD” below).
  - Uses Argo Events to start/resume based on GitHub events.

- **Agents (specialized CodeRun modes)**
  - **Authoring Agent**: writes code for the task based on `requirements.yaml`.
  - **Clippy & Lint Fixer Agent**: enforces 100% green including `-D warnings` and `-W clippy::pedantic`.
  - **Test Runner/Remediator Agent**: ensures unit/integration tests pass; adds/fixes tests as needed.
  - **Deployer Agent**: triggers build + deploy (GitHub Actions + Argo CD) and validates runtime health.
  - **Acceptance Agent**: verifies acceptance criteria (functional checks, endpoints healthy, dashboards, etc.).
  - **PR Comment Remediator Agent**: responds to PR comments with targeted changes.
  - **CI Failure Remediator Agent**: triggered by failing checks to fix the cause.
  - **Issue Triage/Implementation Agent**: reacts to new Issues to implement or groom.

Each “agent” can remain a `CodeRun` with a different prompt/profile and tool set, avoiding a proliferation of CRDs.

### Event‑driven orchestration
- **Event sources (Argo Events)**
  - GitHub: `pull_request`, `issue_comment`, `pull_request_review_comment`, `check_run`, `workflow_run`, `issues`, `push`.
  - Optional: internal webhooks from controller, CI, or observability signals.
- **Sensors**
  - Map events to DAG entrypoints or `resume` paths via labels/parameters (e.g., repo, PR number, task id).
  - Correlate using keys such as `(repo, PR, task-id)` stored as Workflow/CRD labels and annotations.

### Orchestration CRD (optional)
- Introduce `TaskSequence` (namespaced) to define a graph for a backlog or project slice:
  - Spec: tasks, dependencies, batch groups, acceptance criteria, concurrency policy, repo/branches, service id.
  - Status: per‑task phases, artifacts (PRs), last event processed, resumable cursor.
- The Orchestrator Workflow reconciles `TaskSequence` → emits `CodeRun`/`DocsRun` as needed.
- Alternative: encode everything directly in Argo Workflow parameters and labels (no new CRD). Start with Workflow‑only; add CRD if state becomes too complex.

### DAG sketch (conceptual)
```yaml
entrypoint: project-dag
templates:
  - name: project-dag
    dag:
      tasks:
        - name: author
          templateRef: { name: coderun-template, template: coderun-main }
          arguments: { parameters: [...] }

        - name: clippy-fix
          dependencies: [author]
          templateRef: { name: coderun-template, template: coderun-main }
          arguments: { parameters: [{ name: prompt-mod, value: "Fix all clippy pedantic; -D warnings" }, ...] }

        - name: tests
          dependencies: [clippy-fix]
          templateRef: { name: coderun-template, template: coderun-main }
          arguments: { parameters: [{ name: prompt-mod, value: "Ensure tests pass; add missing tests" }, ...] }

        - name: deploy
          dependencies: [tests]
          templateRef: { name: coderun-template, template: coderun-main }
          arguments: { parameters: [{ name: prompt-mod, value: "Trigger GH Actions, ensure rollout healthy" }, ...] }

        - name: acceptance
          dependencies: [deploy]
          templateRef: { name: coderun-template, template: coderun-main }
          arguments: { parameters: [{ name: prompt-mod, value: "Validate acceptance criteria" }, ...] }
```

### Batch and parallel processing
- Parse `docs/examples/example-requirements.yaml`‑like inputs to build a dependency graph.
- Identify independent tasks and run their “author → clippy → tests” chains in parallel using DAG fan‑out.
- Use Argo `synchronization` (semaphores) to rate‑limit shared resources (e.g., runners, clusters).
- Optionally implement “batch windows” to group similar tasks (e.g., repo‑level changes) to reduce churn.

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

### Incremental path
1) Add agent profiles (prompts/tools) for Clippy Fixer, Test Remediator, Deployer, Acceptance.
2) Create an initial Orchestrator Workflow DAG that chains these for a single task.
3) Add Argo Events for GitHub comments and CI failures → trigger remediators.
4) Add parallelization for multiple independent tasks.
5) (Optional) Introduce `TaskSequence` CRD if state management in Workflow becomes cumbersome.

### Open questions
- Where do we define acceptance criteria? Enforce a schema in `requirements.yaml`?
- Ephemeral env strategy: GH Actions‑only vs. Argo CD preview Apps vs. both.
- How to gate merges automatically while respecting required reviews?
- Back‑pressure policies when event rate spikes.


