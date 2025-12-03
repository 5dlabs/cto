# Loki Log Query Guide

## Overview

This guide provides comprehensive documentation for querying logs in Grafana Loki using LogQL syntax. Loki stores all container logs from your Kubernetes cluster, including agent CLIs (Rex, Cleo, Tess), controllers, and infrastructure components.

## Architecture

```
Container Logs (stdout/stderr)
         ↓
   Fluent-bit (collect + enrich)
         ↓
   OTEL Collector (batch + forward)
         ↓
   Loki (store + index)
         ↓
   Grafana (query + visualize)
```

## Accessing Logs

### Via Grafana (Recommended)

1. Open Grafana: `http://grafana.local` (or port-forward)
2. Navigate to "Explore" → Select "Loki" data source
3. Enter your LogQL query
4. Select time range and click "Run query"

### Via API (For Debugging)

```bash
# Port-forward Loki
kubectl port-forward -n observability svc/loki-gateway 3100:80

# Query logs
curl 'http://localhost:3100/loki/api/v1/query_range' \
  -G \
  --data-urlencode 'query={namespace="cto"}' \
  --data-urlencode 'limit=50'
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
{namespace="cto"}

# Multiple labels (AND)
{namespace="cto", app="agent-controller"}

# Regex match
{pod=~"rex-.*"}

# Negative match
{namespace!="kube-system"}

# Regex negative match
{pod!~"fluent-bit-.*"}
```

### Text Filters (Log Line Search)

After selecting streams, filter by log content:

```logql
# Contains text
{namespace="cto"} |= "error"

# Case-insensitive contains
{namespace="cto"} |~ "(?i)error"

# Does not contain
{namespace="cto"} != "debug"

# Regex match
{namespace="cto"} |~ "error.*failed"
```

## Common Label Fields

### Kubernetes Labels (Added by Fluent-bit)

- `namespace` - Namespace name
- `pod` - Full pod name
- `container` - Container name within pod
- `node` - Node name
- `stream` - stdout or stderr

### Application Labels from Pod Metadata

- `app` - Application name
- `component` - Component type (e.g., "agent")
- `instance` - Instance identifier
- `agent_type` - Agent type (rex, cleo, tess) - if added
- `task_id` - Task ID - if added

### Resource Labels (Added by OTEL Collector)

- `cluster_name` - Cluster identifier (e.g., "telemetry-dev")
- `deployment_environment` - Environment (e.g., "development")
- `service_name` - Service name
- `service_namespace` - Service namespace

## Example Queries

### Agent Logs

#### All Rex Agent Logs
```logql
{namespace="cto", pod=~"rex-.*"}
```

#### Cleo Code Review Logs
```logql
{namespace="cto", pod=~"cleo-.*"}
```

#### Tess QA Logs
```logql
{namespace="cto", pod=~"tess-.*"}
```

#### All Agent Logs (Rex + Cleo + Tess)
```logql
{namespace="cto", pod=~"(rex|cleo|tess)-.*"}
```

#### Logs for Specific Task
```logql
{namespace="cto", pod=~"task-.*-task-123-.*"}
```

#### Agent Error Logs Only
```logql
{namespace="cto", pod=~"(rex|cleo|tess)-.*"} |~ "(?i)(error|failed|failure)"
```

### Controller Logs

#### All Controller Logs
```logql
{namespace="cto", pod=~"agent-controller-.*"}
```

#### Controller Errors
```logql
{namespace="cto", pod=~"agent-controller-.*"} |= "ERROR"
```

#### Controller Task Processing
```logql
{namespace="cto", pod=~"agent-controller-.*"} |= "Processing task"
```

### Workflow Logs

#### All Argo Workflows
```logql
{namespace="argo"}
```

#### Specific Workflow Execution
```logql
{namespace="argo", pod=~"task-.*-workflow-.*"}
```

#### Workflow Errors
```logql
{namespace="argo"} |~ "(?i)(error|failed)"
```

### Infrastructure Logs

#### Fluent-bit Logs (Collection Issues)
```logql
{namespace="observability", app="fluent-bit"}
```

#### OTEL Collector Logs (Processing Issues)
```logql
{namespace="observability", pod=~"otel-collector-.*"}
```

#### ArgoCD Sync Logs
```logql
{namespace="argocd"} |~ "(?i)(sync|health|status)"
```

### Advanced Queries

#### Find Pull Request Creation
```logql
{namespace="cto"} |~ "(?i)(pull request|PR)" |= "created"
```

#### Find GitHub API Errors
```logql
{namespace="cto"} |= "github" |~ "(?i)(error|failed|4[0-9]{2}|5[0-9]{2})"
```

#### Find Long-Running Operations
```logql
{namespace="cto"} |= "duration" |~ "[0-9]+m"
```

#### Agent Startup/Shutdown
```logql
{namespace="cto"} |~ "(?i)(starting|stopping|shutdown|terminated)"
```

## JSON Log Parsing

If your logs are in JSON format, you can parse and filter by fields:

```logql
# Parse JSON and filter by level
{namespace="cto"} | json | level="error"

# Parse JSON and filter by custom field
{namespace="cto"} | json | status_code >= 400

# Parse JSON and extract specific fields
{namespace="cto"} | json | line_format "{{.timestamp}} [{{.level}}] {{.message}}"
```

## Aggregation & Statistics

### Count Logs
```logql
count_over_time({namespace="cto"}[5m])
```

### Rate of Logs
```logql
rate({namespace="cto"}[5m])
```

### Bytes per Second
```logql
bytes_over_time({namespace="cto"}[5m])
```

### Count by Label
```logql
sum(count_over_time({namespace="cto"}[1h])) by (pod)
```

## Performance Tips

1. **Use Specific Label Filters**: Start with namespace or pod name filters
   - ✅ `{namespace="cto"}`
   - ❌ `{} |= "error"`

2. **Limit Time Range**: Query shorter time periods for faster results
   - ✅ Last 1 hour
   - ❌ Last 7 days (unless necessary)

3. **Use Regex Carefully**: Complex regex can be slow
   - ✅ `pod=~"rex-.*"`
   - ❌ `|~ "(?i)(error|warn|info|debug).*with.*complex.*pattern"`

4. **Limit Result Count**: Use `limit` parameter
   - Default: 100 results
   - Max recommended: 5,000 results

5. **Use Aggregations for Large Datasets**: Instead of returning all logs, aggregate them
   - ✅ `count_over_time()` to count errors
   - ❌ Return all logs and count manually

## Common Debugging Workflows

### Debug Failed Task

1. Find the task's pod:
```logql
{namespace="cto", pod=~"task-.*-task-123-.*"}
```

2. Look for errors:
```logql
{namespace="cto", pod=~"task-.*-task-123-.*"} |~ "(?i)error"
```

3. Check controller processing:
```logql
{namespace="cto", pod=~"agent-controller-.*"} |= "task-123"
```

### Debug Slow Performance

1. Find long-running operations:
```logql
{namespace="cto"} |= "duration" |~ "[0-9]+m"
```

2. Check for retries:
```logql
{namespace="cto"} |~ "(?i)retr(y|ying|ied)"
```

3. Look for timeouts:
```logql
{namespace="cto"} |~ "(?i)timeout"
```

### Debug GitHub Integration

1. Find GitHub API calls:
```logql
{namespace="cto"} |= "github.com/api"
```

2. Check for rate limits:
```logql
{namespace="cto"} |= "rate limit"
```

3. Find authentication errors:
```logql
{namespace="cto"} |= "github" |~ "(?i)(unauthorized|forbidden|401|403)"
```

### Debug ArgoCD Sync Issues

1. Find sync operations:
```logql
{namespace="argocd"} |~ "(?i)sync"
```

2. Check health status:
```logql
{namespace="argocd"} |= "health" |~ "(?i)(degraded|progressing|unknown)"
```

3. Find deployment errors:
```logql
{namespace="argocd"} |~ "(?i)(error|failed)" |~ "(?i)(deploy|apply|create)"
```

## Creating Alerts

You can create alerts in Grafana based on log patterns:

1. Go to "Alerting" → "Alert rules"
2. Create new rule with LogQL query
3. Set threshold (e.g., error count > 10 in 5 minutes)
4. Configure notification channels

### Example Alert Query
```logql
sum(rate({namespace="cto"} |~ "(?i)error"[5m])) > 0.1
```

This triggers if error rate exceeds 0.1 per second (6 per minute).

## Retention & Storage

- **Retention Period**: 7 days (configured in Loki)
- **Storage**: 20Gi (local-path storage class)
- **Compression**: Enabled by default
- **Index Fields**: Labels are indexed for fast filtering

## Troubleshooting

### No Logs Appearing

1. Check Fluent-bit is running:
```bash
kubectl get pods -n observability -l app.kubernetes.io/name=fluent-bit
```

2. Check Fluent-bit is collecting:
```bash
kubectl logs -n observability <fluent-bit-pod> --tail=50
```

3. Check OTEL Collector is receiving:
```bash
kubectl logs -n observability <otel-collector-pod> --tail=50
```

4. Check Loki is healthy:
```bash
kubectl get pods -n observability -l app.kubernetes.io/name=loki
curl http://localhost:3100/ready
```

### Logs Missing Metadata

If logs don't have Kubernetes labels, check Fluent-bit filter configuration:
```bash
kubectl get configmap -n observability fluent-bit -o yaml
```

Ensure the `kubernetes` filter is enabled and properly configured.

### Slow Queries

1. Add more specific label filters
2. Reduce time range
3. Use aggregations instead of raw logs
4. Check Loki resource usage:
```bash
kubectl top pod -n observability -l app.kubernetes.io/name=loki
```

## Resources

- [Loki Documentation](https://grafana.com/docs/loki/latest/)
- [LogQL Documentation](https://grafana.com/docs/loki/latest/logql/)
- [Grafana Explore Documentation](https://grafana.com/docs/grafana/latest/explore/)

## Next Steps

1. **Create Custom Dashboards**: Build Grafana dashboards for common queries
2. **Set Up Alerts**: Configure alerts for critical error patterns
3. **Add Custom Labels**: Enhance agent pods with additional metadata labels
4. **Integrate with OpenMemory**: Store important log insights in OpenMemory for context
