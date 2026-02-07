# Task 8: Setup Redis integration for caching and rate limiting

## Priority
high

## Description
Integrate Redis client for rate limiting, deduplication cache, and session management

## Dependencies
- Task 7

## Implementation Details
Add redis crate, implement connection pool, create rate limiting middleware using sliding window, implement notification deduplication with TTL.

## Acceptance Criteria
Redis connection established, rate limiting works correctly, deduplication prevents duplicate notifications within TTL window

## Decision Points
- **d8** [performance]: Rate limiting algorithm choice

## Subtasks
- 1. Setup Redis dependencies and connection pool [implementer]
- 2. Implement rate limiting middleware with sliding window [implementer]
- 3. Implement notification deduplication cache with TTL [implementer]
- 4. Review Redis integration architecture and code quality [reviewer]
