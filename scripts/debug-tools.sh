#!/bin/bash
# Local debugging script for tools server
# Tests the stdio semaphore fix
#
# Usage: ./scripts/debug-tools.sh
#
# This will:
# 1. Build the tools server (if not already built)
# 2. Run it with a minimal config on port 3001
# 3. Output detailed logs to /tmp/tools-debug.log and stdout
#
# To stop: Ctrl+C

set -e

# Change to the project directory
cd /Users/jonathonfritz/agents/keeper

# Set environment variables for local testing
export SYSTEM_CONFIG_PATH="/Users/jonathonfritz/agents/keeper/test-mcp-servers.json"
export RUST_LOG="debug,tools=trace"

echo "=== Tools Server Local Debug ==="
echo "Config: $SYSTEM_CONFIG_PATH"
echo "Log level: $RUST_LOG"
echo ""

# Build the tools server (only if binary doesn't exist)
if [ ! -f "./target/debug/tools-server" ]; then
    echo "Building tools server..."
    cargo build -p tools --bin tools-server
else
    echo "Tools server binary already exists"
fi

echo ""
echo "=== Running tools server ==="
echo "Press Ctrl+C to stop"
echo "Logs will be saved to /tmp/tools-debug.log"
echo ""

# Run with verbose logging
./target/debug/tools-server \
  --project-dir /Users/jonathonfritz/agents/keeper \
  --port 3001 \
  2>&1 | tee /tmp/tools-debug.log
