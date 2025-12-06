# Task 1: Initialize Rust project with Axum and core dependencies

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 1.

## Goal

Set up the Rust project structure with Axum 0.7, sqlx for PostgreSQL, Redis client, and configure the workspace with proper directory layout

## Requirements

1. Run `cargo init --name teamsync-api`
2. Add dependencies to Cargo.toml:
   - axum = "0.7"
   - tokio = { version = "1", features = ["full"] }
   - sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "migrate"] }
   - redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
   - tower = "0.4"
   - tower-http = { version = "0.5", features = ["trace", "cors"] }
   - serde = { version = "1.0", features = ["derive"] }
   - serde_json = "1.0"
3. Create directory structure: src/{api, domain, infra}/
4. Set up main.rs with basic Axum server skeleton
5. Create .env.example with DATABASE_URL, REDIS_URL placeholders

## Acceptance Criteria

Run `cargo build` and `cargo test` to ensure compilation succeeds. Start server with `cargo run` and verify it listens on port 3000

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-1): Initialize Rust project with Axum and core dependencies`
