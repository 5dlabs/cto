
## Linear Projects Being Archived Unexpectedly

If you notice Linear projects being archived immediately after creation, check:

### 1. Linear Workspace Automation Rules
Go to: **Linear** → **Settings** → **Workspace** → **Automations**

Look for any rules that:
- Archive projects after a time period
- Archive projects with certain names
- Cleanup duplicate projects

### 2. Production PM Server
If the local dev environment wasn't running when you ran the intake tool, the request may have gone to the production PM server (`pm.5dlabs.ai`) which might have different behavior.

**Fix:** Always run `just preflight` before using MCP tools to ensure:
- Local services are running
- Tunnel is healthy
- `CTO_PM_SERVER_URL` points to dev

### 3. Cleanup Script
To remove old archived test projects:
```bash
just cleanup-test-projects
```

### Verification
Run `just preflight` and ensure:
- ✅ CTO_PM_SERVER_URL points to pm-dev.5dlabs.ai
- ✅ cto-config.json pmServerUrl points to pm-dev.5dlabs.ai
- ✅ Tunnel is healthy
