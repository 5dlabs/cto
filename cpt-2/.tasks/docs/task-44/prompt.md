# Task 44: Implement graceful shutdown

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 44.

## Goal

Add graceful shutdown handling for the authentication service

## Requirements

1. Handle SIGTERM and SIGINT signals
2. Close database connections gracefully
3. Close Redis connections properly
4. Complete in-flight requests before shutdown
5. Setup shutdown timeout handling

## Acceptance Criteria

Test graceful shutdown doesn't lose data or connections

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-44): Implement graceful shutdown`
