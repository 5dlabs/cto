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

The initial task decomposition identified **10 tasks** spanning infrastructure provisioning, backend pipeline extension, frontend surfacing, end-to-end validation, and production hardening.

### Task Summary

| ID | Title | Agent | Stack | Priority | Dependencies |
|----|-------|-------|-------|----------|-------------|
| 1 | Provision Dev Infrastructure for Sigma-1 E2E Pipeline | Bolt | Kubernetes/Helm | High | — |
| 2 | Implement Agent Delegate Resolution in PM Server | Nova | Bun/Elysia | High | 1 |
| 3 | Integrate Hermes Research in Deliberation Path | Nova | Bun/Elysia | High | 1 |
| 4 | Implement Design Snapshot PR Surfacing | Blaze | React/Next.js | Medium | 1 |
| 5 | Implement Discord and Linear Bridge Notifications | Nova | Bun/Elysia | High | 1 |
| 6 | Validate End-to-End Pipeline Completion | Tess | Test frameworks | High | 2, 3, 4, 5 |
| 7 | Verify Linear Issues Have Delegate Assignments | Tess | Test frameworks | High | 2, 6 |
| 8 | Validate Hermes Research Memo Content | Tess | Test frameworks | Medium | 3, 6 |
| 9 | Validate Design Snapshot PR Surfacing in Frontend | Tess | Test frameworks | Medium | 4, 6 |
| 10 | Production Hardening: HA, Ingress, and Security | Bolt | Kubernetes/Helm | Medium | 2–9 |

### Key Services & Components

- **PM Server (Bun/Elysia)** — Core orchestrator; owns `resolve_agent_delegates()`, Hermes integration, Linear issue creation, and notification dispatch (Tasks 2, 3, 5)
- **Web Frontend (React/Next.js)** — Design snapshot PR surfacing dashboard component (Task 4)
- **In-cluster Bridges** — `bots/discord-bridge-http` and `bots/linear-bridge` for notifications (Task 5)
- **External SaaS** — Hermes API via `NOUS_API_KEY` (Task 3)
- **Infrastructure Operators** — CloudNative-PG (Postgres), Redis, NATS, external-secrets, Cilium (Task 1, 10)

### Agent Assignments

- **Bolt** — Infrastructure provisioning and production hardening (Tasks 1, 10)
- **Nova** — Backend pipeline logic in Bun/Elysia (Tasks 2, 3, 5)
- **Blaze** — Frontend React/Next.js work (Task 4)
- **Tess** — End-to-end and integration testing (Tasks 6, 7, 8, 9)

### Cross-Cutting Concerns

- Secret management across 4+ sensitive tokens (Linear, GitHub, NOUS_API_KEY, Discord)
- Service-to-service authentication between PM server and bridge services
- Pipeline orchestration pattern (affects Tasks 2, 3, 5, 6, 10)
- RBAC and network policy enforcement (Tasks 1, 10)
- Hermes availability gating (Tasks 3, 8)

## 3. Resolved Decisions

Nine decision points were raised during deliberation. Seven reached immediate consensus; two were debated substantively and resolved by the committee.

---

### [D1] Should delegate resolution and Hermes integration be PM server extensions or separate microservices?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 5 (Agent Delegate Resolution, Hermes Integration, Notifications)

**Context:** The Optimist argued that `resolve_agent_delegates()` already lives in the PM server, all three tasks share the same agent/stack, and microservice splitting adds deployment surface without scaling justification. The Pessimist agreed but raised the operational concern that a slow Hermes SaaS response could block the shared Bun event loop.

**Decision:** Extend the existing PM server (Bun/Elysia) with new endpoints and logic.

**Consensus:** 2/2 (100%) — both debaters agreed on extension.

**Consequences:**
- **Positive:** Single deployment target, shared testing infrastructure, no new network hops, lower operational overhead for a validation run.
- **Negative / Caveats:** The Hermes call path in Task 3 **must** implement a circuit breaker and explicit timeout to prevent external API latency from cascading into delegate resolution. This is an implementation requirement, not an optional nice-to-have. Bun's single-threaded event loop makes this failure mode concrete.

---

### [D2] Which notification bridge services for Discord and Linear?

**Status:** Accepted

**Task Context:** Task 5 (Discord and Linear Bridge Notifications)

**Context:** Both debaters immediately agreed. In-cluster bridges are deployed and the PRD requires only start/complete notifications.

**Decision:** Use the existing in-cluster `bots/discord-bridge-http` and `bots/linear-bridge` services.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Zero additional cost, no external dependency, no egress latency, aligns with organizational self-hosted preference.
- **Negative:** Bridge service availability becomes a dependency; if bridges are down, notifications fail. Task 5 should implement graceful degradation (log and continue, don't block pipeline).

---

### [D3] How should agent delegate assignments be represented in Linear issue creation?

**Status:** Accepted

**Task Context:** Tasks 2, 7 (Delegate Resolution, Delegate Assignment Verification)

**Context:** The PRD acceptance criterion is unambiguous: "Issues have delegate_id set (visible as assignee in Linear)." Custom fields or labels do not satisfy this requirement.

**Decision:** Set `delegate_id` as the Linear `assigneeId` field directly.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Directly satisfies acceptance criteria. Task 7 validation is straightforward — query the issue and assert `assignee` is non-null and correct.
- **Negative:** Requires that delegate IDs are valid Linear user IDs. Task 2 must handle the case where a delegate ID does not map to a valid Linear user (log error, create issue unassigned, flag for review).

---

### [D4] REST or GraphQL for new PM server endpoints?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 5 (all Nova/Bun/Elysia tasks)

**Context:** Consumers are internal pipeline stages, not external clients needing flexible queries. The existing PM server uses REST via Elysia.

**Decision:** RESTful HTTP endpoints following the existing Elysia pattern.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Consistency with existing codebase, no new schema dependencies, lower cognitive overhead.
- **Negative:** None raised. The scope (2–3 new endpoints) doesn't justify GraphQL overhead.

---

### [D5] Service-to-service auth mechanism?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 5, 10 (Infrastructure, PM Server extensions, Notifications, Production Hardening)

**Context:** The Optimist proposed Kubernetes service account tokens with RBAC. The Pessimist pushed back strongly: K8s SA token projection requires validation middleware in every consuming service, the existing bridge services (`bots/discord-bridge-http`, `bots/linear-bridge`) likely don't support SA token validation, and retrofitting auth into deployed bridges is scope creep for a validation run. API keys managed by external-secrets are operationally simpler — one secret, one header check.

**Decision:** API key-based authentication with keys managed by the external-secrets operator.

**Consensus:** Pessimist's position prevailed. The Optimist did not counter the practical argument about existing bridge service compatibility.

**Consequences:**
- **Positive:** Operationally simple for a validation run. One shared API key, distributed via external-secrets (already agreed in D9), with a single header check in consuming services. No middleware retrofitting needed for existing bridges.
- **Negative:** A shared API key is less granular than per-service-account authentication. For production hardening (Task 10), consider migrating to mTLS or SA tokens if the bridge services are updated to support them.
- **Caveats:** The Optimist correctly noted that K8s RBAC and Cilium network policies should still be enforced as a defense-in-depth layer — the API key is the application-level auth, but network segmentation via Cilium remains mandatory.

---

### [D6] Self-hosted Hermes or external SaaS API?

**Status:** Accepted

**Task Context:** Tasks 3, 8 (Hermes Integration, Hermes Validation)

**Context:** The PRD explicitly references `NOUS_API_KEY` as the integration path. No Hermes instance exists in the cluster infrastructure.

**Decision:** Use the external Hermes SaaS API via `NOUS_API_KEY`.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** No deployment scope creep. PRD-aligned. Tasks 3 and 8 already assume API key gating.
- **Negative:** External SaaS dependency introduces latency and availability risk. Mitigated by the circuit breaker requirement from D1.

---

### [D7] Dedicated dashboard section or activity feed integration for design PRs?

**Status:** Accepted

**Task Context:** Tasks 4, 9 (Design Snapshot PR Surfacing, Frontend Validation)

**Context:** Task 4 is specifically scoped to PR surfacing. Task 9 validates "accurate metadata and links" — deterministic assertions require a known DOM structure, not items buried in a mixed-content feed.

**Decision:** Dedicated section in the dashboard with metadata, status, and links.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Clear discoverability for users. Deterministic test assertions for Task 9. Clean component boundary for React/Next.js implementation.
- **Negative:** Adds a new dashboard section that must be maintained. Acceptable for a validation run.

---

### [D8] Event-driven (NATS) or synchronous request-response for pipeline orchestration?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 5, 6, 10 (Infrastructure, PM Server extensions, Notifications, E2E Validation, Production Hardening)

**Context:** This was the most substantive disagreement in the deliberation. The Optimist argued for NATS-based event-driven orchestration citing deployed infrastructure, natural stage boundaries, independent retries, and observable stage transitions. The Pessimist countered forcefully: the pipeline is a **linear sequence processing one PRD at a time** with no fan-out or concurrent load. NATS adds message ordering complexity (stages must execute sequentially), requires a completion aggregator for Task 6 E2E validation, makes debugging harder (distributed traces vs. stack traces), and introduces NATS itself as a new SPOF. The Pessimist asked the Optimist to "show me the fan-out" — and the Optimist did not provide one in the subsequent turn.

**Decision:** Synchronous request-response with the PM server as orchestrator.

**Consensus:** Pessimist's position prevailed. The pipeline's linear, single-PRD nature makes event-driven orchestration complexity without a customer.

**Consequences:**
- **Positive:** Free ordering (call sequence = execution sequence). Simple error handling (one log stream, one stack trace). Task 6 E2E validation calls one endpoint and asserts on the response. No message ordering logic. No completion aggregator. Easier to debug at 2am.
- **Negative:** No decoupled retry without explicit implementation at each call boundary. If the PM server goes down, the entire pipeline stops — but this is true regardless of orchestration pattern since the PM server is the orchestrator either way.
- **Caveats:** NATS remains deployed in-cluster and available for future use if the pipeline evolves to support concurrent PRD processing or fan-out. This decision is scope-appropriate for a validation run, not a permanent architecture ban. Task 5 (notifications) should implement retries at the HTTP call level.

---

### [D9] External-secrets operator or manual Kubernetes Secrets?

**Status:** Accepted

**Task Context:** Tasks 1, 10 (Infrastructure Provisioning, Production Hardening)

**Context:** The external-secrets operator and CRDs are deployed in-cluster. Task 10 requires secret rotation. The pipeline handles 4+ sensitive tokens.

**Decision:** Use the existing external-secrets operator and CRDs.

**Consensus:** 2/2 (100%)

**Consequences:**
- **Positive:** Automated rotation, audit trail, and scalable secret management out of the box. No manual secret creation/rotation.
- **Negative:** Requires that the backing secret store (e.g., AWS Secrets Manager, Vault) is properly configured. Task 1 should verify connectivity to the secret store as part of infrastructure provisioning.

## 4. Escalated Decisions

No decisions were escalated. All nine decision points reached resolution during the two-turn deliberation.

## 5. Architecture Overview

### Agreed Approach

The Sigma-1 E2E pipeline is implemented as a **monolithic extension of the existing PM server**, using **synchronous request-response orchestration** with **API key authentication** and **external-secrets-managed credentials**.

### Technology Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Backend / Orchestrator | Bun + Elysia | Existing PM server, extended with new endpoints |
| Frontend | React / Next.js | Design snapshot PR dashboard component |
| Database | PostgreSQL (CloudNative-PG) | Existing operator |
| Cache | Redis | Existing in-cluster |
| Messaging | NATS (available, **not used** for pipeline orchestration) | Reserved for future fan-out needs |
| Secret Management | external-secrets operator | Backed by external secret store |
| Network Security | Cilium CNI + Network Policies | Defense-in-depth with API key auth |
| External Services | Hermes SaaS API, Linear API, GitHub API | Via managed API keys |
| Notification Bridges | `bots/discord-bridge-http`, `bots/linear-bridge` | In-cluster, self-hosted |

### Service Architecture

```
┌─────────────────────────────────────────────┐
│                 PM Server (Bun/Elysia)       │
│                                              │
│  ┌──────────────┐  ┌──────────────────────┐  │
│  │ resolve_agent │  │ Hermes Integration   │  │
│  │ _delegates()  │  │ (circuit breaker +   │  │
│  │              │  │  timeout)            │  │
│  └──────┬───────┘  └──────────┬───────────┘  │
│         │                     │              │
│  ┌──────▼─────────────────────▼───────────┐  │
│  │     Pipeline Orchestrator (sync)       │  │
│  │  deliberation → tasks → issues → notify │  │
│  └──────┬──────────┬──────────┬───────────┘  │
│         │          │          │              │
└─────────┼──────────┼──────────┼──────────────┘
          │          │          │
    ┌─────▼───┐ ┌────▼────┐ ┌──▼──────────────┐
    │ Linear  │ │ GitHub  │ │ Notification     │
    │ API     │ │ API     │ │ Bridges          │
    │         │ │         │ │ (Discord/Linear) │
    └─────────┘ └─────────┘ └──────────────────┘
```

### Key Patterns

1. **Synchronous Orchestration** — The PM server calls each pipeline stage in sequence. Ordering is implicit in the call chain. Errors are caught at each boundary with explicit logging.
2. **Circuit Breaker on External APIs** — The Hermes SaaS call path must implement a circuit breaker with configurable timeout to prevent external latency from blocking internal operations.
3. **API Key Auth** — A shared API key, managed by external-secrets, is used for service-to-service authentication. Consuming services validate via a single header check.
4. **Graceful Degradation** — Notification failures (Task 5) must not block pipeline completion. Hermes unavailability (Task 3) falls back to default research memo behavior.
5. **Native Linear Assignee** — `delegate_id` maps directly to Linear's `assigneeId` field. No custom fields or labels.

### Explicitly Ruled Out

| Option | Reason |
|--------|--------|
| Separate microservices for delegate resolution / Hermes | No scaling justification; adds deployment surface and network hops for a validation run |
| GraphQL endpoints | Only 2–3 internal endpoints needed; GraphQL schema overhead unjustified |
| External SaaS notification bridges (Zapier/Pipedream) | In-cluster bridges deployed; external adds cost, latency, and dependency |
| Self-hosted Hermes instance | Not in cluster; deploying one is scope creep for validation |
| NATS event-driven orchestration | Pipeline is linear/sequential with no fan-out; adds ordering complexity, completion aggregation, and debugging overhead |
| K8s SA token-based service-to-service auth | Requires middleware retrofitting in existing bridge services; scope creep for validation |
| Manual Kubernetes Secrets | Operator deployed; manual secrets don't rotate or audit |

## 6. Implementation Constraints

### Security Requirements

- **All secrets** (NOUS_API_KEY, Linear API token, GitHub token, Discord webhook) **must** be managed via external-secrets operator ExternalSecret CRDs. No hardcoded secrets, no manual `kubectl create secret`.
- **API key authentication** is required for all service-to-service calls. The key must be passed in an HTTP header and validated by the receiving service.
- **Cilium network policies** must restrict traffic to only necessary service-to-service paths. No unrestricted pod-to-pod communication.
- **RBAC** must be enforced for all Kubernetes service accounts (Task 10).

### Performance Targets

- **Hermes API timeout**: Configurable, default 30 seconds. Circuit breaker must open after 3 consecutive failures and half-open after 60 seconds.
- **Pipeline completion**: The full pipeline (deliberation → task generation → issue creation → notifications) should complete within 5 minutes for a single PRD under normal conditions.
- **Notification delivery**: Best-effort with retry (3 attempts, exponential backoff). Notification failure must not block pipeline completion.

### Operational Requirements

- **Single log stream**: The synchronous orchestration pattern means all pipeline stages log to the PM server's log stream. Structured logging with stage identifiers is required for debuggability.
- **Idempotency**: Issue creation and PR creation should be idempotent — re-running the pipeline with the same PRD should not create duplicate issues or PRs.
- **Graceful degradation**: Hermes unavailability → fallback to default research memos. Bridge unavailability → log warning, continue pipeline. Invalid delegate_id → create issue unassigned, log error.

### Service Dependencies and Integration Points

| Service | Location | Auth Method | Required By |
|---------|----------|-------------|-------------|
| Linear API | External SaaS | API token (external-secrets) | Tasks 2, 5, 7 |
| GitHub API | External SaaS | API token (external-secrets) | Tasks 4, 6 |
| Hermes API | External SaaS | NOUS_API_KEY (external-secrets) | Tasks 3, 8 |
| `bots/discord-bridge-http` | In-cluster | API key (external-secrets) | Task 5 |
| `bots/linear-bridge` | In-cluster | API key (external-secrets) | Task 5 |
| PostgreSQL (CloudNative-PG) | In-cluster | Connection string (ConfigMap) | Tasks 2, 3 |
| Redis | In-cluster | Connection string (ConfigMap) | Tasks 2, 3 |

### Organizational Preferences

- **Prefer self-hosted services** when available (bridges, operators, secret management).
- **Extend existing services** rather than creating new ones for validation-scoped work.
- **Consistent patterns**: Follow existing Elysia REST conventions. No new paradigms without justification.

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`**: `true`
- **`frontendTargets`**: `web`
- **Provider mode**: `stitch` (ingest + stitch)
- **Stitch status**: `failed` — no design artifacts were generated by the Stitch provider
- **Framer status**: `skipped` (not requested)

### Supplied Design Artifacts

No design artifacts, reference URLs, or component-library artifacts were supplied in the design context.

### Implications for Implementation

1. **Task 4 (Design Snapshot PR Surfacing)** targets a **web** frontend using **React/Next.js**. Since no design artifacts were generated or supplied, Blaze should implement the dedicated dashboard section using the existing application's design system and component patterns.
2. **Task 9 (Frontend Validation)** should assert on component structure and data correctness rather than pixel-perfect visual matching, given the absence of design references.
3. The Stitch failure does not block implementation — it means no external design mockups are available. The implementing agent should follow the existing UI conventions of the application.

### 7a. Selected Design Direction

No design selections were provided.

### 7b. Design Deliberation Decisions

No design deliberation results were provided.

## 8. Open Questions

The following items were not resolved in deliberation and are left to implementing agents' best judgment:

1. **Hermes circuit breaker library**: Which circuit breaker implementation should be used in the Bun/Elysia environment? Options include `opossum`, a custom implementation, or Elysia middleware. Agent Nova should choose based on what's already in the dependency tree or lightest to add.

2. **ConfigMap structure for `sigma-1-infra-endpoints`**: The exact key names and format for the ConfigMap aggregating service endpoints (Task 1) are not specified. Agent Bolt should follow existing cluster conventions for ConfigMap naming.

3. **PR scaffold format**: The PRD requires "PR created in sigma-1 with generated artifacts" but does not specify the directory structure or file format for task scaffolds. Agent Nova should follow existing conventions in the `sigma-1` repository.

4. **Dashboard component placement**: Task 4 specifies a "dedicated section" but the exact route, layout position, and navigation entry point within the Next.js app are unspecified. Agent Blaze should integrate following existing dashboard patterns.

5. **Retry policy for Linear/GitHub API calls**: The deliberation established retries for notifications (Task 5) but did not specify retry policy for core Linear issue creation (Task 2) or GitHub PR creation. Implementing agents should apply reasonable defaults (3 retries, exponential backoff).

6. **Test framework selection for Tess**: Tasks 6–9 are assigned to Tess with "Test frameworks" as the stack. The specific framework (Jest, Vitest, Playwright for frontend, etc.) is left to Tess based on what's already configured in the repository.

7. **Production hardening scope (Task 10)**: The extent of HA scaling, specific ingress controller configuration, and CDN integration details are left to Agent Bolt's judgment based on cluster capabilities. This task is lower priority and depends on all validation tasks completing first.

