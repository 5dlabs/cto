# Task 11: Implement rate limiting middleware

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 11.

## Goal

Create rate limiting system using Redis with different limits for authenticated vs anonymous users

## Requirements

1. Create rate limiting middleware using Redis sliding window algorithm
2. Implement different limits: 100 req/min for authenticated, 20 req/min for anonymous
3. Use IP address for anonymous users, user_id for authenticated users
4. Add rate limit headers in responses (X-RateLimit-Limit, X-RateLimit-Remaining)
5. Return 429 status with retry-after header when limit exceeded

## Acceptance Criteria

Test rate limiting for both user types, verify header inclusion, validate limit reset behavior

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-11): Implement rate limiting middleware`
