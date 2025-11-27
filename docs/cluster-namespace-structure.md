# CTO Platform - Kubernetes Namespace Structure

## Current Namespace Structure

### `cto` - Application Workloads
Core CTO platform applications and agents.

| Service | Purpose | Pods |
|---------|---------|------|
| cto-controller | Agent controller/orchestrator | 2 |
| cto-tools | MCP tools server (Brave, GitHub, etc.) | 2 |
| cto-tools-k8s-mcp | Kubernetes MCP server | 1 |
| openmemory | Agent memory service | 1 |

### `argo` - Event-Driven Automation
Argo Events and Workflows for GitHub event processing and workflow execution.

| Service | Purpose | Pods |
|---------|---------|------|
| argo-events-controller-manager | Event controller | 1 |
| argo-workflows-server | Workflow UI/API | 1 |
| argo-workflows-workflow-controller | Workflow executor | 1 |
| eventbus-default-stan | NATS event bus | 3 |
| github-eventsource | GitHub webhook receiver | 1 |
| *-sensor (17 sensors) | Event handlers for PR, CI, deployments | 17 |

**Sensors include:**
- `atlas-batch-integration-sensor` - Atlas batch processing
- `atlas-conflict-monitor-sensor` - Merge conflict detection
- `atlas-pr-monitor-sensor` - PR monitoring
- `bolt-production-deployment-sensor` - Production deployments
- `ci-failure-remediation-sensor` - CI failure handling
- `play-workflow-*-sensor` - Play workflow triggers
- `stage-aware-*-sensor` - Stage-aware agent triggers
- `tess-label-fallback-sensor` - Tess labeling

### `argocd` - GitOps Control Plane
ArgoCD for GitOps-based deployments.

| Service | Purpose | Pods |
|---------|---------|------|
| argocd-application-controller | App reconciliation | 1 |
| argocd-applicationset-controller | App generation | 1 |
| argocd-dex-server | SSO/Auth | 1 |
| argocd-notifications-controller | Notifications | 1 |
| argocd-redis | Cache | 1 |
| argocd-repo-server | Git operations | 1 |
| argocd-server | UI/API | 1 |

### `operators` - Cluster Operators
Kubernetes operators for managing cluster resources.

| Service | Purpose | Pods |
|---------|---------|------|
| cloudnative-pg-operator | PostgreSQL management | 1 |
| redis-operator | Redis management | 1 |
| ngrok-operator-agent | Ngrok tunnel agent | 1 |
| ngrok-operator-manager | Ngrok management | 2 |
| vault-secrets-operator | Vault secret sync | 1 |

### `telemetry` - Observability Stack
Monitoring, logging, and metrics collection.

| Service | Purpose | Pods |
|---------|---------|------|
| victoria-metrics | Time-series metrics DB | 1 |
| victoria-logs | Log aggregation | 1 |
| grafana | Dashboards/visualization | 1 |
| kube-state-metrics | K8s metrics exporter | 1 |
| otel-collector | OpenTelemetry collector | 1 |
| fluent-bit | Log forwarding | 1 (DaemonSet) |

### `databases` - Database Instances
Application databases managed by operators.

| Service | Purpose | Pods |
|---------|---------|------|
| test-postgres | PostgreSQL instance | 1 |
| test-redis | Redis instance | 1 |

### `vault` - Secrets Management
HashiCorp Vault for secrets storage.

| Service | Purpose | Pods |
|---------|---------|------|
| vault | Secrets vault | 1 |

### `arc-runners` - GitHub Actions Runners
Ephemeral GitHub Actions runner pods.

| Service | Purpose | Pods |
|---------|---------|------|
| k8s-runner-* | Ephemeral runner pods | Variable |
| runner-cache-pruner | Cache cleanup | CronJob |

### `cert-manager` - TLS Certificate Management
Automatic TLS certificate issuance and renewal.

| Service | Purpose | Pods |
|---------|---------|------|
| cert-manager | Certificate controller | 1 |
| cert-manager-cainjector | CA bundle injection | 1 |
| cert-manager-webhook | Admission webhook | 1 |

### `ingress-nginx` - Ingress Controller
NGINX-based ingress for external traffic.

| Service | Purpose | Pods |
|---------|---------|------|
| ingress-nginx-controller | Ingress routing | 1 |

### `external-dns` - DNS Management
Automatic DNS record management via Cloudflare.

| Service | Purpose | Pods |
|---------|---------|------|
| external-dns | DNS sync controller | 1 |

---

## Proposed Consolidation

### Option A: Consolidate Infrastructure

Rename `operators` → `infra` and consolidate small infrastructure namespaces:

| Current Namespace | Proposed | Rationale |
|-------------------|----------|-----------|
| `operators` | `infra` | All cluster infrastructure |
| `cert-manager` | → `infra` | Infrastructure (3 pods) |
| `ingress-nginx` | → `infra` | Infrastructure (1 pod) |
| `external-dns` | → `infra` | Infrastructure (1 pod) |
| `argo` | `automation` | Better describes event-driven automation |

**Result:**
- `infra` - All cluster infrastructure operators/controllers
- `automation` - Event-driven automation (Argo Events/Workflows)
- `argocd` - GitOps control plane
- `cto` - Application workloads
- `telemetry` - Observability
- `databases` - Database instances
- `vault` - Secrets management
- `arc-runners` - CI runner pods

### Option B: Minimal Changes

Keep current structure, just consolidate:
- `external-dns` → `operators`
- Remove empty `arc-systems`

---

## Namespace Purposes Summary

| Namespace | Type | Description |
|-----------|------|-------------|
| `cto` | Application | CTO platform agents and services |
| `argo`/`automation` | Platform | Event-driven workflow automation |
| `argocd` | Platform | GitOps deployment system |
| `operators`/`infra` | Infrastructure | Cluster operators and controllers |
| `telemetry` | Observability | Metrics, logs, monitoring |
| `databases` | Data | Database instances |
| `vault` | Security | Secrets management |
| `arc-runners` | CI/CD | GitHub Actions runner pods |

---

## Questions for Review

1. Should `cert-manager`, `ingress-nginx`, and `external-dns` be consolidated into `infra`/`operators`?
2. Should `argo` be renamed to `automation` to better describe its purpose?
3. Is the separation between `argocd` (GitOps) and `argo` (Workflows) clear enough?
4. Should `vault` remain separate or join `infra`?
5. Is `telemetry` the right name for the observability stack?

