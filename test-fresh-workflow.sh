#!/bin/bash
set -e

echo "🧹 Cleaning up old workflows..."
kubectl delete workflows -n agent-platform -l parent-workflow=play-project-workflow-template-q955h --ignore-not-found=true

echo ""
echo "⏳ Waiting for cleanup..."
sleep 5

echo ""
echo "🚀 Starting fresh workflow with fixed image..."
echo ""
echo "The new workflow will use:"
echo "  - ✅ Ripgrep fix (Factory image rebuilt at 21:25)"
echo "  - ✅ Zero-commit detection (exit 1 if no commits)"
echo "  - ✅ Stale branch cleanup (recreates from closed PRs)"
echo ""
echo "Expected behavior:"
echo "  1. Branch setup detects closed PR #870"
echo "  2. Deletes stale branch and recreates fresh"
echo "  3. Factory makes commits (no ripgrep error)"
echo "  4. PR is created with new changes"
echo "  5. Cleo starts quality review"
echo ""
echo "Ready to start? (Ctrl+C to cancel)"
read -p "Press Enter to continue..."

# Start new workflow via MCP tool
echo ""
echo "Starting workflow..."
echo "Use: play({ task_id: 1, parallel_execution: true })"
