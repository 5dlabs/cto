#!/bin/bash
# Run a single agent/CLI combination LOCALLY
# No Kubernetes required - uses locally installed CLIs
#
# Usage: ./run-local.sh <agent> <cli>
# Example: ./run-local.sh rex claude

set -euo pipefail

AGENT="${1:-}"
CLI="${2:-}"

if [[ -z "$AGENT" || -z "$CLI" ]]; then
    echo "Usage: $0 <agent> <cli>"
    echo "Example: $0 rex claude"
    echo ""
    echo "Available agents: rex, blaze, grizz, nova, tap, spark"
    echo "Available CLIs: claude, codex, cursor, factory, gemini, opencode"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"
SCENARIOS_FILE="${SCRIPT_DIR}/../../scenarios/scenarios.yaml"
TEMPLATES_DIR="${SCRIPT_DIR}/../../output/${AGENT}-${CLI}"
OUTPUT_DIR="${SCRIPT_DIR}/../outputs/${AGENT}-${CLI}"

echo "═══════════════════════════════════════════════════════════════"
echo "  Local Test: ${AGENT} + ${CLI}"
echo "═══════════════════════════════════════════════════════════════"

# Check if templates exist
if [[ ! -d "$TEMPLATES_DIR" ]]; then
    echo "Error: Templates not found at $TEMPLATES_DIR"
    echo "Run the template generation test first:"
    echo "  AGENT_TEMPLATES_PATH=\"\$(pwd)/templates\" cargo test -p controller --test agent_cli_matrix_tests generate_output -- --ignored"
    exit 1
fi

# Check if CLI is installed (handle aliases)
check_cli() {
    local cli_name="$1"
    # Check both command -v and type (for aliases)
    command -v "$cli_name" &>/dev/null || type "$cli_name" &>/dev/null 2>&1
}

CLI_CMD=""
case "$CLI" in
    claude)
        CLI_CMD="/Users/jonathonfritz/.claude/local/claude"  # Use direct path
        if [[ ! -x "$CLI_CMD" ]]; then
            CLI_CMD="claude"  # Fall back to command lookup
            if ! check_cli claude; then
                echo "Error: claude CLI not found"
                exit 1
            fi
        fi
        ;;
    codex)
        CLI_CMD="codex"
        if ! check_cli codex; then
            echo "Error: codex CLI not found"
            exit 1
        fi
        ;;
    cursor)
        CLI_CMD="cursor"
        if ! check_cli cursor; then
            echo "Error: cursor CLI not found"
            exit 1
        fi
        ;;
    factory)
        CLI_CMD="factory"
        if ! check_cli factory; then
            echo "Error: factory CLI not found"
            exit 1
        fi
        ;;
    gemini)
        CLI_CMD="gemini"
        if ! check_cli gemini; then
            echo "Error: gemini CLI not found"
            exit 1
        fi
        ;;
    opencode)
        CLI_CMD="/Users/jonathonfritz/.opencode/bin/opencode"  # Use direct path
        if [[ ! -x "$CLI_CMD" ]]; then
            CLI_CMD="opencode"
            if ! check_cli opencode; then
                echo "Error: opencode CLI not found"
                exit 1
            fi
        fi
        ;;
    *)
        echo "Error: Unknown CLI: $CLI"
        exit 1
        ;;
esac

echo "  CLI: $CLI_CMD ($(which $CLI_CMD))"

# Get task info
TASK_TITLE=$(python3 -c "
import yaml
with open('${SCENARIOS_FILE}', 'r') as f:
    scenarios = yaml.safe_load(f)
agent = '${AGENT}'
if agent in scenarios:
    print(scenarios[agent]['scenario']['title'])
else:
    print('Implement feature')
" 2>/dev/null || echo "Implement feature")

TASK_PROMPT=$(python3 -c "
import yaml
with open('${SCENARIOS_FILE}', 'r') as f:
    scenarios = yaml.safe_load(f)
agent = '${AGENT}'
if agent in scenarios:
    print(scenarios[agent]['scenario']['task_prompt'])
else:
    print('Implement a feature')
" 2>/dev/null || echo "Implement a feature")

echo "  Task: ${TASK_TITLE}"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"
mkdir -p "${OUTPUT_DIR}/code"
mkdir -p "${OUTPUT_DIR}/logs"

# Create a temporary workspace
WORKSPACE=$(mktemp -d)
echo "Step 1: Setting up workspace at ${WORKSPACE}..."

# Initialize git repo
cd "$WORKSPACE"
git init --quiet
git config user.email "test@5dlabs.io"
git config user.name "E2E Test"

# Create task directory structure
mkdir -p task
cat > task/task.md << TASKMD
# Task: ${TASK_TITLE}

${TASK_PROMPT}
TASKMD

# Copy the system prompt (memory file)
if [[ -f "${TEMPLATES_DIR}/CLAUDE.md" ]]; then
    cp "${TEMPLATES_DIR}/CLAUDE.md" ./CLAUDE.md
elif [[ -f "${TEMPLATES_DIR}/AGENTS.md" ]]; then
    cp "${TEMPLATES_DIR}/AGENTS.md" ./AGENTS.md
elif [[ -f "${TEMPLATES_DIR}/GEMINI.md" ]]; then
    cp "${TEMPLATES_DIR}/GEMINI.md" ./GEMINI.md
fi

# Copy CLI config
case "$CLI" in
    claude)
        mkdir -p .claude
        if [[ -f "${TEMPLATES_DIR}/settings.json" ]]; then
            cp "${TEMPLATES_DIR}/settings.json" .claude/settings.json
        fi
        ;;
    codex)
        if [[ -f "${TEMPLATES_DIR}/codex-config.toml" ]]; then
            cp "${TEMPLATES_DIR}/codex-config.toml" ./codex.toml
        fi
        ;;
    cursor)
        mkdir -p .cursor
        if [[ -f "${TEMPLATES_DIR}/cursor-cli-config.json" ]]; then
            cp "${TEMPLATES_DIR}/cursor-cli-config.json" .cursor/config.json
        fi
        ;;
    factory)
        mkdir -p .factory
        if [[ -f "${TEMPLATES_DIR}/factory-cli-config.json" ]]; then
            cp "${TEMPLATES_DIR}/factory-cli-config.json" .factory/config.json
        fi
        ;;
    gemini)
        mkdir -p .gemini
        if [[ -f "${TEMPLATES_DIR}/settings.json" ]]; then
            cp "${TEMPLATES_DIR}/settings.json" .gemini/settings.json
        fi
        ;;
    opencode)
        if [[ -f "${TEMPLATES_DIR}/opencode-config.json" ]]; then
            cp "${TEMPLATES_DIR}/opencode-config.json" ./opencode.json
        fi
        ;;
esac

# Initial commit
git add -A
git commit -m "Initial setup" --quiet

echo "  ✓ Workspace ready"
echo ""
echo "  Files:"
find . -type f -not -path './.git/*' | sort | sed 's/^/    /'
echo ""

# Step 2: Run the CLI
echo "Step 2: Running ${CLI} CLI (non-interactive mode)..."
echo "────────────────────────────────────────────────────────────────"

# Set timeout (15 minutes max)
TIMEOUT_SECONDS=900
EXIT_CODE=0

# Build the CLI command based on CLI type
# All CLIs run in non-interactive/print mode with timeout
case "$CLI" in
    claude)
        # Claude CLI: -p for non-interactive print mode
        echo "Running: claude -p \"<task>\" --permission-mode bypassPermissions"
        echo ""
        timeout $TIMEOUT_SECONDS $CLI_CMD \
            -p "$(cat task/task.md)" \
            --permission-mode bypassPermissions \
            --output-format text \
            2>&1 | tee "${OUTPUT_DIR}/logs/stdout.log" || EXIT_CODE=$?
        ;;
    codex)
        # Codex: non-interactive mode with --quiet and approval mode
        echo "Running: codex --approval-mode full-auto \"<task>\""
        echo ""
        timeout $TIMEOUT_SECONDS $CLI_CMD \
            --approval-mode full-auto \
            --quiet \
            "$(cat task/task.md)" \
            2>&1 | tee "${OUTPUT_DIR}/logs/stdout.log" || EXIT_CODE=$?
        ;;
    cursor)
        # Cursor: --prompt for non-interactive
        echo "Running: cursor agent --prompt \"<task>\""
        echo ""
        timeout $TIMEOUT_SECONDS $CLI_CMD agent \
            --prompt "$(cat task/task.md)" \
            2>&1 | tee "${OUTPUT_DIR}/logs/stdout.log" || EXIT_CODE=$?
        ;;
    factory)
        # Factory: batch mode
        echo "Running: factory --batch \"<task>\""
        echo ""
        timeout $TIMEOUT_SECONDS $CLI_CMD \
            --batch \
            "$(cat task/task.md)" \
            2>&1 | tee "${OUTPUT_DIR}/logs/stdout.log" || EXIT_CODE=$?
        ;;
    gemini)
        # Gemini: non-interactive with --sandbox none for file writes
        echo "Running: gemini -p \"<task>\" --sandbox none"
        echo ""
        timeout $TIMEOUT_SECONDS $CLI_CMD \
            -p "$(cat task/task.md)" \
            --sandbox none \
            2>&1 | tee "${OUTPUT_DIR}/logs/stdout.log" || EXIT_CODE=$?
        ;;
    opencode)
        # OpenCode: non-interactive mode
        echo "Running: opencode --non-interactive \"<task>\""
        echo ""
        timeout $TIMEOUT_SECONDS $CLI_CMD \
            --non-interactive \
            "$(cat task/task.md)" \
            2>&1 | tee "${OUTPUT_DIR}/logs/stdout.log" || EXIT_CODE=$?
        ;;
esac

echo "────────────────────────────────────────────────────────────────"
echo ""

# Check for errors
if [[ $EXIT_CODE -eq 124 ]]; then
    echo "⚠️  CLI timed out after ${TIMEOUT_SECONDS}s"
    echo "timeout" > "${OUTPUT_DIR}/logs/error.txt"
elif [[ $EXIT_CODE -ne 0 ]]; then
    echo "⚠️  CLI exited with code: $EXIT_CODE"
    echo "exit_code: $EXIT_CODE" > "${OUTPUT_DIR}/logs/error.txt"
else
    echo "✓ CLI completed successfully"
fi

# Step 3: Capture generated code
echo "Step 3: Capturing generated code..."

# Copy all files (excluding .git)
rsync -av --exclude='.git' . "${OUTPUT_DIR}/code/"

# Generate diff
git diff HEAD > "${OUTPUT_DIR}/code.patch" 2>/dev/null || true
git diff --stat HEAD 2>/dev/null || true

# Count new files
NEW_FILES=$(git status --porcelain | grep -c "^??" || echo "0")
MODIFIED_FILES=$(git status --porcelain | grep -c "^ M" || echo "0")

# Determine final status
FINAL_STATUS="success"
if [[ $EXIT_CODE -eq 124 ]]; then
    FINAL_STATUS="timeout"
elif [[ $EXIT_CODE -ne 0 ]]; then
    FINAL_STATUS="error"
fi

# Save metadata
cat > "${OUTPUT_DIR}/run-info.json" << EOF
{
  "agent": "${AGENT}",
  "cli": "${CLI}",
  "task_title": "${TASK_TITLE}",
  "workspace": "${WORKSPACE}",
  "completed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "exit_code": ${EXIT_CODE},
  "status": "${FINAL_STATUS}",
  "new_files": ${NEW_FILES},
  "modified_files": ${MODIFIED_FILES},
  "local_run": true
}
EOF

# Cleanup
cd - > /dev/null
# Keep workspace for inspection
# rm -rf "$WORKSPACE"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Results saved to: ${OUTPUT_DIR}"
echo ""
echo "  Workspace (for inspection): ${WORKSPACE}"
echo ""
echo "  Generated files:"
find "${OUTPUT_DIR}/code" -type f -not -name '.gitignore' | head -20 | sed 's|^|    |'
echo ""
echo "═══════════════════════════════════════════════════════════════"

