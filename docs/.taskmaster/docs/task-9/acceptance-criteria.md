# Acceptance Criteria: OpenTelemetry Observability

## Tracer Shim Implementation
- [ ] Lightweight wrapper accepts commands via exec "$@" with minimal overhead
- [ ] Spans created with name format: agentName/stepName (e.g., clippy/verify)
- [ ] Required span attributes: repo, prNumber, ref, taskId, agent, workflowName, nodeId
- [ ] Error spans marked with status=ERROR and exit code recorded
- [ ] Trace ID and span ID exported to environment variables
- [ ] W3C traceparent context propagation implemented

## Metrics Collection
- [ ] agent_step_duration_ms histogram with proper labels
- [ ] agent_step_success_total and agent_step_fail_total counters
- [ ] Metrics exported via OTLP or Prometheus fallback
- [ ] Exemplars attached to duration samples with trace IDs
- [ ] High-cardinality alerts avoided during load testing

## Environment Configuration
- [ ] OTEL environment variables injected into all workflow pods
- [ ] otel-collector:4317 endpoint reachable from pods
- [ ] Sampling configuration tunable per environment
- [ ] InitContainer health checks validate collector connectivity
- [ ] Steps continue gracefully when collector unavailable

## Pod Labeling and Log Correlation
- [ ] Workflow pods labeled with: workflows.argoproj.io/workflow, repo, pr, taskId, agent
- [ ] Log entries enriched with trace ID and span ID fields
- [ ] Logs filterable by repo/PR in logging UI
- [ ] Structured log parsing extracts correlation fields

## Workflow Output Parameters
- [ ] prHtmlUrl extracted from GitHub event payload
- [ ] actionsRunUrl constructed from workflow context
- [ ] previewUrl emitted by deploy step via output parameters
- [ ] All URLs clickable in Argo UI and accessible via CLI

## Grafana Dashboard Integration
- [ ] Duration panels show P50/P90/P99 by agent and repo
- [ ] Success rate and failure count visualizations
- [ ] Exemplar links open corresponding traces
- [ ] Variable filters work without query errors
- [ ] Dashboard panels refresh within 15 seconds

## Resilience Testing
- [ ] Agent pod kills during execution handled gracefully
- [ ] Controller restarts don't break span continuity
- [ ] Collector outages don't fail business logic
- [ ] High concurrency (50+ parallel steps) performs acceptably
- [ ] No orphaned spans without proper parent relationships