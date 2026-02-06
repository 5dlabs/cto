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


## Claude Code & Swarm Mode

### Binary Path
```bash
/Users/jonathonfritz/.local/bin/claudesp
```

Use `claudesp` (not `claude`) for swarm/TeammateTool features.

### One-Shot Coding Task
```bash
# PTY required for interactive terminal
exec pty:true workdir:/path/to/project command:"claudesp 'Your task here'"
```

### Background Coding Task
```bash
# Start in background, get sessionId
exec pty:true workdir:/path/to/project background:true command:"claudesp 'Your task here'"

# Monitor progress
process action:log sessionId:XXX

# Check if done
process action:poll sessionId:XXX
```

### Swarm Mode (Parallel Sub-Agents)

Use TeammateTool for parallel orchestration:

```javascript
// Create a team
Teammate({ operation: "spawnTeam", team_name: "my-team" })

// Spawn a worker
Task({
  team_name: "my-team",
  name: "worker-1",
  subagent_type: "general-purpose",
  prompt: "Your task for the sub-agent",
  run_in_background: true
})

// Check inbox for results
Teammate({ operation: "getInbox", team_name: "my-team" })
```

### Auto-Notify on Completion

For long tasks, append wake trigger:
```
... your task here.

When finished, run: clawdbot gateway wake --text "Done: [summary]" --mode now
```

## Discord Headless Access (agent-browser)

Authenticated profile saved at `~/.agent-browser/profiles/discord`

```bash
# Set profile for Discord access
export AGENT_BROWSER_PROFILE="$HOME/.agent-browser/profiles/discord"

# Open Discord
agent-browser open "https://discord.com/channels/@me"

# Get element refs for interaction
agent-browser snapshot -i

# Click elements, type, etc.
agent-browser click @e1
agent-browser type @e2 "message text"

# Screenshot
agent-browser screenshot /tmp/discord.png

# Close when done
agent-browser close
```

Session persists across restarts. No need to re-login.

## Agent Directory

See `/Users/jonathonfritz/.clawdbot/AGENT_DIRECTORY.md` for a list of all agents and how to contact them.

Quick reference:
- **stitch** — code review
- **metal** — infrastructure  
- **pixel/ctolite** — desktop app
- **research** — web research
- **holt** — bot deployment
- **intake** — PRD processing


---

## Agent Browser (Headless Web Automation)

**ALWAYS use `agent-browser` with `--state` for authenticated web automation.** Runs headless by default.

### Quick Start (Authenticated)

```bash
# Linear - project management
agent-browser --state ~/.agent-browser/linear-auth.json open https://linear.app

# Discord - messaging  
agent-browser --state ~/.agent-browser/discord-auth.json open https://discord.com/channels/@me

# Get snapshot, interact, close
agent-browser snapshot -i
agent-browser click @e2
agent-browser close
```

### Available Auth States

| Service | State File | Example URL |
|---------|-----------|-------------|
| Linear | `~/.agent-browser/linear-auth.json` | `https://linear.app` |
| Discord | `~/.agent-browser/discord-auth.json` | `https://discord.com/channels/@me` |

### Workflow Pattern

```bash
# 1. Open with auth state
agent-browser --state ~/.agent-browser/linear-auth.json open https://linear.app

# 2. Get snapshot to see elements
agent-browser snapshot -i

# 3. Interact using @refs from snapshot
agent-browser click @e5

# 4. ALWAYS close when done
agent-browser close
```

### Important Rules

1. **ALWAYS use `--state`** for authenticated sites
2. **ALWAYS `close` when done** - One browser at a time
3. **Use @refs from snapshots** - More reliable than selectors

