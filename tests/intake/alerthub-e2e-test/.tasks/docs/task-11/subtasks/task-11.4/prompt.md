# Subtask task-11.4: Implement Axum Middleware for Automatic Rate Limiting

## Parent Task
Task 11

## Subagent Type
implementer

## Parallelizable
No - must wait for dependencies

## Description
Create Axum middleware that automatically applies rate limiting to routes based on tenant and endpoint configuration.

## Dependencies
- Subtask 8.2
- Subtask 8.3

## Implementation Details
Build Axum middleware that extracts tenant ID and endpoint information from requests, applies appropriate rate limits using the RateLimiter, returns proper HTTP status codes (429 Too Many Requests), and includes rate limit headers in responses. Support configurable bypass rules and custom error responses.

## Test Strategy
Integration tests with mock Axum applications

---
*Project: alerthub*
