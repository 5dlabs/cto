# Autonomous Agent Prompt: Create Monitoring and Observability

## Your Mission
You are tasked with implementing comprehensive monitoring with Prometheus metrics, structured logging, and alerting for the Agent Remediation Loop. This system must provide complete visibility into remediation operations, performance tracking, and proactive alerting for operational issues.

## Context
The Agent Remediation Loop is a complex distributed system requiring deep observability to ensure reliable operation, quick troubleshooting, and continuous optimization. Your implementation will provide the metrics, logs, traces, and alerts that enable operators to understand system behavior, identify issues proactively, and maintain high service quality.

## Required Actions

### 1. Define and Expose Prometheus Metrics
Create comprehensive metrics coverage for all system components:
- Implement core metrics: remediation_cycles_total, remediation_iteration_count, remediation_duration_seconds
- Add agent metrics: agent_cancellations_total, active_agents_count
- Build escalation metrics: escalations_total with reason and outcome labels
- Create state management metrics: state_operations_total, configmap_size_bytes
- Add GitHub API metrics: github_api_requests_total, github_api_rate_limit_remaining
- Implement system health metrics: system_health_score, error_rate_percent
- Use appropriate metric types (Counter, Gauge, Histogram) for each measurement

### 2. Implement Structured JSON Logging
Build comprehensive logging framework with context awareness:
- Set up zap logger with production configuration and structured output
- Create context-aware logging functions that extract task_id, correlation_id
- Implement event-specific logging: LogRemediationStarted, LogAgentCancellation, LogEscalation
- Add operation logging: LogStateOperation, LogLabelOperation, LogGitHubAPICall
- Build error logging with comprehensive context and metadata
- Create system health logging with component status and check results
- Use consistent timestamp formatting and log levels throughout

### 3. Create Grafana Dashboards
Design comprehensive dashboards for system visibility:
- Build overview dashboard with key metrics: remediation rates, active tasks, duration distribution
- Create agent operations dashboard: cancellation rates, active agent counts
- Add escalation tracking dashboard: escalation frequency, reasons, outcomes
- Implement performance dashboard: operation durations, error rates, health scores
- Create task-specific dashboard with templating for individual task analysis
- Use appropriate visualization types: time series, gauges, heatmaps, pie charts
- Configure proper time ranges, refresh intervals, and alert annotations

### 4. Set Up Alerting Rules
Implement proactive alerting for operational issues:
- Create iteration alerts: ExcessiveRemediationCycles (>7), RemediationStuck (no progress)
- Build escalation alerts: HighEscalationRate, escalation pattern detection
- Add system health alerts: StateOperationFailures, SystemHealthDegraded
- Implement GitHub API alerts: GitHubAPIRateLimitExhausted, GitHubAPIErrors
- Create resource alerts: ConfigMapSizeTooLarge, LabelOperationFailures
- Configure appropriate severity levels, for clauses, and notification channels
- Include runbook URLs and actionable descriptions in alert annotations

### 5. Add Distributed Tracing
Implement OpenTelemetry-based distributed tracing:
- Set up Jaeger exporter with proper resource attributes and service naming
- Create tracing functions: TraceRemediationCycle, TraceAgentOperation, TraceStateOperation
- Add GitHub API call tracing with HTTP method and URL attributes
- Implement span events and status setting for error conditions
- Build context propagation across service boundaries
- Add trace correlation with logs using span context
- Create trace sampling strategies for performance optimization

### 6. Build Health Check System
Create comprehensive health monitoring:
- Implement HealthChecker with pluggable health check interface
- Create specific health checks: GitHubAPIHealthCheck, StateManagerHealthCheck, KubernetesHealthCheck
- Build overall health aggregation with status summary (healthy/degraded/unhealthy)
- Add HTTP endpoints: /health, /healthz, /ready, /live for Kubernetes integration
- Implement timeout handling and concurrent health check execution
- Create health check results caching to prevent excessive load

## Technical Requirements

### Metrics Implementation
```go
var (
    remediationCyclesTotal = promauto.NewCounterVec(
        prometheus.CounterOpts{
            Name: "remediation_cycles_total",
            Help: "Total remediation cycles by outcome",
        },
        []string{"task_id", "outcome", "severity"},
    )
    
    remediationDurationSeconds = promauto.NewHistogramVec(
        prometheus.HistogramOpts{
            Name: "remediation_duration_seconds",
            Help: "Duration of remediation cycles",
            Buckets: prometheus.ExponentialBuckets(10, 2, 10),
        },
        []string{"task_id", "iteration", "outcome"},
    )
)
```

### Logging Framework
```go
func LogRemediationStarted(ctx context.Context, taskID string, iteration int, severity string) {
    WithContext(ctx).Info("remediation_started",
        zap.String("task_id", taskID),
        zap.Int("iteration", iteration),
        zap.String("severity", severity),
        zap.String("trigger", "pr_comment"),
    )
}
```

### Health Check Interface
```go
type HealthCheck interface {
    Name() string
    Check(ctx context.Context) HealthStatus
}

type HealthStatus struct {
    Status    string
    Message   string
    Details   map[string]interface{}
    Timestamp time.Time
}
```

## Implementation Checklist

### Core Development
- [ ] Implement Prometheus metrics collection for all system components
- [ ] Build structured logging framework with context awareness
- [ ] Create Grafana dashboard configurations with appropriate visualizations
- [ ] Set up Prometheus alerting rules with proper severity and thresholds
- [ ] Implement OpenTelemetry distributed tracing with Jaeger integration
- [ ] Build health check system with component-specific checks

### Integration
- [ ] Integrate metrics collection throughout existing codebase
- [ ] Add logging calls to all major operations and state transitions
- [ ] Connect dashboards to Prometheus data sources
- [ ] Configure alert manager integration with notification channels
- [ ] Set up trace propagation across service boundaries
- [ ] Expose health check endpoints for Kubernetes probes

### Performance and Reliability
- [ ] Optimize metrics collection for minimal performance impact
- [ ] Implement efficient logging with appropriate batching
- [ ] Configure retention policies for metrics, logs, and traces
- [ ] Test monitoring system under high load conditions
- [ ] Validate alert accuracy and reduce false positives

## Expected Outputs

1. **Metrics System**: Complete Prometheus metrics with collectors and exporters
2. **Logging Framework**: Structured logging with context awareness and event tracking
3. **Dashboard Configurations**: Grafana dashboards for system and component monitoring
4. **Alerting Rules**: Comprehensive Prometheus alerts with proper thresholds
5. **Tracing System**: OpenTelemetry integration with span creation and propagation
6. **Health Checks**: Component health monitoring with HTTP endpoints
7. **Integration Layer**: Observability client for easy use throughout application
8. **Documentation**: Monitoring runbooks and troubleshooting guides

## Success Validation

Your implementation is successful when:
1. All system components have comprehensive metrics coverage
2. Structured logging provides clear audit trails and debugging information
3. Dashboards enable effective system monitoring and issue identification
4. Alerts trigger appropriately for operational issues with minimal false positives
5. Distributed traces enable end-to-end request flow analysis
6. Health checks accurately reflect component and overall system status
7. Performance impact remains minimal (<5% overhead)
8. Integration with existing infrastructure works seamlessly
9. Monitoring enables quick troubleshooting and issue resolution
10. Operational visibility improves system reliability and performance

## Technical Constraints

### Performance Requirements
- Metrics collection overhead <2% CPU usage
- Logging operations complete within 10ms
- Health checks complete within 5 seconds
- Tracing overhead <1% of request latency
- Dashboard queries complete within 10 seconds

### Storage and Retention
- Metrics retention: 30 days high resolution, 1 year downsampled
- Logs retention: 7 days detailed, 30 days compressed
- Traces retention: 3 days detailed, 14 days sampled
- Dashboard query performance maintained with data growth

### Integration Requirements
- Compatible with existing Prometheus infrastructure
- Works with current Grafana installation
- Integrates with existing alerting channels
- Supports Kubernetes deployment patterns
- Follows company observability standards

## Common Pitfalls to Avoid

- Don't create high-cardinality metrics that exhaust Prometheus
- Avoid logging sensitive information like tokens or user data
- Don't implement synchronous operations that block request processing
- Ensure proper error handling in monitoring code itself
- Test alert thresholds thoroughly to prevent alert fatigue
- Validate dashboard query performance under realistic data loads
- Don't ignore the performance impact of extensive tracing
- Ensure monitoring system resilience doesn't affect primary functionality

## Resources and References

- Prometheus documentation: https://prometheus.io/docs/
- Grafana dashboard best practices
- OpenTelemetry Go SDK documentation  
- Zap structured logging library
- Kubernetes monitoring patterns and conventions

Begin by understanding the complete system architecture and identifying all components that need monitoring. Design the metrics schema carefully to balance visibility with performance. Focus on operational needs and ensure the monitoring system helps rather than hinders system operations.