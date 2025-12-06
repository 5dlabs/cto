# Task 31: Implement CSRF protection

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 31.

## Goal

Add CSRF protection for state-changing authentication operations

## Requirements

1. Install and configure csurf middleware
2. Generate CSRF tokens for forms
3. Validate CSRF tokens on POST requests
4. Handle CSRF token errors gracefully
5. Exclude OAuth callback routes from CSRF validation

## Acceptance Criteria

Test CSRF protection blocks unauthorized requests and allows valid ones

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-31): Implement CSRF protection`
