#!/bin/bash

# =============================================================================
# Master E2E Test Runner
# Runs all end-to-end tests: Rust tests, shell scenarios, and Docker tests
# =============================================================================

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo ""
echo "════════════════════════════════════════════════════════════════════════════"
echo "║                     CTO E2E Test Suite                                   ║"
echo "════════════════════════════════════════════════════════════════════════════"
echo ""

cd "$REPO_ROOT"

# Track results
RUST_RESULT=0
BASIC_RESULT=0
SCENARIO_RESULT=0

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 1: Rust Template Validation Tests
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║ Phase 1: Rust Template Validation Tests                                   ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

if cargo test -p controller --test e2e_template_tests 2>&1; then
    echo -e "${GREEN}✓ Rust template tests passed${NC}"
else
    echo -e "${RED}✗ Rust template tests failed${NC}"
    RUST_RESULT=1
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 2: Basic Structure Tests
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║ Phase 2: Basic Structure Tests                                            ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

if "$SCRIPT_DIR/test-e2e-local.sh" 2>&1; then
    echo -e "${GREEN}✓ Basic structure tests passed${NC}"
else
    echo -e "${RED}✗ Basic structure tests failed${NC}"
    BASIC_RESULT=1
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 3: Full Scenario Tests
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║ Phase 3: Full Scenario Tests                                              ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

if "$SCRIPT_DIR/test-e2e-scenarios.sh" 2>&1; then
    echo -e "${GREEN}✓ Scenario tests passed${NC}"
else
    echo -e "${RED}✗ Scenario tests failed${NC}"
    SCENARIO_RESULT=1
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "════════════════════════════════════════════════════════════════════════════"
echo "║                           Test Summary                                   ║"
echo "════════════════════════════════════════════════════════════════════════════"
echo ""

TOTAL_FAILED=$((RUST_RESULT + BASIC_RESULT + SCENARIO_RESULT))

if [ $RUST_RESULT -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} Rust Template Tests"
else
    echo -e "  ${RED}✗${NC} Rust Template Tests"
fi

if [ $BASIC_RESULT -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} Basic Structure Tests"
else
    echo -e "  ${RED}✗${NC} Basic Structure Tests"
fi

if [ $SCENARIO_RESULT -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} Full Scenario Tests"
else
    echo -e "  ${RED}✗${NC} Full Scenario Tests"
fi

echo ""

if [ $TOTAL_FAILED -eq 0 ]; then
    echo -e "${GREEN}════════════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}║                    ✅ ALL E2E TESTS PASSED                               ║${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════════════════════════${NC}"
    exit 0
else
    echo -e "${RED}════════════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}║                    ❌ $TOTAL_FAILED TEST SUITE(S) FAILED                            ║${NC}"
    echo -e "${RED}════════════════════════════════════════════════════════════════════════════${NC}"
    exit 1
fi

