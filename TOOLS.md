# TOOLS.md - Local Notes

Skills define *how* tools work. This file is for *your* specifics — the stuff that's unique to your setup.

## What Goes Here

Things like:
- Camera names and locations
- SSH hosts and aliases  
- Preferred voices for TTS
- Speaker/room names
- Device nicknames
- Anything environment-specific

## Examples

```markdown
### Cameras
- living-room → Main area, 180° wide angle
- front-door → Entrance, motion-triggered

### SSH
- home-server → 192.168.1.100, user: admin

### TTS
- Preferred voice: "Nova" (warm, slightly British)
- Default speaker: Kitchen HomePod
```

## Why Separate?

Skills are shared. Your setup is yours. Keeping them apart means you can update skills without losing your notes, and share skills without leaking your infrastructure.

---

Add whatever helps you do your job. This is your cheat sheet.

## Credentials

**IMPORTANT**: Before asking for credentials or searching 1Password, check:

1. **Local credentials file**: `local/CREDENTIALS.md` or `~/.config/cto/credentials.md`
2. **Environment variables**: Many tools auto-load from `.env` files
3. **1Password CLI**: Use `op` if you need to fetch new credentials

### Loading Credentials for Development

```bash
# Option 1: Source credentials (if shell exports)
source ~/.config/cto/credentials.md

# Option 2: Use direnv (auto-loads .envrc in directories)
cd /Users/jonathonfritz/agents/conductor

# Option 3: Load into current shell from local/CREDENTIALS.md
set -a
source /Users/jonathonfritz/agents/conductor/local/CREDENTIALS.md
set +a
```

### Available Credentials (in local/CREDENTIALS.md)

- GitHub App credentials (Morgan, Rex, etc.)
- Linear OAuth app credentials
- API keys (Anthropic, OpenAI)
- Cloudflare tunnel tokens
- Database connection strings
- Kubernetes config
