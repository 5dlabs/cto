#!/bin/bash

echo "🧪 EARLY HOOK TEST - PostToolUse triggered at $(date)"
echo "======================================================"

# Create a simple test file to verify hooks are working
HOOK_TEST_FILE=".hook-test-success"
HOOK_LOG_FILE=".hook-test.log"

# Log the hook trigger with timestamp
echo "$(date): Early hook triggered by tool: $CLAUDE_TOOL on file: $CLAUDE_FILE" >> "$HOOK_LOG_FILE"

# Basic environment check
echo "Working directory: $(pwd)"
echo "Tool used: ${CLAUDE_TOOL:-unknown}"
echo "File affected: ${CLAUDE_FILE:-unknown}"

# Create success indicator file
echo "Hook test successful at $(date)" > "$HOOK_TEST_FILE"
echo "Tool: ${CLAUDE_TOOL:-unknown}" >> "$HOOK_TEST_FILE"
echo "File: ${CLAUDE_FILE:-unknown}" >> "$HOOK_TEST_FILE"
echo "Working directory: $(pwd)" >> "$HOOK_TEST_FILE"

# Quick git status to see if files are being modified (non-blocking)
echo "Git status check:"
if git status --porcelain 2>/dev/null | head -5; then
    echo "Git status retrieved successfully"
else
    echo "Git status not available (this is okay)"
fi

echo ""
echo "✅ Hook test completed successfully!"
echo "✅ Created test file: $HOOK_TEST_FILE"
echo "✅ Hook system is working - continuing with docs generation..."
echo "=========================================================="