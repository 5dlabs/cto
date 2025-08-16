# Toolman Guide: OpenTelemetry Observability

## Overview
Guide for implementing and operating OpenTelemetry-based observability for TaskMaster agent workflows.

## Key Components

### OTEL Tracer Wrapper
```bash
# Basic usage
./scripts/otel-wrapper.sh --span-name "clippy/verify" \
  --attributes "repo=myorg/repo,pr=123,agent=clippy" \
  -- actual-command args

# Environment variables
export OTEL_EXPORTER_OTLP_ENDPOINT="http://otel-collector:4317"
export OTEL_SERVICE_NAME="agent-steps"
export OTEL_RESOURCE_ATTRIBUTES="k8s.namespace=workflows"
```

### Metrics Collection
```bash
# Check metrics availability
curl http://prometheus:9090/api/v1/label/__name__/values | \
  grep agent_step

# Query duration metrics
curl -G http://prometheus:9090/api/v1/query \
  --data-urlencode 'query=agent_step_duration_ms{agent="clippy"}'

# Success rate calculation
curl -G http://prometheus:9090/api/v1/query \
  --data-urlencode 'query=rate(agent_step_success_total[5m])'
```

## Template Integration

### Workflow Template Updates
```yaml
# Add OTEL environment to templates
spec:
  templates:
  - name: agent-step
    container:
      env:
      - name: OTEL_EXPORTER_OTLP_ENDPOINT
        value: "http://otel-collector:4317"
      - name: OTEL_SERVICE_NAME
        value: "agent-steps"
      - name: OTEL_RESOURCE_ATTRIBUTES
        value: "service.name=agent-steps,k8s.namespace={{workflow.namespace}}"
      command: ["/usr/local/bin/otel-wrapper.sh"]
      args: ["--span-name", "{{inputs.parameters.agent}}/{{inputs.parameters.step}}", 
             "--attributes", "repo={{workflow.parameters.repo}},pr={{workflow.parameters.prNumber}}", 
             "--", "python", "/app/agent.py"]
```

### Pod Labeling
```yaml
# Ensure proper labels for log correlation
metadata:
  labels:
    workflows.argoproj.io/workflow: "{{workflow.name}}"
    repo: "{{workflow.parameters.repo}}"
    pr: "{{workflow.parameters.prNumber}}"
    taskId: "{{workflow.parameters.taskId}}"
    agent: "{{inputs.parameters.agent}}"
```

## Monitoring Tools

### Grafana Dashboard Queries

#### Step Duration by Agent
```promql
histogram_quantile(0.95, 
  rate(agent_step_duration_ms_bucket{agent=~"$agent"}[5m])
)
```

#### Success Rate by Repository
```promql
rate(agent_step_success_total{repo=~"$repo"}[5m]) /
(rate(agent_step_success_total{repo=~"$repo"}[5m]) + 
 rate(agent_step_fail_total{repo=~"$repo"}[5m]))
```

#### Active Tasks Gauge
```promql
count by (repo, pr) (
  increase(agent_step_success_total[1h]) > 0
)
```

### Log Correlation Queries

#### Loki Query by Trace ID
```logql
{namespace="workflows"} |= "traceId" | json | traceId="abc123def456"
```

#### Filter by Repository and PR
```logql
{repo="myorg/repo", pr="123"} | json | level="error"
```

## Troubleshooting

### Common Issues

#### Missing Spans
```bash
# Check collector connectivity
kubectl exec -it deployment/workflows-agent -- \
  nc -zv otel-collector 4317

# Verify OTEL environment
kubectl exec -it deployment/workflows-agent -- env | grep OTEL

# Check wrapper execution
kubectl logs deployment/workflows-agent -c init-tracer
```

#### High Cardinality Metrics
```bash
# Check metric cardinality
curl -s http://prometheus:9090/api/v1/label/__name__/values | \
  jq -r '.data[]' | grep agent_step | wc -l

# Monitor series count
curl -s http://prometheus:9090/api/v1/query \
  --data-urlencode 'query=prometheus_tsdb_symbol_table_size_bytes'
```

#### Log Correlation Failures
```bash
# Verify trace ID injection
kubectl exec -it pod/workflow-pod -- \
  grep -o 'traceId":"[^"]*' /var/log/application.log

# Check log enrichment
kubectl logs pod/workflow-pod | jq -r '.traceId // "missing"'
```

### Debug Commands

#### Validate Trace Propagation
```bash
# Start manual span
./scripts/otel-wrapper.sh --span-name "debug/test" \
  --attributes "repo=debug,pr=999" -- sleep 5

# Check trace in Jaeger/Tempo
curl "http://jaeger:16686/api/traces?service=agent-steps&limit=1"
```

#### Test Metrics Emission
```bash
# Force metric collection
./scripts/otel-wrapper.sh --span-name "test/metric" \
  --emit-metrics -- echo "testing"

# Verify in Prometheus  
curl -G http://prometheus:9090/api/v1/query \
  --data-urlencode 'query=agent_step_duration_ms{stepName="metric"}'
```

#### Exemplar Validation
```bash
# Query with exemplars
curl -G http://prometheus:9090/api/v1/query \
  --data-urlencode 'query=agent_step_duration_ms_bucket[5m]' \
  | jq '.data[].values[].exemplar'
```

## Performance Optimization

### Sampling Configuration
```yaml
# Environment tuning
OTEL_TRACES_SAMPLER: "parentbased_traceidratio"
OTEL_TRACES_SAMPLER_ARG: "0.1"  # 10% sampling for high-volume
OTEL_METRICS_EXEMPLAR_FILTER: "trace_based"
```

### Resource Limits
```yaml
# Container resource limits
resources:
  limits:
    cpu: 100m
    memory: 128Mi
  requests:
    cpu: 50m
    memory: 64Mi
```

### Batch Configuration
```yaml
# OTEL SDK batching
OTEL_BSP_MAX_QUEUE_SIZE: "2048"
OTEL_BSP_MAX_EXPORT_BATCH_SIZE: "512"
OTEL_BSP_EXPORT_TIMEOUT: "30000"
```

## Best Practices

### Security
- Never log trace context in plaintext
- Use service mesh for secure collector communication
- Implement proper RBAC for metrics access
- Rotate collector certificates regularly

### Performance  
- Set appropriate sampling rates per environment
- Use resource limits to prevent memory leaks
- Implement circuit breakers for collector failures
- Monitor collector resource usage

### Reliability
- Configure graceful degradation when collector unavailable
- Implement retry logic with exponential backoff
- Use health checks for all observability components
- Maintain runbooks for common failure scenarios