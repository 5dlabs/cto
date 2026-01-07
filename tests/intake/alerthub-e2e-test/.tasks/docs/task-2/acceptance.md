# Acceptance Criteria: Task 2

- [ ] Build the high-performance core service that receives, validates, and routes notifications with rate limiting and WebSocket support
- [ ] 1. Unit tests for models, validation, rate limiting logic
2. Integration tests with testcontainers for PostgreSQL and Redis
3. API tests for all endpoints using reqwest
4. WebSocket tests for real-time updates
5. Load test with 10,000 req/min using criterion
6. Verify Kafka events are published correctly
7. Test rate limiting behavior (429 responses)
8. Test deduplication (identical notifications within TTL)
9. Verify metrics endpoint returns valid Prometheus format
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 2.1: Initialize Rust project with dependencies and project structure
- [ ] 2.2: Define data models, enums, and validation schemas
- [ ] 2.3: Implement database layer with sqlx and PostgreSQL migrations
- [ ] 2.4: Implement Redis integration for rate limiting, caching, and pub/sub
- [ ] 2.5: Implement Kafka producer for event publishing
- [ ] 2.6: Build Axum router with REST API endpoints
- [ ] 2.7: Implement WebSocket handler with connection management and pub/sub
- [ ] 2.8: Implement middleware stack for auth, logging, and metrics
- [ ] 2.9: Create optimized multi-stage Dockerfile with security hardening
- [ ] 2.10: Develop comprehensive unit and integration test suite
