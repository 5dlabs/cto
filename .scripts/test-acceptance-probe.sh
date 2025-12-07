#!/bin/bash
# =============================================================================
# Test: Acceptance Criteria Probe
# Validates the acceptance criteria verification mechanism
# =============================================================================

# Note: Not using set -e so we can handle test failures

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DIR="$REPO_ROOT/.test-acceptance"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0

echo "════════════════════════════════════════════════════════════════"
echo "║ Acceptance Criteria Probe Test"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Setup
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR/task"

# Create the acceptance probe function for testing
# (Simplified version without Handlebars)
PROBE_RESULT_FILE="$TEST_DIR/.acceptance_result"

probe_acceptance_criteria() {
    local ACCEPTANCE_FILE="${CLI_WORK_DIR}/task/acceptance-criteria.md"
    
    # Initialize probe result
    echo "pending" > "$PROBE_RESULT_FILE"

    if [ ! -f "$ACCEPTANCE_FILE" ]; then
        echo "  ⚠️ No acceptance-criteria.md found - skipping verification"
        echo "skipped" > "$PROBE_RESULT_FILE"
        return 0
    fi

    # Count total criteria (lines starting with "- [ ]" or "- [x]")
    local TOTAL_CRITERIA
    TOTAL_CRITERIA=$(grep -cE '^\s*-\s*\[([ x])\]' "$ACCEPTANCE_FILE" 2>/dev/null) || TOTAL_CRITERIA=0
    local COMPLETED_CRITERIA
    COMPLETED_CRITERIA=$(grep -cE '^\s*-\s*\[x\]' "$ACCEPTANCE_FILE" 2>/dev/null) || COMPLETED_CRITERIA=0

    if [ "$TOTAL_CRITERIA" -eq 0 ]; then
        echo "  ⚠️ No checkable criteria found in acceptance file"
        echo "skipped" > "$PROBE_RESULT_FILE"
        return 0
    fi

    local COMPLETION_PCT=$((COMPLETED_CRITERIA * 100 / TOTAL_CRITERIA))

    echo "  ✓ Completed: $COMPLETED_CRITERIA / $TOTAL_CRITERIA ($COMPLETION_PCT%)"

    # Determine pass/fail
    local THRESHOLD="${ACCEPTANCE_THRESHOLD:-90}"
    
    if [ "$COMPLETION_PCT" -ge "$THRESHOLD" ]; then
        echo "  ✅ Acceptance criteria met ($COMPLETION_PCT% >= ${THRESHOLD}% threshold)"
        echo "passed" > "$PROBE_RESULT_FILE"
        return 0
    else
        echo "  ❌ Acceptance criteria NOT met ($COMPLETION_PCT% < ${THRESHOLD}% threshold)"
        echo "failed" > "$PROBE_RESULT_FILE"
        return 1
    fi
}

# Test 1: All criteria complete (100%)
test_all_complete() {
    echo -e "${BLUE}Test 1: All criteria complete (100%)${NC}"
    
    cat > "$TEST_DIR/task/acceptance-criteria.md" << 'EOF'
# Acceptance Criteria
- [x] Feature A implemented
- [x] Feature B implemented
- [x] Tests pass
- [x] Documentation updated
EOF
    
    CLI_WORK_DIR="$TEST_DIR"
    ACCEPTANCE_THRESHOLD=90
    
    if probe_acceptance_criteria; then
        echo -e "  ${GREEN}✓${NC} Correctly identified as PASSED"
        ((PASSED++))
    else
        echo -e "  ${RED}✗${NC} Should have passed but failed"
        ((FAILED++))
    fi
}

# Test 2: Partial completion (50%)
test_partial_complete() {
    echo -e "${BLUE}Test 2: Partial completion (50%)${NC}"
    
    cat > "$TEST_DIR/task/acceptance-criteria.md" << 'EOF'
# Acceptance Criteria
- [x] Feature A implemented
- [x] Feature B implemented
- [ ] Tests pass
- [ ] Documentation updated
EOF
    
    CLI_WORK_DIR="$TEST_DIR"
    ACCEPTANCE_THRESHOLD=90
    
    if probe_acceptance_criteria; then
        echo -e "  ${RED}✗${NC} Should have failed but passed"
        ((FAILED++))
    else
        echo -e "  ${GREEN}✓${NC} Correctly identified as FAILED (50% < 90%)"
        ((PASSED++))
    fi
}

# Test 3: Above threshold (90%)
test_above_threshold() {
    echo -e "${BLUE}Test 3: Above threshold (90%)${NC}"
    
    cat > "$TEST_DIR/task/acceptance-criteria.md" << 'EOF'
# Acceptance Criteria
- [x] Feature A implemented
- [x] Feature B implemented
- [x] Feature C implemented
- [x] Feature D implemented
- [x] Feature E implemented
- [x] Feature F implemented
- [x] Feature G implemented
- [x] Feature H implemented
- [x] Feature I implemented
- [ ] Feature J (nice to have)
EOF
    
    CLI_WORK_DIR="$TEST_DIR"
    ACCEPTANCE_THRESHOLD=90
    
    if probe_acceptance_criteria; then
        echo -e "  ${GREEN}✓${NC} Correctly identified as PASSED (90% >= 90%)"
        ((PASSED++))
    else
        echo -e "  ${RED}✗${NC} Should have passed but failed"
        ((FAILED++))
    fi
}

# Test 4: No acceptance file
test_no_file() {
    echo -e "${BLUE}Test 4: No acceptance file${NC}"
    
    rm -f "$TEST_DIR/task/acceptance-criteria.md"
    CLI_WORK_DIR="$TEST_DIR"
    
    if probe_acceptance_criteria; then
        echo -e "  ${GREEN}✓${NC} Correctly handled missing file (skipped)"
        ((PASSED++))
    else
        echo -e "  ${RED}✗${NC} Should have skipped but failed"
        ((FAILED++))
    fi
}

# Test 5: Empty file
test_empty_file() {
    echo -e "${BLUE}Test 5: Empty criteria file${NC}"
    
    cat > "$TEST_DIR/task/acceptance-criteria.md" << 'EOF'
# Acceptance Criteria

No checkboxes in this file.
EOF
    
    CLI_WORK_DIR="$TEST_DIR"
    
    if probe_acceptance_criteria; then
        echo -e "  ${GREEN}✓${NC} Correctly handled empty criteria (skipped)"
        ((PASSED++))
    else
        echo -e "  ${RED}✗${NC} Should have skipped but failed"
        ((FAILED++))
    fi
}

# Test 6: Nested criteria
test_nested_criteria() {
    echo -e "${BLUE}Test 6: Nested criteria structure${NC}"
    
    cat > "$TEST_DIR/task/acceptance-criteria.md" << 'EOF'
# Acceptance Criteria

## Required
- [x] Core feature
  - [x] Sub-feature A
  - [x] Sub-feature B
- [x] API endpoints

## Optional  
- [ ] Nice-to-have feature
EOF
    
    CLI_WORK_DIR="$TEST_DIR"
    ACCEPTANCE_THRESHOLD=80
    
    if probe_acceptance_criteria; then
        echo -e "  ${GREEN}✓${NC} Correctly handled nested criteria"
        ((PASSED++))
    else
        echo -e "  ${RED}✗${NC} Should have passed with 80% threshold"
        ((FAILED++))
    fi
}

# Run tests
echo ""
test_all_complete
echo ""
test_partial_complete
echo ""
test_above_threshold
echo ""
test_no_file
echo ""
test_empty_file
echo ""
test_nested_criteria

# Cleanup
rm -rf "$TEST_DIR"

# Summary
echo ""
echo "════════════════════════════════════════════════════════════════"
echo -e "║ Results: ${GREEN}$PASSED passed${NC}, ${RED}$FAILED failed${NC}"
echo "════════════════════════════════════════════════════════════════"

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi
exit 0

