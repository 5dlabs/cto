# Task 4: Setup Redis connection and caching layer

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 4.

## Goal

Configure Redis connection for session management and rate limiting

## Requirements

1. Add Redis client configuration in redis.rs module
2. Create connection pool with redis::aio::ConnectionManager
3. Implement session storage interface using Redis with TTL
4. Setup rate limiting storage with Redis counters and sliding window
5. Add Redis configuration from environment (REDIS_URL)

## Acceptance Criteria

Integration tests for Redis connectivity and basic set/get operations with TTL verification

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-4): Setup Redis connection and caching layer`
