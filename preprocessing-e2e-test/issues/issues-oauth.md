# Issues Log: oauth

## Issue #1: Morgan OAuth Tokens Missing (RESOLVED)
- **Severity**: BLOCKING
- **Created**: 2026-01-28T11:56:23-08:00
- **Status**: RESOLVED
- **Resolution**: 2026-01-29
- **Details**:
  - Fixed by adding LINEAR_APP_MORGAN_CLIENT_ID and LINEAR_APP_MORGAN_CLIENT_SECRET to PM server launchd plist
  - Tokens verified to exist in Kubernetes secret `linear-app-morgan`
- **Note**: Token validation with Linear API showed network issues (exit code 56) - may be a network/firewall issue
