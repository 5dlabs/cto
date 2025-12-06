# Task 3: Implement JWT authentication system with refresh tokens

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 3.

## Goal

Build JWT-based authentication with access and refresh token generation, validation middleware, and secure token storage in Redis

## Requirements

1. Add dependencies: jsonwebtoken = "9", argon2 = "0.5", uuid = { version = "1", features = ["v4", "serde"] }
2. Create src/domain/auth.rs with:
   - struct Claims { sub: Uuid, exp: i64, role: String }
   - fn generate_access_token(user_id: Uuid, role: String) -> Result<String> (15min expiry)
   - fn generate_refresh_token() -> String (7 days, store in Redis with user_id)
3. Implement password hashing with argon2
4. Create POST /api/auth/register and /api/auth/login endpoints
5. Build Axum middleware in src/api/middleware/auth.rs:
   - Extract JWT from Authorization header
   - Validate signature and expiry
   - Inject user context into request extensions
6. Store refresh tokens in Redis with key pattern: refresh_token:{token} -> user_id

## Acceptance Criteria

Unit test token generation/validation. Integration test: register user, login, access protected endpoint with token, refresh token flow. Test expired token rejection

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-3): Implement JWT authentication system with refresh tokens`
