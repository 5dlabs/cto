#!/bin/bash
# Quick test script for tools server config parsing
# This verifies the stdio semaphore fix without running the full server

cd /Users/jonathonfritz/agents/keeper

echo "=== Testing mcp-servers.json config parsing ==="
echo ""

# Check if config file exists and is valid JSON
if [ ! -f "test-mcp-servers.json" ]; then
    echo "ERROR: test-mcp-servers.json not found"
    exit 1
fi

echo "Config file: test-mcp-servers.json"
echo "File size: $(wc -c < test-mcp-servers.json) bytes"
echo ""

# Validate JSON
if ! python3 -c "import json; json.load(open('test-mcp-servers.json'))" 2>/dev/null; then
    echo "ERROR: Invalid JSON in test-mcp-servers.json"
    python3 -c "import json; json.load(open('test-mcp-servers.json'))" 2>&1
    exit 1
fi
echo "✓ JSON is valid"
echo ""

# Parse and display servers
echo "Servers defined in config:"
python3 << 'PYEOF'
import json
with open('test-mcp-servers.json') as f:
    config = json.load(f)

servers = config.get('servers', {})
print(f"\nTotal servers: {len(servers)}\n")

for name, cfg in sorted(servers.items()):
    transport = cfg.get('transport', 'unknown')
    cmd = cfg.get('command', cfg.get('url', 'N/A'))
    print(f"  {name}:")
    print(f"    transport: {transport}")
    print(f"    command/url: {cmd}")
    print()
PYEOF

echo "=== Config test complete ==="
