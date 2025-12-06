# Task 16: Initialize Rust/Axum project with PostgreSQL and Redis integration

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 16.

## Goal

Set up the foundational Rust project structure with Axum 0.7, configure sqlx for PostgreSQL 15, and establish Redis 7 connections for caching and pub/sub

## Requirements

1. Initialize Cargo project with workspace structure
2. Add dependencies: axum = "0.7", tokio = { version = "1", features = ["full"] }, sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "migrate"] }, redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
3. Create directory structure: src/{api, domain, infra}, frontend/, infra/
4. Configure .env with DATABASE_URL, REDIS_URL
5. Setup sqlx migrations directory
6. Create main.rs with basic Axum server scaffold
7. Implement Redis connection pool in infra/redis.rs
8. Implement PostgreSQL connection pool in infra/database.rs

## Acceptance Criteria

Verify cargo build succeeds, server starts on configured port, PostgreSQL connection pool initializes, Redis ping succeeds

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-16): Initialize Rust/Axum project with PostgreSQL and Redis integration`
