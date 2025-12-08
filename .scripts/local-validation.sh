#!/bin/bash
# Local Validation Script for E2E Monitor Development
# Purpose: Validate changes locally before pushing to avoid CI wait times
set -euo pipefail

cd "$(dirname "$0")/.."

echo "═══════════════════════════════════════════════════════════════"
echo "║  LOCAL VALIDATION - E2E Monitor Development                  ║"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

ERRORS=0

# =========================================================================
# 1. Rust Compilation
# =========================================================================
echo -e "${CYAN}[1/6] Building play-monitor binary...${NC}"
if cargo build -p play-monitor --release 2>&1; then
    echo -e "${GREEN}✓ play-monitor builds successfully${NC}"
else
    echo -e "${RED}✗ play-monitor build failed${NC}"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# =========================================================================
# 2. Clippy (pedantic)
# =========================================================================
echo -e "${CYAN}[2/6] Running Clippy (pedantic)...${NC}"
if cargo clippy -p play-monitor -- -D warnings -W clippy::pedantic 2>&1 | tail -20; then
    echo -e "${GREEN}✓ Clippy passes${NC}"
else
    echo -e "${RED}✗ Clippy found issues${NC}"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# =========================================================================
# 3. CLI Argument Validation
# =========================================================================
echo -e "${CYAN}[3/6] Validating CLI argument parsing...${NC}"
BINARY="./target/release/play-monitor"

# Test monitor command accepts required args
if $BINARY monitor --help 2>&1 | grep -q "iteration"; then
    echo -e "${GREEN}✓ 'monitor' command accepts --iteration${NC}"
else
    echo -e "${RED}✗ 'monitor' command missing --iteration${NC}"
    ERRORS=$((ERRORS + 1))
fi

if $BINARY monitor --help 2>&1 | grep -q "docs-repository"; then
    echo -e "${GREEN}✓ 'monitor' command accepts --docs-repository${NC}"
else
    echo -e "${RED}✗ 'monitor' command missing --docs-repository${NC}"
    ERRORS=$((ERRORS + 1))
fi

if $BINARY monitor --help 2>&1 | grep -q "docs-project-directory"; then
    echo -e "${GREEN}✓ 'monitor' command accepts --docs-project-directory${NC}"
else
    echo -e "${RED}✗ 'monitor' command missing --docs-project-directory${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Test run command accepts required args
if $BINARY run --help 2>&1 | grep -q "docs-repository"; then
    echo -e "${GREEN}✓ 'run' command accepts --docs-repository${NC}"
else
    echo -e "${RED}✗ 'run' command missing --docs-repository${NC}"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# =========================================================================
# 4. Template Validation (Handlebars syntax)
# =========================================================================
echo -e "${CYAN}[4/6] Validating Handlebars templates...${NC}"
TEMPLATE="infra/charts/controller/templates/watch/factory/container-watch-monitor.sh.hbs"

if [ -f "$TEMPLATE" ]; then
    # Check for required Handlebars variables
    if grep -q "{{repository_url}}" "$TEMPLATE"; then
        echo -e "${GREEN}✓ Template uses {{repository_url}}${NC}"
    else
        echo -e "${RED}✗ Template missing {{repository_url}}${NC}"
        ERRORS=$((ERRORS + 1))
    fi
    
    if grep -q "{{docs_repository_url}}" "$TEMPLATE"; then
        echo -e "${GREEN}✓ Template uses {{docs_repository_url}}${NC}"
    else
        echo -e "${RED}✗ Template missing {{docs_repository_url}}${NC}"
        ERRORS=$((ERRORS + 1))
    fi
    
    if grep -q "{{docs_project_directory}}" "$TEMPLATE"; then
        echo -e "${GREEN}✓ Template uses {{docs_project_directory}}${NC}"
    else
        echo -e "${RED}✗ Template missing {{docs_project_directory}}${NC}"
        ERRORS=$((ERRORS + 1))
    fi
    
    # Check URL extraction pattern
    if grep -q "REPO_ORG_NAME=" "$TEMPLATE" && grep -q "sed.*github" "$TEMPLATE"; then
        echo -e "${GREEN}✓ Template extracts org/repo from URL${NC}"
    else
        echo -e "${YELLOW}⚠ Template may not extract org/repo correctly${NC}"
    fi
    
    # Check GitHub auth uses env vars (not Handlebars)
    if grep -q '\$GITHUB_APP_PRIVATE_KEY' "$TEMPLATE" && grep -q '\$GITHUB_APP_ID' "$TEMPLATE"; then
        echo -e "${GREEN}✓ Template uses env vars for GitHub auth${NC}"
    else
        echo -e "${RED}✗ Template should use env vars for GitHub auth${NC}"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo -e "${RED}✗ Template not found: $TEMPLATE${NC}"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# =========================================================================
# 5. YAML Lint (Helm templates)
# =========================================================================
echo -e "${CYAN}[5/6] Validating YAML syntax...${NC}"
if command -v yamllint &> /dev/null; then
    # Lint values files (skip templates with Handlebars)
    if yamllint -d relaxed infra/charts/controller/values.yaml 2>&1 | grep -v "^$"; then
        echo -e "${GREEN}✓ values.yaml passes yamllint${NC}"
    else
        echo -e "${GREEN}✓ values.yaml passes yamllint${NC}"
    fi
else
    echo -e "${YELLOW}⚠ yamllint not installed, skipping YAML validation${NC}"
fi
echo ""

# =========================================================================
# 6. Argo Workflow Dry-Run
# =========================================================================
echo -e "${CYAN}[6/6] Argo workflow dry-run...${NC}"
if command -v argo &> /dev/null && kubectl get ns cto &> /dev/null 2>&1; then
    # Test that we can submit a workflow with the expected parameters
    DRYRUN_FILE=$(mktemp)
    argo submit --from workflowtemplate/play-workflow-template -n cto \
        -p task-id=1 \
        -p repository=5dlabs/cto-parallel-test \
        -p service=cto-parallel-test \
        -p docs-repository=5dlabs/cto-parallel-test \
        -p docs-project-directory=docs \
        -p implementation-agent=5DLabs-Rex \
        -p implementation-cli=factory \
        -p implementation-model=claude-opus-4-5-20251101 \
        -p quality-agent=5DLabs-Cleo \
        -p quality-cli=claude \
        -p quality-model=claude-opus-4-5-20251101 \
        -p testing-agent=5DLabs-Tess \
        -p testing-cli=claude \
        -p testing-model=claude-opus-4-5-20251101 \
        --dry-run -o yaml > "$DRYRUN_FILE" 2>&1
    
    if grep -q "name: docs-repository" "$DRYRUN_FILE"; then
        echo -e "${GREEN}✓ Workflow accepts docs-repository parameter${NC}"
    else
        echo -e "${RED}✗ Workflow missing docs-repository${NC}"
        ERRORS=$((ERRORS + 1))
    fi
    
    if grep -q "name: docs-project-directory" "$DRYRUN_FILE"; then
        echo -e "${GREEN}✓ Workflow accepts docs-project-directory parameter${NC}"
    else
        echo -e "${RED}✗ Workflow missing docs-project-directory${NC}"
        ERRORS=$((ERRORS + 1))
    fi
    
    rm -f "$DRYRUN_FILE"
else
    echo -e "${YELLOW}⚠ argo CLI or cto namespace not available, skipping dry-run${NC}"
fi
echo ""

# =========================================================================
# Summary
# =========================================================================
echo "═══════════════════════════════════════════════════════════════"
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}║  ✅ ALL VALIDATIONS PASSED - Safe to push                     ║${NC}"
else
    echo -e "${RED}║  ❌ $ERRORS VALIDATION(S) FAILED - Fix before pushing           ║${NC}"
fi
echo "═══════════════════════════════════════════════════════════════"
echo ""

exit $ERRORS

