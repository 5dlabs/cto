# Task 5: Implement OAuth2 integration for Google and GitHub

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 5.

## Goal

Add OAuth2 flows for external authentication providers with user account linking

## Requirements

1. Add oauth2 crate dependency
2. Configure OAuth2 clients for Google and GitHub with environment variables
3. Implement authorization URL generation endpoints
4. Create callback handlers to exchange codes for tokens
5. Fetch user profile data and create/link accounts
6. Generate internal JWT tokens after successful OAuth2 flow

## Acceptance Criteria

Test OAuth2 flows end-to-end, verify user profile data extraction, validate account creation/linking logic

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-5): Implement OAuth2 integration for Google and GitHub`
