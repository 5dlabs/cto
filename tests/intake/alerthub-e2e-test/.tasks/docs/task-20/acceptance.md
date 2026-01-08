# Acceptance Criteria: Task 20

- [ ] Build the high-performance core service that receives, validates, and routes notifications. Handles rate limiting, priority queuing, deduplication, and publishes events to Kafka for downstream processing. Provides WebSocket connections for real-time updates and Prometheus metrics for observability.
- [ ] 1. Unit tests for models, validation, and business logic:
- Test Channel and Priority enum serialization
- Test NotificationPayload validation
- Test rate limiter logic with mock Redis

2. Integration tests with test containers:
- Spin up PostgreSQL, Redis, Kafka containers
- Test POST /api/v1/notifications returns 202 with valid ID
- Test rate limiting blocks after threshold
- Test deduplication prevents duplicate notifications
- Test Kafka event publishing
- Test WebSocket connection and message delivery

3. Load testing with k6:
- Sustained 10,000 notifications/minute
- Verify p95 latency < 100ms
- Test concurrent WebSocket connections (1,000+)

4. End-to-end tests:
- Submit notification via API
- Verify record in PostgreSQL
- Verify event in Kafka topic
- Verify WebSocket clients receive update

5. Health check validation:
- GET /health/live returns 200
- GET /health/ready returns 200 when all dependencies available
- GET /metrics returns Prometheus format
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 20.1: Initialize Rust project with Cargo.toml and core dependencies
- [ ] 20.2: Implement data models with validation and serialization
- [ ] 20.3: Implement PostgreSQL database layer with connection pooling
- [ ] 20.4: Implement Redis-based rate limiting with sliding window
- [ ] 20.5: Implement Kafka producer with event publishing and retry logic
- [ ] 20.6: Implement deduplication logic with Redis-based fingerprinting
- [ ] 20.7: Implement Axum route handlers with middleware stack
- [ ] 20.8: Implement WebSocket real-time notification system
- [ ] 20.9: Implement Prometheus metrics instrumentation
- [ ] 20.10: Create main.rs with server initialization and graceful shutdown
- [ ] 20.11: Create multi-stage Dockerfile and Kubernetes manifests
