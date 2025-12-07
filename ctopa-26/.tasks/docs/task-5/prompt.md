# Task 5: Implement JWT authentication system

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 5.

## Goal

Create JWT-based authentication with access and refresh tokens

## Requirements

1. Add jsonwebtoken and bcrypt dependencies
2. Create auth.rs module with JWT token generation/validation
3. Implement AuthService with login, refresh, logout methods
4. Create middleware for JWT validation using axum::middleware
5. Store refresh tokens in Redis with 7-day expiration
6. Implement password hashing with bcrypt
7. Add JWT secret configuration from environment

## Acceptance Criteria

Unit tests for token generation/validation and integration tests for auth flow including token refresh

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-5): Implement JWT authentication system`
