# Victoria Logs Query Guide

## Overview

This guide provides comprehensive documentation for querying logs in Victoria Logs using LogQL syntax. Victoria Logs stores all container logs from your Kubernetes cluster, including agent CLIs (Rex, Cleo, Tess), controllers, and infrastructure components.

## Architecture

```
Container Logs (stdout/stderr)
         ↓
   Fluent-bit (collect + enrich)
         ↓
   OTEL Collector (batch + forward)
         ↓
   Victoria Logs (store + index)
         ↓
   Grafana (query + visualize)
```

## Accessing Logs

### Via Grafana (Recommended)

1. Open Grafana: `http://grafana.local` (or port-forward)
2. Navigate to "Explore" → Select "VictoriaLogs" data source
3. Enter your LogQL query
4. Select time range and click "Run query"

### Via API (For Debugging)

```bash
# Port-forward Victoria Logs
kubectl port-forward -n telemetry svc/victoria-logs-victoria-logs-single-server 9428:9428

# Query logs
curl 'http://localhost:9428/select/logsql/query' \
  -d 'query={kubernetes_namespace="cto"}' \
  -d 'limit=50'
```

## LogQL Query Syntax

### Basic Structure

```
{label_filter} | text_filter | additional_filters
```

### Label Filters (Stream Selectors)

Label filters select log streams based on metadata added by Kubernetes and Fluent-bit:

```logql
# Single label match
{kubernetes_namespace="cto"}

# Multiple labels (AND)
{kubernetes_namespace="cto", app_kubernetes_io_name="agent-controller"}

# Regex match
{kubernetes_pod_name=~"rex-.*"}

# Negative match
{kubernetes_namespace!="kube-system"}

# Regex negative match
{kubernetes_pod_name!~"fluent-bit-.*"}
```

### Text Filters (Log Line Search)

After selecting streams, filter by log content:

```logql
# Contains text
{kubernetes_namespace="cto"} |~ "error"

# Case-insensitive contains
{kubernetes_namespace="cto"} |~ "(?i)error"

# Does not contain
{kubernetes_namespace="cto"} !~ "debug"

# Exact match
{kubernetes_namespace="cto"} | log == "Application started"
```

## Common Label Fields

### Kubernetes Labels (Added by Fluent-bit)

- `kubernetes_namespace` - Namespace name
- `kubernetes_pod_name` - Full pod name
- `kubernetes_container_name` - Container name within pod
- `kubernetes_namespace_name` - Same as kubernetes_namespace
- `kubernetes_pod_id` - Pod UID
- `kubernetes_host` - Node name

### Kubernetes Labels from Pod Metadata

- `app_kubernetes_io_name` - Application name
- `app_kubernetes_io_component` - Component type (e.g., "agent")
- `app_kubernetes_io_instance` - Instance identifier
- `agent_cto_io_type` - Agent type (rex, cleo, tess) - if added
- `agent_cto_io_task_id` - Task ID - if added

### Resource Labels (Added by OTEL Collector)

- `cluster_name` - Cluster identifier (e.g., "telemetry-dev")
- `deployment_environment` - Environment (e.g., "development")
- `service_name` - Service name
- `service_namespace` - Service namespace

## Example Queries

### Agent Logs

#### All Rex Agent Logs
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"rex-.*"}
```

#### Cleo Code Review Logs
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"cleo-.*"}
```

#### Tess QA Logs
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"tess-.*"}
```

#### All Agent Logs (Rex + Cleo + Tess)
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"(rex|cleo|tess)-.*"}
```

#### Logs for Specific Task
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"task-.*-task-123-.*"}
```

#### Agent Error Logs Only
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"(rex|cleo|tess)-.*"} |~ "(?i)(error|failed|failure)"
```

### Controller Logs

#### All Controller Logs
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"agent-controller-.*"}
```

#### Controller Errors
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"agent-controller-.*"} |~ "ERROR"
```

#### Controller Task Processing
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"agent-controller-.*"} |~ "Processing task"
```

### Workflow Logs

#### All Argo Workflows
```logql
{kubernetes_namespace="argo"}
```

#### Specific Workflow Execution
```logql
{kubernetes_namespace="argo", kubernetes_pod_name=~"task-.*-workflow-.*"}
```

#### Workflow Errors
```logql
{kubernetes_namespace="argo"} |~ "(?i)(error|failed)"
```

### Infrastructure Logs

#### Fluent-bit Logs (Collection Issues)
```logql
{kubernetes_namespace="telemetry", app_kubernetes_io_name="fluent-bit"}
```

#### OTEL Collector Logs (Processing Issues)
```logql
{kubernetes_namespace="telemetry", kubernetes_pod_name=~"otel-collector-.*"}
```

#### ArgoCD Sync Logs
```logql
{kubernetes_namespace="argocd"} |~ "(?i)(sync|health|status)"
```

### Time-Based Queries

#### Last Hour
```logql
{kubernetes_namespace="cto"} | __timestamp__ >= now() - 1h
```

#### Specific Time Range
```logql
{kubernetes_namespace="cto"} | __timestamp__ >= "2025-11-24T00:00:00Z" and __timestamp__ < "2025-11-24T23:59:59Z"
```

#### Today's Logs
```logql
{kubernetes_namespace="cto"} | __timestamp__ >= today()
```

### Advanced Queries

#### Find Pull Request Creation
```logql
{kubernetes_namespace="cto"} |~ "(?i)(pull request|PR)" |~ "created"
```

#### Find GitHub API Errors
```logql
{kubernetes_namespace="cto"} |~ "github" |~ "(?i)(error|failed|4[0-9]{2}|5[0-9]{2})"
```

#### Find Long-Running Operations
```logql
{kubernetes_namespace="cto"} |~ "duration" |~ "[0-9]+m"
```

#### Agent Startup/Shutdown
```logql
{kubernetes_namespace="cto"} |~ "(?i)(starting|stopping|shutdown|terminated)"
```

## JSON Log Parsing

If your logs are in JSON format, you can parse and filter by fields:

```logql
# Parse JSON and filter by level
{kubernetes_namespace="cto"} | json | level="error"

# Parse JSON and filter by custom field
{kubernetes_namespace="cto"} | json | status_code >= 400

# Parse JSON and extract specific fields
{kubernetes_namespace="cto"} | json | line_format "{{.timestamp}} [{{.level}}] {{.message}}"
```

## Aggregation & Statistics

### Count Logs
```logql
count_over_time({kubernetes_namespace="cto"}[5m])
```

### Rate of Logs
```logql
rate({kubernetes_namespace="cto"}[5m])
```

### Bytes per Second
```logql
bytes_over_time({kubernetes_namespace="cto"}[5m])
```

### Count by Label
```logql
sum(count_over_time({kubernetes_namespace="cto"}[1h])) by (kubernetes_pod_name)
```

## Performance Tips

1. **Use Specific Label Filters**: Start with namespace or pod name filters
   - ✅ `{kubernetes_namespace="cto"}`
   - ❌ `{} |~ "error"`

2. **Limit Time Range**: Query shorter time periods for faster results
   - ✅ Last 1 hour
   - ❌ Last 7 days (unless necessary)

3. **Use Regex Carefully**: Complex regex can be slow
   - ✅ `kubernetes_pod_name=~"rex-.*"`
   - ❌ `|~ "(?i)(error|warn|info|debug).*with.*complex.*pattern"`

4. **Limit Result Count**: Use `limit` parameter
   - Default: 100 results
   - Max recommended: 10,000 results

5. **Use Aggregations for Large Datasets**: Instead of returning all logs, aggregate them
   - ✅ `count_over_time()` to count errors
   - ❌ Return all logs and count manually

## Common Debugging Workflows

### Debug Failed Task

1. Find the task's pod:
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"task-.*-task-123-.*"}
```

2. Look for errors:
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"task-.*-task-123-.*"} |~ "(?i)error"
```

3. Check controller processing:
```logql
{kubernetes_namespace="cto", kubernetes_pod_name=~"agent-controller-.*"} |~ "task-123"
```

### Debug Slow Performance

1. Find long-running operations:
```logql
{kubernetes_namespace="cto"} |~ "duration.*[0-9]+m"
```

2. Check for retries:
```logql
{kubernetes_namespace="cto"} |~ "(?i)retr(y|ying|ied)"
```

3. Look for timeouts:
```logql
{kubernetes_namespace="cto"} |~ "(?i)timeout"
```

### Debug GitHub Integration

1. Find GitHub API calls:
```logql
{kubernetes_namespace="cto"} |~ "github.com/api"
```

2. Check for rate limits:
```logql
{kubernetes_namespace="cto"} |~ "rate limit"
```

3. Find authentication errors:
```logql
{kubernetes_namespace="cto"} |~ "github" |~ "(?i)(unauthorized|forbidden|401|403)"
```

### Debug ArgoCD Sync Issues

1. Find sync operations:
```logql
{kubernetes_namespace="argocd"} |~ "(?i)sync"
```

2. Check health status:
```logql
{kubernetes_namespace="argocd"} |~ "(?i)health" |~ "(?i)(degraded|progressing|unknown)"
```

3. Find deployment errors:
```logql
{kubernetes_namespace="argocd"} |~ "(?i)(error|failed)" |~ "(?i)(deploy|apply|create)"
```

## Creating Alerts

You can create alerts in Grafana based on log patterns:

1. Go to "Alerting" → "Alert rules"
2. Create new rule with LogQL query
3. Set threshold (e.g., error count > 10 in 5 minutes)
4. Configure notification channels

### Example Alert Query
```logql
sum(rate({kubernetes_namespace="cto"} |~ "(?i)error"[5m])) > 0.1
```

This triggers if error rate exceeds 0.1 per second (6 per minute).

## Retention & Storage

- **Retention Period**: 6 months (configured in Victoria Logs)
- **Storage**: 20Gi (local-path storage class)
- **Compression**: Enabled by default
- **Index Fields**: All labels are indexed for fast filtering

## Troubleshooting

### No Logs Appearing

1. Check Fluent-bit is running:
```bash
kubectl get pods -n telemetry -l app.kubernetes.io/name=fluent-bit
```

2. Check Fluent-bit is collecting:
```bash
kubectl logs -n telemetry <fluent-bit-pod> --tail=50
```

3. Check OTEL Collector is receiving:
```bash
kubectl logs -n telemetry <otel-collector-pod> --tail=50
```

4. Check Victoria Logs is healthy:
```bash
kubectl get pods -n telemetry -l app.kubernetes.io/name=victoria-logs
curl http://localhost:9428/health
```

### Logs Missing Metadata

If logs don't have Kubernetes labels, check Fluent-bit filter configuration:
```bash
kubectl get configmap -n telemetry fluent-bit -o yaml
```

Ensure the `kubernetes` filter is enabled and properly configured.

### Slow Queries

1. Add more specific label filters
2. Reduce time range
3. Use aggregations instead of raw logs
4. Check Victoria Logs resource usage:
```bash
kubectl top pod -n telemetry victoria-logs-victoria-logs-single-server-0
```

## Resources

- [Victoria Logs Documentation](https://docs.victoriametrics.com/VictoriaLogs/)
- [LogQL Documentation](https://grafana.com/docs/loki/latest/logql/)
- [Grafana Explore Documentation](https://grafana.com/docs/grafana/latest/explore/)

## Next Steps

1. **Create Custom Dashboards**: Build Grafana dashboards for common queries
2. **Set Up Alerts**: Configure alerts for critical error patterns
3. **Add Custom Labels**: Enhance agent pods with additional metadata labels
4. **Integrate with OpenMemory**: Store important log insights in OpenMemory for context





