# Ralph Mobile Dashboard

A simple mobile-friendly dashboard for monitoring and controlling Ralph loops remotely.

## Quick Setup (5 minutes)

### 1. Create KV Namespace

```bash
cd marketing

# Create the KV namespace
wrangler kv:namespace create RALPH_STATE

# Copy the ID from the output and update wrangler.toml:
# [[kv_namespaces]]
# binding = "RALPH_STATE"
# id = "<your-kv-id>"
```

### 2. Deploy to Cloudflare

```bash
# Build and deploy
npm run build
wrangler pages deploy out
```

### 3. Access the Dashboard

Go to `https://5dlabs.ai/ralph` on your phone.

## Usage

### Option A: Sync Existing Ralph Loop

If you have a Ralph loop running locally (like latitude-install):

```bash
# In a separate terminal, run the sync watcher
./scripts/ralph-sync-watcher.sh latitude-install/ralph-coordination.json latitude-install/progress.txt
```

This will:
- Watch your local coordination file
- Sync state to the dashboard every 10 seconds
- Process commands from your phone (pause/resume/stop)

### Option B: Integrate Into New Ralph Loops

Add to your Ralph loop scripts:

```bash
#!/bin/bash
source scripts/ralph-dashboard-sync.sh

# Initialize session
ralph_init_session "my-install"

# In your loop, update progress
ralph_update_step "Deploying ArgoCD" 15 25

# Log messages
ralph_log "Starting deployment..."

# Check for mobile commands
cmd=$(ralph_check_commands)
if [ "$cmd" = "pause" ]; then
  echo "Paused by mobile!"
  # Wait for resume...
fi

# Report errors
ralph_error "Connection refused"

# Mark complete
ralph_complete
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/ralph/state` | GET | Get current state |
| `/api/ralph/state` | POST | Update state |
| `/api/ralph/command` | GET | Get pending commands |
| `/api/ralph/command` | POST | Queue a command |
| `/api/ralph/log` | POST | Append to log |

## Mobile Features

- **Live Status**: See current step, progress bar, duration
- **Activity Log**: Last 20 log entries
- **Hardening Actions**: Issues detected and fixed
- **Controls**: Pause, Resume, Stop buttons
- **Auto-refresh**: Every 30 seconds (or tap refresh)

## Architecture

```
┌─────────────────┐         ┌─────────────────┐
│  Your Phone     │◄───────►│  Cloudflare     │
│  (Dashboard)    │  HTTPS  │  Pages + KV     │
└─────────────────┘         └────────┬────────┘
                                     │
                                     │ HTTPS (curl)
                                     ▼
                            ┌─────────────────┐
                            │  Your Laptop    │
                            │  (Ralph Loop)   │
                            │                 │
                            │  sync-watcher   │
                            │       ↓         │
                            │  coordination   │
                            │  .json          │
                            └─────────────────┘
```

The sync watcher runs locally and:
1. Watches your local coordination file for changes
2. Pushes updates to Cloudflare KV
3. Polls for commands from mobile
4. Updates local coordination file when commands received

## Tips

- **Low battery?** The dashboard auto-refreshes every 30s. Manual refresh anytime.
- **Lost connection?** State persists in KV for 1 hour.
- **Multiple sessions?** Only latest session shown (for simplicity).
- **Want notifications?** Add ntfy.sh calls to your loop for push alerts.
