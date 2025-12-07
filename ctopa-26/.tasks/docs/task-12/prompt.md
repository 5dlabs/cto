# Task 12: Add structured logging and observability

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 12.

## Goal

Implement structured JSON logging with trace IDs and Prometheus metrics

## Requirements

1. Add tracing and tracing-subscriber dependencies
2. Configure JSON logging with request trace IDs using tracing spans
3. Add metrics collection using prometheus crate
4. Implement custom metrics: request_duration, active_connections, task_operations
5. Create /metrics endpoint for Prometheus scraping
6. Add request/response logging middleware with sanitization
7. Configure log levels via environment variables

## Acceptance Criteria

Verify JSON log format and metrics endpoint functionality with sample requests

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-12): Add structured logging and observability`
