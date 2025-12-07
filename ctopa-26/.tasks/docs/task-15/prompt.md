# Task 15: Implement Prometheus metrics endpoint

## Role

You are a Senior Mobile Engineer with expertise in React Native and cross-platform development implementing Task 15.

## Goal

Add application metrics collection and exposure for monitoring and alerting

## Requirements

1. Add prometheus and metrics dependencies
2. Create metrics registry with HTTP request duration, database query times
3. Add custom business metrics: active users, task completion rates
4. Implement /metrics endpoint for Prometheus scraping
5. Add metrics middleware to automatically track request patterns
6. Create health metrics for database and Redis connections

## Acceptance Criteria

Verify metrics endpoint returns Prometheus format, validate metric collection accuracy and performance impact

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-15): Implement Prometheus metrics endpoint`
