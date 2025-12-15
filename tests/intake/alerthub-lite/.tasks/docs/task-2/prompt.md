# Task 2: Implement Notification Service Backend (Rust/Axum)

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 2.

## Goal

Build the core notification service with REST API endpoints, database integration, and caching layer

## Requirements

1. Initialize Rust project with Axum 0.7, sqlx, redis, tokio
2. Define Notification struct with Uuid, title, message, Priority enum, created_at, read fields
3. Implement database layer with PostgreSQL connection using sqlx
4. Add Redis caching for recent notifications
5. Create API routes: POST /api/v1/notifications, GET /api/v1/notifications (paginated), GET /api/v1/notifications/:id, DELETE /api/v1/notifications/:id, GET /health
6. Add rate limiting middleware (10 req/sec per IP)
7. Configure structured logging with tracing

## Acceptance Criteria

Unit tests for data models and business logic, integration tests for API endpoints, load testing for rate limiting, verify caching behavior

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-2): Implement Notification Service Backend (Rust/Axum)`
