# Task 2: Notification Router Service (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 2.

## Goal

Build the high-performance notification routing service with rate limiting, deduplication, and Kafka integration

## Requirements

Create Rust/Axum 0.7 service with endpoints: POST /api/v1/notifications, batch endpoint, GET status, WebSocket for real-time, health checks, and Prometheus metrics. Implement rate limiting via Redis, priority queues, deduplication with TTL, and dead letter queue.

## Acceptance Criteria

Unit tests, integration tests with testcontainers, load test for 10k notifications/minute

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-2): Notification Router Service (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
