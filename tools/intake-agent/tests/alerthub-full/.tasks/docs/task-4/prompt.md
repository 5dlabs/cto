# Task 4: Create Rust notification router service skeleton

## Priority
high

## Description
Initialize Rust project with Axum framework, basic routing, and project structure

## Dependencies
- Task 1

## Implementation Details
Setup Cargo.toml with Axum 0.7, tokio, sqlx dependencies. Create basic project structure with main.rs, routes, handlers, and models modules.

## Acceptance Criteria
Service compiles successfully, basic health endpoint responds, Docker image builds

## Decision Points
- **d4** [architecture]: Database ORM choice for Rust service

## Subtasks
- 1. Initialize Rust project with Cargo.toml and dependencies [implementer]
- 2. Create main.rs with Axum server setup [implementer]
- 3. Create modular project structure with routes, handlers, and models [implementer]
- 4. Review project structure and code quality [reviewer]
