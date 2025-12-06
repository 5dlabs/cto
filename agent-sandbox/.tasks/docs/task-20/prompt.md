# Task 20: Implement rate limiting with Redis token bucket algorithm

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 20.

## Goal

Create middleware for rate limiting using Redis-based token bucket: 100 req/min for authenticated users, 20 req/min for anonymous.

## Requirements

1. Create infra/rate_limit.rs with TokenBucket struct
2. Implement token bucket algorithm:
   - Key format: rate_limit:{user_id|ip}:{window}
   - Use Redis INCR with EXPIRE for atomic operations
   - Lua script for atomic check-and-decrement:
     ```lua
     local current = redis.call('GET', KEYS[1])
     if not current then
       redis.call('SET', KEYS[1], ARGV[1] - 1, 'EX', ARGV[2])
       return 1
     elseif tonumber(current) > 0 then
       redis.call('DECR', KEYS[1])
       return 1
     else
       return 0
     end
     ```
3. Create RateLimitLayer middleware:
   - Extract user_id from Claims or use IP address
   - Check bucket, decrement if available
   - Return 429 Too Many Requests with Retry-After header if exhausted
4. Apply different limits based on authentication status
5. Add X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset headers

## Acceptance Criteria

Unit tests for token bucket logic with mocked Redis. Integration tests: make 100 requests as authenticated user, verify 101st fails. Test anonymous limit of 20. Verify rate limit resets after window. Test concurrent requests don't exceed limit.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-20): Implement rate limiting with Redis token bucket algorithm`
