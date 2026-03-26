# Linear-CTO Integration Workflow

> Current-state reference for how Linear, the Lobster intake pipeline, and the bridge services interact.

## Overview

The current integration has three primary pieces:

- The Lobster intake workflows in [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/pipeline.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/pipeline.lobster.yaml) and [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/intake.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/intake.lobster.yaml)
- The HTTP control plane in [`/Users/jonathon/.codex/worktrees/e92e/cto/apps/linear-bridge/src/http-server.ts`](/Users/jonathon/.codex/worktrees/e92e/cto/apps/linear-bridge/src/http-server.ts)
- The deterministic helper CLI in [`/Users/jonathon/.codex/worktrees/e92e/cto/apps/intake-util/src/index.ts`](/Users/jonathon/.codex/worktrees/e92e/cto/apps/intake-util/src/index.ts)

This is no longer the older PM-server-centric description where intake depended on ad hoc orchestration or NATS-driven debate execution. Intake is Lobster-native, and webhook / human-in-the-loop routing goes through `linear-bridge`.

## Credentials

### Linear

- `LINEAR_API_KEY` is the workspace-scoped credential used for project creation, issue creation, and agent activity updates.
- `LINEAR_TEAM_ID` is required for issue and project routing.
- `LINEAR_WEBHOOK_SECRET` is optional but, when set, is enforced by `linear-bridge` for `/webhooks/linear`.

### GitHub

- GitHub credentials remain per-agent and are used when generated task workflows or play workflows create branches and PRs.

## Intake Flow

### 1. Intake request starts the pipeline

An intake request provides PRD content and pipeline inputs such as:

- `project_name`
- `num_tasks`
- `include_codebase`
- `deliberate`
- optional Linear session / issue metadata

The top-level workflow is [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/pipeline.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/pipeline.lobster.yaml).

### 2. Early Linear visibility is created immediately

The `create-linear-project` step runs near the start of the pipeline and calls:

```bash
intake-util sync-linear init
```

That creates:

- a Linear project
- a PRD issue
- agent mapping metadata for downstream issue sync

### 3. The run is registered with `linear-bridge`

The pipeline registers the active run with:

```bash
intake-util register-run --run-id ... --session-id ... --workflow pipeline
```

This enables:

- session-to-run correlation
- elicitation callbacks
- Lobster resume routing
- Loki-to-Linear activity correlation when `linearSessionId` is present

### 4. Optional context-building phases run

Before task generation, the pipeline may run:

- codebase analysis for non-greenfield repos
- deliberation for architectural decision-making
- infrastructure discovery
- live MCP tool discovery

The deliberation workflow is [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/deliberation.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/deliberation.lobster.yaml). Debate turns and decision voting are implemented as Lobster steps and `openclaw.invoke` calls, not as a NATS debate loop inside `intake-agent`.

### 5. Intake generates artifacts and syncs Linear

The `intake` workflow:

- parses the PRD into tasks
- analyzes complexity
- pauses for task review
- runs the vote-gated refinement loop
- generates docs, prompts, workflows, scaffolds, and security outputs
- creates Linear task issues and subtasks with `intake-util sync-linear issues`
- commits generated artifacts and opens an intake PR

The main workflow file is [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/intake.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/intake.lobster.yaml).

## Bridge and Webhook Flow

### Linear bridge is the authority

`linear-bridge` is the HTTP entry point for:

- `POST /webhooks/linear`
- `POST /notify`
- `POST /elicitation`
- `POST /runs/:runId/register`
- `POST /runs/:runId/callback`
- `DELETE /runs/:runId`

Reference: [`/Users/jonathon/.codex/worktrees/e92e/cto/apps/linear-bridge/src/http-server.ts`](/Users/jonathon/.codex/worktrees/e92e/cto/apps/linear-bridge/src/http-server.ts)

### Workflow-to-Linear notifications

Workflow steps emit Linear-visible updates using:

- `intake-util linear-activity`
- `intake-util bridge-notify`
- `intake-util bridge-elicitation`

These commands are used throughout the intake and deliberation workflows to publish:

- thoughts
- plan/progress updates
- elicitation prompts
- PR-created notifications

### Human responses

When human input is required, Lobster approval gates pause execution. The response path is:

1. workflow publishes elicitation through `linear-bridge`
2. user responds in Linear or Discord
3. bridge resolves the winning response and resumes the run

This is the current replacement for the older "agent waits on NATS / PM server callback" mental model.

## Linear Activity Model

For intake, the workflow now emits activity explicitly from steps instead of relying on a separate intake-specific sidecar abstraction.

Common activity types emitted during intake:

- `plan`
- `thought`
- `response`
- `elicitation`

Examples in source:

- [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/intake.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/intake.lobster.yaml)
- [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/deliberation.lobster.yaml`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/workflows/deliberation.lobster.yaml)

## Outputs

An intake run can produce:

- a Linear project
- a PRD issue
- task issues and subtasks in Linear
- `.tasks/` documentation and prompts in the repo
- an intake PR with generated artifacts

The artifact structure is documented in [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/docs/intake-process.md`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/docs/intake-process.md).

## Related Docs

- [`/Users/jonathon/.codex/worktrees/e92e/cto/intake/docs/intake-process.md`](/Users/jonathon/.codex/worktrees/e92e/cto/intake/docs/intake-process.md)
- [`/Users/jonathon/.codex/worktrees/e92e/cto/docs/cloudflare-tunnel-intake-agent.md`](/Users/jonathon/.codex/worktrees/e92e/cto/docs/cloudflare-tunnel-intake-agent.md)
- [`/Users/jonathon/.codex/worktrees/e92e/cto/apps/intake-agent/README.md`](/Users/jonathon/.codex/worktrees/e92e/cto/apps/intake-agent/README.md)
