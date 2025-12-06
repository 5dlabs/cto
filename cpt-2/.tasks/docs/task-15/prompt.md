# Task 15: Implement token refresh endpoint

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 15.

## Goal

Create endpoint to refresh access tokens using refresh tokens

## Requirements

1. Create POST /auth/refresh endpoint
2. Validate refresh token from request body or cookies
3. Check if refresh token exists in Redis
4. Generate new access token with updated expiration
5. Optionally rotate refresh token for security

## Acceptance Criteria

Test token refresh flow, validation, and new token generation

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-15): Implement token refresh endpoint`
