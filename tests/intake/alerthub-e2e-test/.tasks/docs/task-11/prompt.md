# Task 11: Redis Integration and Rate Limiting (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 11.

## Goal

Implement Redis connection and rate limiting middleware for tenant isolation

## Requirements

1. Add redis crate and connection pool\n2. Implement rate limiting middleware with token bucket\n3. Add deduplication cache with TTL\n4. Configure Redis for pub/sub notifications\n5. Add Redis health checks

## Acceptance Criteria

Redis connections work, rate limiting blocks requests after limit, deduplication prevents duplicate notifications

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-11): Redis Integration and Rate Limiting (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 9
