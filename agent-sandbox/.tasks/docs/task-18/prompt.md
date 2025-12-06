# Task 18: Implement JWT authentication with refresh token mechanism

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 18.

## Goal

Build JWT-based authentication system with access tokens (15min expiry) and refresh tokens (7 days), stored in Redis. Include token validation middleware.

## Requirements

1. Add dependencies: jsonwebtoken = "9", argon2 = "0.5"
2. Create domain/auth.rs with structs: Claims, TokenPair, RefreshToken
3. Implement JWT generation/validation functions:
   - generate_access_token(user_id, role) -> Result<String>
   - generate_refresh_token(user_id) -> Result<String>
   - validate_token(token: &str) -> Result<Claims>
4. Store refresh tokens in Redis with 7-day TTL: SET refresh:{token_id} {user_id} EX 604800
5. Create api/auth.rs with endpoints:
   - POST /api/auth/register (email, password)
   - POST /api/auth/login (email, password) -> returns TokenPair
   - POST /api/auth/refresh (refresh_token) -> returns new TokenPair
6. Implement Axum middleware in infra/middleware.rs:
   - RequireAuth extractor that validates Bearer token and injects Claims
7. Add JWT_SECRET and REFRESH_TOKEN_SECRET to config

## Acceptance Criteria

Unit tests for token generation/validation with valid/expired/invalid tokens. Integration tests: register user, login, use access token, refresh token after expiry, verify old refresh token invalidated. Test middleware rejects requests without valid token.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-18): Implement JWT authentication with refresh token mechanism`
