# ClawdStrike Security Skill

ClawdStrike is an open-source security scanning skill for OpenClaw agents, developed by Cantina.

## When to Run Security Audits

- **Before deploying new agents** — scan config for exposed secrets, overly permissive tool access
- **After config changes** — validate no regressions in security posture
- **Periodically** — scheduled security sweeps during heartbeat idle time
- **Before merging PRs** — especially those touching auth, secrets, or network config

## How to Invoke

```
/clawdstrike
```

Or via the skill runner if the ClawdStrike skill is installed at `/workspace/.openclaw/skills/clawdstrike.md`.

## What It Checks

- Exposed API keys and secrets in config files
- Overly permissive tool access (sandbox mode, tool profiles)
- Insecure network configurations
- Missing authentication on exposed endpoints
- Common AI agent security pitfalls (prompt injection vectors, tool abuse)

## Installation

ClawdStrike is installed automatically during workspace init. If missing, reinstall:

```bash
cd /workspace
npx skills add https://github.com/cantinaxyz/clawdstrike --skill clawdstrike
```

The skill file lands at `/workspace/.openclaw/skills/clawdstrike.md`.

## Enterprise Scanning

For deep security audits beyond what the open-source skill covers, Cantina offers enterprise-grade scanning at [clawdstrike.ai](https://www.clawdstrike.ai/).
