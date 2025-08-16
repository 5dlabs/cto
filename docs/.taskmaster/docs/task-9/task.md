# Task 9: OpenTelemetry Observability Implementation

## Overview
This task implements comprehensive observability for the Task Master system using OpenTelemetry traces, metrics, and structured logging. The system provides end-to-end visibility into agent execution, workflow performance, and system health with correlation across all components.

## Architecture
- **Trace Collection**: OpenTelemetry spans for each agent step and workflow stage
- **Metrics Emission**: Performance and business metrics via OTLP or Prometheus
- **Structured Logging**: Correlated logs with trace IDs and workflow context
- **Dashboards**: Grafana dashboards with exemplar linking to traces
- **Alerting**: Proactive alerting based on performance and error rate metrics

## Key Features

### Distributed Tracing
- **Agent Step Spans**: Individual spans for each AI agent execution
- **Workflow Correlation**: Parent spans linking entire workflow executions
- **Context Propagation**: Trace context passed between workflow steps
- **Exemplar Linking**: Metrics samples linked to specific traces

### Comprehensive Metrics
- **Agent Performance**: Duration, success rates, error classifications
- **Workflow Metrics**: End-to-end execution times, stage success rates  
- **Resource Utilization**: CPU, memory, storage usage patterns
- **Business Metrics**: Task completion rates, quality gate effectiveness

## Implementation

### Tracer Shim Implementation
```go
package tracer

import (
    "context"
    "os"
    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/trace"
)

type AgentStepTracer struct {
    tracer trace.Tracer
    ctx    context.Context
    span   trace.Span
}

func NewAgentStepTracer(stepName, agentName string) *AgentStepTracer {
    tracer := otel.Tracer("agent-steps")
    ctx := context.Background()
    
    // Extract parent context from environment if available
    if parentCtx := extractTraceContext(); parentCtx != nil {
        ctx = parentCtx
    }
    
    ctx, span := tracer.Start(ctx, fmt.Sprintf("%s/%s", agentName, stepName))
    
    // Add required attributes
    span.SetAttributes(
        attribute.String("repo", os.Getenv("REPO")),
        attribute.String("pr.number", os.Getenv("PR_NUMBER")),
        attribute.String("task.id", os.Getenv("TASK_ID")),
        attribute.String("agent", agentName),
        attribute.String("workflow.name", os.Getenv("WORKFLOW_NAME")),
        attribute.String("node.id", os.Getenv("NODE_ID")),
    )
    
    return &AgentStepTracer{
        tracer: tracer,
        ctx:    ctx,
        span:   span,
    }
}

func (t *AgentStepTracer) Finish(err error, exitCode int) {
    defer t.span.End()
    
    if err != nil || exitCode != 0 {
        t.span.SetStatus(codes.Error, fmt.Sprintf("exit code: %d", exitCode))
        t.span.RecordError(err)
    } else {
        t.span.SetStatus(codes.Ok, "completed successfully")
    }
    
    t.span.SetAttributes(
        attribute.Int("exit.code", exitCode),
    )
    
    // Export trace ID for log correlation
    traceID := t.span.SpanContext().TraceID().String()
    fmt.Printf(`{"trace_id":"%s","span_id":"%s"}`+"\n", 
        traceID, t.span.SpanContext().SpanID().String())
}
```

### Bash Wrapper for Agent Steps
```bash
#!/bin/bash
# tracer-wrapper.sh - wraps agent execution with OpenTelemetry tracing

set -euo pipefail

AGENT_NAME=${AGENT:-"unknown"}
STEP_NAME=${STEP:-"execute"}
REPO=${REPO:-""}
PR_NUMBER=${PR_NUMBER:-""}
TASK_ID=${TASK_ID:-""}

# Start span and capture trace context
TRACE_OUTPUT=$(otel-cli exec \
  --service agent-steps \
  --name "${AGENT_NAME}/${STEP_NAME}" \
  --attrs "repo=${REPO},pr.number=${PR_NUMBER},task.id=${TASK_ID},agent=${AGENT_NAME}" \
  -- echo "span_started")

# Extract trace ID for correlation
TRACE_ID=$(echo "$TRACE_OUTPUT" | jq -r '.trace_id // empty')
SPAN_ID=$(echo "$TRACE_OUTPUT" | jq -r '.span_id // empty')

# Export for downstream processes
export TRACE_ID SPAN_ID

# Print correlation info for log ingestion
echo "{\"trace_id\":\"$TRACE_ID\",\"span_id\":\"$SPAN_ID\",\"level\":\"info\",\"msg\":\"agent step started\"}"

# Execute actual command with error handling
EXIT_CODE=0
"$@" || EXIT_CODE=$?

# Report completion status
if [ $EXIT_CODE -eq 0 ]; then
  echo "{\"trace_id\":\"$TRACE_ID\",\"span_id\":\"$SPAN_ID\",\"level\":\"info\",\"msg\":\"agent step completed\",\"exit_code\":$EXIT_CODE}"
else
  echo "{\"trace_id\":\"$TRACE_ID\",\"span_id\":\"$SPAN_ID\",\"level\":\"error\",\"msg\":\"agent step failed\",\"exit_code\":$EXIT_CODE}"
fi

exit $EXIT_CODE
```

### Workflow Template Integration
```yaml
- name: traced-agent-step
  inputs:
    parameters:
      - {name: agent}
      - {name: step, value: "execute"}
  container:
    image: ghcr.io/myorg/agent-runner:latest
    command: ["/bin/bash", "/scripts/tracer-wrapper.sh"]
    args: ["/scripts/run-agent.sh"]
    env:
      - name: OTEL_EXPORTER_OTLP_ENDPOINT
        value: "http://otel-collector:4317"
      - name: OTEL_EXPORTER_OTLP_PROTOCOL
        value: "grpc"
      - name: OTEL_SERVICE_NAME
        value: "agent-steps"
      - name: OTEL_RESOURCE_ATTRIBUTES
        value: "service.name=agent-steps,deployment.environment={{workflow.parameters.env}},k8s.namespace={{workflow.namespace}}"
      - name: AGENT
        value: "{{inputs.parameters.agent}}"
      - name: STEP
        value: "{{inputs.parameters.step}}"
      - name: REPO
        value: "{{workflow.parameters.repo}}"
      - name: PR_NUMBER
        value: "{{workflow.parameters.pr}}"
      - name: TASK_ID
        value: "{{workflow.parameters.taskId}}"
      - name: WORKFLOW_NAME
        value: "{{workflow.name}}"
      - name: NODE_ID
        value: "{{workflow.status.nodes.self.id}}"
```

### Metrics Implementation
```go
package metrics

import (
    "github.com/prometheus/client_golang/prometheus"
    "time"
)

var (
    AgentStepDuration = prometheus.NewHistogramVec(
        prometheus.HistogramOpts{
            Name:    "agent_step_duration_seconds",
            Help:    "Duration of agent step execution",
            Buckets: prometheus.ExponentialBuckets(0.1, 2, 10),
        },
        []string{"repo", "pr_number", "task_id", "agent", "workflow_name", "step_name", "outcome"},
    )
    
    AgentStepSuccess = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "agent_step_success_total",
            Help: "Total successful agent step executions",
        },
        []string{"agent", "repo", "pr_number", "step_name"},
    )
    
    AgentStepFail = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "agent_step_fail_total", 
            Help: "Total failed agent step executions",
        },
        []string{"agent", "repo", "pr_number", "step_name", "error_type"},
    )
)

func RecordAgentStep(repo, prNumber, taskId, agent, workflow, step string, 
                     duration time.Duration, success bool, errorType string) {
    
    outcome := "success"
    if !success {
        outcome = "failure"
    }
    
    AgentStepDuration.WithLabelValues(
        repo, prNumber, taskId, agent, workflow, step, outcome,
    ).Observe(duration.Seconds())
    
    if success {
        AgentStepSuccess.WithLabelValues(agent, repo, prNumber, step).Inc()
    } else {
        AgentStepFail.WithLabelValues(agent, repo, prNumber, step, errorType).Inc()
    }
}
```

### OpenTelemetry Collector Configuration
```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024
  resource:
    attributes:
      - key: deployment.environment
        value: ${ENV}
        action: upsert
      - key: k8s.cluster.name
        value: ${CLUSTER_NAME}
        action: upsert

exporters:
  jaeger:
    endpoint: jaeger-collector:14250
    tls:
      insecure: true
  prometheus:
    endpoint: "0.0.0.0:8889"
    const_labels:
      cluster: ${CLUSTER_NAME}

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch, resource]
      exporters: [jaeger]
    metrics:
      receivers: [otlp]
      processors: [batch, resource] 
      exporters: [prometheus]
```

### Pod Labeling and Annotation
```yaml
metadata:
  labels:
    workflows.argoproj.io/workflow: "{{workflow.name}}"
    repo: "{{workflow.parameters.repo}}"
    pr: "{{workflow.parameters.pr}}"
    task-id: "{{workflow.parameters.taskId}}"
    agent: "{{inputs.parameters.agent}}"
  annotations:
    trace.context: "" # Populated by tracer wrapper
    commit.sha: "{{workflow.parameters.commitSha}}"
```

### Log Correlation Configuration
```yaml
# fluent-bit.conf
[INPUT]
    Name tail
    Path /var/log/containers/*workflow*.log
    Parser docker
    Tag kube.*
    
[FILTER]
    Name kubernetes
    Match kube.*
    
[FILTER]
    Name modify
    Match kube.*
    Add service agent-steps
    
[FILTER]
    Name parser
    Match kube.*
    Key_Name log
    Parser json
    Reserve_Data On
    
[OUTPUT]
    Name loki
    Match kube.*
    Host loki
    Port 3100
    Labels job=agent-steps,namespace=$kubernetes['namespace_name'],pod=$kubernetes['pod_name']
```

### Workflow Output Parameters
```yaml
outputs:
  parameters:
    - name: pr_html_url
      value: "{{workflow.parameters.event | jq '.pull_request.html_url // empty'}}"
    - name: actions_run_url
      value: "{{workflow.parameters.event | jq '.workflow_run.html_url // empty'}}"
    - name: preview_url
      valueFrom:
        path: /tmp/preview_url
    - name: trace_id
      valueFrom:
        path: /tmp/trace_id
```

## Grafana Dashboard Configuration

### Agent Performance Dashboard
```json
{
  "dashboard": {
    "title": "Agent Step Performance",
    "panels": [
      {
        "title": "Agent Step Duration",
        "type": "heatmap",
        "targets": [
          {
            "expr": "rate(agent_step_duration_seconds_bucket[5m])",
            "legendFormat": "{{agent}} - {{step_name}}"
          }
        ],
        "exemplars": {
          "datasource": "Tempo",
          "expr": "agent_step_duration_seconds",
          "traceIdLabelName": "trace_id"
        }
      },
      {
        "title": "Success Rate by Agent",
        "type": "stat", 
        "targets": [
          {
            "expr": "rate(agent_step_success_total[5m]) / (rate(agent_step_success_total[5m]) + rate(agent_step_fail_total[5m]))",
            "legendFormat": "{{agent}}"
          }
        ]
      },
      {
        "title": "Active PRs and Tasks",
        "type": "gauge",
        "targets": [
          {
            "expr": "count by (repo) (increase(agent_step_success_total[1h]))"
          }
        ]
      }
    ]
  }
}
```

## Alerting Rules
```yaml
# agent-alerting.yaml
groups:
  - name: agent-performance
    rules:
      - alert: AgentStepFailureRate
        expr: |
          (
            rate(agent_step_fail_total[5m]) / 
            (rate(agent_step_success_total[5m]) + rate(agent_step_fail_total[5m]))
          ) > 0.1
        for: 2m
        labels:
          severity: warning
          component: agent-steps
        annotations:
          summary: "High failure rate for agent {{ $labels.agent }}"
          description: "Agent {{ $labels.agent }} has {{ $value | humanizePercentage }} failure rate"
          
      - alert: AgentStepHighDuration
        expr: |
          histogram_quantile(0.95, rate(agent_step_duration_seconds_bucket[5m])) > 600
        for: 5m
        labels:
          severity: warning
          component: agent-steps
        annotations:
          summary: "Slow agent execution for {{ $labels.agent }}"
          description: "95th percentile duration is {{ $value }}s for agent {{ $labels.agent }}"
```

## Testing and Validation

### Trace Validation
```bash
#!/bin/bash
# test-tracing.sh

# Submit test workflow
WORKFLOW=$(argo submit --from workflowtemplate/coderun-template \
  -p github-app=rex -p repo=test -p pr=123 -o name)

# Wait for completion  
argo wait "$WORKFLOW"

# Extract trace ID from logs
TRACE_ID=$(argo logs "$WORKFLOW" | grep -o '"trace_id":"[^"]*"' | head -1 | cut -d'"' -f4)

if [ -n "$TRACE_ID" ]; then
  echo "Trace ID found: $TRACE_ID"
  
  # Query Jaeger for trace
  curl -s "http://jaeger:16686/api/traces/$TRACE_ID" | jq '.data[0].spans | length'
  
else
  echo "ERROR: No trace ID found in logs"
  exit 1
fi
```

### Metrics Validation
```bash
# Check metrics endpoint
curl -s http://otel-collector:8889/metrics | grep agent_step_duration_seconds

# Verify exemplars
curl -s "http://prometheus:9090/api/v1/query?query=agent_step_duration_seconds" | \
  jq '.data.result[0].exemplars'
```

## Dependencies
- OpenTelemetry Collector
- Jaeger or Tempo for trace storage
- Prometheus for metrics collection
- Grafana for visualization
- Fluent Bit/Fluentd for log processing
- Loki for log storage (optional)

## References
- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [OTLP Protocol](https://opentelemetry.io/docs/specs/otlp/)
- [Grafana Exemplars](https://grafana.com/docs/grafana/latest/fundamentals/exemplars/)
- [Prometheus Exemplars](https://prometheus.io/docs/prometheus/latest/feature_flags/#exemplars-storage)