# Acceptance Criteria: Task 6

- [ ] Add rate limiting using Redis to enforce 100 req/min authenticated, 20 req/min anonymous
- [ ] Verify rate limits enforced correctly for authenticated/anonymous users and proper headers returned
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 6.1: Implement sliding window algorithm with Redis
- [ ] 6.2: Create rate limiting middleware with different limits
- [ ] 6.3: Add proper HTTP headers and error responses
