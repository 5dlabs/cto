#!/bin/bash
set -eo pipefail

# =============================================================================
# Local End-to-End Test Framework
# Tests all job types across all CLIs with realistic inputs
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATES_DIR="$REPO_ROOT/agent-templates"
TEST_DIR="$REPO_ROOT/.test-e2e"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
PASSED=0
FAILED=0

echo "════════════════════════════════════════════════════════════════"
echo "║ Local End-to-End Test Framework"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Setup test directory
setup_test_dir() {
    rm -rf "$TEST_DIR"
    mkdir -p "$TEST_DIR/fixtures"
    mkdir -p "$TEST_DIR/output"
    mkdir -p "$TEST_DIR/task-files"
    echo -e "${BLUE}📁 Created test directory: $TEST_DIR${NC}"
}

# Create test fixtures for different job types
create_fixtures() {
    echo -e "${BLUE}📝 Creating test fixtures...${NC}"
    
    # Coder job fixture (like play parallel)
    mkdir -p "$TEST_DIR/fixtures/coder"
    cat > "$TEST_DIR/fixtures/coder/task.md" << 'EOF'
# Test Task: Add Hello World Endpoint

## Overview
Add a simple `/hello` endpoint that returns "Hello, World!"
EOF

    cat > "$TEST_DIR/fixtures/coder/prompt.md" << 'EOF'
You are implementing a simple REST endpoint.
Add a `/hello` endpoint to the API.
EOF

    # Healer job fixture
    mkdir -p "$TEST_DIR/fixtures/healer"
    cat > "$TEST_DIR/fixtures/healer/incident.md" << 'EOF'
# Incident: Service Unavailable
- Alert: HighErrorRate
- Service: api-gateway
- Symptoms: HTTP 500 errors spiking
EOF

    echo -e "${GREEN}✓ Created test fixtures${NC}"
}

# Test that system prompt exists for agent/job
test_system_prompt() {
    local agent=$1
    local job=$2
    local prompt_file="$TEMPLATES_DIR/$agent/$job/system-prompt.md.hbs"
    
    if [ -f "$prompt_file" ]; then
        echo -e "  ${GREEN}✓${NC} System prompt exists: $agent/$job"
        ((PASSED++))
        return 0
    else
        echo -e "  ${RED}✗${NC} Missing system prompt: $agent/$job"
        ((FAILED++))
        return 1
    fi
}

# Test that container symlink exists
test_container_symlink() {
    local agent=$1
    local job=$2
    local container_file="$TEMPLATES_DIR/$agent/$job/container.sh.hbs"
    
    if [ -L "$container_file" ] || [ -f "$container_file" ]; then
        echo -e "  ${GREEN}✓${NC} Container template: $agent/$job"
        ((PASSED++))
        return 0
    else
        echo -e "  ${RED}✗${NC} Missing container: $agent/$job"
        ((FAILED++))
        return 1
    fi
}

# Test partials exist
test_partials() {
    echo ""
    echo -e "${BLUE}Testing partials...${NC}"
    
    local partials=(
        "header.sh.hbs"
        "rust-env.sh.hbs"
        "go-env.sh.hbs"
        "node-env.sh.hbs"
        "config.sh.hbs"
        "github-auth.sh.hbs"
        "git-setup.sh.hbs"
        "task-files.sh.hbs"
        "acceptance-probe.sh.hbs"
        "completion.sh.hbs"
    )
    
    for partial in "${partials[@]}"; do
        if [ -f "$TEMPLATES_DIR/_shared/partials/$partial" ]; then
            echo -e "  ${GREEN}✓${NC} Partial: $partial"
            ((PASSED++))
        else
            echo -e "  ${RED}✗${NC} Missing partial: $partial"
            ((FAILED++))
        fi
    done
}

# Test Docker container (if available)
test_docker() {
    local cli=$1
    local image="ghcr.io/5dlabs/$cli:latest"
    
    if ! command -v docker &>/dev/null; then
        echo -e "  ${YELLOW}⚠${NC} Docker not available"
        return 0
    fi
    
    # Check if image exists
    if ! docker image inspect "$image" &>/dev/null 2>&1; then
        echo -e "  ${YELLOW}⚠${NC} Image not found: $cli (pulling...)"
        if ! docker pull --platform linux/amd64 "$image" &>/dev/null 2>&1; then
            echo -e "  ${YELLOW}⚠${NC} Could not pull: $cli"
            return 0
        fi
    fi
    
    # Run container test
    echo -e "  ${BLUE}Testing Docker: $cli${NC}"
    
    # Test 1: Container starts
    if docker run --rm --platform linux/amd64 "$image" echo "OK" &>/dev/null 2>&1; then
        echo -e "    ${GREEN}✓${NC} Container starts"
        ((PASSED++))
    else
        echo -e "    ${RED}✗${NC} Container failed to start"
        ((FAILED++))
        return 1
    fi
    
    # Test 2: Templates mount correctly
    if docker run --rm --platform linux/amd64 \
        -v "$TEMPLATES_DIR:/agent-templates:ro" \
        "$image" \
        test -f /agent-templates/_shared/container.sh.hbs &>/dev/null 2>&1; then
        echo -e "    ${GREEN}✓${NC} Templates mount"
        ((PASSED++))
    else
        echo -e "    ${RED}✗${NC} Templates mount failed"
        ((FAILED++))
    fi
    
    # Test 3: Partials accessible
    local partial_count
    partial_count=$(docker run --rm --platform linux/amd64 \
        -v "$TEMPLATES_DIR:/agent-templates:ro" \
        "$image" \
        bash -c "ls /agent-templates/_shared/partials/*.hbs 2>/dev/null | wc -l" 2>/dev/null || echo "0")
    
    if [ "$partial_count" -ge 7 ]; then
        echo -e "    ${GREEN}✓${NC} Partials accessible ($partial_count found)"
        ((PASSED++))
    else
        echo -e "    ${RED}✗${NC} Partials not accessible"
        ((FAILED++))
    fi
    
    # Test 4: Task files can be mounted
    cp -r "$TEST_DIR/fixtures/coder/"* "$TEST_DIR/task-files/" 2>/dev/null || true
    if docker run --rm --platform linux/amd64 \
        -v "$TEST_DIR/task-files:/task-files:ro" \
        "$image" \
        test -f /task-files/task.md &>/dev/null 2>&1; then
        echo -e "    ${GREEN}✓${NC} Task files mount"
        ((PASSED++))
    else
        echo -e "    ${RED}✗${NC} Task files mount failed"
        ((FAILED++))
    fi
}

# Main test runner
run_agent_tests() {
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "║ Testing Agent Templates"
    echo "════════════════════════════════════════════════════════════════"
    
    # Agent/job combinations to test
    local agents=(
        "rex:coder,healer"
        "blaze:coder,healer"
        "grizz:coder,healer"
        "nova:coder,healer,docs"
        "tap:coder,healer"
        "spark:coder,healer"
        "bolt:deploy,healer"
        "cipher:security,healer"
        "cleo:quality"
        "tess:test"
        "stitch:review"
        "morgan:pm,docs"
        "atlas:integration"
    )
    
    for entry in "${agents[@]}"; do
        local agent="${entry%%:*}"
        local jobs="${entry#*:}"
        
        echo ""
        echo -e "${BLUE}Agent: $agent${NC}"
        
        IFS=',' read -ra job_array <<< "$jobs"
        for job in "${job_array[@]}"; do
            test_system_prompt "$agent" "$job" || true
            test_container_symlink "$agent" "$job" || true
        done
    done
}

run_docker_tests() {
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "║ Docker Integration Tests"
    echo "════════════════════════════════════════════════════════════════"
    
    if ! command -v docker &>/dev/null; then
        echo -e "${YELLOW}⚠ Docker not available, skipping integration tests${NC}"
        return 0
    fi
    
    # Test available CLIs
    for cli in claude codex; do
        test_docker "$cli" || true
    done
}

# Print summary
print_summary() {
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo -e "║ Results: ${GREEN}$PASSED passed${NC}, ${RED}$FAILED failed${NC}"
    echo "════════════════════════════════════════════════════════════════"
    
    if [ "$FAILED" -gt 0 ]; then
        return 1
    fi
    return 0
}

# Cleanup
cleanup() {
    if [ "${KEEP_TEST_DIR:-}" != "1" ]; then
        rm -rf "$TEST_DIR"
        echo -e "${BLUE}🧹 Cleaned up test directory${NC}"
    fi
}

# Main
setup_test_dir
create_fixtures
test_partials
run_agent_tests
run_docker_tests
print_summary
cleanup
