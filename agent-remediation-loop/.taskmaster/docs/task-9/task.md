# Task 9: Create Monitoring and Observability

## Overview
Implement comprehensive monitoring with Prometheus metrics, structured logging, and alerting for the Agent Remediation Loop. This system provides complete visibility into remediation operations, performance tracking, and proactive alerting for operational issues.

## Technical Context
The Agent Remediation Loop is a complex distributed system requiring deep observability to ensure reliable operation, quick troubleshooting, and continuous optimization. This monitoring system provides metrics, logs, traces, and alerts that enable operators to understand system behavior, identify issues proactively, and maintain high service quality.

### Observability Pillars
- **Metrics**: Quantitative measurements of system behavior and performance
- **Logs**: Structured event records for debugging and audit trails
- **Traces**: Request flow tracking across distributed components
- **Alerts**: Proactive notifications for operational issues

## Implementation Guide

### Step 1: Define and Expose Prometheus Metrics

#### 1.1 Core Metrics Schema
```go
package metrics

import (
    "github.com/prometheus/client_golang/prometheus"
    "github.com/prometheus/client_golang/prometheus/promauto"
)

var (
    // Remediation cycle metrics
    remediationCyclesTotal = promauto.NewCounterVec(
        prometheus.CounterOpts{
            Name: "remediation_cycles_total",
            Help: "Total number of remediation cycles by task and outcome",
        },
        []string{"task_id", "outcome", "severity"},
    )

    remediationIterationCount = promauto.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "remediation_iteration_count",
            Help: "Current iteration count for active tasks",
        },
        []string{"task_id"},
    )

    remediationDurationSeconds = promauto.NewHistogramVec(
        prometheus.HistogramOpts{
            Name:    "remediation_duration_seconds",
            Help:    "Duration of remediation cycles in seconds",
            Buckets: prometheus.ExponentialBuckets(10, 2, 10), // 10s to ~2.8 hours
        },
        []string{"task_id", "iteration", "outcome"},
    )

    // Agent operation metrics
    agentCancellationsTotal = promauto.NewCounterVec(
        prometheus.CounterOpts{
            Name: "agent_cancellations_total",
            Help: "Total number of agent cancellations by type and reason",
        },
        []string{"agent_type", "reason", "successful"},
    )

    activeAgentsCount = promauto.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "active_agents_count",
            Help: "Number of currently active agents by type",
        },
        []string{"agent_type", "task_id"},
    )

    // Escalation metrics
    escalationsTotal = promauto.NewCounterVec(
        prometheus.CounterOpts{
            Name: "escalations_total",
            Help: "Total number of escalations by reason and outcome",
        },
        []string{"reason", "outcome", "notification_channel"},
    )

    // State management metrics
    stateOperationsTotal = promauto.NewCounterVec(
        prometheus.CounterOpts{
            Name: "state_operations_total",
            Help: "Total state management operations by type and outcome",
        },
        []string{"operation", "outcome"},
    )

    stateOperationDurationSeconds = promauto.NewHistogramVec(
        prometheus.HistogramOpts{
            Name:    "state_operation_duration_seconds",
            Help:    "Duration of state management operations",
            Buckets: prometheus.DefBuckets,
        },
        []string{"operation"},
    )

    configMapSizeBytes = promauto.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "configmap_size_bytes",
            Help: "Size of state ConfigMaps in bytes",
        },
        []string{"task_id", "type"},
    )

    // GitHub API metrics
    githubAPIRequestsTotal = promauto.NewCounterVec(
        prometheus.CounterOpts{
            Name: "github_api_requests_total",
            Help: "Total GitHub API requests by endpoint and status",
        },
        []string{"endpoint", "method", "status_code"},
    )

    githubAPIRateLimitRemaining = promauto.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "github_api_rate_limit_remaining",
            Help: "Remaining GitHub API rate limit",
        },
        []string{"endpoint"},
    )

    // Label management metrics
    labelOperationsTotal = promauto.NewCounterVec(
        prometheus.CounterOpts{
            Name: "label_operations_total",
            Help: "Total label operations by type and outcome",
        },
        []string{"operation_type", "outcome"},
    )

    labelOperationDurationSeconds = promauto.NewHistogramVec(
        prometheus.HistogramOpts{
            Name:    "label_operation_duration_seconds",
            Help:    "Duration of label operations",
            Buckets: prometheus.DefBuckets,
        },
        []string{"operation_type"},
    )

    // System health metrics
    systemHealthScore = promauto.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "system_health_score",
            Help: "Overall system health score (0-1)",
        },
        []string{"component"},
    )

    errorRatePercent = promauto.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "error_rate_percent",
            Help: "Error rate percentage by component",
        },
        []string{"component", "error_type"},
    )
)
```

#### 1.2 Metrics Collection Functions
```go
package metrics

import (
    "context"
    "strconv"
    "time"
)

type MetricsCollector struct {
    enabled bool
}

func NewMetricsCollector(enabled bool) *MetricsCollector {
    return &MetricsCollector{enabled: enabled}
}

func (m *MetricsCollector) RecordRemediationCycle(taskID, outcome, severity string, duration time.Duration) {
    if !m.enabled {
        return
    }
    
    remediationCyclesTotal.WithLabelValues(taskID, outcome, severity).Inc()
    remediationDurationSeconds.WithLabelValues(taskID, outcome).Observe(duration.Seconds())
}

func (m *MetricsCollector) UpdateIterationCount(taskID string, count int) {
    if !m.enabled {
        return
    }
    
    remediationIterationCount.WithLabelValues(taskID).Set(float64(count))
}

func (m *MetricsCollector) RecordAgentCancellation(agentType, reason string, successful bool) {
    if !m.enabled {
        return
    }
    
    agentCancellationsTotal.WithLabelValues(agentType, reason, strconv.FormatBool(successful)).Inc()
}

func (m *MetricsCollector) UpdateActiveAgents(agentType, taskID string, count int) {
    if !m.enabled {
        return
    }
    
    activeAgentsCount.WithLabelValues(agentType, taskID).Set(float64(count))
}

func (m *MetricsCollector) RecordEscalation(reason, outcome, channel string) {
    if !m.enabled {
        return
    }
    
    escalationsTotal.WithLabelValues(reason, outcome, channel).Inc()
}

func (m *MetricsCollector) RecordStateOperation(operation, outcome string, duration time.Duration) {
    if !m.enabled {
        return
    }
    
    stateOperationsTotal.WithLabelValues(operation, outcome).Inc()
    stateOperationDurationSeconds.WithLabelValues(operation).Observe(duration.Seconds())
}

func (m *MetricsCollector) UpdateConfigMapSize(taskID, mapType string, size int64) {
    if !m.enabled {
        return
    }
    
    configMapSizeBytes.WithLabelValues(taskID, mapType).Set(float64(size))
}

func (m *MetricsCollector) RecordGitHubAPIRequest(endpoint, method, statusCode string) {
    if !m.enabled {
        return
    }
    
    githubAPIRequestsTotal.WithLabelValues(endpoint, method, statusCode).Inc()
}

func (m *MetricsCollector) UpdateGitHubRateLimit(endpoint string, remaining int) {
    if !m.enabled {
        return
    }
    
    githubAPIRateLimitRemaining.WithLabelValues(endpoint).Set(float64(remaining))
}

func (m *MetricsCollector) RecordLabelOperation(operationType, outcome string, duration time.Duration) {
    if !m.enabled {
        return
    }
    
    labelOperationsTotal.WithLabelValues(operationType, outcome).Inc()
    labelOperationDurationSeconds.WithLabelValues(operationType).Observe(duration.Seconds())
}

func (m *MetricsCollector) UpdateSystemHealth(component string, score float64) {
    if !m.enabled {
        return
    }
    
    systemHealthScore.WithLabelValues(component).Set(score)
}

func (m *MetricsCollector) UpdateErrorRate(component, errorType string, rate float64) {
    if !m.enabled {
        return
    }
    
    errorRatePercent.WithLabelValues(component, errorType).Set(rate)
}
```

### Step 2: Implement Structured JSON Logging

#### 2.1 Logging Framework Setup
```go
package logging

import (
    "context"
    "fmt"
    "os"
    "time"
    
    "go.uber.org/zap"
    "go.uber.org/zap/zapcore"
)

var logger *zap.Logger

func InitLogger(level string, development bool) error {
    var config zap.Config
    
    if development {
        config = zap.NewDevelopmentConfig()
        config.EncoderConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder
    } else {
        config = zap.NewProductionConfig()
        config.EncoderConfig.TimeKey = "timestamp"
        config.EncoderConfig.EncodeTime = zapcore.ISO8601TimeEncoder
    }
    
    // Parse log level
    var zapLevel zapcore.Level
    switch level {
    case "debug":
        zapLevel = zapcore.DebugLevel
    case "info":
        zapLevel = zapcore.InfoLevel
    case "warn":
        zapLevel = zapcore.WarnLevel
    case "error":
        zapLevel = zapcore.ErrorLevel
    default:
        zapLevel = zapcore.InfoLevel
    }
    config.Level.SetLevel(zapLevel)
    
    // Add common fields
    config.InitialFields = map[string]interface{}{
        "service":   "agent-remediation-loop",
        "version":   os.Getenv("VERSION"),
        "component": "controller",
    }
    
    var err error
    logger, err = config.Build()
    if err != nil {
        return fmt.Errorf("failed to initialize logger: %w", err)
    }
    
    return nil
}

func GetLogger() *zap.Logger {
    return logger
}

// Context-aware logging
func WithContext(ctx context.Context) *zap.Logger {
    if logger == nil {
        // Fallback logger
        logger, _ = zap.NewProduction()
    }
    
    // Extract context values for logging
    if taskID := ctx.Value("task_id"); taskID != nil {
        return logger.With(zap.String("task_id", taskID.(string)))
    }
    
    if correlationID := ctx.Value("correlation_id"); correlationID != nil {
        return logger.With(zap.String("correlation_id", correlationID.(string)))
    }
    
    return logger
}

// Structured event logging
func LogRemediationStarted(ctx context.Context, taskID string, iteration int, severity string) {
    WithContext(ctx).Info("remediation_started",
        zap.String("task_id", taskID),
        zap.Int("iteration", iteration),
        zap.String("severity", severity),
        zap.String("trigger", "pr_comment"),
        zap.Time("timestamp", time.Now()),
    )
}

func LogRemediationCompleted(ctx context.Context, taskID string, iteration int, outcome string, duration time.Duration) {
    WithContext(ctx).Info("remediation_completed",
        zap.String("task_id", taskID),
        zap.Int("iteration", iteration),
        zap.String("outcome", outcome),
        zap.Duration("duration", duration),
        zap.Time("timestamp", time.Now()),
    )
}

func LogAgentCancellation(ctx context.Context, agentType, taskID, reason string, successful bool) {
    WithContext(ctx).Info("agent_cancellation",
        zap.String("agent_type", agentType),
        zap.String("task_id", taskID),
        zap.String("reason", reason),
        zap.Bool("successful", successful),
        zap.Time("timestamp", time.Now()),
    )
}

func LogEscalation(ctx context.Context, taskID string, reason, outcome string, iteration int) {
    WithContext(ctx).Warn("escalation_triggered",
        zap.String("task_id", taskID),
        zap.String("reason", reason),
        zap.String("outcome", outcome),
        zap.Int("iteration", iteration),
        zap.Time("timestamp", time.Now()),
    )
}

func LogStateOperation(ctx context.Context, operation, taskID, outcome string, duration time.Duration) {
    WithContext(ctx).Info("state_operation",
        zap.String("operation", operation),
        zap.String("task_id", taskID),
        zap.String("outcome", outcome),
        zap.Duration("duration", duration),
        zap.Time("timestamp", time.Now()),
    )
}

func LogLabelOperation(ctx context.Context, operationType string, prNumber int, outcome string, labelsAffected []string) {
    WithContext(ctx).Info("label_operation",
        zap.String("operation_type", operationType),
        zap.Int("pr_number", prNumber),
        zap.String("outcome", outcome),
        zap.Strings("labels_affected", labelsAffected),
        zap.Time("timestamp", time.Now()),
    )
}

func LogGitHubAPICall(ctx context.Context, endpoint, method string, statusCode int, duration time.Duration, rateLimitRemaining int) {
    WithContext(ctx).Info("github_api_call",
        zap.String("endpoint", endpoint),
        zap.String("method", method),
        zap.Int("status_code", statusCode),
        zap.Duration("duration", duration),
        zap.Int("rate_limit_remaining", rateLimitRemaining),
        zap.Time("timestamp", time.Now()),
    )
}

func LogSystemError(ctx context.Context, component, operation, errorType string, err error, metadata map[string]interface{}) {
    fields := []zap.Field{
        zap.String("component", component),
        zap.String("operation", operation),
        zap.String("error_type", errorType),
        zap.Error(err),
        zap.Time("timestamp", time.Now()),
    }
    
    // Add metadata fields
    for key, value := range metadata {
        fields = append(fields, zap.Any(key, value))
    }
    
    WithContext(ctx).Error("system_error", fields...)
}

func LogSystemHealth(ctx context.Context, component string, healthScore float64, checks map[string]bool) {
    fields := []zap.Field{
        zap.String("component", component),
        zap.Float64("health_score", healthScore),
        zap.Time("timestamp", time.Now()),
    }
    
    // Add individual check results
    for check, result := range checks {
        fields = append(fields, zap.Bool(fmt.Sprintf("check_%s", check), result))
    }
    
    WithContext(ctx).Info("system_health", fields...)
}
```

### Step 3: Create Grafana Dashboards

#### 3.1 Dashboard Configuration
```json
{
  "dashboard": {
    "id": null,
    "title": "Agent Remediation Loop - Overview",
    "tags": ["remediation", "agents", "monitoring"],
    "timezone": "UTC",
    "panels": [
      {
        "title": "Remediation Cycles Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(remediation_cycles_total[5m])",
            "legendFormat": "{{outcome}}"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "color": {
              "mode": "palette-classic"
            },
            "custom": {
              "displayMode": "list",
              "orientation": "horizontal"
            },
            "unit": "reqps"
          }
        }
      },
      {
        "title": "Active Remediation Tasks",
        "type": "gauge",
        "targets": [
          {
            "expr": "count(remediation_iteration_count > 0)",
            "legendFormat": "Active Tasks"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "min": 0,
            "max": 100,
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 50},
                {"color": "red", "value": 80}
              ]
            }
          }
        }
      },
      {
        "title": "Remediation Duration Distribution",
        "type": "heatmap",
        "targets": [
          {
            "expr": "rate(remediation_duration_seconds_bucket[5m])",
            "legendFormat": "{{le}}"
          }
        ]
      },
      {
        "title": "Agent Operations",
        "type": "timeseries",
        "targets": [
          {
            "expr": "rate(agent_cancellations_total[5m])",
            "legendFormat": "Cancellations ({{agent_type}})"
          },
          {
            "expr": "active_agents_count",
            "legendFormat": "Active ({{agent_type}})"
          }
        ]
      },
      {
        "title": "Escalations by Reason",
        "type": "piechart",
        "targets": [
          {
            "expr": "increase(escalations_total[1h])",
            "legendFormat": "{{reason}}"
          }
        ]
      },
      {
        "title": "State Operation Performance",
        "type": "timeseries",
        "targets": [
          {
            "expr": "rate(state_operations_total{outcome=\"success\"}[5m])",
            "legendFormat": "Success Rate"
          },
          {
            "expr": "histogram_quantile(0.95, rate(state_operation_duration_seconds_bucket[5m]))",
            "legendFormat": "95th Percentile Duration"
          }
        ]
      },
      {
        "title": "GitHub API Health",
        "type": "timeseries",
        "targets": [
          {
            "expr": "rate(github_api_requests_total[5m])",
            "legendFormat": "Request Rate ({{status_code}})"
          },
          {
            "expr": "github_api_rate_limit_remaining",
            "legendFormat": "Rate Limit Remaining"
          }
        ]
      },
      {
        "title": "System Health Scores",
        "type": "stat",
        "targets": [
          {
            "expr": "system_health_score",
            "legendFormat": "{{component}}"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "min": 0,
            "max": 1,
            "thresholds": {
              "steps": [
                {"color": "red", "value": null},
                {"color": "yellow", "value": 0.7},
                {"color": "green", "value": 0.9}
              ]
            }
          }
        }
      },
      {
        "title": "Error Rates by Component",
        "type": "timeseries",
        "targets": [
          {
            "expr": "error_rate_percent",
            "legendFormat": "{{component}} - {{error_type}}"
          }
        ]
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "30s"
  }
}
```

#### 3.2 Task-Specific Dashboard
```json
{
  "dashboard": {
    "title": "Agent Remediation Loop - Task Details",
    "templating": {
      "list": [
        {
          "name": "task_id",
          "type": "query",
          "query": "label_values(remediation_iteration_count, task_id)"
        }
      ]
    },
    "panels": [
      {
        "title": "Iteration Progress",
        "type": "timeseries",
        "targets": [
          {
            "expr": "remediation_iteration_count{task_id=\"$task_id\"}",
            "legendFormat": "Current Iteration"
          }
        ]
      },
      {
        "title": "Remediation Timeline",
        "type": "timeseries",
        "targets": [
          {
            "expr": "remediation_duration_seconds{task_id=\"$task_id\"}",
            "legendFormat": "Duration (Iteration {{iteration}})"
          }
        ]
      },
      {
        "title": "State Operations",
        "type": "logs",
        "targets": [
          {
            "expr": "{task_id=\"$task_id\"} |= \"state_operation\"",
            "refId": "A"
          }
        ]
      },
      {
        "title": "Agent Activity",
        "type": "timeseries",
        "targets": [
          {
            "expr": "active_agents_count{task_id=\"$task_id\"}",
            "legendFormat": "{{agent_type}}"
          }
        ]
      }
    ]
  }
}
```

### Step 4: Set Up Alerting Rules

#### 4.1 Prometheus Alerting Rules
```yaml
# alerts/remediation.yml
groups:
  - name: remediation.rules
    rules:
      - alert: ExcessiveRemediationCycles
        expr: remediation_iteration_count > 7
        for: 5m
        labels:
          severity: warning
          component: remediation-loop
        annotations:
          summary: "Task {{$labels.task_id}} has {{$value}} remediation cycles"
          description: "Task {{$labels.task_id}} has reached {{$value}} iterations, approaching the maximum of 10. Manual intervention may be required soon."
          runbook_url: "https://docs.company.com/runbooks/remediation-cycles"

      - alert: RemediationStuck
        expr: rate(remediation_cycles_total[2h]) == 0 and on() count(remediation_iteration_count > 0) > 0
        for: 30m
        labels:
          severity: critical
          component: remediation-loop
        annotations:
          summary: "No remediation progress in 2 hours"
          description: "Active remediation tasks exist but no cycles have completed in the past 2 hours. System may be stuck."
          runbook_url: "https://docs.company.com/runbooks/remediation-stuck"

      - alert: HighEscalationRate
        expr: rate(escalations_total[1h]) > 0.1
        for: 15m
        labels:
          severity: warning
          component: remediation-loop
        annotations:
          summary: "High escalation rate detected"
          description: "Escalation rate is {{$value}} per second over the last hour, indicating potential system issues."

      - alert: StateOperationFailures
        expr: rate(state_operations_total{outcome="error"}[5m]) > 0.05
        for: 10m
        labels:
          severity: critical
          component: state-management
        annotations:
          summary: "High state operation failure rate"
          description: "State operations are failing at {{$value}} per second, which may cause data inconsistency."

      - alert: GitHubAPIRateLimitExhausted
        expr: github_api_rate_limit_remaining < 100
        for: 1m
        labels:
          severity: warning
          component: github-integration
        annotations:
          summary: "GitHub API rate limit nearly exhausted"
          description: "Only {{$value}} GitHub API requests remaining. System performance may degrade."

      - alert: GitHubAPIErrors
        expr: rate(github_api_requests_total{status_code=~"4..|5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
          component: github-integration
        annotations:
          summary: "High GitHub API error rate"
          description: "GitHub API errors occurring at {{$value}} per second over the last 5 minutes."

      - alert: SystemHealthDegraded
        expr: system_health_score < 0.7
        for: 5m
        labels:
          severity: warning
          component: system-health
        annotations:
          summary: "System health degraded for {{$labels.component}}"
          description: "Health score for {{$labels.component}} is {{$value}}, below the 0.7 threshold."

      - alert: ConfigMapSizeTooLarge
        expr: configmap_size_bytes > 800000  # 800KB, approaching 1MB limit
        for: 5m
        labels:
          severity: warning
          component: state-management
        annotations:
          summary: "ConfigMap approaching size limit"
          description: "ConfigMap for task {{$labels.task_id}} is {{$value}} bytes, approaching the 1MB Kubernetes limit."

      - alert: LabelOperationFailures
        expr: rate(label_operations_total{outcome="error"}[5m]) > 0.02
        for: 10m
        labels:
          severity: warning
          component: label-management
        annotations:
          summary: "Label operation failures detected"
          description: "Label operations are failing at {{$value}} per second, which may affect workflow orchestration."

      - alert: NoActiveRemediationProgress
        expr: count(remediation_iteration_count > 0) == 0 and on() increase(remediation_cycles_total[6h]) == 0
        for: 1h
        labels:
          severity: info
          component: remediation-loop
        annotations:
          summary: "No active remediation tasks"
          description: "No remediation activity detected for the past 6 hours. This may be normal or indicate a problem with task detection."
```

### Step 5: Add Distributed Tracing

#### 5.1 OpenTelemetry Integration
```go
package tracing

import (
    "context"
    "fmt"
    
    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/attribute"
    "go.opentelemetry.io/otel/exporters/jaeger"
    "go.opentelemetry.io/otel/propagation"
    "go.opentelemetry.io/otel/sdk/resource"
    "go.opentelemetry.io/otel/sdk/trace"
    "go.opentelemetry.io/otel/semconv/v1.4.0"
    oteltrace "go.opentelemetry.io/otel/trace"
)

var tracer oteltrace.Tracer

func InitTracing(jaegerEndpoint, serviceName string) error {
    exp, err := jaeger.New(jaeger.WithCollectorEndpoint(jaeger.WithEndpoint(jaegerEndpoint)))
    if err != nil {
        return fmt.Errorf("failed to initialize Jaeger exporter: %w", err)
    }

    tp := trace.NewTracerProvider(
        trace.WithBatcher(exp),
        trace.WithResource(resource.NewWithAttributes(
            semconv.SchemaURL,
            semconv.ServiceNameKey.String(serviceName),
            semconv.ServiceVersionKey.String("1.0.0"),
        )),
    )

    otel.SetTracerProvider(tp)
    otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
        propagation.TraceContext{},
        propagation.Baggage{},
    ))

    tracer = otel.Tracer("agent-remediation-loop")
    return nil
}

func StartSpan(ctx context.Context, operationName string, attributes ...attribute.KeyValue) (context.Context, oteltrace.Span) {
    return tracer.Start(ctx, operationName, oteltrace.WithAttributes(attributes...))
}

// Specific tracing functions for key operations
func TraceRemediationCycle(ctx context.Context, taskID string, iteration int) (context.Context, oteltrace.Span) {
    return StartSpan(ctx, "remediation_cycle",
        attribute.String("task_id", taskID),
        attribute.Int("iteration", iteration),
        attribute.String("component", "remediation-controller"),
    )
}

func TraceAgentOperation(ctx context.Context, agentType, operation, taskID string) (context.Context, oteltrace.Span) {
    return StartSpan(ctx, fmt.Sprintf("agent_%s", operation),
        attribute.String("agent_type", agentType),
        attribute.String("operation", operation),
        attribute.String("task_id", taskID),
        attribute.String("component", "agent-manager"),
    )
}

func TraceStateOperation(ctx context.Context, operation, taskID string) (context.Context, oteltrace.Span) {
    return StartSpan(ctx, fmt.Sprintf("state_%s", operation),
        attribute.String("operation", operation),
        attribute.String("task_id", taskID),
        attribute.String("component", "state-manager"),
    )
}

func TraceLabelOperation(ctx context.Context, operationType string, prNumber int) (context.Context, oteltrace.Span) {
    return StartSpan(ctx, fmt.Sprintf("label_%s", operationType),
        attribute.String("operation_type", operationType),
        attribute.Int("pr_number", prNumber),
        attribute.String("component", "label-orchestrator"),
    )
}

func TraceGitHubAPICall(ctx context.Context, endpoint, method string) (context.Context, oteltrace.Span) {
    return StartSpan(ctx, "github_api_call",
        attribute.String("http.method", method),
        attribute.String("http.url", endpoint),
        attribute.String("component", "github-client"),
    )
}

func TraceEscalation(ctx context.Context, taskID, reason string) (context.Context, oteltrace.Span) {
    return StartSpan(ctx, "escalation",
        attribute.String("task_id", taskID),
        attribute.String("escalation_reason", reason),
        attribute.String("component", "escalation-manager"),
    )
}

// Utility functions for adding span events and attributes
func AddSpanEvent(span oteltrace.Span, name string, attributes ...attribute.KeyValue) {
    span.AddEvent(name, oteltrace.WithAttributes(attributes...))
}

func SetSpanStatus(span oteltrace.Span, err error) {
    if err != nil {
        span.RecordError(err)
        span.SetStatus(codes.Error, err.Error())
    } else {
        span.SetStatus(codes.Ok, "")
    }
}
```

### Step 6: Health Check System

#### 6.1 Health Check Implementation
```go
package health

import (
    "context"
    "encoding/json"
    "fmt"
    "net/http"
    "sync"
    "time"
    
    "go.uber.org/zap"
)

type HealthChecker struct {
    checks map[string]HealthCheck
    mutex  sync.RWMutex
    logger *zap.Logger
}

type HealthCheck interface {
    Name() string
    Check(ctx context.Context) HealthStatus
}

type HealthStatus struct {
    Status    string                 `json:"status"`
    Message   string                 `json:"message,omitempty"`
    Details   map[string]interface{} `json:"details,omitempty"`
    Timestamp time.Time             `json:"timestamp"`
}

type OverallHealth struct {
    Status      string                    `json:"status"`
    Timestamp   time.Time                 `json:"timestamp"`
    Components  map[string]HealthStatus   `json:"components"`
    Summary     map[string]int           `json:"summary"`
}

func NewHealthChecker(logger *zap.Logger) *HealthChecker {
    return &HealthChecker{
        checks: make(map[string]HealthCheck),
        logger: logger,
    }
}

func (hc *HealthChecker) RegisterCheck(check HealthCheck) {
    hc.mutex.Lock()
    defer hc.mutex.Unlock()
    hc.checks[check.Name()] = check
}

func (hc *HealthChecker) CheckAll(ctx context.Context) OverallHealth {
    hc.mutex.RLock()
    defer hc.mutex.RUnlock()
    
    results := make(map[string]HealthStatus)
    summary := map[string]int{
        "healthy":   0,
        "degraded":  0,
        "unhealthy": 0,
    }
    
    for name, check := range hc.checks {
        status := check.Check(ctx)
        results[name] = status
        
        switch status.Status {
        case "healthy":
            summary["healthy"]++
        case "degraded":
            summary["degraded"]++
        default:
            summary["unhealthy"]++
        }
    }
    
    overallStatus := "healthy"
    if summary["unhealthy"] > 0 {
        overallStatus = "unhealthy"
    } else if summary["degraded"] > 0 {
        overallStatus = "degraded"
    }
    
    return OverallHealth{
        Status:     overallStatus,
        Timestamp:  time.Now(),
        Components: results,
        Summary:    summary,
    }
}

func (hc *HealthChecker) ServeHTTP(w http.ResponseWriter, r *http.Request) {
    ctx, cancel := context.WithTimeout(r.Context(), 30*time.Second)
    defer cancel()
    
    health := hc.CheckAll(ctx)
    
    w.Header().Set("Content-Type", "application/json")
    
    statusCode := http.StatusOK
    switch health.Status {
    case "degraded":
        statusCode = http.StatusOK // Still return 200 for degraded
    case "unhealthy":
        statusCode = http.StatusServiceUnavailable
    }
    w.WriteHeader(statusCode)
    
    json.NewEncoder(w).Encode(health)
}

// Specific health checks for remediation loop components

type GitHubAPIHealthCheck struct {
    client GitHubClient
}

func (g *GitHubAPIHealthCheck) Name() string {
    return "github_api"
}

func (g *GitHubAPIHealthCheck) Check(ctx context.Context) HealthStatus {
    start := time.Now()
    
    // Simple API call to check connectivity
    _, err := g.client.GetRateLimit(ctx)
    duration := time.Since(start)
    
    if err != nil {
        return HealthStatus{
            Status:    "unhealthy",
            Message:   fmt.Sprintf("GitHub API check failed: %v", err),
            Details:   map[string]interface{}{"duration_ms": duration.Milliseconds()},
            Timestamp: time.Now(),
        }
    }
    
    status := "healthy"
    if duration > 5*time.Second {
        status = "degraded"
    }
    
    return HealthStatus{
        Status:  status,
        Message: "GitHub API accessible",
        Details: map[string]interface{}{
            "duration_ms": duration.Milliseconds(),
        },
        Timestamp: time.Now(),
    }
}

type StateManagerHealthCheck struct {
    stateManager StateManager
}

func (s *StateManagerHealthCheck) Name() string {
    return "state_manager"
}

func (s *StateManagerHealthCheck) Check(ctx context.Context) HealthStatus {
    start := time.Now()
    
    // Test basic state operation
    testTaskID := fmt.Sprintf("health-check-%d", time.Now().Unix())
    err := s.stateManager.TestConnection(ctx, testTaskID)
    duration := time.Since(start)
    
    if err != nil {
        return HealthStatus{
            Status:    "unhealthy",
            Message:   fmt.Sprintf("State manager check failed: %v", err),
            Details:   map[string]interface{}{"duration_ms": duration.Milliseconds()},
            Timestamp: time.Now(),
        }
    }
    
    status := "healthy"
    if duration > 2*time.Second {
        status = "degraded"
    }
    
    return HealthStatus{
        Status:  status,
        Message: "State manager accessible",
        Details: map[string]interface{}{
            "duration_ms": duration.Milliseconds(),
        },
        Timestamp: time.Now(),
    }
}

type KubernetesHealthCheck struct {
    client kubernetes.Interface
}

func (k *KubernetesHealthCheck) Name() string {
    return "kubernetes"
}

func (k *KubernetesHealthCheck) Check(ctx context.Context) HealthStatus {
    start := time.Now()
    
    // Test cluster connectivity
    _, err := k.client.CoreV1().Namespaces().List(ctx, metav1.ListOptions{Limit: 1})
    duration := time.Since(start)
    
    if err != nil {
        return HealthStatus{
            Status:    "unhealthy",
            Message:   fmt.Sprintf("Kubernetes check failed: %v", err),
            Details:   map[string]interface{}{"duration_ms": duration.Milliseconds()},
            Timestamp: time.Now(),
        }
    }
    
    status := "healthy"
    if duration > 3*time.Second {
        status = "degraded"
    }
    
    return HealthStatus{
        Status:  status,
        Message: "Kubernetes cluster accessible",
        Details: map[string]interface{}{
            "duration_ms": duration.Milliseconds(),
        },
        Timestamp: time.Now(),
    }
}
```

### Step 7: Integration with Existing Systems

#### 7.1 Monitoring Service Integration
```go
package monitoring

import (
    "context"
    "fmt"
    "net/http"
    "time"
    
    "github.com/prometheus/client_golang/prometheus/promhttp"
    "go.uber.org/zap"
)

type MonitoringService struct {
    logger           *zap.Logger
    metricsCollector *MetricsCollector
    healthChecker    *HealthChecker
    tracingEnabled   bool
    httpServer       *http.Server
}

func NewMonitoringService(logger *zap.Logger, tracingEnabled bool) *MonitoringService {
    return &MonitoringService{
        logger:           logger,
        metricsCollector: NewMetricsCollector(true),
        healthChecker:    NewHealthChecker(logger),
        tracingEnabled:   tracingEnabled,
    }
}

func (ms *MonitoringService) Start(ctx context.Context, port int) error {
    mux := http.NewServeMux()
    
    // Prometheus metrics endpoint
    mux.Handle("/metrics", promhttp.Handler())
    
    // Health check endpoint
    mux.Handle("/health", ms.healthChecker)
    mux.Handle("/healthz", ms.healthChecker) // Kubernetes convention
    
    // Ready check endpoint
    mux.HandleFunc("/ready", func(w http.ResponseWriter, r *http.Request) {
        w.WriteHeader(http.StatusOK)
        w.Write([]byte("ready"))
    })
    
    // Live check endpoint
    mux.HandleFunc("/live", func(w http.ResponseWriter, r *http.Request) {
        w.WriteHeader(http.StatusOK)
        w.Write([]byte("live"))
    })
    
    ms.httpServer = &http.Server{
        Addr:    fmt.Sprintf(":%d", port),
        Handler: mux,
    }
    
    ms.logger.Info("Starting monitoring service", zap.Int("port", port))
    
    go func() {
        if err := ms.httpServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
            ms.logger.Error("Monitoring service failed", zap.Error(err))
        }
    }()
    
    return nil
}

func (ms *MonitoringService) Stop(ctx context.Context) error {
    if ms.httpServer != nil {
        return ms.httpServer.Shutdown(ctx)
    }
    return nil
}

func (ms *MonitoringService) RegisterHealthCheck(check HealthCheck) {
    ms.healthChecker.RegisterCheck(check)
}

func (ms *MonitoringService) GetMetricsCollector() *MetricsCollector {
    return ms.metricsCollector
}

// Integration wrapper for easy use throughout the application
type ObservabilityClient struct {
    logger  *zap.Logger
    metrics *MetricsCollector
    tracer  oteltrace.Tracer
}

func NewObservabilityClient(logger *zap.Logger, metrics *MetricsCollector) *ObservabilityClient {
    return &ObservabilityClient{
        logger:  logger,
        metrics: metrics,
        tracer:  tracer,
    }
}

func (oc *ObservabilityClient) WithContext(ctx context.Context) *ObservabilityClient {
    return &ObservabilityClient{
        logger:  WithContext(ctx),
        metrics: oc.metrics,
        tracer:  oc.tracer,
    }
}

// Convenience methods combining metrics, logging, and tracing
func (oc *ObservabilityClient) StartRemediationCycle(ctx context.Context, taskID string, iteration int, severity string) (context.Context, oteltrace.Span) {
    // Log the event
    LogRemediationStarted(ctx, taskID, iteration, severity)
    
    // Start tracing
    span_ctx, span := TraceRemediationCycle(ctx, taskID, iteration)
    
    // Update metrics
    oc.metrics.UpdateIterationCount(taskID, iteration)
    
    return span_ctx, span
}

func (oc *ObservabilityClient) CompleteRemediationCycle(ctx context.Context, span oteltrace.Span, taskID string, iteration int, outcome string, duration time.Duration) {
    // Log completion
    LogRemediationCompleted(ctx, taskID, iteration, outcome, duration)
    
    // Record metrics
    oc.metrics.RecordRemediationCycle(taskID, outcome, "medium", duration)
    
    // Complete trace
    SetSpanStatus(span, nil)
    span.End()
}
```

## Performance Considerations

### Metrics Collection Efficiency
- Use prometheus client library's built-in batching
- Minimize label cardinality to prevent metric explosion
- Implement sampling for high-frequency operations
- Cache metric calculations where appropriate

### Logging Performance
- Use structured logging to avoid string formatting costs
- Implement log level filtering at the source
- Batch log writes for high-throughput scenarios
- Compress logs for long-term storage

### Storage and Retention
- Configure appropriate retention policies for metrics (30 days default)
- Use log rotation and archiving for disk management
- Implement metric downsampling for long-term trends
- Monitor storage usage and implement alerts

## Security Considerations

### Sensitive Data Handling
- Never log sensitive information (tokens, passwords, user data)
- Sanitize error messages to prevent information disclosure
- Use structured logging to control data exposure
- Implement log redaction for compliance requirements

### Access Control
- Secure metrics and health endpoints appropriately
- Implement authentication for sensitive monitoring data
- Control access to logs and traces based on role
- Audit access to monitoring infrastructure

## Success Criteria
- Comprehensive metrics coverage for all system components
- Structured logging provides clear audit trails and debugging information
- Grafana dashboards enable effective system monitoring and troubleshooting
- Alerting rules provide proactive notification of operational issues
- Distributed tracing enables end-to-end request flow analysis
- Health checks provide accurate system status information
- Integration with existing monitoring infrastructure works seamlessly
- Performance impact of monitoring remains minimal (<5% overhead)