# Task 20: Implement Redis-based rate limiting middleware

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 20.

## Goal

Create token bucket rate limiter using Redis to enforce 100 req/min for authenticated users and 20 req/min for anonymous

## Requirements

1. Create infra/rate_limiter.rs with:
   - struct RateLimiter { redis: ConnectionManager }
   - async fn check_rate_limit(key: &str, max_requests: u32, window_secs: u64) -> Result<bool>
2. Use Redis commands: INCR rate:{key}:{window}, EXPIRE if first request
3. Create api/middleware/rate_limit.rs:
   - Extract user_id from JWT or use IP address for anonymous
   - Apply limits: authenticated users 100/60s, anonymous 20/60s
   - Return 429 Too Many Requests if exceeded with Retry-After header
4. Integrate middleware into Axum router before auth middleware
5. Add rate limit headers to all responses: X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset

## Acceptance Criteria

Load test with >100 requests in 60s, verify 429 responses, test both authenticated and anonymous limits, verify Redis keys expire correctly

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-20): Implement Redis-based rate limiting middleware`
