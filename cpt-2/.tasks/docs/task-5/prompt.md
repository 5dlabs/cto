# Task 5: Implement JWT token utilities

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 5.

## Goal

Create JWT token generation, validation, and refresh token management

## Requirements

1. Create generateAccessToken(payload) with 15min expiry
2. Create generateRefreshToken(payload) with 7day expiry
3. Implement verifyToken(token) with error handling
4. Add token blacklist functionality using Redis
5. Create refreshAccessToken(refreshToken) method

## Acceptance Criteria

Test token generation, validation, expiration, and refresh token flow

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-5): Implement JWT token utilities`
