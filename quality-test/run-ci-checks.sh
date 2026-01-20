#!/bin/bash
# Ralph CI Gate - Run before every push
# This mirrors what GitHub Actions will run

set -euo pipefail

cd "$(dirname "$0")/.."

echo "════════════════════════════════════════════════════════════════"
echo "   🔍 Ralph CI Gate - Pre-Push Checks"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Track failures
FAILED=0

# 1. Format check
echo "📐 Step 1/5: Checking code formatting..."
if cargo fmt --all --check; then
    echo "✅ Format check passed"
else
    echo "❌ Format check FAILED - run 'cargo fmt --all'"
    FAILED=1
fi
echo ""

# 2. Controller Clippy pedantic (this is what CI runs)
echo "🔍 Step 2/5: Clippy pedantic on controller..."
if cargo clippy -p controller --all-targets -- -D warnings -W clippy::pedantic 2>&1 | tee /tmp/clippy-controller.log | tail -5; then
    if grep -q "^error" /tmp/clippy-controller.log; then
        echo "❌ Controller Clippy pedantic FAILED"
        FAILED=1
    else
        echo "✅ Controller Clippy pedantic passed"
    fi
else
    echo "❌ Controller Clippy pedantic FAILED"
    FAILED=1
fi
echo ""

# 3. Healer Clippy pedantic
echo "🔍 Step 3/5: Clippy pedantic on healer..."
if cargo clippy -p healer --all-targets -- -D warnings -W clippy::pedantic 2>&1 | tee /tmp/clippy-healer.log | tail -5; then
    if grep -q "^error" /tmp/clippy-healer.log; then
        echo "❌ Healer Clippy pedantic FAILED"
        FAILED=1
    else
        echo "✅ Healer Clippy pedantic passed"
    fi
else
    echo "❌ Healer Clippy pedantic FAILED"
    FAILED=1
fi
echo ""

# 4. Full workspace Clippy (standard warnings)
echo "🔍 Step 4/5: Full workspace Clippy..."
if cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/clippy-all.log | tail -5; then
    if grep -q "^error" /tmp/clippy-all.log; then
        echo "❌ Workspace Clippy FAILED"
        FAILED=1
    else
        echo "✅ Workspace Clippy passed"
    fi
else
    echo "❌ Workspace Clippy FAILED"
    FAILED=1
fi
echo ""

# 5. Unit tests
echo "🧪 Step 5/5: Running unit tests..."
if cargo test --all --lib 2>&1 | tee /tmp/test.log | tail -10; then
    if grep -q "FAILED" /tmp/test.log; then
        echo "❌ Unit tests FAILED"
        FAILED=1
    else
        echo "✅ Unit tests passed"
    fi
else
    echo "❌ Unit tests FAILED"
    FAILED=1
fi
echo ""

# Summary
echo "════════════════════════════════════════════════════════════════"
if [ $FAILED -eq 0 ]; then
    echo "   ✅ ALL CI CHECKS PASSED - Safe to push"
    echo "════════════════════════════════════════════════════════════════"
    exit 0
else
    echo "   ❌ CI CHECKS FAILED - Do NOT push until fixed"
    echo "════════════════════════════════════════════════════════════════"
    exit 1
fi
