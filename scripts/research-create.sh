#!/bin/bash
# Research Create Cron Job - Runs every 5 minutes
#
# Install:
#   crontab -e
#   Add: */5 * * * * /Users/jonathonfritz/agents/research/scripts/research-create.sh >> /tmp/research-create.log 2>&1
#
# Manual run:
#   ./research-create.sh

# Set PATH for cron environment
export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/Users/jonathonfritz/.bun/bin:/Users/jonathonfritz/.nvm/versions/node/v25.5.0/bin"
export HOME="/Users/jonathonfritz"

# Read API key from secure file (created once with: op item get "Grok X API Key" --vault Automation --fields xai_api_key --reveal > ~/.config/openclaw/grok-api-key)
export GROK_API_KEY=$(cat ~/.config/openclaw/grok-api-key 2>/dev/null)

if [ -z "$GROK_API_KEY" ]; then
  echo "[$(date)] ERROR: Could not read Grok API key from ~/.config/openclaw/grok-api-key" >> /tmp/research-create.log
  exit 1
fi

cd /Users/jonathonfritz/agents/research
bun run src/utils/research-create.ts --once >> /tmp/research-create.log 2>&1
