# Task 6: Implement rate limiting middleware

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 6.

## Goal

Add rate limiting using Redis to enforce 100 req/min authenticated, 20 req/min anonymous

## Requirements

1. Create src/middleware/rate_limit.rs
2. Use sliding window algorithm with Redis sorted sets
3. Different limits based on authentication status
4. Return 429 status with retry-after header
5. Add rate limit headers to responses

## Acceptance Criteria

Verify rate limits enforced correctly for authenticated/anonymous users and proper headers returned

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-6): Implement rate limiting middleware`
