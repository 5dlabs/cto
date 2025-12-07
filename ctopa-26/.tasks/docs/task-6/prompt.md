# Task 6: Implement OAuth2 integration (Google and GitHub)

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 6.

## Goal

Add OAuth2 authentication providers for Google and GitHub login

## Requirements

1. Add oauth2 crate dependency
2. Create oauth.rs module with OAuth2 client configuration
3. Implement OAuth2 authorization flow handlers for Google and GitHub
4. Add callback endpoints: /auth/google/callback, /auth/github/callback
5. Store OAuth2 client credentials in environment variables
6. Link OAuth2 accounts to existing users or create new users
7. Generate JWT tokens after successful OAuth2 authentication

## Acceptance Criteria

Integration tests with mock OAuth2 providers and manual testing with real OAuth2 flows

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-6): Implement OAuth2 integration (Google and GitHub)`
