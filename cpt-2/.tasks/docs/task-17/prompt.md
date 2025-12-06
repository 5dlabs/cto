# Task 17: Implement password reset request

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 17.

## Goal

Create password reset request functionality with email sending

## Requirements

1. Create POST /auth/forgot-password endpoint
2. Generate secure reset token with 1-hour expiration
3. Store reset token and expiration in user record
4. Send password reset email with reset link
5. Apply rate limiting to prevent abuse

## Acceptance Criteria

Test reset token generation, email sending, and rate limiting

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-17): Implement password reset request`
