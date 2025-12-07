# Task 13: Implement health check endpoints

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 13.

## Goal

Create liveness and readiness health check endpoints for Kubernetes

## Requirements

1. Create health.rs module with health check handlers
2. Implement GET /health/live - basic liveness check (always returns 200)
3. Implement GET /health/ready - readiness check (database and Redis connectivity)
4. Add dependency health checks with timeout handling
5. Return detailed health status in JSON format
6. Add graceful shutdown handling for health checks

## Acceptance Criteria

Unit tests for health check logic and integration tests with database/Redis failures

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-13): Implement health check endpoints`
