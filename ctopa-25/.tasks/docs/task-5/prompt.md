# Task 5: Implement JWT authentication system

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 5.

## Goal

Create JWT-based authentication with access and refresh tokens

## Requirements

1. Add jsonwebtoken and bcrypt dependencies
2. Create src/auth/ module with JWT generation/validation
3. Implement AuthMiddleware for protected routes
4. Create login/logout endpoints
5. Add refresh token rotation mechanism
6. Store refresh tokens in Redis with expiration

## Acceptance Criteria

Test token generation, validation, refresh flow, and middleware protection

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-5): Implement JWT authentication system`
