#!/bin/bash
set -euo pipefail

# =============================================================================
# Agent Image Integration Test
# Tests that container images work with the agent-templates structure
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATES_DIR="$REPO_ROOT/agent-templates"

# Images to test
IMAGES=(
    "ghcr.io/5dlabs/claude:latest"
    "ghcr.io/5dlabs/codex:latest"
)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "════════════════════════════════════════════════════════════════"
echo "║ Agent Image Integration Test"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Check Docker is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not found${NC}"
    exit 1
fi

# Check templates directory exists
if [ ! -d "$TEMPLATES_DIR" ]; then
    echo -e "${RED}❌ Templates directory not found: $TEMPLATES_DIR${NC}"
    exit 1
fi

echo "📁 Templates directory: $TEMPLATES_DIR"
echo ""

# Test each image
PASSED=0
FAILED=0

for IMAGE in "${IMAGES[@]}"; do
    echo "────────────────────────────────────────────────────────────────"
    echo "Testing: $IMAGE"
    echo "────────────────────────────────────────────────────────────────"
    
    # Pull image
    echo "  📥 Pulling image..."
    if ! docker pull "$IMAGE" 2>/dev/null; then
        echo -e "  ${YELLOW}⚠️  Could not pull $IMAGE (may need auth)${NC}"
        continue
    fi
    
    # Test 1: Verify container starts
    echo "  🔍 Test 1: Container starts..."
    if docker run --rm "$IMAGE" echo "hello" &>/dev/null; then
        echo -e "  ${GREEN}✓ Container starts${NC}"
    else
        echo -e "  ${RED}✗ Container failed to start${NC}"
        ((FAILED++))
        continue
    fi
    
    # Test 2: Verify templates can be mounted
    echo "  🔍 Test 2: Templates mount..."
    if docker run --rm \
        -v "$TEMPLATES_DIR:/agent-templates:ro" \
        "$IMAGE" \
        ls /agent-templates/_shared/container.sh.hbs &>/dev/null; then
        echo -e "  ${GREEN}✓ Templates mounted${NC}"
    else
        echo -e "  ${RED}✗ Templates mount failed${NC}"
        ((FAILED++))
        continue
    fi
    
    # Test 3: Verify partials exist
    echo "  🔍 Test 3: Partials exist..."
    PARTIALS_COUNT=$(docker run --rm \
        -v "$TEMPLATES_DIR:/agent-templates:ro" \
        "$IMAGE" \
        ls /agent-templates/_shared/partials/*.hbs 2>/dev/null | wc -l || echo "0")
    
    if [ "$PARTIALS_COUNT" -ge 7 ]; then
        echo -e "  ${GREEN}✓ Found $PARTIALS_COUNT partials${NC}"
    else
        echo -e "  ${RED}✗ Expected 7+ partials, found $PARTIALS_COUNT${NC}"
        ((FAILED++))
        continue
    fi
    
    # Test 4: Verify runtime is available
    echo "  🔍 Test 4: Runtime check..."
    CLI_NAME=$(echo "$IMAGE" | sed 's|.*/||' | sed 's|:.*||')
    
    case "$CLI_NAME" in
        claude)
            if docker run --rm "$IMAGE" which claude &>/dev/null; then
                echo -e "  ${GREEN}✓ Claude CLI available${NC}"
            else
                echo -e "  ${YELLOW}⚠️ Claude CLI not in PATH${NC}"
            fi
            ;;
        codex)
            if docker run --rm "$IMAGE" which codex &>/dev/null; then
                echo -e "  ${GREEN}✓ Codex CLI available${NC}"
            else
                echo -e "  ${YELLOW}⚠️ Codex CLI not in PATH${NC}"
            fi
            ;;
        *)
            echo -e "  ${YELLOW}⚠️ Unknown CLI: $CLI_NAME${NC}"
            ;;
    esac
    
    ((PASSED++))
    echo -e "  ${GREEN}✓ All tests passed for $IMAGE${NC}"
    echo ""
done

echo "════════════════════════════════════════════════════════════════"
echo "║ Results: $PASSED passed, $FAILED failed"
echo "════════════════════════════════════════════════════════════════"

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi

exit 0

