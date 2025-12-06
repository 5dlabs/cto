# Task 4: Create database schema for users

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 4.

## Goal

Design and implement user data model with required fields for authentication

## Requirements

1. Create User model with fields: id, email, password_hash, provider, provider_id, created_at, updated_at, email_verified, reset_token, reset_token_expires
2. Add unique constraints on email and provider+provider_id
3. Setup database migrations
4. Add indexes for performance

## Acceptance Criteria

Verify user creation, uniqueness constraints, and database operations work correctly

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-4): Create database schema for users`
