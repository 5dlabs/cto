# Implementation Prompt for Task 2

## Context
You are implementing "Notification Router Core Service (Rex - Rust/Axum)" for the AlertHub notification platform.

## PRD Reference
See `../../prd.md` for full requirements.

## Task Requirements
Build the high-performance notification routing service that validates, processes, and routes notifications with rate limiting and priority queues.

## Implementation Details
Implement Axum web server with endpoints for notification submission, batch processing, status queries, and WebSocket support. Add PostgreSQL integration with sqlx, Redis rate limiting, Kafka event publishing, and Prometheus metrics. Include priority queue processing and deduplication logic.

## Dependencies
This task depends on: task-1. Ensure those are complete before starting.

## Testing Requirements
API endpoints return correct responses, rate limiting blocks excess requests, notifications are persisted to PostgreSQL, events are published to Kafka, WebSocket connections receive real-time updates, and /metrics endpoint returns valid Prometheus format

## Decision Points to Address

The following decisions need to be made during implementation:

### d3: Dead letter queue implementation strategy
**Category**: error-handling | **Constraint**: open

Options:
1. Redis-based dead letter queue
2. Kafka dead letter topic
3. PostgreSQL table for failed notifications

Document your choice and rationale in the implementation.

### d4: Deduplication TTL configuration
**Category**: performance | **Constraint**: soft

Options:
1. 1 hour default TTL
2. configurable per tenant
3. 24 hour fixed TTL

Document your choice and rationale in the implementation.


## Deliverables
1. Source code implementing the requirements
2. Unit tests with >80% coverage
3. Integration tests for external interfaces
4. Documentation updates as needed
5. Decision point resolutions documented

## Notes
- Follow project coding standards
- Use Effect TypeScript patterns where applicable
- Ensure proper error handling and logging
