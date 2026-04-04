#!/usr/bin/env bash
# generate-pr-body.sh — Build an implementation-focused PR description.
# Focuses on WHAT is being built, not HOW the pipeline works.
# Usage: generate-pr-body.sh <workspace_root> <project_name> > pr-body.md
set -euo pipefail

ROOT="${1:-.}"
PROJECT="${2:-intake}"
TASKS_JSON="$ROOT/.tasks/tasks/tasks.json"
DESIGN_BRIEF="$ROOT/.tasks/docs/design-brief.md"

TASK_COUNT=0
SUBTASK_COUNT=0
PROMPT_COUNT=0

if [ -f "$TASKS_JSON" ]; then
  TASK_COUNT=$(jq 'length' "$TASKS_JSON" 2>/dev/null || echo 0)
  SUBTASK_COUNT=$(jq '[.[].subtasks | length] | add' "$TASKS_JSON" 2>/dev/null || echo 0)
fi
if [ -d "$ROOT/.tasks" ]; then
  PROMPT_COUNT=$(find "$ROOT/.tasks/docs" -name "prompt.md" 2>/dev/null | wc -l | tr -d ' ')
fi

# --- Header: project name and goal ---
cat <<EOF
## ${PROJECT}

EOF

# Extract the Goal section from the design brief if available
if [ -f "$DESIGN_BRIEF" ]; then
  GOAL=$(sed -n '/^> ## Vision/,/^> ##/{ /^> ## Vision/d; /^> ##/d; /^>[[:space:]]*$/d; s/^> //; s/^>//; p; }' "$DESIGN_BRIEF" 2>/dev/null | head -8)
  # Fallback: try > ## Goal
  [ -z "$GOAL" ] && GOAL=$(sed -n '/^> ## Goal/,/^>/{ /^> ## Goal/d; /^>[[:space:]]*$/d; s/^> //; p; }' "$DESIGN_BRIEF" 2>/dev/null | head -8)
  if [ -n "$GOAL" ]; then
    printf '%s\n' "$GOAL"
    printf '\n'
  fi
fi

# --- Architecture decisions from design brief ---
if [ -f "$DESIGN_BRIEF" ]; then
  # Extract decision summaries
  DECISIONS=$(grep -E '^\*\*Decision\*\*:|^\*\*Decision:\*\*' "$DESIGN_BRIEF" 2>/dev/null | head -8)
  if [ -n "$DECISIONS" ]; then
    cat <<'DECEOF'
---

### Architecture Decisions

| Question | Decision |
|----------|----------|
DECEOF
    # Build a table from the [D1]...[DN] sections
    awk '
      /^### \[D[0-9]+\]/ {
        q = $0
        sub(/^### \[D[0-9]+\] /, "", q)
        sub(/ — ESCALATED$/, "", q)
      }
      /^\*\*Decision\*\*:/ || /^\*\*Decision:\*\*/ {
        d = $0
        sub(/\*\*Decision\*\*: ?/, "", d)
        sub(/\*\*Decision:\*\* ?/, "", d)
        if (q != "") print "| " q " | " d " |"
      }
    ' "$DESIGN_BRIEF" 2>/dev/null
    printf '\n'
  fi
fi

# --- Task breakdown grouped by agent ---
cat <<'EOF'
---

### What's Being Built

EOF

if [ -f "$TASKS_JSON" ]; then
  # Group tasks by agent, emit a section per agent
  jq -r '
    group_by(.agent) | sort_by(-length) | .[] |
    "#### " + .[0].agent + "\n\n" +
    ([.[] |
      "**\(.id). \(.title | split(" (")[0])** — \((.subtasks // []) | length) subtasks\n" +
      ((.subtasks // [])[:6] | map("- " + .title) | join("\n")) +
      (if ((.subtasks // []) | length) > 6 then "\n- _...and \(((.subtasks // []) | length) - 6) more_" else "" end) +
      "\n"
    ] | join("\n"))
  ' "$TASKS_JSON" 2>/dev/null || true
fi

# --- Implementation prompts structure ---
cat <<EOF
---

### Implementation Prompts

Each task directory contains the full implementation spec:

\`\`\`
.tasks/docs/task-{N}/
├── prompt.md        # Implementation instructions for the assigned agent
├── acceptance.md    # Acceptance criteria and verification steps
├── task.md          # Full task description with context
├── decisions.md     # Relevant architecture decisions
└── subtasks/
    └── task-{N}.{M}/
        └── prompt.md  # Subtask-level implementation prompt
\`\`\`

**${TASK_COUNT} tasks · ${SUBTASK_COUNT} subtasks · ${PROMPT_COUNT} implementation prompts**

---

### Assigned Agents

| Agent | Scope |
|-------|-------|
EOF

# Agent summary table
if [ -f "$TASKS_JSON" ]; then
  jq -r '
    group_by(.agent) | sort_by(-length) | .[] |
    "| **\(.[0].agent)** | Tasks " +
    ([.[].id | tostring] | join(", ")) +
    " — " +
    ([.[].title | split(" (")[0] | split(":")[0]] | join("; ")) +
    " |"
  ' "$TASKS_JSON" 2>/dev/null || true
fi
