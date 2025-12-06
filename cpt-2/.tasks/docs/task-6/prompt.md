# Task 6: Setup rate limiting middleware

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 6.

## Goal

Implement rate limiting for authentication endpoints to prevent abuse

## Requirements

1. Configure express-rate-limit with Redis store
2. Create different limits: login (5 attempts/15min), signup (3 attempts/hour), password reset (3 attempts/hour)
3. Implement IP-based and user-based rate limiting
4. Add custom error messages for rate limit exceeded
5. Setup rate limit headers in responses

## Acceptance Criteria

Test rate limiting triggers correctly and resets after time window

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-6): Setup rate limiting middleware`
