# Task 4: Set up Redis connection and rate limiting infrastructure

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 4.

## Goal

Configure Redis connection pool and implement token bucket rate limiting for authenticated and anonymous users

## Requirements

1. Create src/infra/redis.rs with connection manager setup
2. Implement rate limiter in src/api/middleware/rate_limit.rs:
   - Use Redis INCR with TTL for sliding window counter
   - Key pattern: rate_limit:{ip_or_user_id}:{minute_window}
   - Limits: 100 req/min for authenticated, 20 req/min for anonymous
   - Return 429 Too Many Requests when exceeded
3. Create middleware layer that:
   - Extracts user_id from auth context or falls back to IP
   - Checks current count in Redis
   - Increments counter with EXPIRE 60
   - Adds X-RateLimit-* headers to response
4. Apply middleware globally in router configuration

## Acceptance Criteria

Unit test rate limit logic with mock Redis. Integration test: make 21 anonymous requests rapidly, verify 429 on 21st. Test authenticated user gets 100 req/min limit

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-4): Set up Redis connection and rate limiting infrastructure`
