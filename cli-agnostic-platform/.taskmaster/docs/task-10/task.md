# Task 10: Create Controller API and Monitoring Infrastructure

## Overview
Implement REST API endpoints for CLI operations, migration tools, and comprehensive observability with Prometheus metrics and Grafana dashboards.

## Technical Specification

### 1. REST API Endpoints
```rust
// Using Axum v0.7.0 web framework
pub struct ApiServer {
    state: ApiState,
    metrics: PrometheusMetrics,
    tracer: Tracer,
}

// Key endpoints:
// GET /agents/{name}/cli-options - Available CLIs
// POST /agents/{name}/migrate - Initiate migration
// GET /agents/{name}/cli-config - Resolved configuration
// POST /agents/{name}/validate - Test configuration
// GET /health - Liveness probe
// GET /ready - Readiness probe
```

### 2. Prometheus Metrics
```rust
pub struct CliMetrics {
    requests_total: CounterVec,
    response_time_seconds: HistogramVec,
    errors_total: CounterVec,
    tokens_used: CounterVec,
    active_sessions: GaugeVec,
}
```

### 3. Observability Stack
- OpenTelemetry distributed tracing
- Structured logging with tracing-subscriber
- Grafana dashboards for CLI comparison
- Jaeger for trace visualization
- Custom SLO alerting rules

## Success Criteria
- API handles 1000+ requests/second
- Prometheus metrics track all operations
- Grafana dashboards provide CLI insights
- Distributed traces span service boundaries
- OpenAPI documentation auto-generated
- SLO alerts trigger appropriately