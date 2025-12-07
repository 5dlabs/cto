# Task 1: Initialize Rust project with Axum foundation

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 1.

## Goal

Set up the basic project structure with Axum web framework, database connections, and essential middleware

## Requirements

1. Create new Rust project with Cargo.toml dependencies: axum 0.7, tokio, sqlx, redis, serde, uuid
2. Set up main.rs with basic Axum router and server
3. Configure environment variables for database and Redis URLs
4. Add basic middleware for CORS, logging, and request tracing
5. Create modular project structure: src/{handlers, models, services, middleware, config}

## Acceptance Criteria

Verify server starts successfully on configured port, responds to health check endpoint, and logs are properly structured

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-1): Initialize Rust project with Axum foundation`
