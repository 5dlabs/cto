# Acceptance Criteria: Task 17

- [ ] Build the high-performance core notification routing service in Rust using Axum. Handles notification submission, validation, rate limiting, priority queuing, and event streaming to Kafka. Includes WebSocket support for real-time updates and Prometheus metrics.
- [ ] 1. Unit tests for models, rate limiter, deduplication:

```rust
#[tokio::test]
async fn test_rate_limiter_allows_within_limit() {
    let limiter = RateLimiter::new();
    for _ in 0..1000 {
        assert!(limiter.check_rate_limit(Uuid::new_v4()).await.unwrap());
    }
}
```

2. Integration tests for API endpoints:

```rust
#[tokio::test]
async fn test_create_notification_success() {
    let app = create_test_app().await;
    let response = app.oneshot(Request::builder().method("POST").uri("/api/v1/notifications").body(Body::from(json!({"tenant_id": "...", "channel": "slack", ...}).to_string())).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

3. Test batch endpoint with 100 notifications
4. Test rate limiting by exceeding tenant limit
5. Test deduplication by submitting same notification twice
6. Test WebSocket connection and message streaming
7. Load test with k6: 10,000 req/min sustained for 5 minutes
8. Verify Kafka messages are published correctly
9. Check Prometheus metrics endpoint returns valid data
10. Test health checks return correct status
11. Deploy to Kubernetes and verify pod startup, readiness probes
12. Test cross-service communication with Integration Service consuming Kafka events
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 17.1: Initialize Rust Project and Core Dependencies
- [ ] 17.2: Implement Data Models and Database Layer with SQLx
- [ ] 17.3: Implement Redis Rate Limiting and Deduplication Cache
- [ ] 17.4: Implement Kafka Event Producer Integration
- [ ] 17.5: Implement Core Axum HTTP Routes and Business Logic
- [ ] 17.6: Implement WebSocket Handler with Redis Pub/Sub
- [ ] 17.7: Implement Observability Layer (Metrics, Health Checks, Tracing)
- [ ] 17.8: Create Containerization and Kubernetes Deployment Manifests
