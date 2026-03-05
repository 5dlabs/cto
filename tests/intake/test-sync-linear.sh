#!/usr/bin/env bash
# Test sync-linear init + issues using the Alert Hub PRD.
#
# Prerequisites:
#   - LINEAR_API_KEY set in environment
#   - Node.js / npx available (or bun)
#   - intake-util built or runnable via `npx tsx`
#
# Usage:
#   LINEAR_API_KEY=lin_api_xxx ./tests/intake/test-sync-linear.sh [--issues]
#
# Without --issues: runs only init (creates project + PRD issue)
# With --issues:    runs init, then creates task issues from fixture data

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
UTIL_DIR="$REPO_ROOT/intake/util"
PRD_FILE="$SCRIPT_DIR/alerthub-e2e-test/prd.md"
TASK_FIXTURE="$REPO_ROOT/intake/tests/fixtures/tasks-medium.json"

# ── Preflight ────────────────────────────────────────────
if [ -z "${LINEAR_API_KEY:-}" ]; then
  echo "ERROR: LINEAR_API_KEY is not set" >&2
  exit 1
fi

if [ ! -f "$PRD_FILE" ]; then
  echo "ERROR: PRD file not found: $PRD_FILE" >&2
  exit 1
fi

# Determine how to run intake-util
if [ -f "$UTIL_DIR/intake-util" ]; then
  INTAKE_UTIL="$UTIL_DIR/intake-util"
elif command -v npx &>/dev/null; then
  INTAKE_UTIL="npx --prefix $UTIL_DIR tsx $UTIL_DIR/src/index.ts"
else
  echo "ERROR: No intake-util binary or npx available" >&2
  exit 1
fi

# Read team ID from cto-config.json
TEAM_ID=$(python3 -c "
import json
with open('$REPO_ROOT/cto-config.json') as f:
    c = json.load(f)
print(c.get('defaults',{}).get('linear',{}).get('teamId',''))
")

if [ -z "$TEAM_ID" ]; then
  echo "ERROR: Could not read linear.teamId from cto-config.json" >&2
  exit 1
fi

echo "═══════════════════════════════════════════════════════"
echo "  sync-linear test: Alert Hub PRD"
echo "  Team ID: $TEAM_ID"
echo "═══════════════════════════════════════════════════════"

# ── Phase 1: Init (project + PRD issue) ─────────────────
echo ""
echo "▶ Phase 1: sync-linear init"
echo "  PRD: $PRD_FILE"
echo ""

INIT_OUTPUT=$($INTAKE_UTIL sync-linear init \
  --project-name "alerthub-test-$(date +%s)" \
  --team-id "$TEAM_ID" \
  --prd-content "$PRD_FILE")

echo "$INIT_OUTPUT" | python3 -m json.tool

PROJECT_ID=$(echo "$INIT_OUTPUT" | python3 -c "import sys,json; print(json.load(sys.stdin)['projectId'])")
PRD_ISSUE_ID=$(echo "$INIT_OUTPUT" | python3 -c "import sys,json; print(json.load(sys.stdin)['prdIssueId'])")
PRD_IDENTIFIER=$(echo "$INIT_OUTPUT" | python3 -c "import sys,json; print(json.load(sys.stdin)['prdIdentifier'])")
AGENT_MAP=$(echo "$INIT_OUTPUT" | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)['agentMap']))")

echo ""
echo "✓ Project created:     $PROJECT_ID"
echo "✓ PRD issue created:   $PRD_IDENTIFIER ($PRD_ISSUE_ID)"
echo "✓ Agent map entries:   $(echo "$AGENT_MAP" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")"

# ── Phase 2: Issues (optional) ──────────────────────────
if [ "${1:-}" = "--issues" ]; then
  if [ ! -f "$TASK_FIXTURE" ]; then
    echo ""
    echo "WARNING: Task fixture not found: $TASK_FIXTURE" >&2
    echo "Skipping issues phase."
    exit 0
  fi

  echo ""
  echo "▶ Phase 2: sync-linear issues"
  echo "  Tasks: $TASK_FIXTURE"
  echo ""

  ISSUES_OUTPUT=$(cat "$TASK_FIXTURE" | $INTAKE_UTIL sync-linear issues \
    --project-id "$PROJECT_ID" \
    --prd-issue-id "$PRD_ISSUE_ID" \
    --team-id "$TEAM_ID" \
    --base-url "https://github.com/5dlabs/alerthub" \
    --agent-map "$AGENT_MAP")

  echo "$ISSUES_OUTPUT" | python3 -m json.tool

  ISSUE_COUNT=$(echo "$ISSUES_OUTPUT" | python3 -c "import sys,json; print(json.load(sys.stdin)['issueCount'])")
  echo ""
  echo "✓ Created $ISSUE_COUNT task issues (with subtasks)"
fi

echo ""
echo "═══════════════════════════════════════════════════════"
echo "  Test complete!"
echo "═══════════════════════════════════════════════════════"
