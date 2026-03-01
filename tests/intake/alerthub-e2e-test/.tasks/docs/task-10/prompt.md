# Task 10: Database Models and Connections (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 10.

## Goal

Implement PostgreSQL connection pool and data models for notifications

## Requirements

1. Add sqlx with PostgreSQL driver\n2. Create connection pool configuration\n3. Define Notification, User, Tenant structs\n4. Implement database migrations\n5. Add connection health checks

## Acceptance Criteria

Database connections successful, migrations run without errors, can insert/query test notification records

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-10): Database Models and Connections (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 9
