# Acceptance Criteria: Task 20

- [ ] Create middleware for rate limiting using Redis-based token bucket: 100 req/min for authenticated users, 20 req/min for anonymous.
- [ ] Unit tests for token bucket logic with mocked Redis. Integration tests: make 100 requests as authenticated user, verify 101st fails. Test anonymous limit of 20. Verify rate limit resets after window. Test concurrent requests don't exceed limit.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 20.1: Implement Redis token bucket algorithm with Lua script
- [ ] 20.2: Implement Redis integration with key namespacing and connection management
- [ ] 20.3: Create Axum middleware layer for rate limit enforcement
- [ ] 20.4: Implement rate limit response headers injection
- [ ] 20.5: Implement differentiated rate limits based on authentication status
