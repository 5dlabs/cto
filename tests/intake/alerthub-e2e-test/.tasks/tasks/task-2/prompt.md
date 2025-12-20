# Task 2: Notification Router Service (Rust/Axum)

## Agent: Rex
## Priority: High
## Language: Rust 1.75+
## Framework: Axum 0.7

## Objective
Build a high-performance notification routing service that receives, validates, and routes notifications.

## Endpoints
- `POST /api/v1/notifications` - Submit a new notification
- `POST /api/v1/notifications/batch` - Submit batch notifications (up to 100)
- `GET /api/v1/notifications/:id` - Get notification status
- `GET /api/v1/notifications/:id/events` - Get delivery events
- `WS /api/v1/ws` - WebSocket for real-time notification updates
- `GET /metrics` - Prometheus metrics
- `GET /health/live` - Liveness check
- `GET /health/ready` - Readiness check

## Core Features
- Rate limiting per tenant (configurable via Redis)
- Priority queue processing (critical, high, normal, low)
- Deduplication with configurable TTL
- Dead letter queue for failed deliveries
- Structured logging with tracing

## Dependencies
- PostgreSQL: Notification persistence, tenant data
- Redis: Rate limiting, deduplication cache
- Kafka: Event streaming to integration service

## Project Structure
```
services/notification-router/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs
│   │   └── handlers.rs
│   ├── models/
│   ├── services/
│   └── config.rs
└── tests/
```

## Acceptance Criteria
- [ ] All endpoints return correct responses
- [ ] Rate limiting works correctly
- [ ] WebSocket connections work
- [ ] Tests pass with `cargo test`
- [ ] Clippy passes with `cargo clippy -- -D warnings`

