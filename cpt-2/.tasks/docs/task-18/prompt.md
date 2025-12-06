# Task 18: Implement password reset confirmation

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 18.

## Goal

Create password reset confirmation with new password setting

## Requirements

1. Create POST /auth/reset-password endpoint
2. Validate reset token and check expiration
3. Validate new password strength requirements
4. Hash new password and update user record
5. Clear reset token after successful reset
6. Invalidate all existing sessions for the user

## Acceptance Criteria

Test password reset flow, token validation, and session invalidation

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-18): Implement password reset confirmation`
