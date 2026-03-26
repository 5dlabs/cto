# Intake Process: PRD to Tasks

How the intake pipeline transforms a Product Requirements Document into agent-ready task files, using **AlertHub** as the running example.

---

## Input

### Required: PRD

A markdown document describing what needs to be built. The Lobster intake workflow receives it as `prd_content` and also persists it into the generated `.tasks/docs/` output for downstream agents.

**AlertHub example** (`tests/intake/alerthub-e2e-test/prd.md`):

```
# Project: AlertHub - Multi-Platform Notification System

## Vision
AlertHub is a comprehensive notification platform that routes alerts across
web, mobile, and desktop clients. Supports Slack, Discord, email, push
notifications with intelligent routing, rate limiting, and user preferences.

## Features
1. Notification Router Service (Rex - Rust/Axum)
2. Integration Service (Nova - Bun/Elysia + Effect)
3. Admin API (Grizz - Go/gRPC)
4. Web Console (Blaze - Next.js 15 + Effect)
5. Mobile App (Tap - Expo/React Native)
6. Desktop Client (Spark - Electron)
7. Infrastructure (Bolt - Kubernetes)

## Constraints
- API response time < 100ms p95
- 10,000 notifications/minute sustained
- 99.9% uptime SLA
- GDPR compliant
```

### Optional: Architecture Context

An `architecture.md` file with service topology, data flows, infrastructure CRDs, security model, and observability strategy. AlertHub's is 693 lines covering every service boundary, API contract, Effect pattern, and Kubernetes resource definition.

### Optional: Design Inputs (V1)

Design intake accepts any combination of:

- `design_prompt` - freeform design intent or UX goals
- `design_artifacts_path` - relative path to sketches/mockups/assets
- `design_urls` - existing site/app URLs for modernization context
- `design_mode` - `ingest_only` or `ingest_plus_stitch` (default)

Materialized outputs are written to:

```
.intake/design/
├── design-context.json
├── assets/
├── crawled/urls.json
└── stitch/
    ├── stitch-run.json
    └── candidates.json
```

When frontend is not detected (`hasFrontend=false`), intake preserves artifacts/URLs but skips Stitch generation.

### Pipeline Configuration

```json
{
  "project_name": "alerthub",
  "num_tasks": 30,
  "deliberate": true,
  "include_codebase": false,
  "generate_ai_prompts": true,
  "design_mode": "ingest_plus_stitch",
  "design_prompt": "Modernize UI and improve conversion",
  "design_urls": ["https://sigma1.led.video/"]
}
```

| Flag | Purpose | AlertHub |
|------|---------|----------|
| `deliberate` | Run the deliberation workflow before task generation | `true` |
| `include_codebase` | Analyze existing repo via Repomix/OctoCode | `false` (greenfield) |
| `num_tasks` | Target number of top-level tasks | `30` |

---

## Pipeline Phases

### Phase 0: Codebase Analysis *(skipped for AlertHub — greenfield)*

For non-greenfield projects (`include_codebase=true`):

1. **Pack repo** via Repomix (or OctoCode fallback) into a context-optimized representation
2. **Search patterns** relevant to the PRD via targeted code search
3. **LLM summarization** extracts structured context: tech stack, service boundaries, API contracts, data models, architectural patterns, test infrastructure, integration points, constraints

**Output**: `.tasks/docs/codebase-context.md`

This context feeds into both the deliberation debate and the parse-prd step so agents reference existing code rather than starting from scratch.

---

### Phase 1: Deliberation *(ran for AlertHub)*

**Workflow**: `deliberation.lobster.yaml`

#### Step 1: Research (Tavily)

Fires targeted web searches to build evidence memos for each debate position. Produces an optimist memo (best practices, scaling patterns, modern approaches) and a pessimist memo (failure modes, operational risks, technology traps).

#### Step 2: Debate

Two debate turns are generated from the PRD context:

- **Optimist** — advocates for modern, scalable architecture. Uses the `optimist-soul.md` personality prompt.
- **Pessimist** — advocates for simpler, operationally conservative architecture. Uses the `pessimist-soul.md` personality prompt.

The workflow runs these as Lobster steps with stateless LLM calls, posts progress through `intake-util bridge-notify` / `linear-activity`, and parses `DECISION_POINT:` markers from each turn for committee voting.

**AlertHub deliberation result** (from `.tasks/deliberation.md`):

The recorded outcome concluded that the original polyglot architecture should be implemented as designed, rather than phased through a simplified intermediate stack that would create rewrite debt.

#### Step 3: Committee Votes

When a `DECISION_POINT:` is raised during debate, a 5-member committee votes through `decision-voting.lobster.yaml`. Votes are tallied, published through the bridge layer, and can be either fully automatic or human-reviewed depending on `human_review_mode`.

#### Step 4: Compile Design Brief

Claude Opus synthesizes the debate result + original PRD into a **Design Brief** with ADR-format resolved decisions.

**AlertHub design brief** (`.tasks/docs/design-brief.md`) — 434 lines covering:

- 8 key design decisions (polyglot architecture, dual messaging, Effect TypeScript, data store segmentation, WebSocket strategy, gRPC for admin, K8s-first infra, Effect Schema)
- Trade-off analysis (performance vs. operational complexity, dev speed vs. type safety)
- 6 risk assessments with mitigations

**Output**: `.tasks/docs/design-brief.md`

---

### Phase 0.5: Design Intake *(new, required when Stitch mode is enabled)*

Runs immediately after PRD materialization in `pipeline.lobster.yaml`.

1. Copies local design artifacts into `.intake/design/assets/`
2. Normalizes URL inputs and crawls basic page metadata into `.intake/design/crawled/urls.json`
3. Detects frontend scope and targets (`web`, `mobile`, `desktop`)
4. Enforces a Stitch credential gate when `design_mode=ingest_plus_stitch`:
   - Requires `STITCH_API_KEY`
   - Emits credential discovery state (`STITCH_API_KEY`, `STITCH_PROJECT_ID`, `STITCH_ACCESS_TOKEN`, `GOOGLE_CLOUD_PROJECT`) to `.intake/design/auth-discovery.json`
   - Fails fast with explicit gate output if required auth is missing
5. If enabled and credentials exist, generates Stitch candidates and saves:
   - `.intake/design/stitch/stitch-run.json`
   - `.intake/design/stitch/candidates.json`
6. Writes canonical `.intake/design/design-context.json` for downstream steps
7. Saves a committed design bundle under `.tasks/design/`:
   - `design-context.json`
   - `crawled/urls.json` (when available)
   - `stitch/stitch-run.json`, `stitch/candidates.json` (when available)
   - `auth-discovery.json`
   - `manifest.json`
8. Persists a design snapshot to bridge SQLite history for audit/evidence reporting

`design-context.json` is threaded into both deliberation and parse-prd task generation.

If credential discovery in OnePass fails, provision or rotate `STITCH_API_KEY` and re-run preflight before starting the pipeline.

---

### Phase 2: Task Generation *(always runs)*

**Workflow**: `intake.lobster.yaml`

#### Step 1: Parse PRD

**Model**: Claude Sonnet 4
**Input**: Design brief (if deliberation ran) or raw PRD, plus codebase context (if non-greenfield)
**Output**: Structured JSON array of tasks

Each parsed task includes:

```json
{
  "id": 12,
  "title": "Notification Submission API (Rex - Rust/Axum)",
  "description": "Implement POST endpoints for single and batch notification submission",
  "status": "pending",
  "dependencies": [10, 11],
  "priority": "high",
  "details": "1. Create POST /api/v1/notifications endpoint\n2. Add request validation...",
  "test_strategy": "Can submit valid notifications and receive 202 Accepted...",
  "decision_points": [
    {
      "id": "d12",
      "category": "api-design",
      "description": "Request validation strategy",
      "options": ["serde with custom validators", "tower middleware", "axum extractors"],
      "requires_approval": false,
      "constraint_type": "soft"
    }
  ]
}
```

**AlertHub result**: 30 tasks with full dependency graph:

| ID | Task | Agent | Dependencies |
|----|------|-------|-------------|
| 1 | Infrastructure Foundation Setup | Bolt | — |
| 2 | PostgreSQL Database Setup | Bolt | 1 |
| 3 | Redis Cache Setup | Bolt | 1 |
| 4 | Kafka Event Streaming Setup | Bolt | 1 |
| 5 | MongoDB Document Store Setup | Bolt | 1 |
| 6 | RabbitMQ Task Queue Setup | Bolt | 1 |
| 7 | SeaweedFS Object Storage Setup | Bolt | 1 |
| 8 | Monitoring and Observability Setup | Bolt | 1 |
| 9 | Notification Router Core Service | Rex | 2, 3 |
| 10 | Database Models and Connections | Rex | 9 |
| 11 | Redis Integration and Rate Limiting | Rex | 9 |
| 12 | Notification Submission API | Rex | 10, 11 |
| 13 | Kafka Event Publishing | Rex | 4, 12 |
| 14 | WebSocket Real-time Updates | Rex | 11 |
| 15 | Notification Query API | Rex | 10 |
| 16 | Service Deployment and Configuration | Rex | 8, 13, 14, 15 |
| 17 | Integration Service Foundation | Nova | 5, 6 |
| 18 | MongoDB Integration with Drizzle ORM | Nova | 17 |
| 19 | Slack Integration Service | Nova | 18 |
| 20 | Multi-Channel Integration Services | Nova | 19 |
| 21 | Kafka Event Consumer | Nova | 4, 20 |
| 22 | Integration Management API | Nova | 20 |
| 23 | Admin API gRPC Foundation | Grizz | 2 |
| 24 | Tenant and User Management | Grizz | 23 |
| 25 | Notification Rules Engine | Grizz | 24 |
| 26 | Analytics and Reporting Service | Grizz | 24 |
| 27 | Web Console Foundation | Blaze | 16, 22, 26 |
| 28 | Dashboard and Notification Management | Blaze | 27 |
| 29 | Mobile App Foundation | Tap | 16, 22 |
| 30 | Desktop Client Foundation | Spark | 16, 28 |

#### Step 2: Analyze Complexity

**Model**: Claude Sonnet 4
Scores each task 1-10 on complexity. Tasks scoring 5+ get expansion recommendations identifying which areas need subtask decomposition.

#### Step 3: Review Tasks (Human Gate)

Presents parsed tasks + complexity analysis for human approval before expansion. The operator can adjust task count, reorder priorities, or reject and restart.

#### Step 4: Refine Tasks (Expand + Vote + Revise Loop)

**Sub-workflow**: `task-refinement.lobster.yaml`

This is a gated loop with up to 2 revision rounds:

```
expand-round-0 → vote-round-0 → check-round-0
    ↓ (if revise/reject)
expand-round-1 → vote-round-1 → check-round-1
    ↓ (if revise/reject)
expand-round-2 → vote-round-2 → check-round-2
    ↓
resolve-output (pick best expansion)
```

**Expansion** breaks each high-complexity task into subtasks with:
- Subagent types (implementer, reviewer, tester, researcher, documenter)
- Parallelization flags
- Per-subtask deliverables and acceptance criteria

**5-Model Voting Committee** evaluates expansion quality across 5 dimensions. Each voter has a distinct **soul** (evaluation personality) and runs on a different frontier model to maximize perspective diversity:

| Voter | Soul | Model | Provider | Evaluation Focus |
|-------|------|-------|----------|-----------------|
| 1 | **The Architect** | Claude Opus 4.6 | Anthropic | Structural integrity, separation of concerns, pattern consistency |
| 2 | **The Pragmatist** | GPT-5.2 Pro | OpenAI | Implementability, task sizing, timeline realism, agent feasibility |
| 3 | **The Minimalist** | Claude Sonnet 4.6 | Anthropic | Over-engineering detection, essential complexity, scope discipline |
| 4 | **The Operator** | O3 Pro | OpenAI | Deployability, observability, debuggability, operational readiness |
| 5 | **The Strategist** | Gemini 2.5 Pro | Google | Long-term maintainability, migration paths, API contract stability |

Each voter scores: task decomposition, dependency ordering, decision point coverage, test strategy quality, agent assignment — but through their distinct evaluative lens. Verdict: **approve**, **revise**, or **reject**.

If "revise" or "reject": voter suggestions are fed back as additional context for the next expansion round. After max revisions (default 2), the pipeline proceeds with the best attempt and attaches a warning to the PR body.

**AlertHub result**: Task 12 ("Notification Submission API") expanded into 14 subtasks (task-12.1 through task-12.14).

#### Step 5: Generate Docs

The workflow generates task docs in parallel with `intake-util fan-out`, validates the merged output, then writes files into `.tasks/docs/`.

#### Step 6: Generate Prompts

The workflow generates prompts in parallel with `intake-util fan-out`, validates them, and writes `prompt.md`, `prompt.xml`, and subtask prompts to disk.

#### Step 7: Commit Outputs

Stages all generated files, creates a branch `intake/{project_name}-{timestamp}`, and commits.

#### Step 8: Create PR

Pushes branch and opens a GitHub PR with:
- Task count, vote verdict, and revision round count
- Links to design brief and task docs
- Linear metadata for project tracking
- `intake` label

---

## Output Structure

```
.tasks/
├── state.json                           # Pipeline state tracking
├── deliberation.md                      # Debate transcript and arbiter decision
├── tasks/
│   └── tasks.json                       # Full task data (all 30 tasks with metadata)
└── docs/
    ├── prd.txt                          # Original PRD (copied)
    ├── architecture.md                  # Architecture context (if provided)
    ├── design-brief.md                  # Resolved decisions from deliberation
    ├── codebase-context.md              # Codebase analysis (if non-greenfield)
    └── task-{id}/
        ├── prompt.md                    # Implementation prompt (markdown)
        ├── prompt.xml                   # Implementation prompt (XML variant)
        ├── acceptance.md                # Acceptance criteria checklist
        ├── decisions.md                 # Decision log template
        └── subtasks/
            └── task-{id}.{sub}/
                └── prompt.md            # Subtask implementation prompt
```

### Per-File Details

#### `prompt.md` — Agent Implementation Prompt

Role-based prompt with agent identity, goal, requirements, acceptance criteria, constraints, and resource links.

```markdown
# Task 12: Notification Submission API (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role
You are a Senior Rust Engineer with expertise in systems programming
and APIs implementing Task 12.

## Goal
Implement POST endpoints for single and batch notification submission

## Requirements
1. Create POST /api/v1/notifications endpoint
2. Add request validation with serde
3. Implement batch submission endpoint
4. Add authentication middleware
5. Return proper HTTP status codes and error responses

## Acceptance Criteria
Can submit valid notifications and receive 202 Accepted, invalid requests
return 400 with error details, rate limits enforced

## Constraints
- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-12): Notification Submission API (Rex - Rust/Axum)`

## Resources
- PRD: `.tasks/docs/prd.txt`
- Dependencies: 10, 11
```

#### `prompt.xml` — XML Prompt Variant

Structured XML with `<meta>`, `<role>`, `<context>`, `<requirements>`, `<acceptance_criteria>`, `<validation>`, and `<deliverables>` sections. Used by agents that prefer XML-formatted instructions.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<task id="12" priority="high" agent="rex">
    <meta>
        <title>Notification Submission API (Rex - Rust/Axum)</title>
        <dependencies>10, 11</dependencies>
        <agent_hint>rex</agent_hint>
    </meta>
    <role>You are a Senior Rust Engineer...</role>
    <context>
        <overview>Implement POST endpoints for notification submission</overview>
    </context>
    <requirements>...</requirements>
    <acceptance_criteria>
        <criterion>Tests passing with adequate coverage</criterion>
        <criterion>Can submit valid notifications and receive 202 Accepted...</criterion>
    </acceptance_criteria>
    <validation>
        <command>cargo test --all</command>
        <command>cargo clippy -- -D warnings</command>
        <command>cargo fmt --check</command>
    </validation>
</task>
```

#### `acceptance.md` — Acceptance Criteria

Checkbox-format checklist with task-specific criteria plus standard quality gates (tests, lints, formatting, build, PR).

```markdown
# Acceptance Criteria: Task 12

- [ ] Implement POST endpoints for single and batch notification submission
- [ ] Can submit valid notifications and receive 202 Accepted...
- [ ] All requirements implemented
- [ ] Tests passing (`cargo test --workspace` exits 0)
- [ ] Lints passing (`cargo clippy --all-targets -- -D warnings` exits 0)
- [ ] Formatted (`cargo fmt --all --check` exits 0)
- [ ] Build succeeds (`cargo build --release` exits 0)
- [ ] PR created and ready for review
```

Quality gate commands are language-aware — Rust tasks get `cargo test`/`cargo clippy`, Kubernetes tasks get `helm lint`/`yamllint`, TypeScript tasks get the relevant bun/npm commands.

#### `decisions.md` — Decision Log

Template for tracking implementation decisions. Pre-populated with predicted decision points from the parse-prd step (when applicable), plus blank sections for additional decisions discovered during implementation.

```markdown
# Decision Log: Task 12

## Predicted Decision Points
No decision points were predicted for this task during intake.

## Additional Decisions
### (Add decision title here)
**Category:** (architecture | error-handling | data-model | ...)
**Decision:** _________________
**Rationale:** _________________
**Alternatives considered:** _________________
**Confidence (1-5):** ___
```

#### `tasks/tasks.json` — Full Task Registry

Single JSON file with all tasks and metadata, used by the task orchestrator to schedule agent work:

```json
{
  "metadata": {
    "taskCount": 30,
    "completedCount": 0,
    "version": "1.0.0"
  },
  "tasks": [
    {
      "id": "1",
      "title": "Infrastructure Foundation Setup (Bolt - Kubernetes)",
      "description": "Set up Kubernetes cluster with basic operators...",
      "agentHint": "bolt",
      "dependencies": [],
      "priority": "high",
      "status": "pending",
      "subtasks": [],
      "test_strategy": "Cluster is accessible via kubectl..."
    }
  ]
}
```

---

## Conformance: v1 (Rust CLI) vs v2 (Lobster Pipeline)

Both output paths produce the same per-task artifact structure. The key differences are scope, not shape:

| Artifact | v1 (`tests/intake/alerthub/`) | v2 (`tests/intake/alerthub-e2e-test/`) |
|----------|------|------|
| `task.md` | Per-task file with overview, details, decision points, testing strategy | Equivalent data in `tasks/tasks.json` + `prompt.md` |
| `prompt.md` | Basic implementation prompt | Role-based prompt with agent identity, language, constraints, resource links |
| `prompt.xml` | Not generated | Full XML variant with structured sections |
| `acceptance.md` | Checklist with subtask checkboxes | Checklist with language-aware quality gate commands |
| `decisions.md` | Pre-populated recommendations | Template with predicted decision points + blank sections |
| `subtasks/` | Flat markdown files (`task-1.1.md`) | Directories with `prompt.md` per subtask (`task-1.1/prompt.md`) |
| `design-brief.md` | Not generated | ADR-format design brief from deliberation |
| `deliberation.md` | Not generated | Full debate transcript with arbiter decision |
| `tasks.json` | Not generated | Centralized task registry with metadata |
| Decision points | Inline in `task.md` with category, options, approval flags | In `prd.json` parse output with same fields |
| Test strategy | Inline in `task.md` | In `tasks.json` and reflected in `acceptance.md` |

### Preserved from v1

The core output contract is preserved:

- **Tasks** with ID, title, description, details, dependencies, priority, status
- **Decision points** with category, constraint type, options, approval requirements
- **Subtasks** with parent reference, agent assignment, parallelization flags, deliverables, acceptance criteria
- **Implementation prompts** with agent role, goal, requirements, acceptance criteria
- **Acceptance criteria** as actionable checklists
- **Test strategies** per task and subtask

### Added in v2

- `prompt.xml` for agents that prefer structured XML
- Language-aware quality gate commands in `acceptance.md`
- `design-brief.md` with ADR-format resolved decisions (when deliberation runs)
- `deliberation.md` audit trail
- `codebase-context.md` for non-greenfield projects
- Centralized `tasks.json` registry
- Vote-gated revision loop (up to 2 rounds) before doc generation
- Vote verdict and revision count in PR body

---

## Agents Involved

| Agent/Role | Technology | Purpose |
|---|---|---|
| **Morgan** | Shell template (Handlebars) | Sets up the run context and launches the Lobster intake pipeline |
| **intake-agent** | Bun/TypeScript | Binary entry point for `ping` and `prd_research` only |
| **intake-util** | Bun/TypeScript CLI | Deterministic operations: generate-docs, generate-prompts, tally votes |
| **Optimist** | Lobster `llm-task` step with soul prompt | Debate: advocates modern/scalable architecture |
| **Pessimist** | Lobster `llm-task` step with soul prompt | Debate: advocates simplicity/operational safety |
| **Committee (5)** | Lobster `llm-task` steps | Vote on deliberation decision points |
| **5-model voting panel** | LLM calls via `openclaw.invoke` | Quality gate: evaluate task decomposition and revisions |
| **linear-bridge** | HTTP bridge service | Webhooks, run registry, elicitation routing, Linear session updates |
| **Repomix** | MCP tool | Pack existing codebase for analysis |
| **OctoCode** | MCP tool | GitHub code search fallback |
| **Tavily** | Research via intake-agent MCP | Pre-debate evidence gathering |
| **Stitch SDK** | `@google/stitch-sdk` (TypeScript) | Optional UI candidate generation from design intake |

---

## End-to-End Example: AlertHub

```
Input:
  prd.md ..................... 658 lines — 7 services, 6 tech stacks
  architecture.md ........... 693 lines — full service topology + CRDs
  config.json ............... deliberate=true, num_tasks=30

Phase 0 (Codebase Analysis):
  Skipped — greenfield project (include_codebase=false)

Phase 1 (Deliberation):
  Tavily research ........... 2 evidence memos (optimist + pessimist)
  Debate .................... Optimist and Pessimist turns generated as Lobster steps
  Decision resolution ....... Committee voting resolves decision points
  Design brief .............. 434 lines, 8 ADR decisions, 6 risk assessments

Phase 2 (Task Generation):
  parse-prd ................. 30 tasks with dependency graph
  analyze-complexity ........ Complexity scores 1-10 per task
  review-tasks .............. Human approval gate
  refine-tasks .............. Expand → 5-model vote → (revise if needed)
  generate-docs ............. 30 task directories
  generate-prompts .......... prompt.md + prompt.xml + acceptance.md per task
                              + subtask prompts (e.g., task-12 → 14 subtasks)
  commit-outputs ............ Branch: intake/alerthub-{timestamp}
  create-pr ................. PR with intake label + Linear metadata

Output:
  .tasks/docs/ .............. 30 task dirs, design-brief.md, deliberation.md
  .tasks/tasks/tasks.json ... 30 tasks, 25KB
  Total files ............... ~250+ (tasks + subtasks + prompts)
  GitHub PR ................. Ready for review → triggers agent implementation
```

---

## Stitch Authentication and Runtime Notes

For `design_mode=ingest_plus_stitch`, set:

- `STITCH_API_KEY`

The pipeline now enforces a hard gate on this credential in preflight materialization. If missing, the run fails before deliberation/tasking.

### Sigma-1 example invocation

```bash
lobster run --mode tool intake/workflows/pipeline.lobster.yaml --args-json '{
  "project_name": "sigma-1",
  "prd_path": ".intake/run-prd.txt",
  "include_codebase": true,
  "design_mode": "ingest_plus_stitch",
  "design_prompt": "Improve existing site design while preserving brand tone",
  "design_urls": "https://sigma1.led.video/"
}'
```
