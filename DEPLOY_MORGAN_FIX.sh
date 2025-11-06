#!/bin/bash
# Deploy Morgan PM Status Sync Fix
set -euo pipefail

cd /Users/jonathonfritz/code/work-projects/5dlabs/cto

echo "════════════════════════════════════════════════════════════"
echo "Morgan PM - Status Sync Fix Deployment"
echo "════════════════════════════════════════════════════════════"
echo ""

# Stage changes
echo "📝 Staging changes..."
git add infra/charts/controller/agent-templates/pm/github-projects-helpers.sh.hbs
git add infra/charts/controller/agent-templates/pm/morgan-pm.sh.hbs
git add docs/engineering/morgan-status-sync-fix.md
git add scripts/fix-existing-project-status.sh 2>/dev/null || true
git add scripts/create-morgan-pr.sh 2>/dev/null || true

echo ""
echo "📊 Changed files:"
git status --short

echo ""
read -p "Continue with commit? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Aborted"
    exit 1
fi

# Commit
echo ""
echo "💾 Committing..."
git commit -m "fix(morgan): use built-in Status field for auto-column creation

Complete fix for GitHub Projects status sync using pattern from 5dlabs/tasks:

1. Built-in Status Field (Critical Discovery)
   - Use 'Status' field instead of custom 'Stage'/'Current Agent' fields
   - GitHub automatically creates board columns for Status values
   - No manual grouping configuration needed
   - Pattern learned from 5dlabs/tasks reference implementation

2. Agent-Based Columns
   - Status options: Pending | Rex | Blaze | Cleo | Cipher | Tess | Atlas | Complete ✅
   - Each agent name becomes a board column
   - Removed Bolt from standard workflow (runs post-merge separately)

3. Real Cluster State Detection
   - Three-tier: RUNNING pods → Active CodeRuns → Stage mapping
   - Workflow phase priority (Succeeded/Failed checked first)
   - format_agent_name() handles GitHub App identifiers (5DLabs-Rex, etc.)

4. Board View as Default
   - Creates board view with BOARD_LAYOUT
   - Deletes default table view
   - Board opens automatically when visiting project

5. Fixed Silent Failures
   - set_project_item_status() returns proper exit codes (1 on failure, not 0)
   - GraphQL response validation
   - Comprehensive error logging with actionable messages

6. Repo-Level Projects Fixed
   - get_or_create_repo_project() now calls create_default_board_view()
   - Fixes cursor bot issue where repo projects had no board setup

Files Changed:
- infra/charts/controller/agent-templates/pm/github-projects-helpers.sh.hbs
- infra/charts/controller/agent-templates/pm/morgan-pm.sh.hbs
- docs/engineering/morgan-status-sync-fix.md

Result: Board columns auto-create and tasks auto-move based on real
Kubernetes cluster activity. No manual configuration required!

Testing:
After ArgoCD syncs, new projects will automatically have:
- Agent-based board columns
- Real-time cluster state reflection
- Board view as default

Existing projects (created with old code) will need manual Status field updates
or should be recreated."

echo ""
echo "🚀 Pushing to origin..."
CURRENT_BRANCH=$(git branch --show-current)
git push -u origin "$CURRENT_BRANCH"

echo ""
echo "📋 Creating pull request..."
gh pr create \
  --title "fix(morgan): Use built-in Status field for auto-column creation" \
  --body "## Summary

Complete fix for Morgan PM's GitHub Projects status sync.

## Key Changes

✅ Use built-in \"Status\" field → Agent columns auto-create  
✅ Real cluster state detection → Board reflects actual Kubernetes activity  
✅ Proper error handling → No more silent failures  
✅ Board as default → Auto-deletes table view  
✅ Repo-level projects → Fixed board setup

## How It Works

Morgan creates \"Status\" field with agent names:
\`Pending | Rex | Blaze | Cleo | Cipher | Tess | Atlas | Complete ✅\`

GitHub automatically creates columns for each Status value!

## Testing

After merge + ArgoCD sync:
1. \`kubectl delete configmap -n agent-platform agent-templates-pm\`
2. Create new test project
3. Watch columns auto-appear!

Pattern learned from 5dlabs/tasks reference implementation.

See: docs/engineering/morgan-status-sync-fix.md"

echo ""
echo "✅ PR created!"
echo ""
echo "Next steps:"
echo "1. Merge the PR on GitHub"
echo "2. ArgoCD will auto-sync (wait ~2 minutes)"
echo "3. Delete ConfigMap: kubectl delete configmap -n agent-platform agent-templates-pm"
echo "4. Create new test project to see agent columns!"


