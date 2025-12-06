# Acceptance Criteria: Task 27

- [ ] Implement Prometheus metrics endpoint, structured JSON logging with trace IDs, and health check endpoints for Kubernetes.
- [ ] Unit tests for metrics recording. Integration tests: make requests, verify metrics incremented. Check /health/ready returns 503 when DB unavailable. Verify logs are valid JSON with trace_ids. Load test and verify metrics in Prometheus. Test log filtering with RUST_LOG.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 27.1: Setup Prometheus metrics infrastructure and register core metrics
- [ ] 27.2: Implement HTTP metrics collection middleware
- [ ] 27.3: Setup structured logging with tracing-subscriber and JSON formatting
- [ ] 27.4: Implement trace ID generation and propagation throughout request lifecycle
- [ ] 27.5: Implement health check endpoints for Kubernetes probes
- [ ] 27.6: Implement Prometheus metrics exposition endpoint and instrument critical code paths
