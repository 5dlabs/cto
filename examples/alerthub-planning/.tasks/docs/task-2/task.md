# Task 2: Notification Router Core Service (Rex - Rust/Axum)

## Status
pending

## Priority
high

## Dependencies
task-1

## Description
Build the high-performance notification routing service that validates, processes, and routes notifications with rate limiting and priority queues.

## Details
Implement Axum web server with endpoints for notification submission, batch processing, status queries, and WebSocket support. Add PostgreSQL integration with sqlx, Redis rate limiting, Kafka event publishing, and Prometheus metrics. Include priority queue processing and deduplication logic.

## Test Strategy
API endpoints return correct responses, rate limiting blocks excess requests, notifications are persisted to PostgreSQL, events are published to Kafka, WebSocket connections receive real-time updates, and /metrics endpoint returns valid Prometheus format

## Decision Points

### d3: Dead letter queue implementation strategy
- **Category**: error-handling
- **Constraint**: open
- **Requires Approval**: No
- **Options**:
  - Redis-based dead letter queue
  - Kafka dead letter topic
  - PostgreSQL table for failed notifications

### d4: Deduplication TTL configuration
- **Category**: performance
- **Constraint**: soft
- **Requires Approval**: No
- **Options**:
  - 1 hour default TTL
  - configurable per tenant
  - 24 hour fixed TTL

