#!/bin/bash
# shellcheck disable=SC2329  # Setup functions are invoked indirectly via $setup_func
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════════════════
# CLI Integration Test Suite
# ═══════════════════════════════════════════════════════════════════════════════
# Tests actual CLI tool invocations to verify end-to-end functionality.
# Designed to run both locally and in CI environments.
#
# Environment variables:
#   ANTHROPIC_API_KEY  - Required for Claude tests
#   OPENAI_API_KEY     - Required for Codex/OpenAI tests
#   GOOGLE_API_KEY     - Required for Gemini tests
#   CI                 - Set to 'true' in CI environments (auto-detected)
#   TEST_TIMEOUT       - Timeout in seconds (default: 120)
#   SKIP_CLEANUP       - Set to 'true' to preserve test directory
# ═══════════════════════════════════════════════════════════════════════════════

# Colors for output (disabled in CI for cleaner logs)
if [[ "${CI:-false}" == "true" ]] || [[ ! -t 1 ]]; then
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    NC=''
else
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m'
fi

# Configuration
TEST_TIMEOUT="${TEST_TIMEOUT:-120}"
SKIP_CLEANUP="${SKIP_CLEANUP:-false}"

# Results tracking
declare -A RESULTS
declare -A ERRORS
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# ═══════════════════════════════════════════════════════════════════════════════
# Helper Functions
# ═══════════════════════════════════════════════════════════════════════════════

log_header() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}║ ${BOLD}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
}

log_section() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Testing: ${BOLD}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_skip() {
    echo -e "${YELLOW}○${NC} $1"
}

# Check if a command exists
command_exists() {
    command -v "$1" &> /dev/null || type "$1" &> /dev/null 2>&1
}

# Check if an environment variable is set and non-empty
env_var_set() {
    [[ -n "${!1:-}" ]]
}

# ═══════════════════════════════════════════════════════════════════════════════
# Test Functions
# ═══════════════════════════════════════════════════════════════════════════════

# Function to test a CLI tool
# Usage: test_cli "Name" "command" "api_key_var" "config_setup_function"
test_cli() {
    local cli_name=$1
    local cli_cmd=$2
    local api_key_var=$3
    local setup_func=${4:-""}
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    log_section "$cli_name"
    
    # Check if CLI exists
    local cli_binary
    cli_binary=$(echo "$cli_cmd" | awk '{print $1}')
    
    if ! command_exists "$cli_binary"; then
        log_skip "$cli_name CLI not found, skipping"
        RESULTS[$cli_name]="SKIPPED"
        ERRORS[$cli_name]="CLI not installed"
        SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
        return 0
    fi
    
    # Check if API key is available (if required)
    if [[ -n "$api_key_var" ]] && ! env_var_set "$api_key_var"; then
        log_skip "$cli_name: $api_key_var not set, skipping"
        RESULTS[$cli_name]="SKIPPED"
        ERRORS[$cli_name]="API key not configured"
        SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
        return 0
    fi
    
    # Run setup function if provided
    if [[ -n "$setup_func" ]]; then
        log_info "Running setup for $cli_name..."
        $setup_func
    fi
    
    # Reset main.py for this test
    cat > main.py << 'PYEOF'
# TODO: Add a function that adds two numbers
PYEOF
    
    log_info "Starting $cli_name test..."
    log_info "Command: $cli_cmd"
    
    # Run the CLI with timeout
    local exit_code=0
    local output_file="${cli_name,,}_output.log"
    
    set +e
    timeout "$TEST_TIMEOUT" bash -c "$cli_cmd" > "$output_file" 2>&1
    exit_code=$?
    set -e
    
    # Check results
    if grep -q "def add" main.py 2>/dev/null; then
        log_success "$cli_name: SUCCESS - function 'add' was created"
        echo "Generated code:"
        cat main.py
        RESULTS[$cli_name]="SUCCESS"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    elif [[ $exit_code -eq 124 ]]; then
        log_warning "$cli_name: TIMEOUT after ${TEST_TIMEOUT}s"
        RESULTS[$cli_name]="TIMEOUT"
        ERRORS[$cli_name]="Timed out after ${TEST_TIMEOUT}s"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    else
        log_error "$cli_name: FAILED - function 'add' not found in main.py"
        echo "Exit code: $exit_code"
        echo "main.py contents:"
        cat main.py
        if [[ -f "$output_file" ]]; then
            echo ""
            echo "CLI output (last 20 lines):"
            tail -20 "$output_file"
        fi
        RESULTS[$cli_name]="FAILED"
        ERRORS[$cli_name]="Expected function not generated (exit code: $exit_code)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# Setup Functions for Each CLI
# These functions are invoked indirectly via $setup_func in test_cli()
# ═══════════════════════════════════════════════════════════════════════════════

setup_claude() {
    mkdir -p .claude
    cat > .claude/config.json << 'EOF'
{
  "mcpServers": {}
}
EOF
    cat > .claude/settings.json << 'EOF'
{
  "permissions": {
    "mcp": {
      "tools": {
        "allow_all": true
      }
    }
  },
  "preferences": {
    "auto_approve_mcp": true
  }
}
EOF
    # Create CLAUDE.md symlink
    ln -sf AGENTS.md CLAUDE.md 2>/dev/null || cp AGENTS.md CLAUDE.md
}

setup_codex() {
    cat > codex.json << 'EOF'
{
  "instructions": ["AGENTS.md"]
}
EOF
}

setup_cursor() {
    mkdir -p .cursor/rules
    ln -sf ../../AGENTS.md .cursor/rules/agent.mdc 2>/dev/null || cp AGENTS.md .cursor/rules/agent.mdc
    cat > .cursor/mcp.json << 'EOF'
{
  "mcpServers": {}
}
EOF
}

setup_factory() {
    cat > droid.json << 'EOF'
{
  "instructions": ["AGENTS.md"],
  "permissions": {
    "allowAll": true
  }
}
EOF
}

setup_gemini() {
    mkdir -p .gemini
    cat > .gemini/config.json << 'EOF'
{
  "model": "gemini-2.5-pro",
  "tools": []
}
EOF
}

# ═══════════════════════════════════════════════════════════════════════════════
# Main Test Execution
# ═══════════════════════════════════════════════════════════════════════════════

main() {
    log_header "CLI Integration Test Suite"
    echo ""
    log_info "Timeout: ${TEST_TIMEOUT}s per test"
    log_info "CI Mode: ${CI:-false}"
    
    # Create a temp test directory
    TEST_DIR=$(mktemp -d)
    log_info "Test directory: $TEST_DIR"
    cd "$TEST_DIR"
    
    # Create a simple test file to work with
    cat > main.py << 'EOF'
# TODO: Add a function that adds two numbers
EOF
    
    # Create AGENTS.md as our base instruction file
    cat > AGENTS.md << 'EOF'
# Test Agent

You are a test agent. Your task is simple:
1. Read main.py
2. Add a function called `add` that takes two parameters and returns their sum
3. That's it - just add the function, nothing else
EOF
    
    # Simple prompt for the test
    PROMPT="Add a function called 'add' to main.py that takes two numbers and returns their sum. Only modify main.py, nothing else."
    
    # ═══════════════════════════════════════════════════════════════════════════
    # Run Tests
    # ═══════════════════════════════════════════════════════════════════════════
    
    # Test Claude
    test_cli "Claude" \
        "claude --print --dangerously-skip-permissions -p \"$PROMPT\"" \
        "ANTHROPIC_API_KEY" \
        "setup_claude"
    
    # Test Codex
    test_cli "Codex" \
        "codex --full-auto \"$PROMPT\"" \
        "OPENAI_API_KEY" \
        "setup_codex"
    
    # Test Cursor (usually interactive, may not work in CI)
    test_cli "Cursor" \
        "cursor --prompt \"$PROMPT\"" \
        "" \
        "setup_cursor"
    
    # Test Factory (droid)
    test_cli "Factory" \
        "droid --auto --prompt \"$PROMPT\"" \
        "OPENAI_API_KEY" \
        "setup_factory"
    
    # Test Gemini
    test_cli "Gemini" \
        "gemini --auto \"$PROMPT\"" \
        "GOOGLE_API_KEY" \
        "setup_gemini"
    
    # ═══════════════════════════════════════════════════════════════════════════
    # Summary
    # ═══════════════════════════════════════════════════════════════════════════
    
    log_header "TEST SUMMARY"
    echo ""
    
    for cli in "${!RESULTS[@]}"; do
        result=${RESULTS[$cli]}
        case $result in
            SUCCESS)
                log_success "$cli: $result"
                ;;
            SKIPPED)
                log_skip "$cli: $result (${ERRORS[$cli]:-})"
                ;;
            TIMEOUT)
                log_warning "$cli: $result"
                ;;
            *)
                log_error "$cli: $result"
                ;;
        esac
    done
    
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "Total: $TOTAL_TESTS | Passed: $PASSED_TESTS | Failed: $FAILED_TESTS | Skipped: $SKIPPED_TESTS"
    echo "═══════════════════════════════════════════════════════════════"
    
    # Output results in a format that can be parsed by CI
    if [[ "${CI:-false}" == "true" ]]; then
        echo ""
        echo "::group::Test Results JSON"
        echo "{"
        echo "  \"total\": $TOTAL_TESTS,"
        echo "  \"passed\": $PASSED_TESTS,"
        echo "  \"failed\": $FAILED_TESTS,"
        echo "  \"skipped\": $SKIPPED_TESTS,"
        echo "  \"results\": {"
        local first=true
        for cli in "${!RESULTS[@]}"; do
            if [[ "$first" == "true" ]]; then
                first=false
            else
                echo ","
            fi
            echo -n "    \"$cli\": \"${RESULTS[$cli]}\""
        done
        echo ""
        echo "  }"
        echo "}"
        echo "::endgroup::"
    fi
    
    # Cleanup
    if [[ "$SKIP_CLEANUP" != "true" ]]; then
        cd /
        rm -rf "$TEST_DIR"
        log_info "Test directory cleaned up"
    else
        echo ""
        log_warning "Test directory preserved at: $TEST_DIR"
        log_info "Run 'rm -rf $TEST_DIR' to clean up"
    fi
    
    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        exit 1
    fi
    exit 0
}

# Run main function
main "$@"
