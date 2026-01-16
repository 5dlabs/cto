#!/usr/bin/env bash
# Clear all GitHub Actions workflow runs from the repository
# Usage: ./scripts/clear-all-workflow-runs.sh [--dry-run]
#
# This script deletes ALL workflow runs from the beginning of time.
# Run with --dry-run first to see what would be deleted.

set -euo pipefail

DRY_RUN=false
BATCH_SIZE=100
REPO=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --repo)
            REPO="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--dry-run] [--repo owner/repo]"
            exit 1
            ;;
    esac
done

# Get repo from gh if not specified
if [[ -z "$REPO" ]]; then
    REPO=$(gh repo view --json nameWithOwner -q '.nameWithOwner')
fi

echo "Repository: $REPO"
echo "Dry run: $DRY_RUN"
echo ""

# Count total runs first
echo "Counting workflow runs..."
TOTAL=$(gh run list --repo "$REPO" --limit 1 --json databaseId -q 'length' 2>/dev/null || echo "0")

if [[ "$TOTAL" == "0" ]]; then
    # Try to get actual count by fetching more
    TOTAL=$(gh api "repos/$REPO/actions/runs" --jq '.total_count' 2>/dev/null || echo "0")
fi

echo "Total workflow runs to delete: $TOTAL"
echo ""

if [[ "$TOTAL" == "0" ]]; then
    echo "No workflow runs found. Nothing to delete."
    exit 0
fi

if [[ "$DRY_RUN" == "true" ]]; then
    echo "DRY RUN - Would delete $TOTAL workflow runs"
    echo ""
    echo "Sample of runs that would be deleted:"
    gh run list --repo "$REPO" --limit 10 --json databaseId,displayTitle,status,conclusion,createdAt \
        --template '{{range .}}ID: {{.databaseId}} | {{.displayTitle}} | {{.status}} | {{.createdAt}}{{"\n"}}{{end}}'
    echo ""
    echo "Run without --dry-run to actually delete."
    exit 0
fi

echo "Starting deletion..."
echo "This may take a while for large histories."
echo "Progress will be logged below."
echo ""

DELETED=0
ERRORS=0
ITERATION=0

# Loop until no more runs
while true; do
    ITERATION=$((ITERATION + 1))
    
    # Get batch of run IDs
    RUN_IDS=$(gh run list --repo "$REPO" --limit "$BATCH_SIZE" --json databaseId -q '.[].databaseId' 2>/dev/null)
    
    if [[ -z "$RUN_IDS" ]]; then
        echo ""
        echo "No more workflow runs found."
        break
    fi
    
    # Count runs in this batch
    BATCH_COUNT=$(echo "$RUN_IDS" | wc -l | tr -d ' ')
    
    echo "Batch $ITERATION: Deleting $BATCH_COUNT runs..."
    
    # Delete each run in parallel (up to 10 at a time)
    echo "$RUN_IDS" | xargs -P 10 -I {} sh -c '
        if gh run delete {} --repo "'"$REPO"'" 2>/dev/null; then
            echo -n "."
        else
            echo -n "x"
        fi
    '
    
    DELETED=$((DELETED + BATCH_COUNT))
    echo ""
    echo "  Deleted so far: $DELETED"
    
    # Small delay to avoid rate limiting
    sleep 1
done

echo ""
echo "========================================"
echo "Cleanup complete!"
echo "Total runs deleted: $DELETED"
echo "========================================"
