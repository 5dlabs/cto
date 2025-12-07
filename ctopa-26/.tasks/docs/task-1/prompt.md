# Task 1: Setup Rust project foundation and dependencies

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 1.

## Goal

Initialize the TeamSync API project with Rust/Axum foundation and core dependencies

## Requirements

1. Create new Rust project with `cargo init`
2. Add dependencies to Cargo.toml: axum = "0.7", tokio = { version = "1.0", features = ["full"] }, serde = { version = "1.0", features = ["derive"] }, sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid"] }, redis = "0.24", uuid = { version = "1.0", features = ["v4"] }, chrono = { version = "0.4", features = ["serde"] }
3. Setup basic main.rs with axum server skeleton
4. Configure workspace structure with separate modules for handlers, models, database

## Acceptance Criteria

Verify project compiles successfully with `cargo build` and basic server starts on localhost

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-1): Setup Rust project foundation and dependencies`
