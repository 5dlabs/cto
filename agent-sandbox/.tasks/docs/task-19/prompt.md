# Task 19: Implement OAuth2 integration for Google and GitHub

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 19.

## Goal

Add OAuth2 authentication flow supporting Google and GitHub providers with callback handling and user profile synchronization.

## Requirements

1. Add dependency: oauth2 = "4.4"
2. Create domain/oauth.rs with:
   - OAuthProvider enum (Google, GitHub)
   - OAuthClient struct wrapping oauth2::BasicClient
3. Implement OAuth flow in api/oauth.rs:
   - GET /api/auth/oauth/{provider} -> redirect to provider with state parameter
   - GET /api/auth/oauth/{provider}/callback -> exchange code for token, fetch user profile
4. Store OAuth state in Redis with 10min TTL for CSRF protection
5. Create or update user in database:
   - If oauth_provider + oauth_id exists, login
   - Else create new user with email from profile
6. Return TokenPair after successful OAuth
7. Add to config: GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET, OAUTH_REDIRECT_URL
8. Handle provider-specific profile APIs:
   - Google: https://www.googleapis.com/oauth2/v2/userinfo
   - GitHub: https://api.github.com/user

## Acceptance Criteria

Manual testing with real OAuth providers in development. Mock OAuth responses for integration tests. Verify state validation prevents CSRF. Test user creation and login flows. Verify email conflicts handled gracefully.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-19): Implement OAuth2 integration for Google and GitHub`
