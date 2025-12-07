# Task 20: Performance optimization and final testing

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 20.

## Goal

Optimize for API response time < 100ms p95 and 1000 concurrent WebSocket connections

## Requirements

1. Add database query optimization and indexing
2. Implement connection pooling tuning
3. Add response caching for read-heavy endpoints
4. Optimize WebSocket connection handling
5. Add load testing with k6 or similar
6. Profile and optimize critical paths

## Acceptance Criteria

Load test API endpoints for p95 < 100ms, test 1000 concurrent WebSocket connections, verify performance metrics

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-20): Performance optimization and final testing`
