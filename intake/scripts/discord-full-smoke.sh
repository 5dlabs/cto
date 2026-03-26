#!/usr/bin/env bash
# Full Discord-only smoke: /notify (markers + session thread) + /elicitation (decision card in thread).
#
# Prerequisites: discord-bridge on DISCORD_BRIDGE_URL (default http://127.0.0.1:3200)
#
# Usage: ./intake/scripts/discord-full-smoke.sh

set -euo pipefail
URL="${DISCORD_BRIDGE_URL:-http://127.0.0.1:3200}"
URL="${URL%/}"

echo "=== 1) Health ==="
curl -sf "$URL/health" | jq .

SID="full-discord-$(date +%s)"
echo ""
echo "=== 2) Parent-channel marker (no session_id) ==="
curl -sf -X POST "$URL/notify" -H 'Content-Type: application/json' \
  -d "$(jq -nc '{
    from: "intake",
    to: "deliberation",
    subject: "agent.deliberation.inbox",
    message: "**Full Discord smoke** — pipeline-style marker (stays in #intake).",
    priority: "normal",
    timestamp: (now | todate),
    metadata: { step: "discord-full-smoke", project_name: "discord-full-smoke" }
  }')" | jq .

echo ""
echo "=== 3) deliberation-start (parent + thread link) session=$SID ==="
curl -sf -X POST "$URL/notify" -H 'Content-Type: application/json' \
  -d "$(jq -nc --arg sid "$SID" '{
    from: "intake",
    to: "deliberation",
    subject: "agent.deliberation.inbox",
    message: "**Deliberation started** — open the thread below.",
    priority: "normal",
    timestamp: (now | todate),
    metadata: { step: "deliberation-start", session_id: $sid }
  }')" | jq .

echo ""
echo "=== 4) Debate turns (same session → thread only) ==="
for speaker in optimist pessimist; do
  curl -sf -X POST "$URL/notify" -H 'Content-Type: application/json' \
    -d "$(jq -nc --arg sid "$SID" --arg sp "$speaker" '{
      from: $sp,
      to: "deliberation",
      subject: "agent.deliberation.inbox",
      message: ("**" + $sp + "** — full-smoke turn body."),
      priority: "normal",
      timestamp: (now | todate),
      metadata: { session_id: $sid, speaker: $sp, step: "discord-full-smoke-turn" }
    }')" | jq -c .
done

echo ""
echo "=== 5) Elicitation (session thread via HTTP server; card with buttons) ==="
ELID="${SID}-dp-smoke"
curl -sf -X POST "$URL/elicitation" -H 'Content-Type: application/json' \
  -d "$(jq -nc --arg sid "$SID" --arg eid "$ELID" '{
    elicitation_id: $eid,
    session_id: $sid,
    decision_id: "dp-smoke",
    question: "Full smoke: pick an option (Discord thread + type 7)",
    category: "architecture",
    options: [
      { label: "Option A", value: "a", description: "First" },
      { label: "Option B", value: "b", description: "Second" }
    ],
    recommended_option: "a",
    vote_summary: {
      total_voters: 5,
      tally: { "a": 3, "b": 2 },
      consensus_strength: 0.6,
      escalated: false
    },
    allow_redeliberation: true,
    timeout_seconds: 0,
    informational: false,
    timestamp: (now | todate)
  }')" | jq .

echo ""
echo "=== Done ==="
echo "Session: $SID"
echo "In Discord: check #intake for marker + start + thread link; open thread for turns + decision card."
echo "Optional: click a button to confirm type-7 resolution (no stale component error)."
echo "Log: tail -f \"$(cd "$(dirname "$0")/../.." && pwd)/intake/.bridge-logs/discord-bridge.log\""
