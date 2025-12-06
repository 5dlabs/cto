# Task 7: Implement OAuth2 integration for Google and GitHub

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 7.

## Goal

Add OAuth2 authentication flows for Google and GitHub providers with callback handling and user account linking

## Requirements

1. Add dependency: oauth2 = "4.4"
2. Create src/domain/oauth.rs with:
   - OAuth2 client configuration for Google and GitHub
   - GET /api/auth/oauth/:provider: redirect to provider authorization URL with state parameter
   - GET /api/auth/oauth/:provider/callback: handle callback
     * Exchange code for token
     * Fetch user info from provider API
     * Find or create user in database by email
     * Generate JWT tokens
3. Store OAuth state in Redis with 10min TTL for CSRF protection
4. Add oauth_provider, oauth_id columns to users table
5. Environment variables: GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET

## Acceptance Criteria

Manual test: initiate OAuth flow, complete authorization, verify JWT returned. Unit test state validation. Test account linking when email exists. Mock provider APIs for integration tests

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-7): Implement OAuth2 integration for Google and GitHub`
