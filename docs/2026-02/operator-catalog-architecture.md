# Operator Catalog Architecture

**Date:** 2026-02-06
**Status:** Proposed
**Author:** Jonathon Fritz

## Context

The platform needs to support provisioning various infrastructure resources (Postgres, Redis, Kafka, etc.) across customer clusters. The initial assumption was to pre-install and host all operators in the management cluster, but this approach has significant drawbacks:

- **Resource overhead:** Every operator consumes CPU/memory even when unused
- **Version conflicts:** Different operators may have conflicting dependencies or CRD versions
- **Upgrade complexity:** Upgrading 20+ operators simultaneously is a maintenance nightmare
- **Blast radius:** A misbehaving operator affects the entire control plane
- **Unnecessary complexity:** Most users only need 2-3 operators, not the full catalog

## Decision

**On-demand operator installation via a frontend catalog.**

Users browse a catalog of available operators in the platform UI. When they select one, the platform installs it into the target cluster on demand. Operators are only running where they're actually needed.

## Alternatives Considered

### 1. Pre-install All Operators (Rejected)

Install every supported operator in every cluster upfront.

**Why rejected:**
- Massive resource waste - most operators would sit idle
- Exponential maintenance burden as catalog grows
- Version conflicts between operators sharing the same cluster
- Too much "fucking around" to keep everything healthy

### 2. Crossplane as Universal Control Plane (Rejected)

Use Crossplane to abstract all operator resources behind custom Compositions and XRDs.

**What Crossplane is:**
- Open-source CNCF Incubating project (Apache 2.0, not vendor-locked)
- Turns Kubernetes into a universal control plane via Providers (plugins for AWS, GCP, Kubernetes, Helm, etc.)
- You define CompositeResourceDefinitions (XRDs) as your custom API
- Compositions map your API to actual infrastructure resources
- Users create Claims to request resources

**Why rejected:**
- Adds a significant abstraction layer between users and operator CRDs
- Every supported operator requires writing and maintaining Compositions + XRDs
- Debugging goes through multiple reconciliation layers (Crossplane -> Provider -> Operator -> Resource)
- Solves a different problem: Crossplane excels at "give me a database" abstraction for platform teams. Our users are Kubernetes-savvy and want direct access to operator CRDs
- Overkill for what is fundamentally an operator lifecycle management problem
- Compositions are Crossplane-specific - creates a different kind of lock-in (framework lock-in rather than vendor lock-in)

**When Crossplane _would_ make sense:**
- If we wanted to abstract away all infrastructure details from end users
- If users should never see or interact with operator-specific CRDs
- If we needed a single unified API across cloud-managed and self-hosted resources (e.g., "give me Postgres" maps to RDS in AWS, CloudNativePG on bare metal)

### 3. Custom Bridge Operator (Rejected for now)

Write a custom operator that watches CRDs in the management cluster and provisions resources in remote clusters using stored kubeconfigs.

**Why rejected:**
- High development effort for each supported resource type
- We'd be reimplementing what existing operators already do well
- Maintaining parity with upstream operator features is unsustainable

## Architecture: Operator Catalog

### Core Concept

```
┌─────────────────────────────────────────────────┐
│                  Platform UI                     │
│                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ Postgres │ │  Redis   │ │  Kafka   │  ...    │
│  │ CloudNPG │ │ Dragonfly│ │ Strimzi  │        │
│  │          │ │          │ │          │        │
│  │[Install] │ │[Install] │ │[Install] │        │
│  └──────────┘ └──────────┘ └──────────┘        │
└───────────────────┬─────────────────────────────┘
                    │ User clicks "Install"
                    ▼
┌─────────────────────────────────────────────────┐
│              Platform Controller                 │
│                                                  │
│  1. Validates the install request                │
│  2. Resolves the Helm chart + version            │
│  3. Installs operator via Helm into target       │
│  4. Reports status back to UI                    │
│                                                  │
└───────────────────┬─────────────────────────────┘
                    │ Helm install / ArgoCD App
                    ▼
┌─────────────────────────────────────────────────┐
│              Target Cluster                      │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  CloudNativePG Operator (just installed) │    │
│  │                                          │    │
│  │  Watches for: Cluster, Backup, etc.     │    │
│  │  User creates CRDs directly             │    │
│  └─────────────────────────────────────────┘    │
│                                                  │
└─────────────────────────────────────────────────┘
```

### Catalog Entry Schema

Each operator in the catalog is defined as a structured entry:

```yaml
# Example catalog entry
- name: cloudnative-pg
  displayName: CloudNativePG
  description: "PostgreSQL operator for Kubernetes"
  category: databases
  icon: postgres.svg
  source:
    type: helm
    repo: https://cloudnative-pg.github.io/charts
    chart: cloudnative-pg
    defaultVersion: "0.23.0"
    supportedVersions:
      - "0.23.0"
      - "0.22.1"
  namespace: cnpg-system
  crds:
    - Cluster
    - Backup
    - ScheduledBackup
    - Pooler
  documentation: https://cloudnative-pg.io/documentation/
  maturity: stable  # stable | beta | alpha
  tags:
    - database
    - postgresql
    - ha
```

### Installation Flow

1. **User browses catalog** - UI renders available operators with descriptions, categories, maturity levels
2. **User clicks Install** - Selects version, confirms target cluster/namespace
3. **Platform controller receives request** - Validates permissions, checks for conflicts
4. **Operator installed** - Via Helm (direct) or by creating an ArgoCD Application CR (GitOps)
5. **Status reported** - UI shows installed operators, health, version
6. **User interacts with operator CRDs directly** - No abstraction layer, full operator API available

### Implementation Options for the Install Mechanism

| Approach | Pros | Cons |
|---|---|---|
| **Direct Helm install** | Simple, immediate | Need Helm credentials/access to target cluster |
| **ArgoCD Application CR** | GitOps-native, self-healing, already in use | Slightly more indirection |
| **Flux HelmRelease CR** | Similar to ArgoCD, declarative | Another tool if not already using Flux |

**Recommended: ArgoCD Application CR** - Since the platform already uses ArgoCD for GitOps, creating an ArgoCD Application for each installed operator keeps everything in the existing deployment model. ArgoCD handles health checking, drift detection, and upgrades.

### Lifecycle Operations

| Operation | How |
|---|---|
| **Install** | Create ArgoCD Application CR (or Helm install) for the operator |
| **Upgrade** | Update the target revision on the ArgoCD Application |
| **Uninstall** | Delete ArgoCD Application (with cascade to clean up CRDs/resources) |
| **Health check** | ArgoCD sync status + operator pod health |
| **List installed** | Query ArgoCD Applications with a label selector (e.g., `platform.io/type=operator`) |

### Catalog Management

The catalog itself is a data file (YAML/JSON) that can be:
- Versioned in Git alongside the platform
- Periodically updated with new operators / versions
- Extended by users (custom catalog entries for internal operators)

### Future Considerations

- **Dependency resolution:** Some operators depend on cert-manager, OPA, etc. The catalog should express and auto-resolve dependencies
- **Operator compatibility matrix:** Track which operator versions work with which Kubernetes versions
- **Resource quotas:** Estimate and enforce resource consumption per operator
- **Multi-tenancy:** Namespace isolation for operators in shared clusters
- **Marketplace:** Allow third parties to contribute catalog entries
- **Upgrade policies:** Auto-upgrade patch versions, require approval for minor/major

## Key Principles

1. **No unnecessary abstraction** - Users get direct access to operator CRDs. The platform manages lifecycle, not API translation
2. **Install only what you use** - Zero resource overhead for unused capabilities
3. **Leverage existing tooling** - ArgoCD for deployment, Helm for packaging. No new control plane
4. **Catalog, not monolith** - Operators are independent entries. Adding a new one is adding a YAML entry, not writing code
5. **No vendor lock-in** - Everything is standard Kubernetes, Helm, and ArgoCD. Swappable at every layer
