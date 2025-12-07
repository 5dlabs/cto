# Task 3: Set up Redis for caching and session management

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 3.

## Goal

Configure Redis connection for rate limiting, session storage, and pub/sub notifications

## Requirements

1. Add redis crate dependency and configure connection pool
2. Create Redis service layer with methods for session management
3. Implement rate limiting storage using Redis with sliding window
4. Set up pub/sub channels for real-time notifications
5. Add Redis health check to startup sequence

## Acceptance Criteria

Verify Redis connectivity, test session storage/retrieval, validate rate limiting counters work correctly

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-3): Set up Redis for caching and session management`
