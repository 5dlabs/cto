# Task 1: Setup Rust project foundation with Axum

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 1.

## Goal

Initialize the Rust project with Axum framework, PostgreSQL, Redis, and essential dependencies

## Requirements

1. cargo init --name teamsync-api
2. Add dependencies: axum = "0.7", tokio, sqlx with postgres feature, redis, serde, uuid
3. Setup Cargo.toml with workspace structure
4. Create src/main.rs with basic Axum server
5. Add .env.example and .gitignore
6. Configure tracing and structured logging

## Acceptance Criteria

Verify project compiles and basic HTTP server starts on localhost:3000

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-1): Setup Rust project foundation with Axum`
