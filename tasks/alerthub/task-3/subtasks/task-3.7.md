# Subtask 3.7: Implement Rate Limiting

## Parent Task
Task 3

## Agent
code-implementer

## Parallelizable
Yes

## Description
Add rate limiting middleware for API protection.

## Details
- Implement token bucket algorithm
- Configure limits per endpoint
- Add rate limit headers to responses
- Handle rate limit exceeded gracefully
- Store counters in Redis

## Deliverables
- `src/middleware/rate_limit.rs` - Rate limiting logic
- `src/config/rate_limit.rs` - Configuration

## Acceptance Criteria
- [ ] Rate limits are enforced
- [ ] Headers show remaining requests
- [ ] 429 returned when exceeded
