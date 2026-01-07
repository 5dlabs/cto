# Task 2: Implement Notification Router Service (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 2.

## Goal

Build the high-performance core service that receives, validates, and routes notifications with rate limiting and WebSocket support

## Requirements

1. Initialize Rust project with Axum 0.7, tokio, sqlx, redis:
   cargo init notification-router
   cargo add axum@0.7 tokio@1 sqlx@0.7 redis@0.24 serde@1 serde_json@1 uuid@1 chrono@0.4 validator@0.18 tracing@0.1 prometheus@0.13

2. Define data models in src/models.rs:
   struct Notification { id: Uuid, tenant_id: Uuid, channel: Channel, priority: Priority, payload: NotificationPayload, metadata: HashMap<String, Value>, created_at: DateTime<Utc>, status: NotificationStatus }
   enum Channel { Slack, Discord, Email, Push, Webhook }
   enum Priority { Critical, High, Normal, Low }
   enum NotificationStatus { Pending, Processing, Delivered, Failed { reason: String, attempts: u32 } }

3. Implement database layer with sqlx:
   - Create migration for notifications table
   - Implement NotificationRepository with CRUD operations
   - Add connection pool initialization

4. Implement Redis integration:
   - Rate limiter using token bucket algorithm (redis INCR with TTL)
   - Deduplication cache using notification hash as key
   - Pub/sub for WebSocket broadcasting

5. Build Axum router with endpoints:
   POST /api/v1/notifications - Submit notification (validate, rate limit, dedupe, insert DB, publish Kafka)
   POST /api/v1/notifications/batch - Batch submit (max 100, parallel processing)
   GET /api/v1/notifications/:id - Get status (query DB)
   GET /api/v1/notifications/:id/events - Get delivery events (query DB)
   WS /api/v1/ws - WebSocket connection (subscribe to Redis pub/sub)
   GET /health/live - Liveness check
   GET /health/ready - Readiness check (test DB, Redis, Kafka)
   GET /metrics - Prometheus metrics

6. Implement Kafka producer:
   - Use rdkafka crate
   - Publish to alerthub.notifications.created topic
   - Include notification ID, tenant ID, channel, priority in event

7. Implement WebSocket handler:
   - Maintain connection map (tenant_id -> Vec<WebSocket>)
   - Subscribe to Redis pub/sub channel per tenant
   - Broadcast updates to connected clients

8. Add middleware:
   - JWT authentication (extract tenant_id from token)
   - Request ID generation and propagation
   - Structured logging with tracing
   - Prometheus metrics collection

9. Create Dockerfile:
   FROM rust:1.75 AS builder
   WORKDIR /app
   COPY . .
   RUN cargo build --release
   FROM debian:bookworm-slim
   COPY --from=builder /app/target/release/notification-router /usr/local/bin/
   CMD ["notification-router"]

## Acceptance Criteria

1. Unit tests for models, validation, rate limiting logic
2. Integration tests with testcontainers for PostgreSQL and Redis
3. API tests for all endpoints using reqwest
4. WebSocket tests for real-time updates
5. Load test with 10,000 req/min using criterion
6. Verify Kafka events are published correctly
7. Test rate limiting behavior (429 responses)
8. Test deduplication (identical notifications within TTL)
9. Verify metrics endpoint returns valid Prometheus format

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-2): Implement Notification Router Service (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
