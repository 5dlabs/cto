

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

The initial task decomposition identified **10 tasks** across two agents and two technology stacks.

### Task Summary

| ID | Title | Agent | Stack | Priority | Dependencies |
|----|-------|-------|-------|----------|--------------|
| 1 | Provision Dev Infrastructure for Sigma-1 E2E Pipeline | Bolt | Kubernetes/Helm | High | — |
| 2 | Extend PM Server for Agent Delegation in Linear Issues | Nova | Bun/Elysia | High | 1 |
| 3 | Integrate Hermes Research Content in Deliberation Path | Nova | Bun/Elysia | High | 1 |
| 4 | Implement Design Snapshot PR Surfacing | Nova | Bun/Elysia | Medium | 1 |
| 5 | Enable Discord and Linear Bridge Notifications | Nova | Bun/Elysia | Medium | 1 |
| 6 | Modernize Web Frontend for Agent Assignment Visualization | Blaze | React/Next.js | Medium | 2 |
| 7 | Add Research Memo Display to Web Frontend | Blaze | React/Next.js | Medium | 3, 6 |
| 8 | Display Design Snapshot PRs in Web Frontend | Blaze | React/Next.js | Low | 4, 6 |
| 9 | Show Pipeline Status Notifications in Web Frontend | Blaze | React/Next.js | Low | 5, 6 |
| 10 | Production Hardening: HA, Ingress, and Security | Bolt | Kubernetes/Helm | High | 2–9 |

### Key Services and Components

- **PM Server** (Bun/Elysia) — existing service with `resolve_agent_delegates()`; extended for delegation, Hermes integration, PR surfacing, and notifications
- **Hermes Research API** — external service accessed via NOUS_API_KEY for research memo content
- **Bridge Services** — `bots/discord-bridge-http` and `bots/linear-bridge` already deployed in-cluster
- **Cloudflare Operator** — `cloudflare-operator-system` with accesstunnels/clustertunnels CRDs for ingress
- **Web Frontend** (React/Next.js) — Tasks 6–9 add visualization for assignments, memos, PRs, and notifications

### Agent Assignments

- **Bolt** (Kubernetes/Helm): Infrastructure provisioning (Task 1) and production hardening (Task 10)
- **Nova** (Bun/Elysia): All backend pipeline work (Tasks 2–5)
- **Blaze** (React/Next.js): All frontend visualization work (Tasks 6–9)

### Cross-Cutting Concerns

- Secret management via Kubernetes secrets and external-secrets operator
- Service discovery via `sigma-1-infra-endpoints` ConfigMap
- 8 unique decision points spanning service-topology, api-design, platform-choice, data-model, ux-behavior, component-library, security, and ingress

## 3. Resolved Decisions

### [D1] Should agent delegation and Hermes research integration be separate microservices or extensions to the existing PM server?

**Status:** Accepted

**Task Context:** Task 2 (Extend PM Server for Agent Delegation), Task 3 (Integrate Hermes Research Content)

**Context:** Both debaters agreed unanimously. The PM server already contains `resolve_agent_delegates()`. Extracting this into microservices for a validation pipeline adds deployment coordination and network hops with no functional benefit at current scale.

**Decision:** Extend the existing PM server (Bun/Elysia) with both agent delegation and Hermes research integration as internal modules with clean interface boundaries, extractable later if needed.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Zero new services to deploy, no inter-service network hops, no deployment coordination overhead. Module boundaries provide separation of concerns within the codebase.
- *Negative:* PM server grows in responsibility. If Hermes integration becomes complex, extraction will require a later migration.
- *Caveats:* The Hermes integration should be implemented as a pluggable module with a clean interface so it can be extracted into its own service if the research pipeline grows significantly.

---

### [D3] Use existing in-cluster bridge services or integrate directly with Discord/Linear APIs?

**Status:** Accepted

**Task Context:** Task 5 (Enable Discord and Linear Bridge Notifications), Task 9 (Show Pipeline Status Notifications)

**Context:** Both debaters agreed unanimously. `bots/discord-bridge-http` and `bots/linear-bridge` are already deployed and operational in-cluster. Reimplementing their functionality inside the PM server duplicates tested infrastructure and couples the PM server to notification concerns (token management, rate limiting, webhook formatting).

**Decision:** Use the existing `bots/discord-bridge-http` and `bots/linear-bridge` services for all notification delivery.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Reuses tested, deployed infrastructure. Avoids managing Discord bot tokens, Linear API keys, and rate limiting inside the PM server. Aligns with organizational preference for self-hosted in-cluster services.
- *Negative:* PM server depends on bridge service availability. If bridge services are down, notifications fail silently unless error handling is implemented.
- *Caveats:* None raised — both speakers strongly aligned.

---

### [D4] How should delegate_id and research memos be represented in the data model?

**Status:** Accepted

**Task Context:** Task 2 (PM Server Delegation), Task 3 (Hermes Integration), Task 6 (Frontend Assignment Visualization), Task 7 (Frontend Memo Display)

**Context:** Both debaters agreed unanimously. The PRD specifies 1:1 assignment (one delegate per task). Creating separate entities for a 1:1 relationship adds join complexity with no normalization benefit.

**Decision:** Extend the existing task schema with `delegate_id` (string) and `research_memo` (embedded object with `content`, `source`, and `timestamp` fields) directly on the task entity.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Single task fetch returns everything the frontend needs. No joins, simple API surface. Appropriate for the 1:1 cardinality in this validation pipeline.
- *Negative:* If multi-assignment becomes a requirement later, a schema migration will be needed.
- *Caveats:* None — both speakers considered this straightforward given the cardinality.

---

### [D8] Which ingress/CDN solution for Sigma-1?

**Status:** Accepted

**Task Context:** Task 1 (Provision Dev Infrastructure), Task 10 (Production Hardening)

**Context:** Both debaters agreed unanimously. The `cloudflare-operator-system` is deployed with webhook and metrics services active. Cloudflare Tunnel CRDs (`accesstunnels.networking.cfargotunnel.com`, `clustertunnels.networking.cfargotunnel.com`) are registered and operational.

**Decision:** Use the existing Cloudflare operator with networking CRDs (accesstunnels, clustertunnels) for ingress, CDN, and TLS termination. Do not deploy a separate ingress controller.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Zero-trust ingress, automatic TLS, and CDN without deploying additional infrastructure. Task 10 benefits directly from existing CRDs.
- *Negative:* Dependency on Cloudflare ecosystem. Debugging tunnel issues requires Cloudflare-specific knowledge.
- *Caveats:* Deploying NGINX alongside the Cloudflare operator would create dual-ingress complexity — explicitly ruled out.

---

### [D6] Which component library for new frontend UI elements?

**Status:** Accepted (conditional — contingent on frontend tasks remaining in scope per D5)

**Task Context:** Tasks 6, 7, 8, 9 (all frontend visualization tasks)

**Context:** Both debaters agreed on the choice itself. The Pessimist noted this decision is moot if frontend tasks are deferred (per dp-5) but conceded that if frontend work proceeds, shadcn/ui is the obvious choice. The existing `cto/tweakcn` service deployed in-cluster indicates the team is already in the shadcn/ui ecosystem.

**Decision:** Use shadcn/ui (built on Radix UI primitives) for all new components in Tasks 6–9.

**Consensus:** 2/2 (100%) — conditional on frontend tasks being in scope

**Consequences:**
- *Positive:* Components are copy-paste-owned, not imported — no version lock-in. Radix primitives provide accessibility compliance. Most adopted component approach in the Next.js ecosystem. `tweakcn` in-cluster confirms existing team familiarity.
- *Negative:* Copy-paste-own model means the team is responsible for component maintenance and updates.
- *Caveats:* This decision only applies if Tasks 6–9 remain in scope. See [D5] for the scope question.

## 4. Escalated Decisions

### [D2] What API paradigm should be used for PM server integrations? — ESCALATED

**Status:** Pending human decision

**Task Context:** Task 2 (PM Server Delegation), Task 3 (Hermes Integration), Task 5 (Discord/Linear Notifications)

**Options:**

| | Option A (Optimist) | Option B (Pessimist) |
|---|---|---|
| **Approach** | REST for synchronous (Hermes calls in Task 3), NATS pub/sub for asynchronous notifications (Task 5) | REST/HTTP for all integrations — synchronous calls to Hermes and direct HTTP POSTs to bridge services |
| **Async model** | NATS subjects for pipeline events; bridge services subscribe | Direct HTTP POSTs from PM server to bridge services |
| **Failure mode** | NATS subject mismatches are silent failures | HTTP errors are immediate (404, connection refused) |

**Optimist argued:** NATS is already deployed in-cluster (`openclaw-nats.openclaw.svc.cluster.local`). Notifications are textbook fire-and-forget async — Discord and Linear don't need synchronous responses. NATS subjects provide decoupling, retry semantics, and observability. The bridge services can subscribe independently, and the PM server doesn't need to know about notification delivery details.

**Pessimist argued:** The bridge services are HTTP services by name and design (`discord-bridge-http`). NATS requires either adding NATS subscribers to both bridge services or building a NATS-to-HTTP adapter — both add code and testing surface for exactly **two notification calls per pipeline run**. NATS subject misconfiguration (`pipeline.complete` vs `pipeline.completed`) is a silent failure — messages vanish without errors. HTTP failures are immediate and debuggable. The decoupling benefit of pub/sub is irrelevant when there are exactly 2 known consumers. REST everywhere keeps the integration surface uniform.

**Recommendation:** The Pessimist's argument is more compelling for this specific scope. The pipeline fires two notifications per run. The blast radius of failure is a missing Discord message, not data loss. HTTP's immediate error feedback is more valuable than NATS's decoupling for two known consumers. However, if the team plans to add more notification consumers (e.g., Slack, email) beyond this E2E validation, NATS becomes the better long-term choice. **For E2E validation scope, recommend Option B (REST everywhere).** If NATS adoption is a strategic goal, consider it for a follow-up iteration when there are more than 2 consumers.

---

### [D5] How should agent assignments and research memos be visualized? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 6, 7, 8, 9 (all frontend work — 40% of project scope)

**Options:**

| | Option A (Optimist) | Option B (Pessimist) |
|---|---|---|
| **Approach** | Build inline display within task cards — delegate_id as avatar/badge, research memo as collapsible preview | No new frontend visualization; verify delegate_id in Linear's UI and research memos in PR content; defer Tasks 6–9 entirely |
| **Scope impact** | 10 tasks (full scope) | 6 tasks (Tasks 1–5, 10 only) |
| **Acceptance criteria coverage** | Extends beyond AC | Covers all 5 AC as written |

**Optimist argued:** For a validation/E2E pipeline dashboard, information density matters more than progressive disclosure. Operators verifying pipeline correctness need to scan assignments and memo status at a glance without clicking into each task. Inline display with collapsible memo sections gives both scannability and detail access.

**Pessimist argued:** The PRD has five acceptance criteria, none of which mention a web frontend. The criteria say "visible as assignee **in Linear**" — Linear is the UI. Tasks 6–9 represent 40% of the task list and add scope that satisfies zero acceptance criteria. The design context shows `stitch_status=failed`, meaning there are no design artifacts to implement against. Building 4 frontend tasks to visualize data that Linear and GitHub already display is scope creep. Validate the pipeline first; build a dashboard later if needed.

**Recommendation:** The Pessimist raises a legitimate scope concern. The five acceptance criteria are all satisfiable without a custom frontend. However, the original task decomposition included these tasks, suggesting the project sponsor may have intended a dashboard. **This is a scope decision that should be made by the project owner.** Two paths:

1. **Minimal (recommended for speed):** Defer Tasks 6–9. Focus on Tasks 1–5 and a reduced Task 10. Validate the pipeline E2E against all 5 acceptance criteria. This cuts 40% of scope and eliminates frontend dependencies.
2. **Full scope:** Keep Tasks 6–9 if a pipeline dashboard is desired for ongoing operational use beyond this single E2E validation run. If choosing this path, use shadcn/ui per [D6].

---

### [D7] Authentication mechanism for frontend and API? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 6, 7, 8, 9 (frontend auth), Task 10 (production hardening RBAC)

**Options:**

| | Option A (Optimist) | Option B (Pessimist) |
|---|---|---|
| **Approach** | JWT-based authentication with RBAC, validated at ingress/gateway | Cloudflare Access via existing accesstunnels CRD; no application-level auth for E2E scope |
| **Implementation** | JWT issuance, validation middleware, RBAC roles, key rotation | Zero application code — Cloudflare Access handles SSO/MFA at the tunnel layer |
| **Future RBAC** | Built-in from day one | Layer RBAC on `Cf-Access-Jwt-Assertion` header identity later |

**Optimist argued:** JWT-based auth with RBAC is stateless, scales horizontally, and is the standard pattern for Kubernetes-native services. The cluster uses external-secrets operator for key management. Task 10 explicitly mentions RBAC enforcement, so building it now avoids retrofitting.

**Pessimist argued:** Cloudflare Access is already available via the `accesstunnels` CRD (resolved in [D8]). It provides SSO, MFA, and identity at the tunnel layer with zero application code. Building JWT issuance, validation middleware, and RBAC for an internal validation tool is overengineered when infrastructure already provides authentication at the edge. RBAC is a production hardening concern for Task 10, not an E2E validation concern. If RBAC is needed later, it can be layered on the `Cf-Access-Jwt-Assertion` header identity that Cloudflare Access already provides.

**Recommendation:** This decision is strongly coupled to [D5]. If Tasks 6–9 are deferred, there is no frontend auth surface and this decision is moot for E2E scope — Cloudflare Access alone is sufficient. If Tasks 6–9 proceed, Cloudflare Access still provides the authentication layer, but RBAC may need to be added at the application level. **Recommend Option B (Cloudflare Access) for E2E validation scope.** Revisit JWT/RBAC when production hardening (Task 10) is in active development and multi-user access patterns are defined.

## 5. Architecture Overview

### Agreed Approach

The architecture extends the **existing PM server (Bun/Elysia)** rather than introducing new services. All backend pipeline logic — agent delegation, Hermes research integration, design snapshot PR surfacing, and notification dispatch — lives within the PM server as internal modules with clean interface boundaries.

### Technology Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| **Backend** | Bun/Elysia | Existing PM server, extended |
| **Frontend** (if in scope) | React/Next.js with shadcn/ui | Contingent on [D5] resolution |
| **Infrastructure** | Kubernetes/Helm | Existing cluster |
| **Ingress/CDN/TLS** | Cloudflare operator + accesstunnels/clustertunnels CRDs | Resolved in [D8] |
| **Notifications** | Existing `bots/discord-bridge-http` and `bots/linear-bridge` | Resolved in [D3] |
| **Messaging** (pending) | NATS or direct HTTP | Pending [D2] resolution |
| **Auth** (pending) | Cloudflare Access or JWT+RBAC | Pending [D7] resolution |

### Service Architecture

```
┌──────────────────────────────────────────────────────┐
│                  PM Server (Bun/Elysia)              │
│                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │
│  │ Agent        │  │ Hermes       │  │ Design    │  │
│  │ Delegation   │  │ Research     │  │ Snapshot  │  │
│  │ Module       │  │ Module       │  │ Module    │  │
│  └──────┬───────┘  └──────┬───────┘  └─────┬─────┘  │
│         │                 │                 │        │
│  ┌──────┴─────────────────┴─────────────────┴─────┐  │
│  │          Notification Dispatch Layer            │  │
│  └──────────────────┬─────────────────────────────┘  │
└─────────────────────┼────────────────────────────────┘
                      │ HTTP or NATS (pending D2)
          ┌───────────┴───────────┐
          ▼                       ▼
┌──────────────────┐   ┌──────────────────┐
│ discord-bridge-  │   │ linear-bridge    │
│ http             │   │                  │
└──────────────────┘   └──────────────────┘

External Services:
  - Linear API (issue creation with delegate_id)
  - Hermes/NOUS API (research memos, requires NOUS_API_KEY)
  - GitHub API (PR creation in 5dlabs/sigma-1)

Ingress:
  Cloudflare Tunnel → accesstunnel CRD → PM Server / Frontend
```

### Data Model

The task entity is extended with two fields:
- `delegate_id: string` — Linear user ID resolved from agent hint
- `research_memo: { content: string, source: string, timestamp: Date }` — embedded Hermes research output

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| Separate microservices for delegation/research | `resolve_agent_delegates()` already exists in PM server; network hops add latency for zero benefit at current scale |
| Direct Discord/Linear API integration from PM server | Bridge services are deployed, tested, and purpose-built; reimplementation duplicates work |
| NGINX ingress controller | Cloudflare operator is deployed with CRDs active; dual-ingress creates split-brain complexity |
| gRPC for integrations | Protobuf compilation and code generation overhead unjustified for 2–3 integration points |
| Separate data entities for delegate_id/research_memo | 1:1 cardinality; joins add complexity with no normalization benefit |

## 6. Implementation Constraints

### Security Requirements

- All secrets (Linear API key, Discord webhook URL, NOUS_API_KEY, GitHub tokens) must be stored in Kubernetes secrets, not ConfigMaps or environment variables
- Ingress must use Cloudflare Tunnel with TLS termination (no plain HTTP exposure)
- Authentication mechanism pending [D7] resolution — **do not implement JWT or RBAC until this is resolved**
- Task 10 must enforce RBAC for all service accounts and enable audit logging

### Performance Targets

- Pipeline must complete all stages (deliberation → task generation → issue creation) without fatal errors
- At least 5 tasks must be generated with valid agent assignments per run
- Hermes research integration must gracefully degrade (skip, not fail) when NOUS_API_KEY is unavailable

### Operational Requirements

- All services provisioned in a dedicated `sigma-1-dev` namespace
- Service endpoints aggregated in `sigma-1-infra-endpoints` ConfigMap for consistent discovery
- Error handling required for: agent mapping failures (fallback assignee or logged error), notification delivery failures, Hermes API unavailability
- Bridge service integration must handle bridge service downtime without crashing the pipeline

### Service Dependencies and Integration Points

| Service | Cluster Address | Purpose |
|---------|----------------|---------|
| `bots/discord-bridge-http` | In-cluster | Discord notifications |
| `bots/linear-bridge` | In-cluster | Linear notifications |
| `openclaw-nats` | `openclaw-nats.openclaw.svc.cluster.local` | Messaging (if [D2] resolves to NATS) |
| `cloudflare-operator-system` | In-cluster | Ingress, TLS, CDN |
| `cto/tweakcn` | In-cluster | Confirms shadcn/ui ecosystem adoption |

### Organizational Preferences

- Prefer self-hosted in-cluster services over external SaaS equivalents
- Prefer extending existing services over creating new ones
- Prefer copy-paste-own component patterns (shadcn/ui) over opaque library dependencies

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`:** true
- **`frontendTargets`:** web
- **`stitch_status`:** failed — no Stitch design artifacts were generated
- **`stitch_reason`:** (empty — no reason provided)

### Supplied Design Artifacts

None. No design mockups, wireframes, or reference URLs were supplied.

### Implications for Implementation

1. **No visual reference exists.** Frontend tasks (6–9), if they proceed, have no design artifacts to implement against. Implementing agents must use shadcn/ui defaults and their best judgment for layout, spacing, and visual hierarchy.
2. **Stitch generation failed.** There are no AI-generated design candidates to select from. This reinforces the Pessimist's argument in [D5] that frontend work lacks specification.
3. **If frontend tasks proceed:** Agents should use shadcn/ui's default theme (or the team's tweakcn configuration if accessible), Radix UI primitives for accessibility, and a minimal, functional layout focused on data display (task lists, collapsible memos, status badges).
4. **If frontend tasks are deferred:** All validation criteria can be confirmed through Linear's UI, GitHub's PR interface, and Discord channel inspection. No design work is needed.

### 7a. Selected Design Direction

No design selections were provided.

### 7b. Design Deliberation Decisions

No design deliberation was conducted.

## 8. Open Questions

The following items were not resolved in deliberation and are non-blocking. Implementing agents should use their best judgment.

1. **Fallback behavior for failed agent mapping (Task 2):** When `resolve_agent_delegates()` cannot map an agent hint to a Linear user ID, should the issue be created unassigned with a log entry, or assigned to a default/fallback user? The PRD implies issues should have assignees; a logged warning with unassigned creation is the safer default.

2. **Research memo format and depth (Task 3):** The embedded `research_memo` object structure (`content`, `source`, `timestamp`) is agreed, but the expected content length and format from Hermes is unspecified. Agents should accept whatever Hermes returns and store it verbatim.

3. **Design snapshot trigger conditions (Task 4):** The PRD says "Design snapshot PR surfacing works" but doesn't define what constitutes a "design change" that triggers PR creation. Given `stitch_status=failed`, implementing agents should trigger PR creation for any task scaffold generation and include whatever artifacts the pipeline produces.

4. **Notification payload format (Task 5):** The exact structure of Discord and Linear notification payloads (what fields, what formatting) is left to implementing agents. Should include at minimum: pipeline status, link to Linear session, link to PR, and task count summary.

5. **Task 10 scope reduction:** If Tasks 6–9 are deferred per [D5], Task 10's dependency list should be updated to `[2, 3, 4, 5]` only. Production hardening scope (HA, RBAC, audit logging) applies to whatever services remain in scope.

6. **NATS infrastructure in Task 1:** If [D2] resolves to REST-everywhere, the NATS provisioning step in Task 1 may be unnecessary (NATS is already deployed in-cluster regardless, but no new NATS configuration would be needed for this pipeline).

