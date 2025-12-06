# Task 7: Create authentication middleware

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 7.

## Goal

Implement middleware to verify JWT tokens and protect routes

## Requirements

1. Create authenticateToken middleware
2. Extract token from Authorization header
3. Verify token and attach user to request object
4. Handle token expiration and invalid tokens
5. Create optional authentication middleware for public routes

## Acceptance Criteria

Test middleware correctly authenticates valid tokens and rejects invalid ones

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-7): Create authentication middleware`
