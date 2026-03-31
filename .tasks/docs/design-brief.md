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

### Task Inventory

The initial decomposition identified **1 task** in this phase, focused on foundational infrastructure:

| ID | Title | Agent | Stack | Priority |
|----|-------|-------|-------|----------|
| 1 | Provision Staging/Production Infra and Artifact Storage | Bolt | Kubernetes/Helm | High |

### Task 1 Summary

Set up distinct staging and production namespaces, storage for site screenshots and variant snapshots, and a ConfigMap for endpoint aggregation to support the Hermes E2E pipeline and web experience. Steps include:

- Create two Kubernetes namespaces (`sigma1-staging`, `sigma1-production`)
- Deploy required operators (CloudNative-PG for Postgres, Redis, NATS) in both namespaces
- Provision storage for current-site screenshots and generated variant snapshots
- Create a ConfigMap aggregating connection strings and storage endpoints
- Set up Kubernetes Secrets for sensitive credentials
- Document all endpoints and storage locations for downstream agents
- Label and annotate all resources for environment distinction

### Key Services and Components Discovered

- **Kubernetes namespaces** — environment isolation primitive
- **MinIO** — existing in-cluster S3-compatible object store (`gitlab/gitlab-minio-svc`)
- **CloudNative-PG (Postgres)**, **Redis**, **NATS** — backing services for the pipeline
- **Cilium** — in-cluster CNI with NetworkPolicy support (CRDs confirmed: `ciliumnetworkpolicies.cilium.io`)
- **External Secrets Operator** — CRDs present in-cluster (`externalsecrets.external-secrets.io`, `clustersecretstores.external-secrets.io`)
- **Helm** — deployment and templating mechanism

### Cross-Cutting Concerns

- Clear staging/production separation is a PRD-level requirement, not optional
- Artifact preservation (screenshots, variant snapshots) must be durable and HTTP-addressable for the web experience
- Rollout risks and migration concerns must be explicitly called out in implementation
- Self-hosted services are preferred when available in-cluster (organizational preference)

### Decision Points Identified

Three decision points were raised during decomposition (dp-1, dp-2, dp-3) and carried into deliberation.

## 3. Resolved Decisions

### [D1] Should artifact storage for screenshots and variant snapshots use the existing in-cluster MinIO service or per-environment PVCs?

**Status:** Accepted

**Task Context:** Task 1 — Provision Staging/Production Infra and Artifact Storage

**Context:** Both Optimist and Pessimist agreed that the existing MinIO service (`gitlab/gitlab-minio-svc`) is the correct storage backend for binary blob artifacts. MinIO is already deployed in-cluster, S3-compatible, and provides HTTP-addressable presigned URLs needed by the web experience. PVCs were rejected as a regression due to node-affinity constraints, lack of HTTP addressability, and manual capacity management overhead.

**Decision:** Use the existing MinIO service (`gitlab/gitlab-minio-svc`) with dedicated buckets per environment (e.g., `hermes-staging-artifacts`, `hermes-prod-artifacts`), leveraging S3-compatible APIs for artifact access.

**Consensus:** 2/2 (100%) — unanimous agreement

**Consequences:**
- **Positive:** Reuses existing infrastructure (self-hosted preference satisfied); provides presigned URLs for the web frontend; supports versioning and lifecycle policies natively; decouples storage from Pod scheduling; no new infrastructure to provision
- **Negative:** Shares I/O and storage capacity with GitLab's MinIO instance, creating potential blast radius to GitLab CI (LFS, artifacts, registry) if Hermes writes burst large artifact volumes
- **Caveats (from Pessimist, accepted by both):** Task 1 **must include**:
  1. Verifying MinIO's backing PV capacity and IOPS headroom
  2. Creating dedicated buckets with lifecycle policies (auto-expire artifacts older than N days)
  3. Setting bucket quotas to cap Hermes storage consumption
  4. If backing PV is undersized, provisioning a **second MinIO instance** rather than risking GitLab — this is the explicit fallback path

### [D2] Should staging and production be separated via distinct Kubernetes namespaces or via labeling within a single namespace?

**Status:** Accepted

**Task Context:** Task 1 — Provision Staging/Production Infra and Artifact Storage

**Context:** Both debaters agreed immediately and without contention. The PRD mandates a "clear staging/production distinction." Namespace separation is the Kubernetes consensus best practice, enables Cilium NetworkPolicy enforcement at the namespace level, provides RBAC as a true security boundary (labels are not), and supports independent ResourceQuotas to prevent cross-environment resource contention.

**Decision:** Provision separate namespaces (`hermes-staging`, `hermes-production`) with per-namespace RBAC RoleBindings, ResourceQuotas, LimitRanges, and Cilium NetworkPolicies for hard isolation.

**Consensus:** 2/2 (100%) — unanimous agreement

**Consequences:**
- **Positive:** Hard network isolation via Cilium NetworkPolicies; RBAC scoped per namespace limits blast radius; independent ResourceQuotas prevent staging load tests from starving production; aligns with research memo consensus and PRD requirements
- **Negative:** Two namespaces mean duplicated ConfigMaps/Secrets and slightly more Helm templating
- **Caveats:** Duplication overhead is trivially handled by Helm's `values-staging.yaml` / `values-production.yaml` pattern — one-time setup cost

> **Note on namespace naming:** The initial task decomposition used `sigma1-staging` / `sigma1-production`. The deliberation converged on `hermes-staging` / `hermes-production`. Implementing agents should use `hermes-staging` / `hermes-production` as the canonical namespace names, reflecting the Hermes pipeline identity.

### [D3] Should endpoint aggregation configuration use Kubernetes ConfigMaps or the External Secrets Operator?

**Status:** Accepted

**Task Context:** Task 1 — Provision Staging/Production Infra and Artifact Storage

**Context:** This was the only contested decision point. The Optimist argued for using the External Secrets Operator (ESO), citing its in-cluster CRD presence, drift reconciliation, and alignment with credential rotation best practices. The Pessimist pushed back forcefully, arguing that (a) endpoint aggregation is configuration, not secrets; (b) ESO CRDs being present does not confirm a functioning `ClusterSecretStore` backend; (c) ESO adds an external dependency to the pod startup path, widening blast radius; and (d) the PRD does not require credential rotation. The Pessimist's position was that ConfigMaps are the correct abstraction for non-sensitive routing data, with native Kubernetes Secrets for actual credentials, and ESO migration deferred until the secret store backend is verified.

**Decision:** Use native Kubernetes ConfigMaps for endpoint aggregation data (URLs, paths, non-sensitive routing configuration) and native Kubernetes Secrets for any credentials, managed via Helm values per environment. Migrate to ESO only after confirming a healthy `ClusterSecretStore` backend exists.

**Consensus:** The Pessimist's position prevails on evidence strength. The Optimist's own hedge — "If it's truly unconfigured, I'd concede that ConfigMaps are the pragmatic starting point" — combined with the inability to confirm a functioning store backend, makes ConfigMaps the only defensible choice for Task 1. The Optimist acknowledged this contingency.

**Consequences:**
- **Positive:** Zero external dependencies on pod startup path; ConfigMaps are native, debuggable with `kubectl`, stored in etcd; failure mode is narrow ("wrong YAML applied," fixable in 30 seconds); no bootstrapping cost for ESO store backend
- **Negative:** No drift reconciliation; manual credential rotation; no automatic sync from external secret stores
- **Caveats:** If future tasks confirm a healthy `ClusterSecretStore` backend and credential rotation becomes a requirement, a follow-up task should migrate Secret resources to ESO. This is an explicit deferred item, not a dropped concern. ConfigMap data must never contain secrets — the boundary between ConfigMap (routing/config) and Secret (credentials) must be enforced in Helm templates.

## 4. Escalated Decisions

No decision points were escalated. All three decision points reached resolution during deliberation.

## 5. Architecture Overview

### Agreed Approach

The Hermes E2E pipeline infrastructure follows a **namespace-isolated, self-hosted-first** architecture on the existing Kubernetes cluster.

#### Technology Stack
- **Orchestration:** Kubernetes (existing cluster) with Helm for templating and deployment
- **Networking/Policy:** Cilium CNI with CiliumNetworkPolicies for namespace-level isolation
- **Artifact Storage:** MinIO (existing, `gitlab/gitlab-minio-svc`) with S3-compatible API, dedicated buckets per environment
- **Configuration:** Native Kubernetes ConfigMaps (endpoint aggregation) + native Kubernetes Secrets (credentials), managed via per-environment Helm values files
- **Backing Services:** CloudNative-PG (Postgres), Redis, NATS — deployed per namespace
- **RBAC:** Namespace-scoped RoleBindings (not ClusterRoleBindings)
- **Resource Governance:** Per-namespace ResourceQuotas and LimitRanges

#### Service Architecture
```
┌─────────────────────────────────────────────────────┐
│                  Kubernetes Cluster                   │
│                                                       │
│  ┌──────────────────┐    ┌──────────────────┐        │
│  │  hermes-staging   │    │ hermes-production │        │
│  │                    │    │                    │        │
│  │  ConfigMap (eps)   │    │  ConfigMap (eps)   │        │
│  │  Secrets (creds)   │    │  Secrets (creds)   │        │
│  │  Postgres (CNPG)   │    │  Postgres (CNPG)   │        │
│  │  Redis              │    │  Redis              │        │
│  │  NATS               │    │  NATS               │        │
│  │  ResourceQuota      │    │  ResourceQuota      │        │
│  │  LimitRange         │    │  LimitRange         │        │
│  │  CiliumNetPolicy    │    │  CiliumNetPolicy    │        │
│  │  RoleBinding        │    │  RoleBinding        │        │
│  └──────────────────┘    └──────────────────┘        │
│                                                       │
│  ┌──────────────────┐                                 │
│  │  gitlab namespace  │                                 │
│  │  gitlab-minio-svc  │◄── Shared, with dedicated     │
│  │   ├─ hermes-staging-artifacts (bucket)              │
│  │   └─ hermes-prod-artifacts    (bucket)              │
│  └──────────────────┘                                 │
└─────────────────────────────────────────────────────┘
```

#### Key Patterns
- **Hard namespace isolation:** Staging and production workloads never share a namespace; network policies enforce zero cross-namespace traffic by default
- **Self-hosted storage:** MinIO reuse over new infrastructure; PVCs explicitly ruled out for artifact storage
- **Config/Secret separation:** Non-sensitive endpoint data in ConfigMaps; credentials in Secrets; boundary enforced in Helm templates
- **Capacity gating:** MinIO capacity and IOPS verified before pipeline goes live; fallback to dedicated MinIO instance if undersized

#### Explicitly Ruled Out
- **PVCs for artifact storage** — node-affinity coupling, no HTTP addressability, manual capacity management; unanimously rejected
- **Single-namespace with label separation** — labels are not a security boundary; cannot enforce network isolation as cleanly; unanimously rejected
- **External Secrets Operator for Task 1** — unverified backend dependency; adds external failure mode to pod startup; deferred until `ClusterSecretStore` health is confirmed

## 6. Implementation Constraints

All implementing agents **must** respect the following:

### Security Requirements
- RBAC must be scoped to namespaces (`RoleBinding`), never cluster-wide (`ClusterRoleBinding`) unless explicitly justified
- CiliumNetworkPolicies must enforce default-deny ingress between `hermes-staging` and `hermes-production` namespaces
- Credentials must never appear in ConfigMaps — they belong in Kubernetes Secrets, mounted as environment variables
- MinIO bucket access should use per-environment service accounts or access keys, not shared credentials

### Performance / Resource Governance
- Each namespace must have a `ResourceQuota` and `LimitRange` to prevent resource contention
- MinIO capacity and IOPS must be verified before the pipeline writes artifacts; if GitLab's MinIO is undersized, a second MinIO instance must be provisioned (not PVCs)
- Bucket lifecycle policies must auto-expire artifacts older than a configurable retention period
- Bucket quotas must cap Hermes artifact storage consumption

### Operational Requirements
- Helm deployment must use per-environment values files (`values-staging.yaml`, `values-production.yaml`)
- All resources must be labeled with environment, project, and component metadata for observability and future scaling
- Endpoint aggregation ConfigMap must be named consistently and documented for downstream agents
- PodDisruptionBudgets should be considered for production namespace workloads

### Service Dependencies and Integration Points
- **MinIO** (`gitlab/gitlab-minio-svc`): Shared dependency — treat as external service; verify health and capacity before integration
- **Cilium**: In-cluster CNI — CiliumNetworkPolicy resources are the enforcement mechanism for namespace isolation
- **External Secrets Operator**: Present but unverified — do not depend on for Task 1; document as future migration target

### Organizational Preferences
- Prefer self-hosted services when available in-cluster (confirmed: MinIO, ESO CRDs, Cilium)
- Prefer boring, proven Kubernetes patterns over novel abstractions
- Infrastructure decisions should minimize external dependency chains

## 7. Design Intake Summary

### Frontend Context
- **`hasFrontend`:** true
- **`frontendTargets`:** web
- **Mode:** `ingest_plus_stitch`
- **Stitch generation status:** Failed (no Stitch candidates available)

### Supplied Design Artifacts
No design artifacts or reference URLs were supplied in the intake.

### Implications for Implementation
- The PRD requires "a web experience with a clear staging/production distinction" — this means a web frontend must be built or integrated as part of the broader project scope (beyond Task 1)
- Since Stitch generation failed and no design artifacts were supplied, the web experience design is currently **unspecified**
- Implementing agents working on frontend tasks (when scoped) will need to either:
  1. Receive design direction in a subsequent design intake cycle, or
  2. Use best judgment to create a minimal, functional web interface for pipeline status, artifact browsing (screenshots/snapshots), and environment switching (staging ↔ production)
- The MinIO S3-compatible API with presigned URLs (from D1) directly supports serving screenshot and variant snapshot artifacts to the web frontend without a custom file-serving layer

### 7a. Selected Design Direction

No design selections were provided. This section will be populated if/when a design review cycle produces variant selections.

### 7b. Design Deliberation Decisions

No design deliberation was conducted. This section will be populated if/when the Designer persona evaluates visual-identity, design-system, component-library, layout-pattern, or ux-behavior decision points for the web experience.

## 8. Open Questions

The following items are **non-blocking** — implementing agents should use their best judgment:

1. **MinIO capacity headroom:** The exact backing PV size and IOPS profile of `gitlab/gitlab-minio-svc` is unknown. Task 1 should investigate and document findings. If undersized, provision a dedicated MinIO instance for Hermes. This is a gated implementation detail, not an architectural question.

2. **Artifact retention policy duration:** The lifecycle policy's retention period (N days) for auto-expiring artifacts was not specified in the PRD. Implementing agents should default to a reasonable value (e.g., 30 days for staging, 90 days for production) and make it configurable.

3. **Namespace naming:** The deliberation converged on `hermes-staging` / `hermes-production`, diverging from the initial parse's `sigma1-staging` / `sigma1-production`. Implementing agents should use `hermes-staging` / `hermes-production` unless project leadership specifies otherwise.

4. **ESO backend verification:** A future task should verify whether a `ClusterSecretStore` is configured and healthy. If confirmed, credential management should migrate from native Secrets to ExternalSecret resources. This is an explicit deferred item.

5. **Web experience design:** No design artifacts, Stitch candidates, or design deliberation outputs are available. Frontend implementation tasks will need design direction from a subsequent intake cycle or must proceed with a minimal functional interface.

6. **Backing service versions:** Specific versions for CloudNative-PG, Redis, and NATS were not discussed in deliberation. Implementing agents should use the latest stable versions available in the cluster's Helm chart repositories unless version constraints are discovered during provisioning.

7. **MinIO bucket naming convention:** The deliberation used `hermes-staging-artifacts` / `hermes-prod-artifacts` as examples. Implementing agents should confirm bucket naming conventions align with any existing MinIO bucket policies in the `gitlab` namespace.

