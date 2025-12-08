# Log Collection Architecture

This document describes the log collection pipeline that sends logs from all platform services to Victoria Logs.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Kubernetes Cluster                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │  Controller  │  │    Tools     │  │  OpenMemory  │  │    Agents    │   │
│  │   (cto ns)   │  │   (cto ns)   │  │   (cto ns)   │  │(agent-platform)│ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘   │
│         │                 │                 │                 │           │
│         └─────────────────┼─────────────────┼─────────────────┘           │
│                           │                 │                              │
│                           ▼                 ▼                              │
│                  /var/log/containers/*.log                                 │
│                           │                                                │
│                           ▼                                                │
│                 ┌──────────────────┐                                       │
│                 │    Fluent-Bit    │  (DaemonSet - observability ns)       │
│                 │   Log Collector  │                                       │
│                 └────────┬─────────┘                                       │
│                          │ OTLP/HTTP                                       │
│                          ▼                                                 │
│                 ┌──────────────────┐                                       │
│                 │  OTEL Collector  │  (Deployment - observability ns)      │
│                 │   Processing     │                                       │
│                 └────────┬─────────┘                                       │
│                          │ OTLP/HTTP                                       │
│                          ▼                                                 │
│                 ┌──────────────────┐                                       │
│                 │  Victoria Logs   │  (StatefulSet - observability ns)     │
│                 │     Storage      │                                       │
│                 └────────┬─────────┘                                       │
│                          │                                                 │
│                          ▼                                                 │
│                 ┌──────────────────┐                                       │
│                 │     Grafana      │  (Deployment - observability ns)      │
│                 │  Visualization   │                                       │
│                 └──────────────────┘                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Services with Log Collection Enabled

All services are configured with standard labels and annotations for log collection:

### Labels
- `platform.5dlabs.io/log-collection: enabled` - Indicates the service is part of log collection
- `app.kubernetes.io/component: <component>` - Identifies the component type

### Annotations
- `logs.platform.5dlabs.io/collect: "true"` - Explicitly enables log collection
- `logs.platform.5dlabs.io/service: <service-name>` - Service identifier for queries

## Complete Service Inventory

### CTO Namespace (`cto`)

| Service | Component | Description |
|---------|-----------|-------------|
| agent-controller | controller | CodeRun/DocsRun orchestrator |
| tools-mcp | tools | MCP server proxy |
| openmemory | memory | Agent memory system |

### Observability Namespace (`observability`)

| Service | Component | Description |
|---------|-----------|-------------|
| victoria-logs | logging | Log storage |
| victoria-metrics | metrics | Metrics storage |
| otel-collector | telemetry | Log/metrics processing |
| grafana | visualization | Dashboards and queries |
| kube-state-metrics | metrics-exporter | Kubernetes state metrics |

### Automation Namespace (`automation`)

| Service | Component | Description |
|---------|-----------|-------------|
| argo-workflows | workflow-engine | Workflow orchestration |
| argo-events | event-bus | Event processing |
| github-webhooks | webhooks | GitHub webhook receiver |

### Infrastructure Namespace (`infra`)

| Service | Component | Description |
|---------|-----------|-------------|
| ingress-nginx | ingress | Ingress controller |
| cert-manager | certificates | TLS certificate management |
| arc-controller | runner-controller | GitHub Actions runner controller |
| external-dns | dns | DNS record management |
| cloudnative-pg-operator | postgres-operator | PostgreSQL operator |
| redis-operator | redis-operator | Redis operator |
| vault-secrets-operator | secrets-operator | Vault secret sync |

### Vault Namespace (`vault`)

| Service | Component | Description |
|---------|-----------|-------------|
| vault | secrets | Secret management |

### ArgoCD Namespace (`argocd`)

| Service | Component | Description |
|---------|-----------|-------------|
| argocd | gitops | GitOps controller |

### Cloudflare Operator System Namespace (`cloudflare-operator-system`)

| Service | Component | Description |
|---------|-----------|-------------|
| cloudflare-operator | tunnel-operator | Cloudflare tunnel management |

### Kube-System Namespace (`kube-system`)

| Service | Component | Description |
|---------|-----------|-------------|
| kilo | vpn | WireGuard VPN mesh |
| metrics-server | metrics-server | Resource metrics collection |

### Agent Platform Namespace (`agent-platform`)

| Service | Component | Description |
|---------|-----------|-------------|
| agent-runtime | agent | AI agent execution |

### ARC Runners Namespace (`arc-runners`)

| Service | Component | Description |
|---------|-----------|-------------|
| github-runner | ci-runner | GitHub Actions runners |

### Databases Namespace (`databases`)

| Service | Component | Description |
|---------|-----------|-------------|
| database-instance | database | PostgreSQL/Redis instances |

## Fluent-Bit Service Enrichment

Fluent-Bit enriches logs with the following fields based on pod patterns:

```yaml
# Fields added to all logs
cluster.name: telemetry-dev
platform.name: cto
platform.version: v1

# Service-specific fields (added based on pod name patterns)
service.name: <service-name>
service.component: <component-type>
service.namespace: <kubernetes-namespace>
```

## Victoria Logs Stream Fields

The OTEL Collector configures Victoria Logs with optimized stream fields for efficient indexing:

```
VL-Stream-Fields: cluster.name,platform.name,service.name,service.component,service.namespace,kubernetes.namespace_name,kubernetes.pod_name,kubernetes.container_name
```

## Querying Logs

### By Service
```logsql
service.name: agent-controller
```

### By Namespace
```logsql
kubernetes.namespace_name: cto
```

### By Component Type
```logsql
service.component: controller
```

### By Platform (all CTO services)
```logsql
platform.name: cto
```

### Combined Queries
```logsql
service.name: agent-controller AND _time:[now-1h, now]
```

```logsql
kubernetes.namespace_name: cto AND level: error
```

### Query All Infrastructure Services
```logsql
service.namespace: infra
```

### Query All Operators
```logsql
service.component: *-operator
```

## Adding a New Service

To add log collection for a new service:

### 1. Add Pod Labels to Deployment

For Helm charts (ArgoCD applications):
```yaml
helm:
  values: |
    podLabels:
      app.kubernetes.io/component: <component>
      platform.5dlabs.io/log-collection: enabled
    podAnnotations:
      logs.platform.5dlabs.io/collect: "true"
      logs.platform.5dlabs.io/service: <service-name>
```

For raw Kubernetes manifests:
```yaml
spec:
  template:
    metadata:
      labels:
        app.kubernetes.io/component: <component>
        platform.5dlabs.io/log-collection: enabled
      annotations:
        logs.platform.5dlabs.io/collect: "true"
        logs.platform.5dlabs.io/service: <service-name>
```

### 2. Add Fluent-Bit Filter (Optional)

For enhanced service identification, add a filter in `fluent-bit.yaml`:

```ini
[FILTER]
    Name                modify
    Match               kube.var.log.containers.*<service>*_<namespace>_*
    Add                 service.name        <service-name>
    Add                 service.component   <component>
    Add                 service.namespace   <namespace>
```

### 3. Update This Documentation

Add the new service to the appropriate namespace table above.

## Troubleshooting

### Logs Not Appearing

1. **Check Fluent-Bit is running**:
```bash
kubectl get pods -n observability -l app.kubernetes.io/name=fluent-bit
```

2. **Check OTEL Collector is running**:
```bash
kubectl get pods -n observability -l app.kubernetes.io/name=otel-collector
```

3. **Check Victoria Logs is healthy**:
```bash
kubectl get pods -n observability -l app=server
```

4. **Verify log path exists**:
```bash
kubectl exec -n observability <fluent-bit-pod> -- ls /var/log/containers/
```

5. **Check Fluent-Bit logs for errors**:
```bash
kubectl logs -n observability -l app.kubernetes.io/name=fluent-bit
```

### Service Not Being Enriched

1. Verify pod labels are applied:
```bash
kubectl get pods -n <namespace> -l platform.5dlabs.io/log-collection=enabled
```

2. Check if Fluent-Bit filter pattern matches the pod name
3. Add or adjust the Fluent-Bit filter if needed

### Query Performance

If queries are slow:
1. Use stream fields (`service.name`, `kubernetes.namespace_name`) for filtering
2. Limit time range with `_time` filter
3. Avoid regex on high-cardinality fields

## References

- [Victoria Logs Documentation](https://docs.victoriametrics.com/victorialogs/)
- [LogsQL Query Language](https://docs.victoriametrics.com/victorialogs/logsql/)
- [Fluent-Bit Kubernetes Filter](https://docs.fluentbit.io/manual/pipeline/filters/kubernetes)
- [OpenTelemetry Collector](https://opentelemetry.io/docs/collector/)
