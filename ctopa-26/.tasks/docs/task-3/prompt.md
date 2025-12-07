# Task 3: Implement database connection and models

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 3.

## Goal

Setup PostgreSQL connection pool and define Rust models for database entities

## Requirements

1. Create database.rs module with connection pool setup using sqlx::PgPool
2. Define models in models/ directory: User, Team, TeamMember, Task structs with sqlx derives
3. Implement From traits for converting between database rows and models
4. Add database configuration from environment variables (DATABASE_URL)
5. Create database repository pattern with traits for each entity

## Acceptance Criteria

Unit tests for model serialization/deserialization and integration tests for database connectivity

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-3): Implement database connection and models`
