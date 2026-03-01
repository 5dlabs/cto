# Task 9: Notification Router Core Service (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 9.

## Goal

Create Rust/Axum service foundation with basic HTTP server and health endpoints

## Requirements

1. Initialize Rust project with Axum 0.7\n2. Set up basic HTTP server with routes\n3. Implement health check endpoints (/health/live, /health/ready)\n4. Add structured logging with tracing\n5. Configure Prometheus metrics endpoint

## Acceptance Criteria

Service starts successfully, health endpoints return 200, metrics endpoint exposes basic HTTP metrics

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-9): Notification Router Core Service (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 2, 3
