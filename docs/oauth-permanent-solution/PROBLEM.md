# Linear OAuth Permanent Solution - Problem Statement

## The Pain Point

Morgan (and other agents) need to authenticate with Linear to create projects, issues, and manage tasks. Currently, we keep running into OAuth token expiration/refresh issues that require manual intervention.

## Current State

1. **Client Credentials Flow** (30-day token):
   - Works for reading data (teams, issues)
   - Fails when PM server tries to create issues ("App user not valid")
   - Token is `app` actor, not `user` actor

2. **User OAuth Flow** (10-day token with refresh):
   - Requires initial user authorization
   - Refresh tokens can expire or be revoked
   - We have refresh infrastructure but it keeps failing

3. **PM Server Configuration**:
   - Reads `LINEAR_OAUTH_TOKEN` or `LINEAR_API_KEY` env vars
   - Has agent-specific token handling (per-agent OAuth apps)
   - Has refresh logic but it's not working reliably

## What We Have

- Linear OAuth App: `5DLabs-Morgan` (Client ID: `752f67d53b2b0dab2191832ba0aa43d9`)
- Client credentials flow configured
- PM server with refresh endpoint at `/oauth/refresh/{agent}`
- Kubernetes secrets infrastructure
- 1Password for credential storage
- launchd services for local dev

## Requirements for Solution

1. **Zero manual intervention** - Morgan should never need human to reauthorize
2. **Automatic token refresh** - Before tokens expire
3. **Graceful failure handling** - If refresh fails, auto-recover
4. **Works in both local dev and K8s cluster**
5. **Secure credential storage**

## Linear OAuth Documentation

From https://linear.app/developers/oauth-2-0-authentication:

- **Authorization Code Flow**: User grants access, get access_token (10 days) + refresh_token
- **Client Credentials Flow**: App-level token (30 days), limited to public teams, `app` actor
- **Refresh**: POST to `/oauth/token` with `grant_type=refresh_token`
- **Scopes**: `read`, `write`, `issues:create`, `admin`

## Key Questions

1. Why does client_credentials token fail for issue creation?
2. How to ensure refresh tokens never expire silently?
3. Should we use per-agent OAuth apps or a single service account?
4. How to handle the initial authorization flow in a way that's one-time only?
5. What monitoring/alerting should exist for token health?

## Files to Reference

- `/Users/jonathonfritz/.cursor/worktrees/cto-e2e-testing/fzm/crates/pm/src/config.rs` - PM config handling
- `/Users/jonathonfritz/.cursor/worktrees/cto-e2e-testing/fzm/crates/pm/src/handlers/oauth.rs` - OAuth handlers
- `/Users/jonathonfritz/.cursor/worktrees/cto-e2e-testing/fzm/crates/pm/src/server.rs` - PM server with Linear client
- `/Users/jonathonfritz/.cursor/worktrees/cto-e2e-testing/fzm/crates/pm/src/client.rs` - Linear GraphQL client
