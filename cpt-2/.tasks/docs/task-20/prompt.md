# Task 20: Implement account linking functionality

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 20.

## Goal

Allow users to link multiple OAuth providers to single account

## Requirements

1. Create POST /auth/link/:provider endpoint
2. Check if user is already authenticated
3. Initiate OAuth flow for additional provider
4. Link new provider to existing user account
5. Handle conflicts when provider account already linked

## Acceptance Criteria

Test provider linking, conflict resolution, and account merging

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-20): Implement account linking functionality`
