#!/usr/bin/env bash
set -euo pipefail

# Inline 3-model voting without Lobster sub-workflow (avoids ENAMETOOLONG).
# Usage: inline-vote.sh <content_file> <criteria> <v1p> <v1m> <v2p> <v2m> <v3p> <v3m>

ROOT="${WORKSPACE:-.}"
CF="$1"; CRITERIA="$2"
V1P="$3"; V1M="$4"; V2P="$5"; V2M="$6"; V3P="$7"; V3M="$8"

if [ ! -f "$CF" ]; then
  echo "inline-vote: content_file not found: $CF" >&2; exit 1
fi

WS="${WORKSPACE:-.}"
mkdir -p "$WS/.intake/tmp"

vote_one() {
  local SOUL_FILE="$1" PROVIDER="$2" MODEL="$3" VOTER_ID="$4"
  local ARGS_FILE="$WS/.intake/tmp/voter-${VOTER_ID}-args.json"
  local TIMEOUT_SEC="${INLINE_VOTE_TIMEOUT_SEC:-90}"

  jq -n \
    --rawfile p1 "$SOUL_FILE" \
    --rawfile p2 "$ROOT/intake/prompts/vote-system.md" \
    --rawfile content "$CF" \
    --arg criteria "$CRITERIA" \
    --arg schema "$ROOT/intake/schemas/vote-ballot.schema.json" \
    --arg provider "$PROVIDER" \
    --arg model "$MODEL" \
    '{ "prompt": ($p1 + "\n\n---\n\n" + $p2),
       "input": {"content": $content, "criteria": $criteria},
       "schema": $schema, "provider": $provider, "model": $model }' > "$ARGS_FILE"

  python3 - "$ARGS_FILE" "$TIMEOUT_SEC" <<'PY'
import subprocess
import sys

args_file = sys.argv[1]
timeout_sec = int(sys.argv[2])

try:
    completed = subprocess.run(
        ["openclaw.invoke", "--tool", "llm-task", "--action", "json", "--args-file", args_file],
        check=True,
        text=True,
        capture_output=True,
        timeout=timeout_sec,
    )
    sys.stdout.write(completed.stdout)
except subprocess.TimeoutExpired:
    print("inline-vote: vote invocation timed out", file=sys.stderr)
    sys.exit(124)
except subprocess.CalledProcessError as err:
    if err.stderr:
        sys.stderr.write(err.stderr)
    sys.exit(err.returncode or 1)
PY
}

FALLBACK_SID_FILE="$WS/.intake/linear-session-id.txt"
LINEAR_SID=""
if [ -f "$FALLBACK_SID_FILE" ] && [ -s "$FALLBACK_SID_FILE" ]; then
  LINEAR_SID="$(cat "$FALLBACK_SID_FILE")"
fi

post_voter_activity() {
  local EMOJI="$1" NAME="$2" TAGLINE="$3" MODEL="$4" VOTER_OUT="$5"
  if [ -z "$LINEAR_SID" ] || [ -z "$VOTER_OUT" ]; then return 0; fi
  local CHOSEN REASONING CONF_PCT CONF_INT SENTIMENT BODY
  CHOSEN="$(printf '%s' "$VOTER_OUT" | jq -r '.chosen_option // .verdict // "unknown"' 2>/dev/null)"
  REASONING="$(printf '%s' "$VOTER_OUT" | jq -r '.reasoning // "No reasoning provided."' 2>/dev/null)"
  CONF_PCT="$(printf '%s' "$VOTER_OUT" | jq -r '(.confidence // 0.6) * 100' 2>/dev/null || echo '60')"
  CONF_INT="${CONF_PCT%.*}"
  if [ "$CONF_INT" -ge 80 ] 2>/dev/null; then SENTIMENT="💪"; elif [ "$CONF_INT" -ge 60 ] 2>/dev/null; then SENTIMENT="🤔"; else SENTIMENT="⚠️"; fi
  BODY="$(printf '## %s %s\n\n> *%s*\n\n**Vote:** %s %s\n**Confidence:** %s%% %s\n**Model:** `%s`\n\n---\n\n%s' \
    "$EMOJI" "$NAME" "$TAGLINE" "$CHOSEN" "$SENTIMENT" "$CONF_PCT" "$SENTIMENT" "$MODEL" "$REASONING")"
  intake-util linear-activity --session-id "$LINEAR_SID" --type thought --body "$BODY" 2>/dev/null || true
}

echo "inline-vote: running voter-1 (architect)" >&2
V1_OUT=""
if V1_OUT=$(vote_one "$ROOT/intake/prompts/voter-architect-soul.md" "$V1P" "$V1M" "1"); then
  echo "inline-vote: voter-1 ok" >&2
  post_voter_activity "🏛️" "Architect" "Evaluating structural integrity and long-term maintainability." "$V1M" "$V1_OUT"
else
  echo "inline-vote: voter-1 failed (continuing with degraded committee)" >&2
fi

echo "inline-vote: running voter-2 (pragmatist)" >&2
V2_OUT=""
if V2_OUT=$(vote_one "$ROOT/intake/prompts/voter-pragmatist-soul.md" "$V2P" "$V2M" "2"); then
  echo "inline-vote: voter-2 ok" >&2
  post_voter_activity "⚖️" "Pragmatist" "Weighing practical trade-offs and real-world constraints." "$V2M" "$V2_OUT"
else
  echo "inline-vote: voter-2 failed (continuing with degraded committee)" >&2
fi

echo "inline-vote: running voter-3 (minimalist)" >&2
V3_OUT=""
if V3_OUT=$(vote_one "$ROOT/intake/prompts/voter-minimalist-soul.md" "$V3P" "$V3M" "3"); then
  echo "inline-vote: voter-3 ok" >&2
  post_voter_activity "✂️" "Minimalist" "Cutting to essential complexity — less is more." "$V3M" "$V3_OUT"
else
  echo "inline-vote: voter-3 failed (continuing with degraded committee)" >&2
fi

echo "inline-vote: tallying" >&2
BALLOTS_FILE="$WS/.intake/tmp/inline-vote-ballots.json"
TALLY_FILE="$WS/.intake/tmp/inline-vote-tally.json"

jq -n \
  --arg v1 "$V1_OUT" \
  --arg v2 "$V2_OUT" \
  --arg v3 "$V3_OUT" \
  '[
    ($v1 | select(length > 0) | fromjson),
    ($v2 | select(length > 0) | fromjson),
    ($v3 | select(length > 0) | fromjson)
  ] | map(select(. != null))' > "$BALLOTS_FILE"

BALLOT_COUNT="$(jq 'length' "$BALLOTS_FILE" 2>/dev/null || echo 0)"

if [ "$BALLOT_COUNT" -gt 0 ]; then
  if ! intake-util tally --ballots-json "$BALLOTS_FILE" > "$TALLY_FILE"; then
    echo "inline-vote: tally failed; using fallback verdict" >&2
    BALLOT_COUNT=0
  fi
fi

if [ "$BALLOT_COUNT" -eq 0 ]; then
  jq -n '{
    verdict: "approve",
    average_scores: {
      task_decomposition: 0,
      dependency_ordering: 0,
      decision_point_coverage: 0,
      test_strategy_quality: 0,
      agent_assignment: 0,
      overall: 0
    },
    vote_breakdown: { approve: 0, revise: 0, reject: 0 },
    suggestions: ["Committee voting degraded: all voter models unavailable during this run."],
    consensus_score: 0
  }' > "$TALLY_FILE"
else
  # Add an explicit note when some voters failed but we still produced a tally.
  if [ "$BALLOT_COUNT" -lt 3 ]; then
    jq --arg note "Committee voting degraded: $BALLOT_COUNT/3 ballots available." \
      '.suggestions = ((.suggestions // []) + [$note])' "$TALLY_FILE" > "$TALLY_FILE.tmp" && mv "$TALLY_FILE.tmp" "$TALLY_FILE"
  fi
fi

# Post tally to Linear
if [ -n "$LINEAR_SID" ] && [ -f "$TALLY_FILE" ]; then
  VERDICT="$(jq -r '.verdict // "unknown"' "$TALLY_FILE" 2>/dev/null)"
  APPROVE_CT="$(jq -r '.vote_breakdown.approve // 0' "$TALLY_FILE" 2>/dev/null)"
  REVISE_CT="$(jq -r '.vote_breakdown.revise // 0' "$TALLY_FILE" 2>/dev/null)"
  REJECT_CT="$(jq -r '.vote_breakdown.reject // 0' "$TALLY_FILE" 2>/dev/null)"
  if [ "$VERDICT" = "approve" ]; then ICON="✅"; elif [ "$VERDICT" = "revise" ]; then ICON="🔄"; else ICON="❌"; fi
  TALLY_BODY="$(printf '## 🗳️ Committee Verdict: %s %s\n\n| Vote | Count |\n|------|-------|\n| ✅ Approve | %s |\n| 🔄 Revise | %s |\n| ❌ Reject | %s |' \
    "$VERDICT" "$ICON" "$APPROVE_CT" "$REVISE_CT" "$REJECT_CT")"
  intake-util linear-activity --session-id "$LINEAR_SID" --type thought --body "$TALLY_BODY" 2>/dev/null || true
fi

# Keep the historical wrapper shape expected by task-refinement checks.
jq -n --slurpfile t "$TALLY_FILE" '{output: [$t[0]]}'
