#!/bin/bash
# MCP Investor Research Server

cd "$(dirname "$0")/.."

# Load API key from environment or 1Password
if [ -z "$GROK_API_KEY" ]; then
  export GROK_API_KEY=$(op item get "Grok X API Key" --vault Automation --fields xai_api_key --reveal 2>/dev/null)
fi

# Run the server
bun run dev
