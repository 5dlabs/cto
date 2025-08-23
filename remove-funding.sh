#!/bin/bash

if [ $# -eq 0 ]; then
    echo "Usage: ./remove-funding.sh <repo-name>"
    echo "Example: ./remove-funding.sh questdb-operator"
    exit 1
fi

REPO=$1
echo "🗑️  Removing funding configuration from forked repo: $REPO"
echo "======================================================"

# Get current file info for deletion
FUNDING_FILE=$(gh api repos/5dlabs/$REPO/contents/.github/FUNDING.yml 2>/dev/null)

if [ $? -eq 0 ]; then
    # Extract sha for deletion
    SHA=$(echo "$FUNDING_FILE" | jq -r '.sha')
    
    echo "Found FUNDING.yml in $REPO, removing..."
    
    # Delete the file
    gh api -X DELETE "repos/5dlabs/$REPO/contents/.github/FUNDING.yml" \
        -f message="Remove inappropriate funding from forked repository" \
        -f sha="$SHA"
    
    echo "✅ Removed .github/FUNDING.yml from $REPO"
else
    echo "ℹ️  No .github/FUNDING.yml found in $REPO"
fi

# Also check root FUNDING.yml
ROOT_FUNDING=$(gh api repos/5dlabs/$REPO/contents/FUNDING.yml 2>/dev/null)

if [ $? -eq 0 ]; then
    SHA=$(echo "$ROOT_FUNDING" | jq -r '.sha')
    
    echo "Found root FUNDING.yml in $REPO, removing..."
    
    gh api -X DELETE "repos/5dlabs/$REPO/contents/FUNDING.yml" \
        -f message="Remove inappropriate funding from forked repository" \
        -f sha="$SHA"
    
    echo "✅ Removed root FUNDING.yml from $REPO"
else
    echo "ℹ️  No root FUNDING.yml found in $REPO"
fi

echo ""
echo "🎯 $REPO is now clean of funding configurations!"
