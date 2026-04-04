# Enhanced PRD

## 1. Original Requirements

> # Project: Sigma-1 Agent Delegation E2E
>
> ## Goal
> Validate the full intake pipeline end-to-end with agent delegation enabled, confirming:
> 1. Linear issues are created with the correct agent assignee (delegate_id)
> 2. Hermes research integration produces content in the deliberation path
> 3. Design snapshot PR surfacing works
> 4. Discord and Linear bridge notifications fire correctly
>
> ## Context
> The PM server now resolves agent hints to Linear user IDs at issue creation time via `resolve_agent_delegates()`. Previously all issues were created unassigned. This run validates the full delegation flow from PRD → deliberation → task generation → issue creation with assigned agents.
>
> ## Requirements
> - Pipeline completes through all stages (deliberation, task generation, issue creation)
> - At least 5 tasks generated with agent assignments
> - Linear issues show assigned agents (not just "agent:pending" labels)
> - Research memos include Hermes-sourced content where NOUS_API_KEY is available
> - PR created in target repo with task scaffolds
>
> ## Target Repository
> - Organization: 5dlabs
> - Repository: sigma-1
> - Visibility: private
>
> ## Acceptance Criteria
> - [ ] Pipeline completes without fatal errors
> - [ ] Linear session created with issues
> - [ ] Issues have delegate_id set (visible as assignee in Linear)
> - [ ] PR created in sigma-1 with generated artifacts
> - [ ] Discord notification posted for pipeline start/complete

## 2. Project Scope

The initial task decomposition identified **10 tasks** spanning three agents and two technology stacks. A significant finding from deliberation is that several tasks exceed the PRD's stated acceptance criteria — this is documented in the Resolved Decisions section below.

### Task Summary

| ID | Title | Agent | Stack | Priority | Dependencies |
|----|-------|-------|-------|----------|-------------|
| 1 | Provision Dev Infrastructure for Sigma-1 E2E Pipeline | Bolt | Kubernetes/Helm | High | — |
| 2 | Extend PM Server for Agent Delegation in Linear Issues | Nova | Bun/Elysia | High | 1 |
| 3 | Integrate Hermes Research into Deliberation Path | Nova | Bun/Elysia | High | 1 |
| 4 | Implement Design Snapshot PR Surfacing | Nova | Bun/Elysia | Medium | 1 |
| 5 | Enable Discord and Linear Bridge Notifications | Nova | Bun/Elysia | High | 1 |
| 6 | Validate Task Generation with Agent Assignment | Nova | Bun/Elysia | High | 2, 3 |
| 7 | Implement Web Frontend for Delegation Status | Blaze | React/Next.js | Medium | 6 |
| 8 | End-to-End Pipeline Integration Test | Tess | Test frameworks | High | 2, 3, 4, 5, 6, 7 |
| 9 | Production Hardening: HA Scaling, Ingress, Network Policies | Bolt | Kubernetes/Helm | Medium | 8 |
| 10 | Production Hardening: RBAC, Secret Rotation, Audit Logging | Bolt | Kubernetes/Helm | Medium | 9 |

### Key Services and Components

- **PM Server** (`cto/cto-pm`): Existing Bun/Elysia service containing `resolve_agent_delegates()` — the core of the delegation pipeline
- **Discord Bridge** (`bots/discord-bridge-http`): In-cluster notification service for Discord
- **Linear Bridge** (`bots/linear-bridge`): In-cluster notification service for Linear
- **External-Secrets Operator**: Deployed in-cluster with CRDs for secret management
- **Hermes Agent / NOUS API**: Research integration (in-cluster primary, external fallback)
- **tweakcn** (`cto/tweakcn`): Deployed in-cluster, indicating organizational shadcn/ui familiarity

### Agent Assignments

- **Bolt** (Kubernetes/Helm): Infrastructure provisioning and production hardening (Tasks 1, 9, 10)
- **Nova** (Bun/Elysia): Core pipeline logic — delegation, research, PR surfacing, notifications, validation (Tasks 2, 3, 4, 5, 6)
- **Blaze** (React/Next.js): Web frontend dashboard (Task 7)
- **Tess** (Test frameworks): E2E integration test (Task 8)

### Cross-Cutting Concerns

- Secret management touches Tasks 1, 3, 9, and 10
- Service topology decision (dp-1) affects Tasks 2, 3, and 6
- Scope questions around Tasks 7, 9, and 10 were a central point of debate (see Section 3)
- The dependency chain is deep: Task 10 → 9 → 8 → {2,3,4,5,6,7} → 1

## 3. Resolved Decisions

### [D1] Should agent delegation and Hermes integration be extensions to the existing PM server, or separate microservices?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 6 — PM server delegation logic, Hermes research integration, task generation validation

**Context:** Both debaters agreed unanimously. The PM server (`cto/cto-pm`) already contains `resolve_agent_delegates()` and is deployed in-cluster. The PRD is a validation exercise for existing behavior, not a greenfield product. Splitting into microservices would add Helm charts, health checks, network policies, and inter-service failure modes for zero demonstrated scaling need.

**Decision:** Extend the existing PM server (Bun/Elysia) — add delegation resolution and Hermes integration logic as endpoints within `cto-pm`. No new services.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Zero new deployment surface; leverages existing operational infrastructure; simplest path to validation
- **Negative:** If delegation or research logic later needs independent scaling, extraction will be required
- **Caveats:** None raised — both debaters agreed this is the correct topology for a validation run with no scaling requirements

---

### [D2] Which notification bridge services should be used for Discord and Linear integration?

**Status:** Accepted

**Task Context:** Tasks 5, 8 — Notification enablement and E2E test

**Context:** Both debaters agreed unanimously. `bots/discord-bridge-http` and `bots/linear-bridge` are already deployed and operational in-cluster. Using external SaaS (Zapier, Pipedream) would introduce egress latency, third-party auth tokens, billing dependencies, and external infrastructure routing for an internal private pipeline.

**Decision:** Use the existing in-cluster `bots/discord-bridge-http` and `bots/linear-bridge` services.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Zero new dependencies; low-latency in-cluster HTTP calls; no external auth surface
- **Negative:** If bridge services are down, notifications fail with no external fallback
- **Caveats:** None — both debaters considered external SaaS clearly inappropriate for this use case

---

### [D3] How should secrets be managed and injected into the pipeline?

**Status:** Accepted

**Task Context:** Tasks 1, 3, 9, 10 — Infrastructure provisioning, Hermes integration, production hardening

**Context:** Both debaters agreed on using the external-secrets operator, which is already deployed with CRDs in the cluster. The Pessimist raised a critical operational caveat: ExternalSecret CRDs that point to nonexistent paths in the backing secret provider will silently create empty Kubernetes Secrets, causing downstream services to fail with opaque "unauthorized" errors rather than clear "missing secret" errors.

**Decision:** Use the existing external-secrets operator with ExternalSecret CRDs backed by the cluster's SecretStore. **Task 1 must include a validation step** that verifies ExternalSecret resources resolve to non-empty values before downstream tasks proceed.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Automated, auditable, rotation-capable secret management using already-deployed infrastructure
- **Negative:** Depends on backing provider (Vault, AWS SSM, etc.) being correctly configured with the required paths
- **Caveats (Pessimist):** Empty-secret failure mode is a known issue with external-secrets operator. Task 1 **must** verify that `NOUS_API_KEY`, Linear tokens, and Discord webhook URLs resolve to non-empty values. This is a blocking prerequisite for Task 3.

---

### [D4] What API paradigm should the frontend use for delegation status?

**Status:** Accepted (conditional — see Scope Caveat)

**Task Context:** Tasks 2, 7 — PM server API and web frontend dashboard

**Context:** The Pessimist argued that Task 7 (web dashboard) is not in the PRD's acceptance criteria and should be descoped entirely, which would make this decision point moot. The Optimist proposed REST endpoints as the appropriate paradigm for flat task-list data. Both agreed that if Task 7 is retained, REST is the correct choice — GraphQL adds schema/resolver overhead for a single-view dashboard with no nested data.

**Decision:** REST endpoints from the PM server. The data model is a flat list of tasks with assignee and status fields — no graph-shaped data justifies GraphQL complexity.

**Consensus:** 2/2 (100%) on REST if Task 7 is retained

**Consequences:**
- **Positive:** Native to Bun/Elysia; no additional schema layer or client dependencies (Apollo/urql)
- **Negative:** If future dashboards need flexible querying across multiple entities, REST may require multiple endpoints
- **Scope Caveat:** The Pessimist's position is that Task 7 should be descoped. If Task 7 is descoped, this decision is moot. **Implementing agents should treat Task 7 as lower priority than core pipeline tasks (1–6, 8).**

---

### [D5] How should agent assignments and delegation status be modeled?

**Status:** Accepted

**Task Context:** Tasks 2, 6, 7 — Delegation logic, task generation validation, dashboard

**Context:** Both debaters agreed unanimously. The PRD's acceptance criterion is "issues have delegate_id set" — this is a field, not an event stream. Event sourcing would add projection layers, replay logic, and eventual consistency semantics for what amounts to writing and reading two columns. Audit trail needs (if in scope via Task 10) are satisfied by infrastructure-layer logging, not data model complexity.

**Decision:** Extend the existing task schema with `delegate_id` (string, nullable) and `status` (enum) fields using a standard CRUD model.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Minimal schema change; straightforward read/write; no new infrastructure (event store, projections)
- **Negative:** No built-in history of status transitions at the data layer
- **Caveats:** If audit logging of status transitions is later required, it should be implemented at the application/infrastructure logging layer, not by retrofitting event sourcing

---

### [D6] What authentication mechanism for the web frontend dashboard?

**Status:** Accepted (conditional — see Scope Caveat)

**Task Context:** Task 7 — Web frontend dashboard

**Context:** The Pessimist argued Task 7 should be descoped entirely, making this decision moot. If retained, both agreed: JWT-based stateless auth with httpOnly cookies. The PM server is a stateless Bun process — adding a session store (Redis) introduces a new dependency; in-memory sessions break on restart. JWT with short-lived tokens in httpOnly cookies provides XSS mitigation without operational overhead.

**Decision:** JWT-based stateless authentication with short-lived tokens stored in httpOnly cookies.

**Consensus:** 2/2 (100%) on JWT if Task 7 is retained

**Consequences:**
- **Positive:** No session store dependency; works natively with Next.js middleware; httpOnly cookies mitigate XSS
- **Negative:** Token revocation requires a blocklist (adds complexity) or short expiry with refresh tokens
- **Scope Caveat:** Same as D4 — conditional on Task 7 remaining in scope

---

### [D7] How should the frontend handle pipeline progress and error states?

**Status:** Accepted (conditional — see Scope Caveat)

**Task Context:** Task 7 — Web frontend dashboard

**Context:** The Pessimist argued Task 7 should be descoped. If retained, both agreed: polling at 5-second intervals with inline error banners. Pipeline runs take minutes, making 5s polling indistinguishable from real-time. WebSocket infrastructure in Bun/Elysia adds complexity for no perceptible UX benefit. Inline banners (not modal dialogs) because errors shouldn't block the user from viewing other tasks.

**Decision:** Polling with 5-second periodic refresh. Inline error banners for error states (not modal dialogs).

**Consensus:** 2/2 (100%) on polling with inline errors if Task 7 is retained

**Consequences:**
- **Positive:** No WebSocket infrastructure needed; simple implementation; non-blocking error UX
- **Negative:** Slightly higher server load from polling vs. push; 0–5s latency on status updates
- **Scope Caveat:** Same as D4 — conditional on Task 7 remaining in scope

---

### [D8] Should Hermes integration use the in-house agent or external research API?

**Status:** Accepted

**Task Context:** Task 3 — Hermes research integration

**Context:** Both debaters agreed on in-house Hermes as primary with NOUS API as fallback. The Pessimist raised a critical point about failure semantics: the PRD says "where NOUS_API_KEY is available," using conditional language that implies graceful degradation. If neither Hermes nor NOUS is available, the pipeline **must not fail** — research is enrichment, not a gate. The risk is Task 3 treating missing research as a fatal error that blocks the entire validation run.

**Decision:** In-house Hermes agent as primary, NOUS API as fallback when `NOUS_API_KEY` is present, and **graceful skip** if neither is available. Research is conditional enrichment, not a pipeline gate.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Keeps data in-cluster when possible; reduces external API costs; graceful degradation preserves pipeline reliability
- **Negative:** Research memos may be absent in some runs, reducing deliberation quality
- **Caveats (Pessimist):** Task 3 must implement a circuit breaker or skip pattern. Three explicit cases must be handled: (1) Hermes available → use it, (2) Hermes unavailable + NOUS_API_KEY present → call NOUS, (3) neither available → skip research, log warning, continue pipeline. **Case 3 must not be a fatal error.**

---

### [D9] What component library for the frontend dashboard?

**Status:** Accepted (conditional — see Scope Caveat)

**Task Context:** Task 7 — Web frontend dashboard

**Context:** Both agreed: shadcn/ui with Radix primitives. `cto/tweakcn` is deployed in-cluster, confirming organizational familiarity and likely existing theme customization. Custom components for an internal validation dashboard would be waste. The Pessimist's primary position was to descope Task 7 entirely.

**Decision:** shadcn/ui with Radix primitives as the component library.

**Consensus:** 2/2 (100%) on shadcn/ui if Task 7 is retained

**Consequences:**
- **Positive:** Copy-paste model (no npm vendor lock-in); accessible primitives via Radix; organizational alignment confirmed by tweakcn deployment
- **Negative:** shadcn components require Tailwind CSS — adds to bundle if not already configured
- **Scope Caveat:** Same as D4 — conditional on Task 7 remaining in scope

---

### [D-SCOPE] Should Tasks 7, 9, and 10 be included in the validation run?

**Status:** Accepted — **Tasks 7, 9, 10 are deprioritized; core validation is Tasks 1–6 and 8**

**Task Context:** Tasks 7, 9, 10 — Web dashboard, HA scaling, RBAC/audit logging

**Context:** The Pessimist raised the strongest meta-argument of the deliberation: Tasks 7, 9, and 10 represent ~40% of the task list but support zero PRD acceptance criteria. The PRD's acceptance criteria are: pipeline completes, Linear issues have assignees, PR is created, Discord notifications fire. A web dashboard, production HA scaling, and RBAC/audit logging do not validate delegation — they build a product around a validation run. The Optimist did not contest this point and acknowledged Task 7 is "the lowest-priority task."

**Decision:** The core validation scope is **Tasks 1–6 and 8**. Tasks 7, 9, and 10 are acknowledged as beyond the PRD's acceptance criteria and should be treated as **stretch/deferred work**. Implementing agents should complete Tasks 1–6 and 8 before beginning Tasks 7, 9, or 10. Task 8 (E2E test) should remove its dependency on Task 7 to avoid blocking validation on dashboard completion.

**Consensus:** 2/2 (100%) that Tasks 7, 9, 10 are not required by the PRD

**Consequences:**
- **Positive:** Focuses effort on the 6 tasks that directly validate the PRD's acceptance criteria; reduces timeline risk by ~40%
- **Negative:** No dashboard for visual monitoring; no production hardening in this phase
- **Caveats:** If organizational stakeholders require Task 7, it should be implemented after core validation passes. Task 8's dependency on Task 7 should be removed — the E2E test should validate pipeline artifacts (Linear issues, PR, notifications), not dashboard rendering.

## 4. Escalated Decisions

No decisions were escalated. All decision points reached consensus during deliberation.

## 5. Architecture Overview

### Agreed Approach

The Sigma-1 E2E validation run follows a **monolithic extension** pattern — all new functionality is added to the existing `cto/cto-pm` Bun/Elysia service. No new microservices are created.

### Technology Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Runtime | Bun | Existing PM server runtime |
| API Framework | Elysia | REST endpoints; existing patterns |
| Frontend (if retained) | React/Next.js + shadcn/ui + Radix | Stretch scope — Task 7 |
| Infrastructure | Kubernetes/Helm | Existing cluster |
| Secret Management | external-secrets operator | Already deployed with CRDs |
| Notifications | In-cluster discord-bridge-http + linear-bridge | Already operational |
| Research | In-house Hermes → NOUS API fallback → graceful skip | Three-tier degradation |
| Data Model | CRUD — extend task schema with `delegate_id` + `status` | No event sourcing |

### Service Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                        │
│                                                             │
│  ┌───────────────┐     ┌─────────────────────────────────┐ │
│  │  cto/cto-pm   │────▶│  Linear API (external)          │ │
│  │  (Bun/Elysia) │     │  - Create issues w/ delegate_id │ │
│  │               │     └─────────────────────────────────┘ │
│  │  Extended:    │                                         │
│  │  - delegation │     ┌─────────────────────────────────┐ │
│  │  - research   │────▶│  Hermes (in-cluster) or NOUS    │ │
│  │  - PR surface │     │  (external, conditional)         │ │
│  │               │     └─────────────────────────────────┘ │
│  │               │                                         │
│  │               │     ┌─────────────────────────────────┐ │
│  │               │────▶│  bots/discord-bridge-http        │ │
│  │               │     └─────────────────────────────────┘ │
│  │               │                                         │
│  │               │     ┌─────────────────────────────────┐ │
│  │               │────▶│  bots/linear-bridge              │ │
│  └───────────────┘     └─────────────────────────────────┘ │
│                                                             │
│  ┌───────────────┐     ┌─────────────────────────────────┐ │
│  │ external-     │────▶│  Backing store (Vault/SSM/etc.) │ │
│  │ secrets op    │     └─────────────────────────────────┘ │
│  └───────────────┘                                         │
│                                                             │
│  ┌───────────────┐                                         │
│  │  sigma-1 repo │◀── PR with task scaffolds + design      │
│  │  (5dlabs)     │    snapshots                            │
│  └───────────────┘                                         │
└─────────────────────────────────────────────────────────────┘
```

### Key Patterns

1. **Monolithic extension**: All logic within `cto-pm` — no new service deployments
2. **Three-tier research degradation**: Hermes (in-cluster) → NOUS API (external fallback) → graceful skip
3. **Secret validation gate**: ExternalSecret resources must resolve to non-empty values before pipeline proceeds
4. **CRUD data model**: `delegate_id` and `status` fields on existing task schema
5. **In-cluster notification**: HTTP calls to co-located bridge services

### Explicitly Ruled Out

| Approach | Reason |
|----------|--------|
| Separate microservices for delegation/research | Premature decomposition; adds deployment surface for a validation run with no scaling needs |
| External SaaS notifications (Zapier/Pipedream) | Unnecessary egress, auth surface, and cost for private internal pipeline |
| Manual secret provisioning | Compliance risk; operator already deployed; no audit trail |
| GraphQL API | Over-engineering for flat task-list data; adds schema/resolver overhead |
| Event sourcing for delegation state | "Issues have delegate_id set" is a column, not an event stream |
| Session-based auth for dashboard | Requires session store (Redis) — PM server is stateless |
| WebSocket real-time updates | Pipeline runs take minutes; 5s polling is indistinguishable from real-time |
| Custom component library | Internal tooling doesn't justify custom UI when shadcn is organizationally aligned |

## 6. Implementation Constraints

### Security Requirements

- **Secrets must use the external-secrets operator** — no manual Kubernetes secrets in manifests or CI
- **ExternalSecret validation is mandatory**: Task 1 must confirm that secrets resolve to non-empty values before marking infrastructure as ready
- **If Task 7 is implemented**: JWT with short-lived tokens in httpOnly cookies; no localStorage token storage
- Target repository (`5dlabs/sigma-1`) is private — all Git operations must use authenticated credentials

### Performance Targets

- Pipeline must complete through all stages without fatal errors (PRD acceptance criterion)
- At least 5 tasks generated with valid agent assignments per run
- Notification delivery to Discord and Linear should complete within the pipeline run (not queued indefinitely)

### Operational Requirements

- **Research integration must not block the pipeline**: Missing Hermes + missing NOUS_API_KEY = skip research with warning log, not fatal error
- **Backward compatibility**: If agent mapping is unavailable for a specific task, fall back to `agent:pending` label (do not fail)
- All resources in the sigma-1 namespace must be labeled for easy cleanup and traceability
- ConfigMap `sigma-1-infra-endpoints` must aggregate all connection strings and service URLs

### Service Dependencies and Integration Points

| Service | Type | Required For |
|---------|------|-------------|
| `cto/cto-pm` | In-cluster, existing | Tasks 2, 3, 4, 5, 6 |
| `bots/discord-bridge-http` | In-cluster, existing | Task 5, 8 |
| `bots/linear-bridge` | In-cluster, existing | Task 5, 8 |
| Linear API | External | Task 2, 6 (issue creation with delegate_id) |
| Hermes agent | In-cluster, conditional | Task 3 (primary research) |
| NOUS API | External, conditional | Task 3 (fallback research, requires NOUS_API_KEY) |
| GitHub API | External | Task 4 (PR creation in sigma-1) |
| external-secrets operator | In-cluster, existing | Task 1 (secret provisioning) |

### Organizational Preferences

- **Prefer in-cluster services** over external SaaS when equivalent functionality exists
- **Use existing infrastructure** (operators, bridges, services) rather than deploying new tooling
- **shadcn/ui** is the organizational standard for React dashboards (confirmed by `cto/tweakcn` deployment)

### Task Priority and Ordering

Implementing agents must respect the following priority:

1. **Critical path (must complete):** Tasks 1 → 2, 3, 4, 5 → 6 → 8
2. **Stretch/deferred:** Task 7 (dashboard), Task 9 (HA), Task 10 (RBAC)
3. **Dependency fix:** Task 8 (E2E test) should **not** depend on Task 7 — remove this dependency to avoid blocking validation on stretch scope

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`:** true
- **`frontendTargets`:** web
- **Mode:** `ingest_plus_stitch`

### Provider Status

| Provider | Status | Notes |
|----------|--------|-------|
| Stitch | Failed | No generated design artifacts available |
| Framer | Unknown | Not attempted or no results |

### Supplied Design Artifacts

No design artifacts were supplied with the PRD. No reference URLs were provided.

### Component Library Context

- **tweakcn** (`cto/tweakcn`) is deployed in-cluster, confirming organizational adoption of shadcn/ui
- Decision D9 establishes shadcn/ui with Radix primitives as the component library if Task 7 (web dashboard) is implemented

### Implications for Implementation

1. **Task 7 (if retained)** should use shadcn/ui with Radix primitives and Tailwind CSS, consistent with the tweakcn deployment
2. **No generated designs are available** — the dashboard UI must be implemented from component library defaults with functional requirements only (task list, assignee display, pipeline status)
3. **Stitch failure** means there are no visual mockups to reference; implementing agents should prioritize functional correctness over visual polish for this internal validation tool
4. Since no design artifacts exist and this is an internal tooling dashboard, visual design decisions should default to shadcn/ui's standard theme unless organizational overrides exist via tweakcn

### 7a. Selected Design Direction

No design selections were provided.

### 7b. Design Deliberation Decisions

No design deliberation was conducted.

## 8. Open Questions

The following items were not resolved in deliberation and should be handled by implementing agents using best judgment:

1. **Backing secret store configuration**: Which backing provider (Vault, AWS SSM, GCP Secret Manager) is configured for the external-secrets operator? Task 1 must discover this during infrastructure provisioning. If the backing store paths for `NOUS_API_KEY`, Linear tokens, and Discord webhook URLs are not pre-configured, the agent should document what's needed and flag it.

2. **Hermes agent endpoint discovery**: The exact in-cluster endpoint for the Hermes agent is not specified. Task 3 should check `cto/cto-tools` or service discovery for a Hermes-compatible API. If not found, fall back directly to NOUS API.

3. **Linear user ID mapping source**: `resolve_agent_delegates()` maps agent hints to Linear user IDs, but the source of this mapping (hardcoded, API lookup, configuration file) is not specified. Task 2 should use the existing mapping logic and extend it only if the current implementation is insufficient for 5+ agent assignments.

4. **Design snapshot generation mechanism**: Task 4 references "design snapshot generation" but the PRD doesn't specify what generates the snapshots. The implementing agent should integrate with whatever design snapshot mechanism currently exists in the pipeline, or create placeholder scaffolds in the PR if no mechanism is found.

5. **E2E test mocking strategy**: Task 8 notes "mock external dependencies as needed for repeatability" — the implementing agent should decide which dependencies (Linear, Discord, GitHub) to mock vs. call live based on test environment constraints and idempotency requirements.

6. **Dashboard scope (if Task 7 proceeds)**: The exact fields to display, filtering/sorting capabilities, and any role-based visibility are undefined. Implement a minimal read-only view: task title, agent assignment, delegate_id, pipeline status. No CRUD operations from the dashboard.

