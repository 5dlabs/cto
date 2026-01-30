#!/usr/bin/env bash
# =========================================================================
# Acceptance Criteria Verification Probe
# Mirrors: templates/_shared/partials/acceptance-probe.sh.hbs
# =========================================================================

set -euo pipefail

ACCEPTANCE_FILE="${ACCEPTANCE_FILE:-/workspace/task/acceptance-criteria.md}"
PROBE_RESULT_FILE="${PROBE_RESULT_FILE:-/workspace/.acceptance_result}"
ACCEPTANCE_THRESHOLD="${ACCEPTANCE_THRESHOLD:-90}"

# Initialize probe result
echo "pending" > "$PROBE_RESULT_FILE"

probe_acceptance_criteria() {
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "║ Acceptance Criteria Verification"
    echo "════════════════════════════════════════════════════════════════"
    echo ""

    if [ ! -f "$ACCEPTANCE_FILE" ]; then
        echo "⚠️ No acceptance-criteria.md found at: $ACCEPTANCE_FILE"
        echo "skipped" > "$PROBE_RESULT_FILE"
        return 0
    fi

    echo "📋 Checking acceptance criteria from: $ACCEPTANCE_FILE"

    # Count total criteria (lines starting with "- [ ]" or "- [x]")
    TOTAL_CRITERIA=$(grep -cE '^\s*-\s*\[([ x])\]' "$ACCEPTANCE_FILE" 2>/dev/null) || TOTAL_CRITERIA=0
    COMPLETED_CRITERIA=$(grep -cE '^\s*-\s*\[x\]' "$ACCEPTANCE_FILE" 2>/dev/null) || COMPLETED_CRITERIA=0

    if [ "$TOTAL_CRITERIA" -eq 0 ]; then
        echo "⚠️ No checkable criteria found in acceptance file"
        echo "skipped" > "$PROBE_RESULT_FILE"
        return 0
    fi

    COMPLETION_PCT=$((COMPLETED_CRITERIA * 100 / TOTAL_CRITERIA))

    echo "  ✓ Completed: $COMPLETED_CRITERIA / $TOTAL_CRITERIA ($COMPLETION_PCT%)"
    echo ""

    # Show uncompleted criteria
    if [ "$COMPLETED_CRITERIA" -lt "$TOTAL_CRITERIA" ]; then
        echo "📝 Remaining criteria:"
        grep -E '^\s*-\s*\[ \]' "$ACCEPTANCE_FILE" | head -10 | while read -r line; do
            echo "  $line"
        done
        REMAINING=$((TOTAL_CRITERIA - COMPLETED_CRITERIA))
        if [ "$REMAINING" -gt 10 ]; then
            echo "  ... and $((REMAINING - 10)) more"
        fi
        echo ""
    fi

    # Determine pass/fail based on threshold (default 90%)
    if [ "$COMPLETION_PCT" -ge "$ACCEPTANCE_THRESHOLD" ]; then
        echo "✅ Acceptance criteria met ($COMPLETION_PCT% >= ${ACCEPTANCE_THRESHOLD}% threshold)"
        echo "passed" > "$PROBE_RESULT_FILE"
        return 0
    else
        echo "❌ Acceptance criteria NOT met ($COMPLETION_PCT% < ${ACCEPTANCE_THRESHOLD}% threshold)"
        echo "failed" > "$PROBE_RESULT_FILE"
        return 1
    fi
}

# Run if executed directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    probe_acceptance_criteria
    exit $?
fi
