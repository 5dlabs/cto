# Task 2: Notification Router Core Service (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Rust Engineer specializing in high-performance systems implementing Task 2.

## Goal

Build the high-performance notification routing service that validates, processes, and routes notifications with rate limiting and priority queues.

## Requirements

Implement Axum web server with endpoints for notification submission, batch processing, status queries, and WebSocket support. Add PostgreSQL integration with sqlx, Redis rate limiting, Kafka event publishing, and Prometheus metrics. Include priority queue processing and deduplication logic.

## Acceptance Criteria

API endpoints return correct responses, rate limiting blocks excess requests, notifications are persisted to PostgreSQL, events are published to Kafka, WebSocket connections receive real-time updates, and /metrics endpoint returns valid Prometheus format

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-2): Notification Router Core Service (Rex - Rust/Axum)`

## Decision Points

### d3: Dead letter queue implementation strategy
**Category**: error-handling | **Constraint**: open

Options:
1. Redis-based dead letter queue
2. Kafka dead letter topic
3. PostgreSQL table for failed notifications

### d4: Deduplication TTL configuration
**Category**: performance | **Constraint**: soft

Options:
1. 1 hour default TTL
2. configurable per tenant
3. 24 hour fixed TTL


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-1
