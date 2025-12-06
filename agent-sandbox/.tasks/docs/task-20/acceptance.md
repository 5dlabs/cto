# Acceptance Criteria: Task 20

- [ ] Create token bucket rate limiter using Redis to enforce 100 req/min for authenticated users and 20 req/min for anonymous
- [ ] Load test with >100 requests in 60s, verify 429 responses, test both authenticated and anonymous limits, verify Redis keys expire correctly
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 20.1: Implement core Redis-based rate limiter with token bucket algorithm
- [ ] 20.2: Create rate limiting middleware with user/IP identification
- [ ] 20.3: Implement rate limit headers and 429 response handling
- [ ] 20.4: Integrate middleware into Axum router with comprehensive load testing
