# Task 2: Configure environment variables and secrets

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 2.

## Goal

Setup environment configuration for authentication providers and services

## Requirements

1. Create .env.example with required variables
2. Setup environment validation using joi or similar
3. Configure variables: GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET, JWT_SECRET, JWT_REFRESH_SECRET, REDIS_URL, EMAIL_SERVICE_CONFIG
4. Implement config loader with validation

## Acceptance Criteria

Verify environment variables load correctly and validation catches missing required variables

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-2): Configure environment variables and secrets`
