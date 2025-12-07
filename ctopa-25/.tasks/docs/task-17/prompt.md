# Task 17: Add observability and monitoring

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 17.

## Goal

Implement Prometheus metrics, structured logging, and health checks

## Requirements

1. Add prometheus and tracing-subscriber dependencies
2. Create src/metrics.rs with custom metrics
3. Add /metrics endpoint for Prometheus scraping
4. Implement structured JSON logging with trace IDs
5. Add health check endpoints: /health/live and /health/ready
6. Add request duration and error rate metrics

## Acceptance Criteria

Verify metrics collection, log formatting, and health check responses

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-17): Add observability and monitoring`
