# Linear OAuth Token Management - Problem Statement

## The Pain Point

Morgan (and other agents) require Linear OAuth tokens to create issues, projects, and manage workflows. These tokens have the following characteristics:

1. **User OAuth tokens** (from authorization flow):
   - Valid for 10 days
   - Can be refreshed using refresh_token
   - Required for agent-specific operations (creating agent sessions, etc.)
   - Currently stored in 1Password and `.env.local`

2. **Client credentials tokens**:
   - Valid for 30 days
   - Cannot refresh - must re-generate
   - Limited functionality ("App user not valid" on certain operations)
   - Not suitable for full agent operations

## Current Failure Modes

1. Token expires silently → API calls fail with 401
2. Refresh token expires → must re-authorize manually via browser
3. Multiple services (PM server, controller, MCP) each need the token
4. No automatic refresh mechanism → manual intervention every ~10 days
5. Environment variables scattered across launchd plists, .env files, 1Password

## Requirements for Solution

1. **Zero manual intervention** - token should stay valid indefinitely
2. **Multi-service coordination** - all services should get updated tokens
3. **Failure recovery** - if refresh fails, alert and provide fallback
4. **Observability** - know when tokens are refreshed, when they'll expire
5. **Security** - tokens must be stored securely, rotated properly

## Constraints

- Linear OAuth spec: https://linear.app/developers/oauth-2-0-authentication
- Refresh tokens are single-use (new refresh_token issued with each refresh)
- Must work both locally (launchd) and in Kubernetes cluster
- Morgan is an OAuth app installed in Linear workspace

## Questions Each Solution Must Address

1. Where are tokens stored?
2. What triggers a refresh?
3. How do services get the new token?
4. What happens if refresh fails?
5. How is this deployed/maintained?
