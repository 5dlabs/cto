# Task 19: Implement OAuth2 integration for Google and GitHub

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 19.

## Goal

Add OAuth2 authentication flows for Google and GitHub providers with user account linking

## Requirements

1. Add dependency: oauth2 = "4.4"
2. Create domain/oauth.rs with:
   - enum OAuthProvider { Google, GitHub }
   - struct OAuthConfig { client_id, client_secret, redirect_uri }
   - fn get_authorization_url(provider: OAuthProvider) -> String
   - async fn exchange_code(provider: OAuthProvider, code: String) -> Result<UserInfo>
3. Create api/oauth.rs handlers:
   - GET /api/auth/oauth/{provider} -> redirect to provider
   - GET /api/auth/oauth/{provider}/callback?code=xxx -> exchange code, create/link user, return JWT
4. Store OAuth tokens in users table with oauth_provider, oauth_id fields
5. Handle account linking: if email exists, link OAuth to existing account
6. Configure Google OAuth 2.0 client (https://console.cloud.google.com)
7. Configure GitHub OAuth app (https://github.com/settings/developers)

## Acceptance Criteria

Manual OAuth flow testing with real providers, verify account creation, test account linking, verify JWT returned after successful OAuth

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-19): Implement OAuth2 integration for Google and GitHub`
