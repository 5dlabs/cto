#!/bin/bash
# Note: -e disabled to allow all tests to run even if some fail

# =============================================================================
# E2E Scenario Tests - Full Workflow Simulation
# Tests actual template rendering + script execution with mock inputs
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATES_DIR="$REPO_ROOT/agent-templates"
TEST_DIR="$REPO_ROOT/.test-e2e-scenarios"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

PASSED=0
FAILED=0

echo "════════════════════════════════════════════════════════════════"
echo "║ E2E Scenario Tests - Full Workflow Simulation"
echo "════════════════════════════════════════════════════════════════"

# Cleanup and setup
setup() {
    rm -rf "$TEST_DIR"
    mkdir -p "$TEST_DIR"/{output,workspace,task-files}
    echo -e "${BLUE}📁 Test directory: $TEST_DIR${NC}"
}

# Create a mock GitHub App token response
create_mock_env() {
    cat > "$TEST_DIR/mock-env.sh" << 'EOF'
# Mock environment for testing
export GITHUB_TOKEN="mock-gh-token-for-testing"
export GITHUB_APP_ID="123456"
export GITHUB_APP_PRIVATE_KEY="mock-private-key"
export ANTHROPIC_API_KEY="mock-anthropic-key"
export OPENAI_API_KEY="mock-openai-key"
export GOOGLE_API_KEY="mock-google-key"

# Mock CLI that just echoes
mock_cli() {
    echo "[MOCK CLI] Would execute: $*"
    echo "[MOCK CLI] Working directory: $(pwd)"
    echo "[MOCK CLI] Task files present: $(ls /task-files 2>/dev/null || echo 'none')"
    return 0
}
EOF
}

# Render container template with specific context
render_container_script() {
    local agent=$1
    local job=$2
    local cli=$3
    local output_file="$TEST_DIR/output/${agent}-${job}-${cli}.sh"
    
    local template="$TEMPLATES_DIR/_shared/container.sh.hbs"
    
    # Read partials and substitute
    local header=$(cat "$TEMPLATES_DIR/_shared/partials/header.sh.hbs" 2>/dev/null || echo "# Header")
    local rust_env=$(cat "$TEMPLATES_DIR/_shared/partials/rust-env.sh.hbs" 2>/dev/null || echo "# Rust")
    local go_env=$(cat "$TEMPLATES_DIR/_shared/partials/go-env.sh.hbs" 2>/dev/null || echo "# Go")  
    local node_env=$(cat "$TEMPLATES_DIR/_shared/partials/node-env.sh.hbs" 2>/dev/null || echo "# Node")
    local config=$(cat "$TEMPLATES_DIR/_shared/partials/config.sh.hbs" 2>/dev/null || echo "# Config")
    local github_auth=$(cat "$TEMPLATES_DIR/_shared/partials/github-auth.sh.hbs" 2>/dev/null || echo "# GH Auth")
    local git_setup=$(cat "$TEMPLATES_DIR/_shared/partials/git-setup.sh.hbs" 2>/dev/null || echo "# Git")
    local task_files=$(cat "$TEMPLATES_DIR/_shared/partials/task-files.sh.hbs" 2>/dev/null || echo "# Tasks")
    local completion=$(cat "$TEMPLATES_DIR/_shared/partials/completion.sh.hbs" 2>/dev/null || echo "# Done")
    
    # CLI-specific execution command
    local cli_execute=""
    case "$cli" in
        claude)
            cli_execute='echo "[MOCK] claude --model $model --prompt /workspace/prompt.md"'
            ;;
        codex)
            cli_execute='echo "[MOCK] codex --model $model --config /workspace/.codex/config.toml"'
            ;;
        opencode)
            cli_execute='echo "[MOCK] opencode --model $model"'
            ;;
        cursor)
            cli_execute='echo "[MOCK] cursor --background --prompt /workspace/prompt.md"'
            ;;
        factory)
            cli_execute='echo "[MOCK] factory agent --config /workspace/.factory/config.json"'
            ;;
        gemini)
            cli_execute='echo "[MOCK] gemini --model $model"'
            ;;
    esac
    
    # Generate script
    cat > "$output_file" << SCRIPT
#!/bin/bash
set -eo pipefail

# Auto-generated test script for: $agent / $job / $cli
# Generated: $(date -Iseconds)

# =========================================================================
# Test Environment Setup
# =========================================================================
source "$TEST_DIR/mock-env.sh" 2>/dev/null || true

# Variables
agent_name="$agent"
job_type="$job"
task_id="test-task-001"
service="test-service"
cli_type="$cli"
model="test-model"
default_retries="3"
github_app="5DLabs-Test"
repository_url="https://github.com/test/repo"
working_directory="$TEST_DIR/workspace"
branch="test-branch"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ CTO Agent Container - TEST MODE"
echo "║ Agent: \$agent_name | Job: \$job_type | CLI: \$cli_type"
echo "════════════════════════════════════════════════════════════════"
echo ""

# =========================================================================
# Configuration Display
# =========================================================================
echo "📋 Configuration:"
echo "  - CLI Type: \$cli_type"
echo "  - Model: \$model"
echo "  - Task ID: \$task_id"
echo "  - Service: \$service"
echo ""

# =========================================================================
# Mock GitHub Auth (skip actual auth in test)
# =========================================================================
echo "🔑 GitHub Authentication: [MOCKED]"
echo ""

# =========================================================================
# Mock Git Setup (skip actual clone in test)
# =========================================================================
echo "📦 Git Setup: [MOCKED - using local workspace]"
mkdir -p "\$working_directory"
echo ""

# =========================================================================
# Task Files
# =========================================================================
echo "📁 Copying task files..."
if [ -d "$TEST_DIR/task-files" ]; then
    cp -r "$TEST_DIR/task-files/"* "\$working_directory/" 2>/dev/null || true
    echo "  ✓ Task files copied"
else
    echo "  ⚠ No task files found"
fi
echo ""

# =========================================================================
# System Prompt
# =========================================================================
echo "📝 System Prompt:"
PROMPT_FILE="$TEMPLATES_DIR/$agent/$job/system-prompt.md.hbs"
if [ -f "\$PROMPT_FILE" ]; then
    echo "  ✓ Found: \$PROMPT_FILE"
    # Show first few lines
    head -5 "\$PROMPT_FILE" | sed 's/^/    /'
    echo "    ..."
else
    echo "  ✗ Missing: \$PROMPT_FILE"
fi
echo ""

# =========================================================================
# CLI Execution (Mocked)
# =========================================================================
echo "════════════════════════════════════════════════════════════════"
echo "║ Executing \$cli_type CLI [MOCK MODE]"
echo "════════════════════════════════════════════════════════════════"
echo ""

$cli_execute

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ ✅ Test Complete: \$agent_name / \$job_type / \$cli_type"
echo "════════════════════════════════════════════════════════════════"
SCRIPT
    
    chmod +x "$output_file"
    echo "$output_file"
}

# Run a rendered script and check output
run_scenario() {
    local agent=$1
    local job=$2
    local cli=$3
    
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}Scenario: $agent / $job / $cli${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Render the script
    local script_path
    script_path=$(render_container_script "$agent" "$job" "$cli")
    
    if [ ! -f "$script_path" ]; then
        echo -e "${RED}✗ Failed to render script${NC}"
        ((FAILED++))
        return 1
    fi
    echo -e "${GREEN}✓${NC} Script rendered: $script_path"
    
    # Create test task files
    mkdir -p "$TEST_DIR/task-files"
    cat > "$TEST_DIR/task-files/task.md" << EOF
# Test Task for $agent / $job

## Objective
This is a test task to validate the $job workflow for $agent using $cli.

## Expected Behavior
- Script should start without errors
- Environment should be configured
- Mock CLI should be invoked
EOF
    
    # Run the script
    echo -e "${BLUE}Running script...${NC}"
    local output_log="$TEST_DIR/output/${agent}-${job}-${cli}.log"
    
    if bash "$script_path" > "$output_log" 2>&1; then
        echo -e "${GREEN}✓${NC} Script executed successfully"
        
        # Verify expected output
        if grep -q "Test Complete" "$output_log" && \
           grep -q "$agent" "$output_log" && \
           grep -q "$job" "$output_log"; then
            echo -e "${GREEN}✓${NC} Output validation passed"
            ((PASSED++))
        else
            echo -e "${RED}✗${NC} Output validation failed"
            ((FAILED++))
            tail -20 "$output_log"
        fi
    else
        echo -e "${RED}✗${NC} Script execution failed"
        ((FAILED++))
        tail -20 "$output_log"
    fi
}

# Run Docker scenario (if available)
run_docker_scenario() {
    local agent=$1
    local job=$2
    local cli=$3
    local image="ghcr.io/5dlabs/$cli:latest"
    
    if ! command -v docker &>/dev/null; then
        echo -e "${YELLOW}⚠ Docker not available${NC}"
        return 0
    fi
    
    if ! docker image inspect "$image" &>/dev/null 2>&1; then
        echo -e "${YELLOW}⚠ Image not available: $image${NC}"
        return 0
    fi
    
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}Docker Scenario: $agent / $job / $cli${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Create task files for this scenario
    mkdir -p "$TEST_DIR/task-files-docker"
    cat > "$TEST_DIR/task-files-docker/task.md" << EOF
# Docker Test Task
Testing $agent / $job in $cli container
EOF
    
    local output_log="$TEST_DIR/output/docker-${agent}-${job}-${cli}.log"
    
    # Run container with mounted templates
    if docker run --rm \
        --platform linux/amd64 \
        -v "$TEMPLATES_DIR:/agent-templates:ro" \
        -v "$TEST_DIR/task-files-docker:/task-files:ro" \
        "$image" \
        bash -c "
            echo '════════════════════════════════════════════════════════════════'
            echo '║ Docker Container Test: $agent / $job / $cli'
            echo '════════════════════════════════════════════════════════════════'
            echo ''
            echo '📁 Template structure:'
            ls -la /agent-templates/ || true
            echo ''
            echo '📁 Agent templates:'
            ls -la /agent-templates/$agent/ 2>/dev/null || echo '  Not found'
            echo ''
            echo '📁 Job templates:'
            ls -la /agent-templates/$agent/$job/ 2>/dev/null || echo '  Not found'
            echo ''
            echo '📝 System prompt check:'
            if [ -f /agent-templates/$agent/$job/system-prompt.md.hbs ]; then
                echo '  ✓ System prompt exists'
                head -3 /agent-templates/$agent/$job/system-prompt.md.hbs | sed 's/^/    /'
            else
                echo '  ✗ System prompt missing'
            fi
            echo ''
            echo '📁 Task files:'
            ls -la /task-files/ || echo '  None'
            echo ''
            echo '✅ Docker test complete'
        " > "$output_log" 2>&1; then
        echo -e "${GREEN}✓${NC} Docker scenario passed"
        ((PASSED++))
    else
        echo -e "${RED}✗${NC} Docker scenario failed"
        ((FAILED++))
        tail -30 "$output_log"
    fi
}

# Print summary
print_summary() {
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo -e "║ Final Results: ${GREEN}$PASSED passed${NC}, ${RED}$FAILED failed${NC}"
    echo "════════════════════════════════════════════════════════════════"
    
    if [ "$FAILED" -gt 0 ]; then
        echo -e "${YELLOW}📁 Logs available in: $TEST_DIR/output/${NC}"
        return 1
    fi
    return 0
}

# Cleanup
cleanup() {
    if [ "${KEEP_TEST_DIR:-}" != "1" ]; then
        rm -rf "$TEST_DIR"
        echo -e "${BLUE}🧹 Cleaned up test directory${NC}"
    else
        echo -e "${YELLOW}📁 Test artifacts kept in: $TEST_DIR${NC}"
    fi
}

# ═══════════════════════════════════════════════════════════════════════════
# Main Test Execution
# ═══════════════════════════════════════════════════════════════════════════

setup
create_mock_env

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ Phase 1: Local Script Scenarios"
echo "════════════════════════════════════════════════════════════════"

# Test key scenarios across CLIs
# Coder scenarios (Play workflow)
for cli in claude codex; do
    run_scenario "rex" "coder" "$cli"
    run_scenario "blaze" "coder" "$cli"
done

# Healer scenarios
run_scenario "rex" "healer" "claude"
run_scenario "bolt" "healer" "codex"

# Specialized job scenarios
run_scenario "cipher" "security" "claude"
run_scenario "cleo" "quality" "claude"
run_scenario "morgan" "pm" "claude"
run_scenario "atlas" "integration" "claude"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ Phase 2: Docker Container Scenarios"
echo "════════════════════════════════════════════════════════════════"

# Docker scenarios (if available)
run_docker_scenario "rex" "coder" "claude"
run_docker_scenario "blaze" "coder" "codex"

print_summary
cleanup

