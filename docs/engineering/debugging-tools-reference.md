# Debugging Tools Reference

This document provides a reference for MCP tools available for debugging the CTO platform controller and workflows.

## Quick Reference

| Category | Tools | Purpose |
|----------|-------|---------|
| Logs | `grafana_query_loki_logs`, `victoriametrics_query` | Query application logs and metrics |
| ArgoCD | `argocd_get_application`, `argocd_list_applications` | Check deployment status and sync state |
| Kubernetes | `kubernetes_pods_log`, `kubernetes_events_list` | Debug pod issues and cluster events |
| GitHub | `github_get_pull_request`, `github_list_issues` | Track PR status and issues |

---

## Grafana/Loki Tools

### grafana_list_datasources

Lists available datasources. Use this first to get datasource UIDs.

```json
// No parameters required
```

**Example Response:**
- `VictoriaLogs` (UID: `PD775F2863313E6C7`) - Log storage
- `VictoriaMetrics` (UID: `P4169E866C3094E38`) - Metrics storage

### grafana_query_loki_logs

Query logs from VictoriaLogs/Loki datasource.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `datasourceUid` | string | Yes | Datasource UID (use `PD775F2863313E6C7` for VictoriaLogs) |
| `logql` | string | Yes | LogQL query expression |
| `limit` | integer | No | Max log lines (default: 10, max: 100) |
| `startRfc3339` | string | No | Start time in RFC3339 format |
| `endRfc3339` | string | No | End time in RFC3339 format |

**Common Queries:**

```logql
# Controller logs
{service.name="agent-controller"}

# Error logs only
{service.name="agent-controller"} |= "error"

# Specific workflow logs
{service.name="agent-controller"} |= "workflow" |= "play-"

# Filter by log level
{service.name="agent-controller"} | json | level="ERROR"
```

### grafana_list_loki_label_names

Get available label names for filtering logs.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `datasourceUid` | string | Yes | Datasource UID |

### grafana_list_loki_label_values

Get values for a specific label.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `datasourceUid` | string | Yes | Datasource UID |
| `labelName` | string | Yes | Label to get values for (e.g., `service.name`) |

### grafana_find_error_pattern_logs

Automatically find elevated error patterns compared to baseline.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Investigation name |
| `labels` | object | Yes | Labels to scope analysis |

### grafana_search_dashboards

Search for Grafana dashboards by name.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | No | Search query |

---

## VictoriaMetrics Tools

### victoriametrics_query

Execute instant PromQL/MetricsQL query.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | PromQL/MetricsQL expression |
| `time` | string | No | Evaluation timestamp |

**Common Queries:**

```promql
# Check if controller is up
up{job="agent-controller"}

# Memory usage
container_memory_usage_bytes{namespace="cto", pod=~"cto-controller.*"}

# CPU usage
rate(container_cpu_usage_seconds_total{namespace="cto"}[5m])

# Pod restart count
kube_pod_container_status_restarts_total{namespace="cto"}
```

### victoriametrics_query_range

Execute range query over time period.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | PromQL/MetricsQL expression |
| `start` | string | Yes | Start timestamp |
| `end` | string | No | End timestamp (defaults to now) |
| `step` | string | No | Query resolution (e.g., `1m`, `5m`) |

### victoriametrics_labels

List all available metric label names.

### victoriametrics_label_values

Get values for a specific label.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `label_name` | string | Yes | Label name to query |

### victoriametrics_alerts

List firing and pending alerts.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `state` | string | No | Filter: `firing`, `pending`, or `all` |

---

## ArgoCD Tools

### argocd_list_applications

List all ArgoCD applications with sync status.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | integer | No | Max applications to return |
| `search` | string | No | Filter by application name |

### argocd_get_application

Get detailed application status including sync state and health.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `applicationName` | string | Yes | Application name |
| `applicationNamespace` | string | No | ArgoCD namespace (default: `argocd`) |

**Key Status Fields:**
- `status.sync.status`: `Synced`, `OutOfSync`, `Unknown`
- `status.health.status`: `Healthy`, `Degraded`, `Progressing`, `Missing`
- `status.operationState.phase`: `Succeeded`, `Failed`, `Running`

### argocd_get_application_events

Get Kubernetes events for an application.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `applicationName` | string | Yes | Application name |

### argocd_get_application_resource_tree

Get the resource tree showing all managed resources.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `applicationName` | string | Yes | Application name |

### argocd_get_application_workload_logs

Get logs from application workloads (Deployments, Pods, etc.)

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `applicationName` | string | Yes | Application name |
| `applicationNamespace` | string | Yes | ArgoCD namespace |
| `resourceRef` | object | Yes | Resource reference with `uid`, `kind`, `namespace`, `name`, `version`, `group` |
| `container` | string | Yes | Container name |

### argocd_sync_application

Trigger a sync operation for an application.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `applicationName` | string | Yes | Application name |
| `prune` | boolean | No | Remove resources not in source |
| `dryRun` | boolean | No | Preview without applying |

---

## Kubernetes Tools

### kubernetes_namespaces_list

List all namespaces in the cluster.

### kubernetes_pods_list_in_namespace

List pods in a specific namespace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `namespace` | string | Yes | Namespace name |
| `labelSelector` | string | No | Filter by labels (e.g., `app=controller`) |

### kubernetes_pods_get

Get detailed pod information.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Pod name |
| `namespace` | string | No | Namespace |

### kubernetes_pods_log

Get pod logs.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Pod name |
| `namespace` | string | No | Namespace |
| `container` | string | No | Container name |
| `tail` | integer | No | Lines from end (default: 100) |
| `previous` | boolean | No | Get previous container logs |

### kubernetes_pods_exec

Execute command in a pod.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Pod name |
| `namespace` | string | No | Namespace |
| `command` | array | Yes | Command and args (e.g., `["ls", "-la"]`) |
| `container` | string | No | Container name |

### kubernetes_pods_top

Get pod resource usage (CPU/memory).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `namespace` | string | No | Namespace |
| `all_namespaces` | boolean | No | Query all namespaces |

### kubernetes_events_list

List Kubernetes events (useful for debugging).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `namespace` | string | No | Filter by namespace |

### kubernetes_resources_get

Get any Kubernetes resource by type.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `apiVersion` | string | Yes | API version (e.g., `v1`, `apps/v1`) |
| `kind` | string | Yes | Resource kind (e.g., `Pod`, `Deployment`) |
| `name` | string | Yes | Resource name |
| `namespace` | string | No | Namespace |

### kubernetes_resources_list

List resources of a specific type.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `apiVersion` | string | Yes | API version |
| `kind` | string | Yes | Resource kind |
| `namespace` | string | No | Namespace |
| `labelSelector` | string | No | Label filter |

---

## GitHub Tools

### github_list_pull_requests

List pull requests in a repository.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | Repository owner |
| `repo` | string | Yes | Repository name |
| `state` | string | No | `open`, `closed`, or `all` |
| `per_page` | number | No | Results per page (max 100) |

### github_get_pull_request

Get detailed PR information.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | Repository owner |
| `repo` | string | Yes | Repository name |
| `pull_number` | number | Yes | PR number |

### github_get_pull_request_status

Get combined status of all PR checks.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | Repository owner |
| `repo` | string | Yes | Repository name |
| `pull_number` | number | Yes | PR number |

### github_get_pull_request_files

Get list of files changed in a PR.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | Repository owner |
| `repo` | string | Yes | Repository name |
| `pull_number` | number | Yes | PR number |

### github_list_issues

List repository issues.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | Repository owner |
| `repo` | string | Yes | Repository name |
| `state` | string | No | `open`, `closed`, or `all` |
| `labels` | array | No | Filter by labels |

### github_get_issue

Get detailed issue information.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner` | string | Yes | Repository owner |
| `repo` | string | Yes | Repository name |
| `issue_number` | number | Yes | Issue number |

### github_search_code

Search for code across repositories.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | Search query |

---

## Common Debugging Workflows

### 1. Check Controller Health

```
1. argocd_get_application(applicationName: "cto-controller")
   → Check sync.status and health.status

2. kubernetes_pods_list_in_namespace(namespace: "cto")
   → Verify pod is Running with correct READY count

3. kubernetes_pods_log(name: "cto-controller-xxx", namespace: "cto", tail: 50)
   → Check recent logs for errors
```

### 2. Debug Failed Workflow

```
1. kubernetes_events_list(namespace: "cto")
   → Check for pod scheduling or resource issues

2. grafana_query_loki_logs with query:
   {service.name="agent-controller"} |= "workflow-name" |= "error"
   → Find error messages

3. kubernetes_pods_log for workflow pod
   → Get detailed execution logs
```

### 3. Check Deployment After PR Merge

```
1. github_get_pull_request_status(owner: "5dlabs", repo: "cto", pull_number: X)
   → Verify CI passed

2. argocd_get_application(applicationName: "cto-controller")
   → Check if ArgoCD synced the changes

3. kubernetes_pods_list_in_namespace(namespace: "cto")
   → Verify new pod is running
```

### 4. Investigate Resource Issues

```
1. kubernetes_pods_top(namespace: "cto")
   → Check CPU/memory usage

2. victoriametrics_query with:
   container_memory_usage_bytes{namespace="cto"}
   → Get detailed memory metrics

3. kubernetes_events_list(namespace: "cto")
   → Check for OOMKilled or resource pressure events
```

---

## Important Datasource UIDs

| Datasource | UID | Type |
|------------|-----|------|
| VictoriaLogs | `PD775F2863313E6C7` | Logs |
| VictoriaMetrics | `P4169E866C3094E38` | Metrics (Prometheus) |

## Key Namespaces

| Namespace | Purpose |
|-----------|---------|
| `cto` | Controller, tools, workflows |
| `automation` | Argo Workflows, Events |
| `argocd` | ArgoCD applications |
| `observability` | VictoriaMetrics, Grafana |

## Key Service Names (for log queries)

| Service | Description |
|---------|-------------|
| `agent-controller` | CTO controller |
| `tools-mcp` | MCP tools server |
| `argo-workflow-controller` | Argo Workflows |
| `argocd-server` | ArgoCD server |

