# Telemetry Infrastructure Implementation Plan

## Executive Summary

Your telemetry infrastructure components (Victoria Logs, Victoria Metrics, Fluent-bit, OTEL Collector) are all deployed and running. However, logs are not being stored in Victoria Logs due to a configuration mismatch in the OTEL Collector.

**Root Cause**: The ArgoCD application for OTEL Collector uses a minimal inline configuration that only outputs to a debug exporter, instead of using the comprehensive configuration in `infra/telemetry/values/otel-collector.yaml` which properly forwards logs to Victoria Logs.

## Current State

### ✅ Working Components

1. **Victoria Logs** (victoria-logs-victoria-logs-single-server-0)
   - Status: Running (113 days)
   - Port: 9428
   - Storage: 20Gi with local-path
   - Retention: 6 months

2. **Fluent-bit** (fluent-bit-dwnrb)
   - Status: Running (102 days)
   - Successfully collecting container logs
   - Successfully sending to OTEL Collector (HTTP 200)
   - Enriching logs with Kubernetes metadata

3. **OTEL Collector** (otel-collector-opentelemetry-collector-788799985b-48jfg)
   - Status: Running (87 days)
   - Receiving logs from Fluent-bit ✅
   - NOT forwarding to Victoria Logs ❌

### ❌ Configuration Issues

**OTEL Collector Configuration Problem**:
- Current: Uses minimal inline values (mode + image only)
- Result: Only exports to debug (console output)
- Missing: Victoria Logs exporter configuration
- Missing: Proper resource processors and batching

**Current OTEL Config** (from deployed ConfigMap):
```yaml
service:
  pipelines:
    logs:
      exporters:
      - debug  # ❌ Only debugging, not storing
      processors:
      - memory_limiter
      - batch
      receivers:
      - otlp
```

**Should Be** (from infra/telemetry/values/otel-collector.yaml):
```yaml
exporters:
  otlphttp:  # ✅ Proper Victoria Logs exporter
    logs_endpoint: http://victoria-logs-victoria-logs-single-server:9428/insert/opentelemetry/v1/logs
    tls:
      insecure: true
    retry_on_failure:
      enabled: true
    headers:
      VL-Stream-Fields: "cluster.name,deployment.environment,service.name,service.namespace"

service:
  pipelines:
    logs:
      receivers: [otlp]
      processors: [memory_limiter, batch, resource]
      exporters: [otlphttp, debug]  # ✅ Both Victoria Logs and debug
```

## Implementation Tasks

### Task 1: Fix OTEL Collector Configuration ⚡ CRITICAL

**Goal**: Update ArgoCD application to use comprehensive OTEL Collector values

**File**: `infra/gitops/applications/otel-collector.yaml`

**Changes Needed**:
Replace the minimal inline values with the full configuration from `infra/telemetry/values/otel-collector.yaml`. This will:
- Add Victoria Logs exporter (otlphttp)
- Add proper resource processors
- Add metric transformations
- Configure proper batching and memory limits
- Keep debug exporter for troubleshooting

**Testing**:
1. Apply the change (merge to main for ArgoCD to pick up)
2. Verify OTEL Collector restarts with new config
3. Check Victoria Logs receives data: `curl 'http://localhost:9428/select/logsql/query' -d 'query=*' -d 'limit=10'`

### Task 2: Configure Grafana for Log Viewing

**Goal**: Set up Victoria Logs as a data source in Grafana

**Steps**:
1. Add Victoria Logs data source to Grafana
2. Configure LogQL query language support
3. Create basic log exploration dashboard

**Configuration**:
```yaml
datasources:
  - name: Victoria Logs
    type: victorialogs-datasource
    url: http://victoria-logs-victoria-logs-single-server:9428
    access: proxy
    jsonData:
      httpMethod: POST
```

### Task 3: Agent CLI Log Collection Strategy

**Goal**: Ensure agent CLI outputs are captured in the telemetry system

**Current Approach** (Already Working ✅):
- Agent CLIs run in Kubernetes pods
- Pods output to stdout/stderr
- Fluent-bit automatically collects from `/var/log/containers/*.log`
- Kubernetes metadata adds context (pod name, namespace, labels)

**What's Already Captured**:
- Rex/Cleo/Tess agent outputs
- Agent-controller logs
- MCP server logs
- All infrastructure component logs

**No Additional Work Needed**: Your current Fluent-bit configuration already collects all container logs, including agent CLIs. Once Task 1 is complete, all these logs will flow into Victoria Logs.

**For Enhanced Filtering**:
Add labels to agent pods to make log queries easier:
```yaml
metadata:
  labels:
    app.kubernetes.io/component: agent
    agent.cto.io/type: rex|cleo|tess
    agent.cto.io/task-id: "<task-id>"
```

### Task 4: Log Query Examples & Documentation

**Goal**: Document common log queries for debugging

**Example Queries** (LogQL/Victoria Logs):

1. **All logs from a specific agent run**:
```
{kubernetes_pod_name=~"task-.*"} | json | task_id="task-123"
```

2. **Error logs from agents**:
```
{kubernetes_namespace="agent-platform"} | json | level="error"
```

3. **Logs from specific time range**:
```
{kubernetes_pod_name=~"rex-.*"} | json | __timestamp__ >= "2025-11-24T00:00:00Z"
```

4. **Filter by agent type**:
```
{app_kubernetes_io_component="agent", agent_cto_io_type="rex"}
```

5. **Search for specific text**:
```
{kubernetes_namespace="agent-platform"} |~ "pull request"
```

### Task 5: Validation & Testing

**Success Criteria**:
1. ✅ Fluent-bit continues collecting logs (already working)
2. ✅ OTEL Collector receives logs from Fluent-bit (already working)
3. ❌ → ✅ OTEL Collector forwards logs to Victoria Logs (FIX NEEDED)
4. ❌ → ✅ Victoria Logs stores and indexes logs (after fix)
5. ✅ Grafana can query logs from Victoria Logs (after Task 2)
6. ✅ Historical logs are available for debugging

**Testing Steps**:
```bash
# 1. Check Fluent-bit is sending logs
kubectl logs -n telemetry fluent-bit-<pod> --tail=20

# 2. Check OTEL Collector is receiving
kubectl logs -n telemetry otel-collector-<pod> --tail=50

# 3. Query Victoria Logs directly
kubectl port-forward -n telemetry svc/victoria-logs-victoria-logs-single-server 9428:9428
curl 'http://localhost:9428/select/logsql/query' -d 'query=*' -d 'limit=10'

# 4. Check specific agent logs
curl 'http://localhost:9428/select/logsql/query' -d 'query={kubernetes_namespace="agent-platform"}' -d 'limit=50'

# 5. Verify metadata enrichment
curl 'http://localhost:9428/select/logsql/query' -d 'query={kubernetes_pod_name=~"rex-.*"}' -d 'limit=10'
```

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                         │
│                                                               │
│  ┌─────────────┐                                             │
│  │ Agent Pods  │                                             │
│  │ (Rex/Cleo/  │──stdout/stderr──┐                           │
│  │  Tess CLIs) │                 │                           │
│  └─────────────┘                 │                           │
│                                   │                           │
│  ┌──────────────┐                │                           │
│  │ Controller   │──stdout/stderr─┤                           │
│  │ MCP Servers  │                │                           │
│  └──────────────┘                │                           │
│                                   ↓                           │
│                    ┌──────────────────────────┐              │
│                    │ /var/log/containers/*.log│              │
│                    └──────────────┬───────────┘              │
│                                   │                           │
│                    ┌──────────────▼──────────┐               │
│                    │   Fluent-bit DaemonSet  │               │
│                    │  - Tail container logs  │               │
│                    │  - Parse CRI format     │               │
│                    │  - Add K8s metadata     │               │
│                    │  - Filter & enrich      │               │
│                    └──────────────┬──────────┘               │
│                                   │                           │
│                            OTLP/HTTP (port 4318)              │
│                                   │                           │
│                    ┌──────────────▼──────────┐               │
│                    │   OTEL Collector        │               │
│                    │  - Receive OTLP logs    │               │
│                    │  - Batch processing     │               │
│                    │  - Memory limiting      │               │
│                    │  - Resource attributes  │               │
│                    │  - Retry logic          │               │
│                    └──────────────┬──────────┘               │
│                                   │                           │
│                            OTLP/HTTP (port 9428)              │
│                                   │                           │
│                    ┌──────────────▼──────────┐               │
│                    │   Victoria Logs         │               │
│                    │  - Store logs           │               │
│                    │  - Index by streams     │               │
│                    │  - 6 month retention    │               │
│                    │  - 20Gi storage         │               │
│                    └──────────────┬──────────┘               │
│                                   │                           │
│                         Query API (port 9428)                 │
│                                   │                           │
│                    ┌──────────────▼──────────┐               │
│                    │      Grafana            │               │
│                    │  - Log exploration      │               │
│                    │  - Dashboards           │               │
│                    │  - Alerts               │               │
│                    └─────────────────────────┘               │
└──────────────────────────────────────────────────────────────┘
```

## Benefits After Implementation

1. **Centralized Logging**: All agent CLI outputs, controller logs, and infrastructure logs in one place
2. **Historical Analysis**: 6 months of log retention for debugging past issues
3. **Rich Context**: Kubernetes metadata (pod, namespace, labels) automatically added
4. **Efficient Querying**: LogQL queries to filter by task ID, agent type, time range, etc.
5. **Performance**: Victoria Logs is optimized for high-volume log ingestion and storage
6. **Integration**: Works seamlessly with your existing Grafana dashboards

## Next Steps

1. **Immediate**: Fix OTEL Collector configuration (Task 1)
2. **Short-term**: Set up Grafana data source and basic dashboard (Task 2)
3. **Medium-term**: Add enhanced labels to agent pods for better filtering (Task 3)
4. **Ongoing**: Document common queries and debugging workflows (Task 4)

## Files to Modify

1. `infra/gitops/applications/otel-collector.yaml` - Update to use full values
2. `infra/telemetry/values/grafana.yaml` - Add Victoria Logs data source (optional, can be added via UI)
3. `infra/charts/controller/agent-templates/*/deployment.yaml` - Add agent labels for filtering (optional enhancement)

## No Breaking Changes

This implementation:
- ✅ Doesn't change existing metrics collection
- ✅ Doesn't require changes to agent code
- ✅ Doesn't impact current workflows
- ✅ Is backward compatible
- ✅ Only adds new capabilities

The fix is primarily a configuration update to make the already-deployed infrastructure work as intended.





