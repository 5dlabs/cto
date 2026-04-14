#!/usr/bin/env bash
# generate-pr-body.sh — Build an implementation-focused PR description.
# Focuses on WHAT is being built, not HOW the pipeline works.
# Usage: generate-pr-body.sh <workspace_root> <project_name> > pr-body.md
set -euo pipefail

ROOT="${1:-.}"
PROJECT="${2:-intake}"
TASKS_JSON="$ROOT/.tasks/tasks/tasks.json"
DESIGN_BRIEF="$ROOT/.tasks/docs/design-brief.md"
ARCH_AUDIO="$ROOT/.tasks/audio/architecture-deliberation.mp3"
ARCH_TRANSCRIPT="$ROOT/.tasks/audio/architecture-deliberation.transcript.json"
ARCH_STATUS="$ROOT/.intake/audio/architecture-deliberation.status.json"
DESIGN_AUDIO="$ROOT/.tasks/audio/design-deliberation.mp3"
DESIGN_TRANSCRIPT="$ROOT/.tasks/audio/design-deliberation.transcript.json"
DESIGN_STATUS="$ROOT/.intake/audio/design-deliberation.status.json"

TASK_COUNT=0
SUBTASK_COUNT=0
PROMPT_COUNT=0

audio_status() {
  local mp3="$1"
  local status_file="$2"
  if [ -f "$mp3" ]; then
    printf 'ready'
    return
  fi
  if [ -f "$status_file" ]; then
    jq -r '.status // "pending"' "$status_file" 2>/dev/null || printf 'pending'
    return
  fi
  printf 'not started'
}

if [ -f "$TASKS_JSON" ]; then
  TASK_COUNT=$(jq 'length' "$TASKS_JSON" 2>/dev/null || echo 0)
  SUBTASK_COUNT=$(jq '[.[].subtasks | length] | add' "$TASKS_JSON" 2>/dev/null || echo 0)
fi

ARCH_AUDIO_STATUS="$(audio_status "$ARCH_AUDIO" "$ARCH_STATUS")"
DESIGN_AUDIO_STATUS="$(audio_status "$DESIGN_AUDIO" "$DESIGN_STATUS")"
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

# --- Design section ---
COMPONENT_LIB="$ROOT/.tasks/design/component-library.json"
CANDIDATES_JSON="$ROOT/.tasks/design/candidates.normalized.json"
SOURCE_SCREENSHOTS="$ROOT/.tasks/design/source-screenshots.json"
DESIGN_CONTEXT="$ROOT/.tasks/design/design-context.json"
FRAMER_RUN="$ROOT/.tasks/design/framer/framer-run.json"

if [ -f "$CANDIDATES_JSON" ] || [ -f "$COMPONENT_LIB" ]; then
  cat <<'DEOF'
---

### 🎨 Design System

DEOF

  # Screenshot
  if [ -f "$SOURCE_SCREENSHOTS" ]; then
    SCREENSHOT_MD=$(jq -r '.[] | select(.asset_url != null and .asset_url != "") | "<a href=\"\(.source_url)\"><img src=\"\(.asset_url)\" alt=\"\(.source_url)\" width=\"600\" /></a>"' "$SOURCE_SCREENSHOTS" 2>/dev/null || true)
    if [ -n "$SCREENSHOT_MD" ]; then
      printf '%s\n\n' "$SCREENSHOT_MD"
    fi
  fi

  # Provider status
  if [ -f "$CANDIDATES_JSON" ]; then
    printf '| Provider | Target | Status |\n'
    printf '|----------|--------|--------|\n'
    jq -r '.[] | "| \(.provider) | \(.target) | \(if .status == "generated" then "✅ generated" elif .status == "failed" then "❌ failed" else .status end) |"' "$CANDIDATES_JSON" 2>/dev/null
    printf '\n'
  fi

  # Design tokens (compact)
  if [ -f "$COMPONENT_LIB" ]; then
    HAS_TOKENS=$(jq '.tokens | keys | length' "$COMPONENT_LIB" 2>/dev/null || echo 0)
    if [ "$HAS_TOKENS" -gt 0 ]; then
      COLORS=$(jq -r '.tokens.color // [] | map("`\(.value)` \(.description // .name)") | join(" · ")' "$COMPONENT_LIB" 2>/dev/null)
      FONT=$(jq -r '.tokens.typography // [] | map("\(.value)") | join(", ")' "$COMPONENT_LIB" 2>/dev/null)
      if [ -n "$COLORS" ] || [ -n "$FONT" ]; then
        printf '**Tokens:** %s\n' "$COLORS"
        printf '**Typography:** %s\n\n' "$FONT"
      fi
    fi

    # Components summary
    PRIMS=$(jq -r '[.primitives[].name] | join(", ")' "$COMPONENT_LIB" 2>/dev/null)
    PATS=$(jq -r '[.patterns[].name] | join(", ")' "$COMPONENT_LIB" 2>/dev/null)
    if [ -n "$PRIMS" ] || [ -n "$PATS" ]; then
      printf '**Primitives:** %s\n' "${PRIMS:-none}"
      printf '**Patterns:** %s\n\n' "${PATS:-none}"
    fi
  fi

  # Framer link
  if [ -f "$FRAMER_RUN" ]; then
    FRAMER_URL=$(jq -r '.projectUrl // empty' "$FRAMER_RUN" 2>/dev/null)
    if [ -n "$FRAMER_URL" ]; then
      FRAMER_COMPONENTS=$(jq -r '[.framer_code_components[].name] | join(", ")' "$COMPONENT_LIB" 2>/dev/null || true)
      printf '🔗 **Framer Project:** [%s](%s)\n' "$FRAMER_URL" "$FRAMER_URL"
      if [ -n "$FRAMER_COMPONENTS" ]; then
        printf '**Framer Components:** %s\n' "$FRAMER_COMPONENTS"
      fi
      printf '\n'
    fi
  fi

  printf '<details><summary>Full design spec → <code>.tasks/design/DESIGN.md</code></summary>\n\n'
  printf 'See `DESIGN.md` in the `.tasks/design/` directory for the complete design system reference including tokens, component inventory, Framer code components with property controls, and variant details.\n\n'
  printf '</details>\n\n'
fi

cat <<EOF
---

### 🎧 Deliberation Audio

- Architecture audio: **${ARCH_AUDIO_STATUS}**
  - MP3: \`.tasks/audio/architecture-deliberation.mp3\`
  - Transcript: \`.tasks/audio/architecture-deliberation.transcript.json\`
- Design audio: **${DESIGN_AUDIO_STATUS}**
  - MP3: \`.tasks/audio/design-deliberation.mp3\`
  - Transcript: \`.tasks/audio/design-deliberation.transcript.json\`

> 💡 After the PR is created, the pipeline will attach a GitHub Release with downloadable MP3 files and comment the direct links on this PR.

EOF

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

# --- Harness dispatch table (from cto-config.json agentHarness mapping) ---
CTO_CONFIG="$ROOT/cto-config.json"
if [ -f "$CTO_CONFIG" ] && [ -f "$TASKS_JSON" ]; then
  HARNESS_MAP=$(jq -r '.defaults.play.agentHarness // empty' "$CTO_CONFIG" 2>/dev/null)
  if [ -n "$HARNESS_MAP" ]; then
    cat <<'HEOF'

---

### Harness Dispatch

| Task | Agent | CLI | Model | Fallback |
|------|-------|-----|-------|----------|
HEOF
    jq -r --argjson harness "$HARNESS_MAP" '
      .[] |
      ($harness[.agent] // $harness["_default"] // {primary:"claude",model:"claude-opus-4-6",fallback:"codex",fallbackModel:"gpt-5.2-codex"}) as $h |
      "| \(.id) | **\(.agent)** | \($h.primary) | `\($h.model)` | \($h.fallback // "-")/\($h.fallbackModel // "-") |"
    ' "$TASKS_JSON" 2>/dev/null || true
  fi
fi
