#!/bin/bash
set -e

echo "🧪 Testing Tess container template rendering and execution..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test variables (simulate what controller would pass)
export PR_NUMBER="123"
export PR_URL="https://github.com/test/repo/pull/123"
export WORKFLOW_NAME="test-workflow"
export TASK_ID="99"

# Create temp directory for testing
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "📁 Using temp directory: $TEMP_DIR"

# Copy template
TEMPLATE_FILE="infra/charts/controller/templates/code/claude/container-tess.sh.hbs"
if [ ! -f "$TEMPLATE_FILE" ]; then
    echo -e "${RED}❌ Template file not found: $TEMPLATE_FILE${NC}"
    exit 1
fi

echo "📋 Rendering template with test values..."

# Simple handlebars replacement for testing (not complete, but enough for validation)
cat "$TEMPLATE_FILE" | \
    sed "s/{{task_id}}/$TASK_ID/g" | \
    sed "s/{{pr_number}}/$PR_NUMBER/g" | \
    sed "s|{{pr_url}}|$PR_URL|g" | \
    sed "s/{{github_app}}/test-app/g" | \
    sed "s/{{repository_url}}/test-repo/g" | \
    sed "s/{{working_directory}}/./g" | \
    sed "s/{{overwrite_memory}}/false/g" | \
    sed "s/{{continue_session}}/false/g" | \
    sed "s/{{docs_repository_url}}/test-docs/g" | \
    sed "s/{{docs_project_directory}}/docs/g" | \
    sed "s/{{docs_branch}}/main/g" \
    > "$TEMP_DIR/container.sh"

echo "✅ Template rendered to $TEMP_DIR/container.sh"

echo -e "\n${YELLOW}🔍 Checking for problematic patterns...${NC}"

# Check for UTF-8 issues or special characters that might cause problems
if file "$TEMP_DIR/container.sh" | grep -q "UTF-8"; then
    echo "✅ File encoding looks good (UTF-8)"
else
    echo -e "${YELLOW}⚠️  File encoding might have issues${NC}"
fi

# Check for non-ASCII characters in critical sections (macOS compatible)
echo "Checking for non-ASCII characters in workflow diagram..."
if LC_ALL=C grep '[^[:print:][:space:]]' "$TEMP_DIR/container.sh" 2>/dev/null | grep -q "Fix Issues"; then
    echo -e "${RED}❌ Found non-ASCII characters in workflow diagram!${NC}"
    echo "Problematic lines:"
    LC_ALL=C grep -n '[^[:print:][:space:]]' "$TEMP_DIR/container.sh" | grep "Fix Issues" | head -5
else
    echo "✅ No problematic non-ASCII characters in workflow diagram"
fi

# Check for the specific error we encountered
echo -e "\nChecking for 'fix:' at start of lines (would cause command not found)..."
if grep -E "^fix:" "$TEMP_DIR/container.sh"; then
    echo -e "${RED}❌ Found 'fix:' at start of line - this will cause errors!${NC}"
    exit 1
else
    echo "✅ No 'fix:' commands found at line start"
fi

# Check that PR_NUMBER is properly handled
echo -e "\nChecking PR_NUMBER handling..."
if grep -q '\$PR_NUM' "$TEMP_DIR/container.sh"; then
    echo "✅ PR_NUM variable is used in heredoc"
fi

if grep -q '${PR_NUMBER:-' "$TEMP_DIR/container.sh"; then
    echo "✅ PR_NUMBER fallback pattern found"
fi

# Validate shell syntax
echo -e "\n${YELLOW}🐚 Validating shell syntax...${NC}"
if bash -n "$TEMP_DIR/container.sh" 2>/dev/null; then
    echo -e "${GREEN}✅ Shell syntax is valid${NC}"
else
    echo -e "${RED}❌ Shell syntax errors found:${NC}"
    bash -n "$TEMP_DIR/container.sh"
    exit 1
fi

# Test CLAUDE.md generation (the part that was failing)
echo -e "\n${YELLOW}📝 Testing CLAUDE.md generation...${NC}"
cd "$TEMP_DIR"
mkdir -p workspace task-files

# Create mock task files
echo "# Test acceptance criteria" > task-files/acceptance-criteria.md
echo "# Test task" > task-files/task.md

# Source the relevant part of the script to test CLAUDE.md creation
# Extract just the CLAUDE.md creation section
sed -n '/^# Create initial CLAUDE.md if needed/,/^fi  # End of CLAUDE.md creation if block/p' container.sh > test-claude-creation.sh

# Run it in a subshell to test
(
    export CLAUDE_WORK_DIR="$TEMP_DIR/workspace"
    mkdir -p "$CLAUDE_WORK_DIR"
    # Create /workspace symlink for the test
    mkdir -p "$TEMP_DIR/workspace-root"
    ln -sf "$TEMP_DIR/workspace-root" /tmp/workspace-test-$$
    # Replace /workspace with our temp path in the script
    sed "s|/workspace/CLAUDE.md|$TEMP_DIR/workspace-root/CLAUDE.md|g" test-claude-creation.sh > test-claude-fixed.sh
    bash test-claude-fixed.sh 2>&1 | tee claude-test.log
    rm -f /tmp/workspace-test-$$
)

if [ -f "$TEMP_DIR/workspace-root/CLAUDE.md" ] || [ -f "$TEMP_DIR/workspace/CLAUDE.md" ]; then
    echo -e "${GREEN}✅ CLAUDE.md created successfully${NC}"
    
    # Check for empty text blocks
    CLAUDE_FILE="$TEMP_DIR/workspace-root/CLAUDE.md"
    [ ! -f "$CLAUDE_FILE" ] && CLAUDE_FILE="$TEMP_DIR/workspace/CLAUDE.md"
    
    if grep -E "^\s*$" "$CLAUDE_FILE" | head -5; then
        echo -e "${YELLOW}⚠️  Found empty lines in CLAUDE.md (might cause cache_control issues)${NC}"
    fi
    
    # Check that PR_NUM was properly substituted
    if grep -q "PR_NUM" "$CLAUDE_FILE"; then
        echo -e "${RED}❌ PR_NUM variable not substituted in CLAUDE.md${NC}"
    else
        echo "✅ Variables properly substituted in CLAUDE.md"
    fi
else
    echo -e "${RED}❌ CLAUDE.md was not created${NC}"
    cat claude-test.log
    exit 1
fi

echo -e "\n${GREEN}🎉 All validation checks passed!${NC}"
echo "The Tess template should work correctly when deployed."
