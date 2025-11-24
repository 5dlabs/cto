#!/bin/bash
# Test script for Atlas Bugbot resolution feature
set -euo pipefail

echo "🧪 Testing Atlas Bugbot Resolution Feature"
echo "==========================================="

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name="$1"
    local test_cmd="$2"
    
    echo -n "Testing $test_name... "
    if eval "$test_cmd"; then
        echo -e "${GREEN}✓${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Test 1: Check if the container script exists
run_test "Atlas container script exists" \
    "[ -f 'infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs' ]"

# Test 2: Check if Bugbot detection logic is present
run_test "Bugbot detection logic present" \
    "grep -q 'BUGBOT_COMMENTS_JSON' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 3: Check if Claude integration is present
run_test "Claude CLI integration present" \
    "grep -q 'command -v claude' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 4: Check for fix request template
run_test "Bugbot fix request template present" \
    "grep -q 'Bugbot Issue Resolution Request' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 5: Check for error handling
run_test "Error count extraction present" \
    "grep -q 'HAS_ERRORS=.*grep -c \"🔴\"' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 6: Check for warning handling
run_test "Warning count extraction present" \
    "grep -q 'HAS_WARNINGS=.*grep -c \"🟡\"' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 7: Check for suggestion handling
run_test "Suggestion count extraction present" \
    "grep -q 'HAS_SUGGESTIONS=.*grep -c \"💡\"' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 8: Check for commit message generation
run_test "Automated commit message present" \
    "grep -q 'fix: address Bugbot feedback from comment' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 9: Check for success notification
run_test "Success notification present" \
    "grep -q 'Atlas: Bugbot Issues Resolved' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 10: Check for fallback when Claude unavailable
run_test "Claude fallback mechanism present" \
    "grep -q 'Claude CLI not available - falling back' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 11: Validate template syntax (basic check)
run_test "Template has valid Handlebars syntax" \
    "! grep -E '{{[^}]*{{' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 12: Check for GitHub API usage
run_test "GitHub API for comments present" \
    "grep -q 'gh api.*issues.*comments' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 13: Check for git operations
run_test "Git checkout operations present" \
    "grep -q 'git checkout.*PR_BRANCH' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 14: Check for push operations
run_test "Git push operations present" \
    "grep -q 'git push origin.*PR_BRANCH' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

# Test 15: Check JSON processing
run_test "JSON processing with jq present" \
    "grep -q 'jq -c' infra/charts/controller/agent-templates/code/integration/container-atlas.sh.hbs"

echo "==========================================="
echo -e "Tests Passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Tests Failed: ${RED}${TESTS_FAILED}${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi