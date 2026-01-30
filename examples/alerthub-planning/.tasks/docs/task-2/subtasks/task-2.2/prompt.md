# Subtask 2.2: Implement core notification processing and rate limiting

**Parent Task:** Implement Notification Router Service (Rex - Rust/Axum)
**Agent:** rex | **Language:** rust

## Description

Build the main notification endpoint with validation, rate limiting using Redis, and deduplication logic to handle incoming notification requests efficiently.

## Details

Create POST /api/v1/notifications endpoint with comprehensive input validation, implement Redis-based rate limiting middleware with configurable limits per user/IP, add deduplication logic using Redis with configurable TTL to prevent duplicate notifications, implement request parsing and validation using serde with proper error responses.

## Dependencies

task-2.1

## Acceptance Criteria

- [ ] Subtask requirements implemented
- [ ] Parent task requirements still satisfied

## Resources

- Parent task: `.tasks/docs/task-2/prompt.md`
- PRD: `.tasks/docs/prd.md`
