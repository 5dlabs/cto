#!/bin/bash

echo "🔍 Auditing Forked Repositories for Inappropriate Funding Configurations..."
echo "================================================================="

# Get all forked repos
FORKED_REPOS=$(gh repo list 5dlabs --limit 100 --json name,isFork,url --jq '.[] | select(.isFork == true) | .name')

if [ -z "$FORKED_REPOS" ]; then
    echo "✅ No forked repositories found."
    exit 0
fi

echo "Found forked repositories:"
echo "$FORKED_REPOS"
echo ""

# Check each forked repo for FUNDING.yml
for repo in $FORKED_REPOS; do
    echo "Checking $repo..."
    
    # Check for FUNDING.yml in .github directory
    if gh api "repos/5dlabs/$repo/contents/.github/FUNDING.yml" >/dev/null 2>&1; then
        echo "⚠️  WARNING: $repo has .github/FUNDING.yml (SHOULD BE REMOVED)"
        echo "   Run: gh api -X DELETE repos/5dlabs/$repo/contents/.github/FUNDING.yml"
    else
        echo "✅ $repo: No funding configuration found"
    fi
    
    # Also check root directory for FUNDING.yml
    if gh api "repos/5dlabs/$repo/contents/FUNDING.yml" >/dev/null 2>&1; then
        echo "⚠️  WARNING: $repo has root FUNDING.yml (SHOULD BE REMOVED)"  
        echo "   Run: gh api -X DELETE repos/5dlabs/$repo/contents/FUNDING.yml"
    fi
done

echo ""
echo "🎯 REMEMBER: Only remove funding from FORKED repositories!"
echo "   Keep funding for original 5D Labs projects (cto, toolman, etc.)"
