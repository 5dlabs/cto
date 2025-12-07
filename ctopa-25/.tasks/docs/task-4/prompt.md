# Task 4: Create core application structure and routing

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 4.

## Goal

Setup Axum router, middleware stack, and application state management

## Requirements

1. Create src/app.rs with AppState struct containing DB and Redis pools
2. Setup router with /api prefix and versioning
3. Add CORS, request tracing, and error handling middleware
4. Create src/handlers/ directory structure
5. Implement graceful shutdown handling

## Acceptance Criteria

Verify routing works, middleware executes in order, and graceful shutdown functions

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-4): Create core application structure and routing`
