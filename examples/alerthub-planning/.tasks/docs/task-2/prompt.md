# Task 2: Implement Notification Router Service (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Rust Engineer specializing in high-performance systems implementing Task 2.

## Goal

Build the high-performance core notification routing service in Rust using Axum. This service receives notifications, applies rate limiting, handles deduplication, and routes messages to the integration service via Kafka.

## Requirements

1. Set up Axum application structure with middleware
2. Implement PostgreSQL connection pool with sqlx
3. Create notification data models and database schema
4. Build POST /api/v1/notifications endpoint with validation
5. Implement rate limiting using Redis
6. Add deduplication logic with configurable TTL
7. Set up Kafka producer for routing to integration service
8. Create priority queue processing (critical, high, normal, low)
9. Add WebSocket endpoint for real-time updates
10. Implement health checks and Prometheus metrics

## Acceptance Criteria

Service starts successfully, all endpoints return expected responses, notifications are persisted to PostgreSQL, rate limiting blocks excessive requests, messages are published to Kafka, WebSocket connections work, and health/metrics endpoints are accessible.

## Constraints

- Match existing codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-2): Implement Notification Router Service (Rex - Rust/Axum)`

## Decision Points

### d3: How should we handle database connection failures and recovery?
**Category**: error-handling | **Constraint**: open

Options:
1. fail-fast
2. circuit-breaker
3. retry-with-backoff

### d4: What should be the default rate limit per tenant?
**Category**: performance | **Constraint**: soft | ⚠️ **Requires Approval**

Options:
1. 100-per-minute
2. 1000-per-minute
3. configurable-per-tenant


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-1
