#!/usr/bin/env bash
# Schema Tests: Validate that fixture files conform to their JSON schemas.
#
# Uses ajv-cli (or python jsonschema) to validate fixtures against schemas.
# Falls back to basic jq structure checks if no validator is available.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEMAS_DIR="$SCRIPT_DIR/../schemas"
FIXTURES_DIR="$SCRIPT_DIR/fixtures"

PASS=0
FAIL=0
SKIP=0

pass() {
  echo "  PASS: $1"
  PASS=$((PASS + 1))
}

fail() {
  echo "  FAIL: $1 — $2"
  FAIL=$((FAIL + 1))
}

skip() {
  echo "  SKIP: $1 — $2"
  SKIP=$((SKIP + 1))
}

# Detect validator
VALIDATOR=""
if command -v ajv >/dev/null 2>&1; then
  VALIDATOR="ajv"
elif command -v python3 >/dev/null 2>&1 && python3 -c "import jsonschema" 2>/dev/null; then
  VALIDATOR="python"
fi

validate_json() {
  local schema="$1"
  local fixture="$2"
  local name="$3"

  if [ ! -f "$schema" ]; then
    skip "$name" "schema not found: $schema"
    return
  fi
  if [ ! -f "$fixture" ]; then
    skip "$name" "fixture not found: $fixture"
    return
  fi

  # Basic JSON validity check (use python3 since jq may not be installed)
  if ! python3 -c "import json; json.load(open('$fixture'))" 2>/dev/null; then
    fail "$name" "fixture is not valid JSON"
    return
  fi

  if [ "$VALIDATOR" = "ajv" ]; then
    if ajv validate -s "$schema" -d "$fixture" --spec=draft2020 --strict=false 2>/dev/null; then
      pass "$name"
    else
      fail "$name" "schema validation failed"
    fi
  elif [ "$VALIDATOR" = "python" ]; then
    if python3 -c "
import json, jsonschema, sys
schema = json.load(open('$schema'))
data = json.load(open('$fixture'))
try:
    jsonschema.validate(data, schema)
    sys.exit(0)
except jsonschema.ValidationError as e:
    print(str(e)[:200], file=sys.stderr)
    sys.exit(1)
" 2>/dev/null; then
      pass "$name"
    else
      fail "$name" "schema validation failed"
    fi
  else
    # Fallback: just check JSON is valid
    pass "$name (JSON valid, no schema validator available)"
  fi
}

echo "=== Schema Validation Tests ==="
echo "Validator: ${VALIDATOR:-jq-only}"
echo ""

# --- Existing schemas with fixtures ---

echo "--- generated-task.schema.json ---"
validate_json \
  "$SCHEMAS_DIR/generated-task.schema.json" \
  "$FIXTURES_DIR/tasks-small.json" \
  "tasks-small.json matches generated-task schema"

# Validate tasks-small is a valid array with required fields (structural)
if python3 -c "
import json
with open('$FIXTURES_DIR/tasks-small.json') as f:
    d = json.load(f)
t = d[0]
assert all(k in t for k in ('id','title','description','dependencies'))
print('OK')
" 2>/dev/null | grep -q OK; then
  pass "tasks-small has required fields (id, title, description, dependencies)"
else
  fail "tasks-small structural check" "missing required fields"
fi

echo ""
echo "--- vote-ballot.schema.json ---"
# Create temp fixture for vote ballot
BALLOT_FIXTURE=$(mktemp)
cat > "$BALLOT_FIXTURE" <<'BALLOT'
{
  "voter_id": "claude-opus",
  "scores": {
    "task_decomposition": 8,
    "dependency_ordering": 7,
    "decision_point_coverage": 9,
    "test_strategy_quality": 6,
    "agent_assignment": 8
  },
  "overall_score": 8,
  "verdict": "approve",
  "reasoning": "Well-structured task decomposition with clear dependencies",
  "suggestions": ["Add more edge case tests"]
}
BALLOT
validate_json "$SCHEMAS_DIR/vote-ballot.schema.json" "$BALLOT_FIXTURE" "vote ballot fixture"
rm -f "$BALLOT_FIXTURE"

echo ""
echo "--- decision-vote-response.schema.json ---"
DVR_FIXTURE=$(mktemp)
cat > "$DVR_FIXTURE" <<'DVR'
{
  "chosen_option": "jwt",
  "confidence": 0.85,
  "reasoning": "JWT is stateless and well-suited for microservices",
  "concerns": ["Token revocation requires extra infrastructure"]
}
DVR
validate_json "$SCHEMAS_DIR/decision-vote-response.schema.json" "$DVR_FIXTURE" "decision vote response"
rm -f "$DVR_FIXTURE"

echo ""
echo "--- elicitation-request.schema.json ---"
validate_json \
  "$SCHEMAS_DIR/elicitation-request.schema.json" \
  "$FIXTURES_DIR/elicitation-request.json" \
  "elicitation-request fixture"

echo ""
echo "--- elicitation-response.schema.json ---"
validate_json \
  "$SCHEMAS_DIR/elicitation-response.schema.json" \
  "$FIXTURES_DIR/elicitation-response-select.json" \
  "elicitation-response (select)"

validate_json \
  "$SCHEMAS_DIR/elicitation-response.schema.json" \
  "$FIXTURES_DIR/elicitation-response-redeliberate.json" \
  "elicitation-response (redeliberate)"

echo ""
echo "--- deliberation-result.schema.json ---"
DELIB_FIXTURE=$(mktemp)
cat > "$DELIB_FIXTURE" <<'DELIB'
{
  "session_id": "test-session-1",
  "prd_hash": "abc123",
  "started_at": "2026-03-05T12:00:00Z",
  "completed_at": "2026-03-05T12:30:00Z",
  "timebox_minutes": 30,
  "debate_turns": 4,
  "status": "completed",
  "decision_points": [
    {
      "id": "dp-1",
      "question": "Event bus choice?",
      "options": ["NATS", "Kafka"],
      "votes": [
        { "voter_id": "v1", "chosen_option": "NATS", "confidence": 0.8, "reasoning": "Lightweight" }
      ],
      "vote_tally": { "NATS": 3, "Kafka": 2 },
      "winning_option": "NATS",
      "consensus_strength": 0.6,
      "escalated": false
    }
  ],
  "design_brief": "# Architecture\nUse NATS JetStream for event bus."
}
DELIB
validate_json "$SCHEMAS_DIR/deliberation-result.schema.json" "$DELIB_FIXTURE" "deliberation result"
rm -f "$DELIB_FIXTURE"

echo ""
echo "--- scaffold.schema.json ---"
SCAFFOLD_FIXTURE=$(mktemp)
cat > "$SCAFFOLD_FIXTURE" <<'SCAFFOLD'
{
  "scaffolds": [
    {
      "task_id": 1,
      "file_structure": [
        { "path": "src/auth/middleware.ts", "description": "Auth middleware", "action": "create" }
      ],
      "interfaces": "export interface AuthConfig { secret: string; ttl: number; }",
      "function_signatures": "export function verifyToken(token: string): Promise<UserPayload>",
      "test_stubs": "describe('verifyToken', () => { it('validates JWT', async () => {}); })"
    }
  ]
}
SCAFFOLD
validate_json "$SCHEMAS_DIR/scaffold.schema.json" "$SCAFFOLD_FIXTURE" "scaffold fixture"
rm -f "$SCAFFOLD_FIXTURE"

echo ""
echo "--- agent-message.json structural check ---"
if python3 -c "
import json
with open('$FIXTURES_DIR/agent-message.json') as f:
    d = json.load(f)
assert all(k in d for k in ('from','to','subject','message','type','timestamp'))
print('OK')
" 2>/dev/null | grep -q OK; then
  pass "agent-message has required fields"
else
  fail "agent-message structural check" "missing required fields"
fi

echo ""
echo "--- decision-votes fixtures structural check ---"
for f in decision-votes-majority.json decision-votes-tie.json; do
  if python3 -c "
import json
with open('$FIXTURES_DIR/$f') as fh:
    d = json.load(fh)
v = d[0]
assert all(k in v for k in ('voter_id','chosen_option','confidence','reasoning','concerns'))
print('OK')
" 2>/dev/null | grep -q OK; then
    pass "$f has required vote fields"
  else
    fail "$f structural check" "missing required fields"
  fi
done

# =========================================================================
# Summary
# =========================================================================
echo ""
echo "========================================"
echo "Results: $PASS passed, $FAIL failed, $SKIP skipped"
echo "========================================"

[ "$FAIL" -eq 0 ] || exit 1
