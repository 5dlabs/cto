# Task 16: Create health check endpoints

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 16.

## Goal

Implement liveness and readiness probes for Kubernetes deployment health monitoring

## Requirements

1. Create GET /health/live endpoint for liveness probe (basic server health)
2. Create GET /health/ready endpoint checking database and Redis connectivity
3. Add dependency health checks with timeout handling
4. Return appropriate HTTP status codes (200 healthy, 503 unhealthy)
5. Include health check details in response body for debugging

## Acceptance Criteria

Test health endpoints under normal and failure conditions, verify dependency checking works correctly

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-16): Create health check endpoints`
