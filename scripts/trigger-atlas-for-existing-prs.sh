#!/bin/bash
# Trigger Atlas PR Guardian for existing open PRs that were missed due to the sensor bug

set -euo pipefail

REPO="5dlabs/cto"
DRY_RUN="${DRY_RUN:-false}"

echo "🔄 Atlas PR Guardian Remediation Script"
echo "========================================"
echo "Repository: $REPO"
echo "Dry Run: $DRY_RUN"
echo

# Get all open PRs
echo "📋 Fetching open PRs..."
OPEN_PRS=$(gh pr list --repo "$REPO" --json number,title,author,createdAt,mergeable --limit 100)
PR_COUNT=$(echo "$OPEN_PRS" | jq '. | length')

if [ "$PR_COUNT" -eq 0 ]; then
    echo "✅ No open PRs found"
    exit 0
fi

echo "Found $PR_COUNT open PR(s)"
echo

# Function to check if Atlas CodeRun exists for a PR
check_atlas_coderun() {
    local pr_number=$1
    kubectl get coderun -n agent-platform -l "agent=atlas,pr-number=$pr_number" --no-headers 2>/dev/null | wc -l | tr -d ' '
}

# Function to trigger Atlas by adding a comment
trigger_atlas() {
    local pr_number=$1
    local pr_title=$2
    
    if [ "$DRY_RUN" = "true" ]; then
        echo "  [DRY RUN] Would comment on PR #$pr_number to trigger Atlas"
        return 0
    fi
    
    # Add a comment to trigger the issue_comment webhook
    gh pr comment "$pr_number" --repo "$REPO" --body "🤖 **Atlas PR Guardian Activation**

This PR was opened before the Atlas sensor fix was deployed. Adding this comment to trigger Atlas activation.

Atlas will now:
- ✅ Monitor for Bugbot comments
- ✅ Watch CI status
- ✅ Check for merge conflicts
- ✅ Auto-merge when all criteria are met

_This is an automated remediation comment._" || {
        echo "  ❌ Failed to comment on PR #$pr_number"
        return 1
    }
    
    echo "  ✅ Triggered Atlas for PR #$pr_number"
}

# Process each PR
echo "🔍 Checking PRs for Atlas activation..."
echo

TRIGGERED_COUNT=0
ALREADY_ACTIVE_COUNT=0
FAILED_COUNT=0

while IFS= read -r pr; do
    PR_NUMBER=$(echo "$pr" | jq -r '.number')
    PR_TITLE=$(echo "$pr" | jq -r '.title')
    PR_AUTHOR=$(echo "$pr" | jq -r '.author.login')
    PR_CREATED=$(echo "$pr" | jq -r '.createdAt')
    PR_MERGEABLE=$(echo "$pr" | jq -r '.mergeable')
    
    echo "PR #$PR_NUMBER: $PR_TITLE"
    echo "  Author: $PR_AUTHOR"
    echo "  Created: $PR_CREATED"
    echo "  Mergeable: $PR_MERGEABLE"
    
    # Check if Atlas CodeRun exists
    CODERUN_COUNT=$(check_atlas_coderun "$PR_NUMBER")
    
    if [ "$CODERUN_COUNT" -gt 0 ]; then
        echo "  ✅ Atlas already active ($CODERUN_COUNT CodeRun(s) found)"
        ALREADY_ACTIVE_COUNT=$((ALREADY_ACTIVE_COUNT + 1))
    else
        echo "  ⚠️  No Atlas CodeRun found - triggering activation..."
        if trigger_atlas "$PR_NUMBER" "$PR_TITLE"; then
            TRIGGERED_COUNT=$((TRIGGERED_COUNT + 1))
        else
            FAILED_COUNT=$((FAILED_COUNT + 1))
        fi
    fi
    
    echo
done < <(echo "$OPEN_PRS" | jq -c '.[]')

# Summary
echo "========================================"
echo "📊 Remediation Summary"
echo "========================================"
echo "Total PRs checked: $PR_COUNT"
echo "Already active: $ALREADY_ACTIVE_COUNT"
echo "Triggered: $TRIGGERED_COUNT"
echo "Failed: $FAILED_COUNT"
echo

if [ "$DRY_RUN" = "true" ]; then
    echo "⚠️  This was a DRY RUN - no changes were made"
    echo "Run with DRY_RUN=false to actually trigger Atlas"
else
    echo "✅ Remediation complete!"
    echo
    echo "Next steps:"
    echo "1. Wait 30-60 seconds for Atlas CodeRuns to be created"
    echo "2. Verify with: kubectl get coderun -n agent-platform -l agent=atlas"
    echo "3. Monitor sensor logs: kubectl logs -f \$(kubectl get pods -n argo -l sensor-name=atlas-pr-guardian -o name | head -1) -n argo"
fi

