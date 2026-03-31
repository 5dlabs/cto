

# Enhanced PRD

## 1. Original Requirements

> # Project: Sigma 1 Hermes E2E
>
> ## Goal
> Validate the intake pipeline end to end with Hermes enabled in the deliberation research path while preserving design snapshot PR surfacing.
>
> ## Scope
> Generate a concise but production-minded task plan for a creative operations platform update.
>
> ## Requirements
> - include a web experience with a clear staging/production distinction
> - preserve current-site screenshots and generated variant snapshots in output artifacts
> - call out rollout risks, migration concerns, and infrastructure sequencing
> - produce implementation-ready tasks for engineering agents

## 2. Project Scope

The initial task decomposition identified **10 tasks** spanning infrastructure, backend, frontend, testing, documentation, and production hardening. Three agents and three technology stacks are involved.

### Tasks Identified

| ID | Title | Agent | Stack | Priority |
|----|-------|-------|-------|----------|
| 1 | Provision Dev and Staging Infrastructure | bolt | Kubernetes/Helm | High |
| 2 | Implement Hermes Deliberation Path API | nova | Bun/Elysia | High |
| 3 | Integrate Hermes Path with Snapshot Artifact Generation | nova | Bun/Elysia | High |
| 4 | Update Web Experience for Hermes Path Surfacing | blaze | React/Next.js | High |
| 5 | Preserve and Migrate Existing Snapshot Artifacts | nova | Bun/Elysia | Medium |
| 6 | Implement Rollout and Migration Risk Logging | nova | Bun/Elysia | Medium |
| 7 | Automate E2E Testing for Hermes Intake Pipeline | tess | Test frameworks | High |
| 8 | Document Migration and Rollout Plan | atlas | CI/CD platforms | Medium |
| 9 | Production Hardening: HA, CDN, TLS, Ingress | bolt | Kubernetes/Helm | High |
| 10 | Production Hardening: RBAC, Secret Rotation, Audit Logging | bolt | Kubernetes/Helm | High |

### Key Services and Components

- **Backend service**: Bun/Elysia Node.js runtime — handles Hermes deliberation API, snapshot artifact generation, artifact migration, and structured logging
- **Frontend**: React/Next.js web application — surfaces Hermes results, snapshot artifacts, and environment distinction
- **Infrastructure**: Kubernetes cluster with CloudNative-PG (Postgres), Redis, NATS operators; MinIO for object storage; Loki for logging; ArgoCD for GitOps (present in cluster)
- **Testing**: E2E test suite covering both API and browser automation

### Agent Assignments

- **bolt** (Kubernetes/Helm): Tasks 1, 9, 10 — all infrastructure provisioning and production hardening
- **nova** (Bun/Elysia): Tasks 2, 3, 5, 6 — all backend application logic
- **blaze** (React/Next.js): Task 4 — frontend web experience
- **tess** (Test frameworks): Task 7 — E2E test automation
- **atlas** (CI/CD platforms): Task 8 — documentation and rollout planning

### Cross-Cutting Concerns

- **9 decision points** were identified across tasks, covering architecture (dp-1), storage (dp-2), API design (dp-3), UX behavior (dp-4), security/auth (dp-5), data model (dp-6), logging (dp-7), testing framework (dp-8), and frontend component approach (dp-9)
- Dependency chain is sequential: Infrastructure (T1) → API (T2) → Artifact integration (T3) → Frontend (T4) / Migration (T5) / Logging (T6) → Testing (T7) / Documentation (T8) → Production hardening (T9) → RBAC hardening (T10)
- Deployment promotion path (staging → production) was identified as a gap during deliberation

## 3. Resolved Decisions

### [D1] Should the Hermes deliberation path be a tightly-coupled extension of the existing Bun/Elysia service or a separate microservice?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 5, 6 (all nova/Bun/Elysia backend tasks)

**Context:** The Optimist argued that the PRD scope is validation of an intake pipeline path, not a new product line, and that extracting to a microservice introduces deployment coordination overhead with no scaling benefit. The Pessimist explicitly concurred, calling modular monolith "correct for a validation pipeline."

**Decision:** Extend the existing Bun/Elysia service as a well-bounded internal module with clear interface contracts. NATS (already deployed in-cluster) is available for future decoupling if load data justifies it.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Simpler deployment — no cross-service orchestration, no service mesh complexity
- ✅ Task 1 provisioning is simpler; no additional service discovery or auth needed
- ✅ Future extraction path via NATS events is a low-cost refactor
- ⚠️ Module boundaries must be enforced via clear interface contracts to prevent coupling creep

---

### [D2] Which storage backend for snapshot artifact preservation?

**Status:** Accepted

**Task Context:** Tasks 3, 5 (artifact generation and migration)

**Context:** Both debaters agreed on MinIO as the platform choice given organizational preference for self-hosted services and its existing in-cluster deployment. The Pessimist raised a critical isolation caveat: `gitlab/gitlab-minio-svc` is owned by GitLab's Helm chart, meaning GitLab upgrades could disrupt Hermes artifact writes.

**Decision:** MinIO (in-cluster, S3-compatible) with a **dedicated bucket and independent credentials** — not sharing GitLab's MinIO instance without isolation. Task 1 must provision either a separate MinIO tenant or at minimum a dedicated bucket with independent lifecycle policy.

**Consensus:** 2/2 (100% — agreed on platform, Pessimist's isolation caveat incorporated)

**Consequences:**
- ✅ Zero egress costs, zero external credentials management, in-cluster low-latency writes
- ✅ S3 API compatibility means migration to external storage requires only endpoint config change
- ✅ Dedicated bucket/credentials isolates from GitLab Helm chart lifecycle
- ⚠️ Task 1 must explicitly provision the isolated MinIO bucket — not use gitlab-minio-svc directly
- ⚠️ MinIO availability monitoring should be included in Task 6's logging scope

---

### [D3] REST, GraphQL, or gRPC for Hermes deliberation path endpoints?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 7 (API implementation and E2E testing)

**Context:** The Optimist argued that Elysia has first-class OpenAPI/Swagger generation, the use case is straightforward request/response patterns, and REST gives Task 7 the simplest integration surface. The Pessimist concurred without objection.

**Decision:** RESTful endpoints via Elysia's built-in route handlers with OpenAPI schema generation (`@elysiajs/swagger`).

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Elysia's type-safe route definitions provide strong typing without protobuf compilation
- ✅ OpenAPI spec auto-generation aids documentation and client generation
- ✅ Every E2E testing framework speaks HTTP natively — simplifies Task 7
- ⚠️ If complex relational queries emerge later, a GraphQL layer can be added atop REST

---

### [D4] How should the web experience distinguish staging from production?

**Status:** Accepted

**Task Context:** Task 4 (frontend web experience)

**Context:** The Optimist proposed combining both a persistent banner and accent color theming, citing accessibility concerns (color-blind users) and industry precedent (Vercel, AWS Console, Stripe). The Pessimist concurred without objection.

**Decision:** Persistent environment banner (top bar, e.g., "⚠ STAGING") **combined** with accent color theming (e.g., amber for staging, brand color for production). Implementation via env var injection at build time — conditionally render an `<EnvironmentBanner>` component and swap a CSS custom property.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Dual-signal approach (text + color) is accessible and unmistakable
- ✅ Minimal implementation cost — env var + conditional render + CSS custom property
- ✅ No additional confirmation step needed (avoids friction for a read-heavy tool)
- ⚠️ Feature flags for Hermes path visibility (Task 4 details) should respect the same environment awareness

---

### [D5] What auth model for the Hermes path and artifact access?

**Status:** Accepted

**Task Context:** Tasks 2, 4, 9, 10 (API, frontend, production hardening, RBAC)

**Context:** The Optimist argued that introducing JWT or OAuth2 alongside existing sessions creates two auth planes and a security surface area nightmare. Extending sessions with RBAC claims maintains a single auth boundary. The Pessimist concurred, noting that NATS has built-in NKey/JWT auth available for future service-to-service needs.

**Decision:** Reuse existing session-based authentication. Add scoped RBAC claims for Hermes-specific resources (e.g., `hermes:read`, `hermes:trigger`). Single auth boundary.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Single auth plane — no dual-auth complexity for Tasks 9 and 10
- ✅ Task 10's RBAC implementation is a natural extension of existing sessions
- ✅ NATS NKey/JWT available if service-to-service auth is needed later
- ⚠️ RBAC claim granularity must be defined early — Task 2 and Task 10 should agree on claim names

---

### [D7] Which logging/monitoring stack for rollout and migration risk tracking?

**Status:** Accepted

**Task Context:** Tasks 6, 9, 10 (logging, production hardening)

**Context:** The Optimist noted that Loki (with canary) is already operational in-cluster. The Pessimist concurred. Both agreed that structured JSON logging with queryable fields (`rollout_phase`, `migration_step`, `error_code`) via LogQL is the appropriate approach.

**Decision:** Loki (existing deployment: `openclaw/loki-*` services) with structured JSON logging from the Bun/Elysia service. Grafana dashboards for visualization of rollback triggers.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ No new infrastructure to provision — Loki + canary already operational
- ✅ LogQL supports structured field queries needed by Task 6
- ✅ Grafana companion dashboards can visualize rollout health
- ⚠️ If Loki proves insufficient during Task 6, revisit — but unlikely for validation pipeline scale

---

### [D8] Custom E2E test harness vs. open-source framework?

**Status:** Accepted

**Task Context:** Task 7 (E2E testing)

**Context:** The Optimist recommended Playwright for its dual capability (browser automation for Task 4's frontend + API testing via `request` context for Tasks 2/3) in a single framework, citing 75k+ GitHub stars and Microsoft backing. The Pessimist concurred.

**Decision:** Playwright for E2E testing — browser automation for frontend verification and API testing via `request` context for backend validation.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Single framework covers both frontend browser tests and backend API tests
- ✅ Trace viewer and CI integration compatible with existing GitHub Actions runners (in-cluster via `actions.github.com` CRDs)
- ✅ Multi-browser support and parallelism model
- ⚠️ No custom harness — reduces maintenance burden per research memo guidance

---

### [D9] What component approach for Hermes path surfacing in the frontend?

**Status:** Accepted

**Task Context:** Task 4 (frontend web experience)

**Context:** The Optimist proposed shadcn/ui (built on Radix primitives) as copy-paste, ownable components with zero vendor lock-in and accessibility primitives built in. The Pessimist concurred.

**Decision:** Integrate shadcn/ui components (built on Radix accessibility primitives) for new Hermes-specific UI elements (cards, comparison views, status indicators), composed within the existing component library. Tailwind CSS foundation.

**Consensus:** 2/2 (100%)

**Consequences:**
- ✅ Components live in the codebase — no runtime dependency or vendor lock-in
- ✅ Radix primitives ensure accessibility compliance out of the box
- ✅ Tailwind foundation aligns with modern Next.js patterns
- ⚠️ Task 4's real complexity is data surfacing logic, not component chrome — allocate effort accordingly

## 4. Escalated Decisions

### [D6] How to model snapshot artifacts to support both legacy and Hermes paths? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 3, 5 (artifact generation and migration)

**Options:**

| | Option A: Schema Extension | Option B: Parallel Table |
|---|---|---|
| **Approach** | Extend existing artifact table with `source` enum (`'legacy'` \| `'hermes'`), JSONB `metadata` column, `schema_version` integer | Create `hermes_artifacts` table with FK to existing artifact IDs |
| **Migration type** | ALTER TABLE (mutative) | CREATE TABLE (additive) |
| **Query model** | Single table, discriminated by `source` field | JOIN across two tables for Hermes artifacts |
| **Legacy impact** | Existing rows get default values for new columns | Zero changes to existing table |

**Optimist argued:** A parallel schema doubles query complexity and forces Task 5 to maintain two codepaths indefinitely. A discriminated union on `source` lets legacy artifacts remain untouched (default value), Hermes artifacts carry deliberation metadata, and Task 3 writes to one table with one set of indexes. The `schema_version` field gives forward migration safety. This is proportionate to the actual requirement — "add a provenance field to artifacts."

**Pessimist argued:** ALTER TABLE on a live artifact table risks lock contention if Task 5's migration runs while Task 3's snapshot generation is active, and risks silent failures if legacy code encounters unexpected NULL values in new columns. An additive CREATE TABLE has zero blast radius on existing data. Consolidation can happen post-validation when the schema is proven. For a *validation* pipeline, you don't need to get the schema right on day one.

**Recommendation:** The Pessimist's concern about ALTER TABLE lock contention on a production table is operationally valid and hard to mitigate without downtime coordination. However, the Optimist's concern about long-term dual-codepath maintenance is equally valid. A pragmatic middle path exists: **use the parallel table approach (Option B) for the validation phase** to minimize risk to production data, with an explicit consolidation task planned post-validation. This gives the safety of additive migration now while committing to the Optimist's cleaner single-table model as a follow-up. The human should decide whether (A) the existing artifact table is small enough and the deployment window flexible enough to tolerate an ALTER TABLE safely, or (B) the additive approach is warranted given production risk tolerance.

## 5. Architecture Overview

### Technology Stack

| Layer | Technology | Version Notes |
|-------|-----------|---------------|
| **Runtime** | Bun | (as deployed) |
| **API Framework** | Elysia | With `@elysiajs/swagger` for OpenAPI generation |
| **Frontend** | React / Next.js | With shadcn/ui (Radix + Tailwind CSS) |
| **Database** | PostgreSQL via CloudNative-PG operator | Single-replica dev/staging, HA for production |
| **Cache** | Redis via operator | |
| **Messaging** | NATS via operator | Available for future decoupling |
| **Object Storage** | MinIO (in-cluster, S3-compatible) | Dedicated bucket, isolated from GitLab's instance |
| **Logging** | Loki + Grafana | Existing in-cluster deployment |
| **E2E Testing** | Playwright | Browser + API testing |
| **CI/CD** | GitHub Actions (in-cluster runners) | ArgoCD available for GitOps promotion |
| **Orchestration** | Kubernetes + Helm | |

### Service Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                     │
│                                                          │
│  ┌──────────────┐    ┌──────────────────────────────┐   │
│  │  Next.js App  │───▶│   Bun/Elysia Service          │   │
│  │  (blaze)      │    │   (nova)                      │   │
│  │               │    │                                │   │
│  │  - Env banner │    │  ┌─────────────────────────┐  │   │
│  │  - shadcn/ui  │    │  │  Hermes Module          │  │   │
│  │  - Artifact   │    │  │  (internal boundary)    │  │   │
│  │    viewer     │    │  │  - Deliberation API     │  │   │
│  └──────────────┘    │  │  - Snapshot trigger      │  │   │
│                       │  │  - Artifact writer       │  │   │
│                       │  └─────────────────────────┘  │   │
│                       │                                │   │
│                       │  ┌──────────┐ ┌────────────┐  │   │
│                       │  │ Legacy   │ │ RBAC/Auth  │  │   │
│                       │  │ Pipeline │ │ (sessions) │  │   │
│                       │  └──────────┘ └────────────┘  │   │
│                       └──────┬───────┬───────┬────────┘   │
│                              │       │       │            │
│                    ┌─────────▼┐  ┌───▼──┐  ┌─▼──────┐    │
│                    │PostgreSQL│  │Redis │  │ MinIO  │    │
│                    │(CNPG)   │  │      │  │(dedicd)│    │
│                    └─────────┘  └──────┘  └────────┘    │
│                                                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  NATS    │  │  Loki/Grafana│  │  ArgoCD (GitOps) │   │
│  │(future)  │  │  (logging)   │  │  (promotion)     │   │
│  └──────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Communication Patterns

- **Frontend → Backend:** REST (HTTP) via Elysia route handlers, OpenAPI-documented
- **Backend → Storage:** S3 API to dedicated MinIO bucket
- **Backend → Database:** Direct PostgreSQL connection via CloudNative-PG
- **Backend → Cache:** Direct Redis connection
- **Logging:** Structured JSON → Loki (via standard stdout/log shipping)
- **Future decoupling:** NATS pub/sub available but not wired for validation phase

### Key Patterns

- **Modular monolith:** Hermes deliberation path is an internal module within the Bun/Elysia service, with clear interface contracts separating it from legacy pipeline logic
- **Single auth plane:** Session-based authentication with scoped RBAC claims — no dual-auth architectures
- **Environment-aware frontend:** Build-time env var injection drives banner rendering and accent color theming
- **Ownable component library:** shadcn/ui components are copied into the codebase, not installed as runtime dependencies

### Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| Separate microservice for Hermes | Disproportionate orchestration overhead for a validation pipeline; NATS available for future extraction |
| GraphQL | No complex relational query patterns; REST is simpler and better supported by Elysia |
| gRPC | No inter-service high-throughput path; adds protobuf compilation complexity |
| JWT/OAuth2 for new endpoints | Would create dual-auth planes alongside existing sessions; security surface area concern |
| External cloud storage (S3, GCS) | Organizational preference for self-hosted; MinIO is already in-cluster and S3-compatible |
| External logging SaaS (Datadog, Sentry) | Loki is already deployed and operational in-cluster |
| Custom E2E test harness | Increases maintenance burden; Playwright covers both browser and API testing |
| Event-sourced artifact model | Architecturally disproportionate for "add a provenance field to artifacts" |

## 6. Implementation Constraints

### Security Requirements

- **Single auth boundary:** All Hermes endpoints must use existing session-based authentication — no introduction of JWT or OAuth2 for new endpoints
- **Scoped RBAC claims:** Hermes-specific resources must be gated by granular claims (e.g., `hermes:read`, `hermes:trigger`); claim names must be agreed between Task 2 (API) and Task 10 (RBAC hardening)
- **Secret isolation:** Dev, staging, and production namespaces must have independent secrets for DB, Redis, NATS, and MinIO
- **Automated secret rotation:** Required for production (Task 10)
- **Audit logging:** All critical resource access must be logged in production (Task 10)
- **TLS termination:** Required for all production web and API endpoints (Task 9)

### Performance Targets

- No explicit latency or throughput targets in the PRD — this is a validation pipeline, not a high-traffic production service
- Production hardening (Task 9) requires HA (multi-replica) for critical services

### Operational Requirements

- **Infrastructure sequencing:** Task 1 (infra) must complete before any application tasks begin; dependency chain is strictly ordered
- **MinIO isolation:** Task 1 must provision a dedicated MinIO bucket with independent credentials and lifecycle policy — must NOT use `gitlab/gitlab-minio-svc` directly without isolation
- **Structured logging fields:** All Hermes path operations must emit structured JSON logs with at minimum: `rollout_phase`, `migration_step`, `error_code` — queryable via Loki's LogQL
- **Rollback triggers:** Task 6 must implement alerting for failed migrations or critical errors; rollback procedures must be documented in Task 8
- **Environment distinction:** Staging and production builds must be visually and programmatically distinguishable via env var injection

### Service Dependencies and Integration Points

- **CloudNative-PG:** PostgreSQL operator CRs in each namespace
- **Redis operator:** CRs in each namespace
- **NATS operator:** CRs in each namespace (available for future use, not actively wired)
- **MinIO:** Dedicated bucket with S3-compatible API endpoint exposed via ConfigMap
- **Loki:** Existing `openclaw/loki-*` services — no provisioning required
- **ArgoCD:** Present in cluster — should be leveraged for deployment promotion (see gap note below)
- **GitHub Actions:** In-cluster runners via `actions.github.com` CRDs — CI for Playwright E2E tests

### Organizational Preferences

- **Self-hosted services preferred** when available in-cluster
- **Additive changes preferred** over mutative changes to production infrastructure during validation

### Identified Gap: Deployment Promotion Path

The Pessimist identified a critical operational gap: **there is no task explicitly covering the CI/CD pipeline that promotes a validated staging build to production.** Task 8 documents a rollout plan, but no task implements the GitOps promotion via ArgoCD. If ArgoCD isn't configured for this pipeline, Task 9's production deployment risks being a manual `kubectl apply` — the #1 cause of production incidents in Kubernetes environments.

**Recommendation:** Task 1 or Task 8 must include ArgoCD Application CR configuration for the Hermes pipeline, with automated promotion from staging to production gated by E2E test passage (Task 7). This should be addressed before application code is written.

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`:** true
- **`frontendTargets`:** web
- **Mode:** `ingest_plus_stitch` (Stitch generation was attempted but failed)

### Supplied Design Artifacts

No design artifacts or reference URLs were supplied in the design context.

### Stitch Generation Status

- **Status:** Failed
- **Reason:** Not specified (empty `stitch_reason` field)
- **Implication:** No AI-generated design variants are available. Task 4 (frontend) must rely on the resolved design-system decisions (shadcn/ui + Radix + Tailwind) and the environment distinction pattern (banner + accent color) without visual mockups.

### Implications for Web Implementation (Task 4)

1. **Component library:** shadcn/ui components (Radix + Tailwind CSS) as resolved in D9 — copy into codebase, do not install as runtime dependency
2. **Environment distinction:** Persistent banner + accent color theming as resolved in D4 — env var driven
3. **UI patterns needed:** Cards (deliberation results), comparison views (snapshot artifacts), status indicators (pipeline state), artifact viewers (screenshots and variant snapshots)
4. **No visual mockups available:** Implementing agent (blaze) should follow shadcn/ui's default design language and Tailwind's spacing/typography system; design review should occur post-implementation
5. **Accessibility:** Radix primitives ensure baseline accessibility; environment distinction uses dual signals (text + color) for color-blind accessibility

### 7a. Selected Design Direction

No design selections were provided (`design_selections` not present).

### 7b. Design Deliberation Decisions

No design deliberation results were provided (`design_deliberation_result` not present).

## 8. Open Questions

The following items are non-blocking. Implementing agents should use their best judgment, documented in code comments or ADRs:

1. **Artifact retention policy:** How long should Hermes snapshot artifacts be retained in MinIO? No retention period is specified in the PRD. Suggest: 90 days for dev, 1 year for staging/production, configurable via env var.

2. **Feature flag implementation:** Task 4 mentions "feature flags or toggles for Hermes path visibility." No specific feature flag service is identified. Agents should use a simple env-var-based toggle unless a feature flag service is already deployed.

3. **Hermes module interface contracts:** D1 resolved that Hermes is an internal module, but the specific interface boundary (TypeScript interface definitions, event contracts) is left to the implementing agent (nova) on Task 2.

4. **RBAC claim granularity:** D5 resolved session-based auth with RBAC claims but did not specify the full claim taxonomy beyond `hermes:read` and `hermes:trigger`. Task 2 and Task 10 agents should coordinate on the complete set.

5. **Grafana dashboard scope:** D7 resolved Loki for logging with Grafana dashboards, but the specific dashboards (rollout health, error rates, migration progress) are left to Task 6 implementation.

6. **ArgoCD Application configuration:** The deployment promotion gap identified by the Pessimist needs to be resolved — either as part of Task 1 (preferred, since it's infrastructure) or Task 8 (documentation + implementation). The implementing agent should determine ArgoCD's current configuration state and extend it.

7. **Data model (pending D6 resolution):** Until the human resolves the escalated D6 decision, Tasks 3 and 5 should design their artifact read/write interfaces behind an abstraction layer that can accommodate either schema extension or parallel table approach.

