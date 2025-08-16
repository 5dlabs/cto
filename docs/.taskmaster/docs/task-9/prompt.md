# Autonomous Implementation Prompt: OpenTelemetry Observability

## Mission Statement
Implement comprehensive OpenTelemetry observability for TaskMaster agent workflows with distributed tracing, metrics, and correlated logging.

## Technical Requirements
1. **OTEL Tracer Shim** creating spans per agent step with attributes
2. **Metrics Emission** for duration histograms and success/failure counters  
3. **Environment Configuration** injecting OTEL variables into workflow pods
4. **Log Correlation** with trace IDs and structured pod labeling
5. **Workflow Outputs** exposing PR, Actions, and preview URLs
6. **Grafana Dashboards** with exemplars linking to traces

## Key Implementation Points
- Lightweight wrapper (bash + otel-cli) for span lifecycle management
- Span attributes: repo, prNumber, ref, taskId, agent, workflowName, nodeId
- OTLP metrics endpoint: http://otel-collector:4317
- Resource attributes: service.name=agent-steps, k8s.namespace, k8s.pod.name
- Context propagation via W3C traceparent headers
- Prometheus metrics fallback if OTLP unavailable

## Success Criteria
- All agent steps produce spans with correct attributes and timing
- Duration and success/failure metrics visible in monitoring dashboards
- Logs can be queried by repo/PR with trace correlation
- Workflow outputs include clickable PR, Actions, and preview URLs
- System remains resilient during collector outages
- Grafana dashboards show exemplars linking metrics to traces