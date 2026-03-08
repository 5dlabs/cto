#!/usr/bin/env bash
# =============================================================================
# Bridge Integration Test
# =============================================================================
# Tests NATS → Discord bridge and NATS → Linear bridge message flow.
#
# Prerequisites:
#   - Docker running (for NATS)
#   - nats CLI installed (brew install nats-io/nats-tools/nats)
#   - .env file with DISCORD_BRIDGE_TOKEN, LINEAR_API_KEY, LINEAR_TEAM_ID
#
# Usage:
#   cd tests/bridges
#   cp .env.example .env   # fill in real values
#   docker compose up -d
#   ./test-bridges.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NATS_URL="${NATS_URL:-nats://localhost:4222}"
NATS_MONITOR="${NATS_MONITOR:-http://localhost:8222}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}PASS${NC} $1"; }
fail() { echo -e "${RED}FAIL${NC} $1"; exit 1; }
info() { echo -e "${YELLOW}INFO${NC} $1"; }

# ---------------------------------------------------------------------------
# Check prerequisites
# ---------------------------------------------------------------------------
if ! command -v nats &>/dev/null; then
  echo "nats CLI not found. Install with: brew install nats-io/nats-tools/nats"
  exit 1
fi

if ! command -v docker &>/dev/null; then
  echo "docker not found."
  exit 1
fi

# ---------------------------------------------------------------------------
# Wait for NATS to be ready
# ---------------------------------------------------------------------------
info "Waiting for NATS at ${NATS_URL}..."
for i in $(seq 1 30); do
  if nats --server="${NATS_URL}" server ping --count=1 &>/dev/null; then
    pass "NATS is ready"
    break
  fi
  if [ "$i" -eq 30 ]; then
    fail "NATS not reachable after 30s"
  fi
  sleep 1
done

# ---------------------------------------------------------------------------
# Test 1: Publish AgentMessage with metadata to agent inbox
# ---------------------------------------------------------------------------
info "Test 1: Publishing AgentMessage with metadata to agent.bolt.inbox"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
nats --server="${NATS_URL}" pub agent.bolt.inbox "$(cat <<EOF
{
  "from": "rex",
  "to": "bolt",
  "subject": "agent.bolt.inbox",
  "message": "Bridge integration test — metadata enrichment check",
  "priority": "normal",
  "timestamp": "${TIMESTAMP}",
  "type": "message",
  "role": "rust-engineer",
  "metadata": {
    "model": "claude-opus-4-6",
    "provider": "anthropic",
    "coordinator": "openclaw",
    "step": "deliberation.optimist"
  }
}
EOF
)"
pass "AgentMessage published to agent.bolt.inbox"

# ---------------------------------------------------------------------------
# Test 2: Publish ElicitationRequest
# ---------------------------------------------------------------------------
info "Test 2: Publishing ElicitationRequest to elicitation.request"
nats --server="${NATS_URL}" pub elicitation.request "$(cat <<EOF
{
  "id": "test-elicit-001",
  "from": "nova",
  "question": "Should we add rate limiting to the API gateway?",
  "options": ["yes", "no", "defer"],
  "timestamp": "${TIMESTAMP}",
  "metadata": {
    "model": "claude-sonnet-4-6",
    "provider": "anthropic",
    "step": "deliberation.vote"
  }
}
EOF
)"
pass "ElicitationRequest published to elicitation.request"

# ---------------------------------------------------------------------------
# Test 3: Publish urgent message (tests priority coloring)
# ---------------------------------------------------------------------------
info "Test 3: Publishing urgent message to agent.cipher.inbox"
nats --server="${NATS_URL}" pub agent.cipher.inbox "$(cat <<EOF
{
  "from": "atlas",
  "to": "cipher",
  "subject": "agent.cipher.inbox",
  "message": "Security vulnerability found in auth module — immediate review needed",
  "priority": "urgent",
  "timestamp": "${TIMESTAMP}",
  "type": "message",
  "role": "security-engineer",
  "metadata": {
    "model": "claude-opus-4-6",
    "provider": "anthropic",
    "step": "security-scan"
  }
}
EOF
)"
pass "Urgent message published to agent.cipher.inbox"

# ---------------------------------------------------------------------------
# Test 4: Check NATS monitoring endpoint
# ---------------------------------------------------------------------------
info "Test 4: Checking NATS monitoring endpoint"
if curl -sf "${NATS_MONITOR}/varz" >/dev/null 2>&1; then
  SUBS=$(curl -sf "${NATS_MONITOR}/varz" | python3 -c "import sys,json; print(json.load(sys.stdin).get('subscriptions',0))" 2>/dev/null || echo "?")
  pass "NATS monitoring OK — ${SUBS} active subscriptions"
else
  info "NATS monitoring endpoint not reachable (non-fatal)"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "=============================="
echo "  Bridge tests complete"
echo "=============================="
echo ""
echo "Verify manually:"
echo "  - Discord #planning channel shows enriched embeds with model/provider in footer"
echo "  - Linear issue shows enriched comments with model/provider in header"
echo "  - Urgent message shows red embed (Discord) / urgent tag (Linear)"
echo ""
