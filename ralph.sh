#!/bin/bash
# CTO Lite Ralph Loop
# Usage: ./ralph.sh [plan|build] [max_iterations]
#
# Examples:
#   ./ralph.sh              # Build mode, 20 iterations
#   ./ralph.sh plan         # Plan mode, 5 iterations  
#   ./ralph.sh build 50     # Build mode, 50 iterations
#   ./ralph.sh plan 3       # Plan mode, 3 iterations

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Parse arguments
MODE="build"
MAX_ITERATIONS=20

if [ "$1" = "plan" ]; then
    MODE="plan"
    MAX_ITERATIONS=${2:-5}
elif [ "$1" = "build" ]; then
    MODE="build"
    MAX_ITERATIONS=${2:-20}
elif [[ "$1" =~ ^[0-9]+$ ]]; then
    MAX_ITERATIONS=$1
fi

# Select prompt file based on mode
if [ "$MODE" = "plan" ]; then
    PROMPT_FILE="PROMPT_plan.md"
else
    PROMPT_FILE="PROMPT_build.md"
fi

# Verify prompt file exists
if [ ! -f "$PROMPT_FILE" ]; then
    echo "Error: $PROMPT_FILE not found"
    echo "Creating default prompt files..."
    
    # Create default prompts if missing
    if [ ! -f "PROMPT_plan.md" ]; then
        cat > PROMPT_plan.md << 'EOF'
# CTO Lite - Planning Mode

You are planning the next phase of CTO Lite implementation.

## Your Task

1. Read `docs/cto-lite.md` for the full plan
2. Read `TASKS.md` for the current task queue
3. Read `PROGRESS.md` for what's been completed
4. Read `AGENTS.md` for file boundaries and rules

## Output

Update `TASKS.md` with:
- Refined task breakdown for current phase
- Dependencies identified
- Estimated complexity

Update `PROGRESS.md` with:
- Planning session notes
- Decisions made
- Blockers identified

Do NOT implement anything. Planning only.
EOF
    fi
    
    if [ ! -f "PROMPT_build.md" ]; then
        cat > PROMPT_build.md << 'EOF'
# CTO Lite - Build Mode

You are implementing CTO Lite. Work autonomously until the task is complete.

## Your Task

1. Read `TASKS.md` - pick the FIRST unchecked task
2. Read `AGENTS.md` - respect file boundaries strictly
3. Read `PROGRESS.md` - understand current state
4. Implement the task completely
5. Verify with `cargo check` (for Rust) or `npm run typecheck` (for TypeScript)
6. Update `TASKS.md` - mark task complete with [x]
7. Update `PROGRESS.md` - log what you did
8. Commit your changes with a descriptive message

## Rules

- ONE task per iteration
- Stay within allowed paths (see AGENTS.md)
- Fork by copying, never modify existing CTO code
- If stuck after 3 attempts, log blocker and exit

## Backpressure Commands

```bash
# Rust
cd crates/cto-lite/tauri && cargo check

# TypeScript  
cd crates/cto-lite/ui && npm run typecheck
```

Exit when task is complete and committed.
EOF
    fi
fi

# Initialize progress file if needed
if [ ! -f "PROGRESS.md" ]; then
    cat > PROGRESS.md << EOF
# CTO Lite Progress Log

## Session Started: $(date)

---
EOF
fi

CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🔁 CTO Lite Ralph Loop"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Mode:       $MODE"
echo "  Prompt:     $PROMPT_FILE"
echo "  Branch:     $CURRENT_BRANCH"
echo "  Iterations: $MAX_ITERATIONS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Detect available CLI
CLI=""
if command -v claude &> /dev/null; then
    CLI="claude"
elif command -v codex &> /dev/null; then
    CLI="codex"
elif command -v amp &> /dev/null; then
    CLI="amp"
else
    echo "Error: No supported CLI found (claude, codex, amp)"
    echo "Install one of: claude-code, codex, amp"
    exit 1
fi

echo "Using CLI: $CLI"
echo ""

for i in $(seq 1 $MAX_ITERATIONS); do
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║  Iteration $i of $MAX_ITERATIONS                                           ║"
    echo "║  $(date '+%Y-%m-%d %H:%M:%S')                                      ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    
    # Log iteration start
    echo "" >> PROGRESS.md
    echo "### Iteration $i - $(date '+%Y-%m-%d %H:%M:%S')" >> PROGRESS.md
    
    # Run the agent
    if [ "$CLI" = "claude" ]; then
        # Claude Code
        OUTPUT=$(claude --dangerously-skip-permissions --print < "$PROMPT_FILE" 2>&1 | tee /dev/stderr) || true
    elif [ "$CLI" = "codex" ]; then
        # Codex
        OUTPUT=$(cat "$PROMPT_FILE" | codex --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
    else
        # Amp
        OUTPUT=$(cat "$PROMPT_FILE" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
    fi
    
    # Check for completion signals
    if echo "$OUTPUT" | grep -qi "all tasks complete\|phase complete\|no more tasks"; then
        echo ""
        echo "✅ Ralph detected completion signal!"
        echo "Completion detected at iteration $i" >> PROGRESS.md
        break
    fi
    
    # Check for blocker signals
    if echo "$OUTPUT" | grep -qi "blocker\|stuck\|cannot proceed"; then
        echo ""
        echo "⚠️  Ralph hit a blocker. Check PROGRESS.md for details."
        echo "Blocker hit at iteration $i" >> PROGRESS.md
        break
    fi
    
    # Small delay between iterations
    sleep 2
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Ralph loop completed"
echo "  Check PROGRESS.md for session log"
echo "  Check TASKS.md for remaining work"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
