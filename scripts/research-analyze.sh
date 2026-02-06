#!/bin/bash
# Research Create - Runs every 5 minutes
# Automatically creates PRDs for worthwhile projects

# Set PATH for cron environment (ensure op is available)
export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/Users/jonathonfritz/.bun/bin:/Users/jonathonfritz/.local/bin"
export HOME="/Users/jonathonfritz"

# Get 1Password session token for cron
export OP_SESSION=$(op account token --account G2H2QB3NWJERPKXJMQEU424NU4 2>/dev/null)
if [ -z "$OP_SESSION" ]; then
  echo "[$(date)] ERROR: Could not get 1Password session" >> /tmp/research-create.log
  exit 1
fi

BUN_PATH="/Users/jonathonfritz/.bun/bin/bun"

echo "=== [$(date)] Research Create ===" >> /tmp/research-create.log

cd /Users/jonathonfritz/agents/research

# Run research + PRD creation
$BUN_PATH run src/utils/research-create.ts --once 2>&1 >> /tmp/research-create.log

echo "=== [$(date)] Complete ===" >> /tmp/research-create.log
