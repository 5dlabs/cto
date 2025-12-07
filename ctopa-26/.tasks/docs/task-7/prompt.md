# Task 7: Implement rate limiting middleware

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 7.

## Goal

Create rate limiting middleware using Redis for API protection

## Requirements

1. Create rate_limit.rs module with sliding window algorithm
2. Implement middleware that checks Redis counters per IP/user
3. Configure limits: 100 req/min for authenticated users, 20 req/min for anonymous
4. Use Redis EXPIRE and INCR commands for efficient counting
5. Return 429 Too Many Requests with Retry-After header
6. Add rate limit headers to all responses (X-RateLimit-Limit, X-RateLimit-Remaining)

## Acceptance Criteria

Unit tests for rate limiting logic and load tests to verify limits are enforced correctly

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-7): Implement rate limiting middleware`
