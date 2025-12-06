# Task 16: Setup Rust/Axum project foundation with PostgreSQL and Redis

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 16.

## Goal

Initialize the Rust project with Axum 0.7, configure PostgreSQL 15 with sqlx, and set up Redis 7 connections. Establish the directory structure following the architecture specification.

## Requirements

1. Run `cargo init --name teamsync-api`
2. Add dependencies to Cargo.toml:
   - axum = "0.7"
   - tokio = { version = "1", features = ["full"] }
   - sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "migrate"] }
   - redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
   - tower = "0.4"
   - tower-http = { version = "0.5", features = ["trace", "cors"] }
3. Create directory structure: src/{api, domain, infra}
4. Create config.rs for environment variables (DATABASE_URL, REDIS_URL)
5. Implement connection pools in infra/db.rs and infra/redis.rs
6. Create main.rs with basic Axum router and health check endpoint
7. Add .env.example with required environment variables

## Acceptance Criteria

Verify cargo build succeeds, health check endpoint returns 200, database and Redis connections establish successfully with docker-compose up

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-16): Setup Rust/Axum project foundation with PostgreSQL and Redis`
