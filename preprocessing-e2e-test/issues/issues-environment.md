# Issues Log: environment

## ISSUE-1: Transient Kubernetes connection errors at startup (RESOLVED)
- **Status**: RESOLVED
- **Severity**: LOW
- **Discovered**: 2026-01-29T00:18:07
- **Description**: 28 ERROR entries in controller.log during initial startup due to Kubernetes API being unreachable before tunnel was established
- **Root Cause**: Network tunnel (cloudflared) was not yet connected when controller started
- **Resolution**: Tunnel connected at ~03:52, all services recovered and are now healthy with 0 recent errors

## ISSUE-2: LINEAR_ENABLED not set (informational)
- **Status**: RESOLVED
- **Severity**: LOW
- **Discovered**: 2026-01-29T04:00:00-08:00
- **Description**: PM server logs show "LINEAR_ENABLED is not set to true. PM service will not process webhooks."
- **Root Cause**: Environment variable not configured - expected for local development
- **Resolution**: Informational only. PM server runs normally without Linear webhooks.
