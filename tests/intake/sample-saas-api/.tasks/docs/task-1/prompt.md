# Task 1: Setup project foundation and database infrastructure

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 1.

## Goal

Initialize Rust/Axum project with PostgreSQL and Redis integration, establishing core infrastructure for the TeamSync API

## Requirements

1. Create Cargo workspace with axum 0.7, sqlx, redis dependencies
2. Setup PostgreSQL connection pool with sqlx migrations
3. Configure Redis client for sessions and rate limiting
4. Implement health check endpoints (/health/live, /health/ready)
5. Add structured JSON logging with tracing and trace IDs
6. Create Docker multi-stage build with Rust 1.75+
7. Setup basic error handling and middleware stack

```rust
// Main structure
struct AppState {
    db: PgPool,
    redis: redis::Client,
}

// Health checks
async fn liveness() -> StatusCode { StatusCode::OK }
async fn readiness(State(state): State<AppState>) -> Result<StatusCode> {
    // Check DB and Redis connectivity
}
```

## Acceptance Criteria

Unit tests for health endpoints, integration tests for DB/Redis connectivity, verify Docker build completes successfully and container starts

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-1): Setup project foundation and database infrastructure`
