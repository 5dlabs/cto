# Task 13: Implement user login with email/password

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 13.

## Goal

Create traditional email/password login functionality

## Requirements

1. Create POST /auth/login endpoint
2. Validate email and password inputs
3. Compare password hash using bcrypt
4. Generate access and refresh tokens on successful login
5. Store refresh token in Redis with user association
6. Apply rate limiting to login attempts

## Acceptance Criteria

Test successful login, password validation, token generation, and rate limiting

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-13): Implement user login with email/password`
