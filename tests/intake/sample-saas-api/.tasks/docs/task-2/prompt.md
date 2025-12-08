# Task 2: Implement authentication and authorization system

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 2.

## Goal

Build JWT-based auth with refresh tokens, OAuth2 integration, and role-based permissions with rate limiting

## Requirements

1. Create user model with roles (owner, admin, member, viewer)
2. Implement JWT token generation/validation with refresh tokens
3. Add OAuth2 flows for Google and GitHub using oauth2 crate
4. Create middleware for role-based authorization
5. Implement rate limiting (100 req/min auth, 20 req/min anon) using Redis
6. Add password hashing with argon2

```rust
#[derive(sqlx::Type, Clone)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
enum UserRole { Owner, Admin, Member, Viewer }

struct AuthMiddleware;
struct RateLimitMiddleware { redis: redis::Client }

// JWT claims structure
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    role: UserRole,
    exp: usize,
}
```

## Acceptance Criteria

Unit tests for JWT validation, integration tests for OAuth flows, rate limiting tests with Redis, authorization middleware tests for each role level

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-2): Implement authentication and authorization system`
