#!/bin/bash
# Collect output from a completed agent run
#
# Usage: ./collect-output.sh <agent> <cli>
# Example: ./collect-output.sh rex claude

set -euo pipefail

AGENT="${1:-}"
CLI="${2:-}"

if [[ -z "$AGENT" || -z "$CLI" ]]; then
    echo "Usage: $0 <agent> <cli>"
    exit 1
fi

REPO="5dlabs/cto-test-${AGENT}-${CLI}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="${SCRIPT_DIR}/../outputs/${AGENT}-${CLI}"

echo "═══════════════════════════════════════════════════════════════"
echo "  Collecting Output: ${AGENT} + ${CLI}"
echo "  Repository: ${REPO}"
echo "═══════════════════════════════════════════════════════════════"

mkdir -p "$OUTPUT_DIR"
mkdir -p "${OUTPUT_DIR}/code"

# Clone the repo
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"
gh repo clone "$REPO" . -- --quiet

echo ""
echo "Commits since reset:"
echo "────────────────────────────────────────────────────────────────"
git log --oneline -20

echo ""
echo "Files created:"
echo "────────────────────────────────────────────────────────────────"
find . -type f -not -path './.git/*' | sort

# Get the diff from the initial commit
INITIAL_COMMIT=$(git rev-list --max-parents=0 HEAD 2>/dev/null | tail -1)
if [[ -n "$INITIAL_COMMIT" ]]; then
    git diff "$INITIAL_COMMIT" HEAD > "${OUTPUT_DIR}/diff.patch" 2>/dev/null || true
    echo ""
    echo "Diff saved to: ${OUTPUT_DIR}/diff.patch"
fi

# Copy all files (except .git)
rsync -av --exclude='.git' . "${OUTPUT_DIR}/code/"

# Get file stats
FILE_COUNT=$(find "${OUTPUT_DIR}/code" -type f | wc -l | tr -d ' ')
TOTAL_SIZE=$(du -sh "${OUTPUT_DIR}/code" 2>/dev/null | cut -f1)

# Update test info
cat > "${OUTPUT_DIR}/test-info.json" << EOF
{
  "agent": "${AGENT}",
  "cli": "${CLI}",
  "repository": "${REPO}",
  "collected_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "status": "collected",
  "files_created": ${FILE_COUNT},
  "total_size": "${TOTAL_SIZE}"
}
EOF

cd - > /dev/null
rm -rf "$TEMP_DIR"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Output collected to: ${OUTPUT_DIR}"
echo "  Files: ${FILE_COUNT}"
echo "  Size: ${TOTAL_SIZE}"
echo "═══════════════════════════════════════════════════════════════"

