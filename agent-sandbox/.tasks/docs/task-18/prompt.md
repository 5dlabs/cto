# Task 18: Implement JWT authentication with refresh token mechanism

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 18.

## Goal

Build JWT-based authentication system with access and refresh tokens, including token generation, validation, and rotation

## Requirements

1. Add dependencies: jsonwebtoken = "9", argon2 = "0.5"
2. Create domain/auth.rs with:
   - struct Claims { sub: Uuid, exp: i64, role: String }
   - fn generate_access_token(user_id: Uuid, role: String) -> Result<String> (15 min expiry)
   - fn generate_refresh_token(user_id: Uuid) -> Result<String> (7 day expiry)
   - fn validate_token(token: &str) -> Result<Claims>
3. Store refresh tokens in Redis with SET refresh:{user_id} {token} EX 604800
4. Create api/auth.rs handlers:
   - POST /api/auth/register (email, password)
   - POST /api/auth/login (email, password) -> returns {access_token, refresh_token}
   - POST /api/auth/refresh (refresh_token) -> returns new access_token
5. Implement Axum middleware for JWT validation in api/middleware/auth.rs
6. Hash passwords with argon2 before storage

## Acceptance Criteria

Unit tests for token generation/validation, integration tests for register/login flow, verify expired tokens rejected, test refresh token rotation

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-18): Implement JWT authentication with refresh token mechanism`
