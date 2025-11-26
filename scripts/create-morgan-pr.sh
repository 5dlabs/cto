#!/bin/bash
# Create Morgan PM Status Sync Fix PR
set -euo pipefail

cd /Users/jonathonfritz/code/work-projects/5dlabs/cto

echo "🔍 Current branch:"
git branch --show-current

echo ""
echo "📊 Changed files:"
git status --short

echo ""
echo "📝 Staging Morgan PM changes..."
git add infra/charts/controller/agent-templates/pm/github-projects-helpers.sh.hbs
git add infra/charts/controller/agent-templates/pm/morgan-pm.sh.hbs
git add docs/engineering/morgan-status-sync-fix.md

echo ""
echo "💾 Committing..."
git commit -m "fix(morgan): use built-in Status field for auto-column creation

Complete fix for GitHub Projects status sync using pattern from 5dlabs/tasks:

1. Built-in Status Field (Critical Discovery)
   - Use 'Status' field instead of custom 'Stage' field
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
   - format_agent_name() handles GitHub App identifiers

4. Board View as Default
   - Creates board view with BOARD_LAYOUT
   - Deletes default table view
   - Board opens automatically

5. Fixed Silent Failures
   - set_project_item_status() returns proper exit codes
   - GraphQL response validation
   - Comprehensive error logging

6. Repo-Level Projects Fixed
   - get_or_create_repo_project() now calls create_default_board_view()
   - Fixes issue where repo projects had no board setup

Result: Board columns auto-create and tasks auto-move based on real
Kubernetes cluster activity. No manual configuration required!"

echo ""
echo "🚀 Pushing to origin..."
git push origin HEAD

echo ""
echo "📋 Creating pull request..."
gh pr create \
  --title "fix(morgan): Use built-in Status field for auto-column creation" \
  --body "## Summary

Complete fix for Morgan PM's GitHub Projects status sync issue.

## The Problem

Morgan PM wasn't updating GitHub Projects board to reflect Kubernetes cluster activity:
- ❌ Used custom \"Stage\" field → Required manual grouping configuration
- ❌ Silent failures → Functions returned success when GraphQL calls failed
- ❌ No cluster state detection → Used theoretical stage mappings
- ❌ Board defaulted to table view

## The Solution

### 1. Built-in \"Status\" Field (Critical Discovery)

**Pattern from 5dlabs/tasks:**
GitHub's built-in \"Status\" field **automatically creates board columns**!

\`\`\`bash
# BEFORE: Custom field
create_single_select_field \"Stage\" [\"To Do\", \"In Progress\", ...]
# ❌ Required manual \"Group by: Stage\" in UI

# AFTER: Built-in field
create_single_select_field \"Status\" [\"Pending\", \"Rex (Implementation)\", ...]  
# ✅ Columns auto-create!
\`\`\`

### 2. Agent-Based Columns

**Status Field Options:**
\`\`\`
Pending | Rex (Implementation) | Blaze (Frontend) | Cleo (Quality) | 
Cipher (Security) | Tess (QA) | Atlas (Integration) | Complete ✅
\`\`\`

Each agent name becomes a board column that auto-creates!

### 3. Real Cluster State Detection

**Three-Tier Priority:**
1. RUNNING pods (\`kubectl get pods --field-selector=status.phase=Running\`)
2. Active CodeRuns (\`kubectl get coderuns\`)  
3. Stage mapping (fallback only)

**Workflow phase checked FIRST:**
- Succeeded → \"Complete ✅\"
- Failed/Error → Detect which agent failed
- Running → Detect actual running agent from cluster

### 4. Fixed Silent Failures

- \`set_project_item_status()\` now returns proper exit codes
- GraphQL response validation
- Comprehensive error logging
- Failed updates trigger warnings

### 5. Board View as Default

- Creates board view with BOARD_LAYOUT
- Deletes default table view
- Board opens automatically when visiting project

### 6. Repo-Level Projects Fixed

- \`get_or_create_repo_project()\` now calls \`create_default_board_view()\`
- Fixes cursor bot issue where repo projects had no board setup

## Result

✅ **Auto-Column Creation** - No manual configuration needed  
✅ **Real Cluster State** - Board reflects actual Kubernetes activity  
✅ **Proper Error Handling** - Failed updates logged and debuggable  
✅ **Board as Default** - Opens to board view automatically  
✅ **Agent Workflow Visibility** - See who's working on what at a glance

## Expected Behavior

1. Morgan creates project → Status field created with agent names
2. Morgan creates board view → Columns auto-appear!
3. Morgan detects running agent → Updates Status field
4. Task moves to agent column automatically
5. Workflow completes → Task moves to \"Complete ✅\"

## Testing

After merge:
1. ArgoCD will sync new Morgan template
2. Delete ConfigMap: \`kubectl delete configmap -n cto agent-templates-pm\`
3. Create new test project
4. Verify agent columns appear automatically
5. Watch tasks move through agent columns as workflows progress

## Files Changed

- \`infra/charts/controller/agent-templates/pm/github-projects-helpers.sh.hbs\`
- \`infra/charts/controller/agent-templates/pm/morgan-pm.sh.hbs\`
- \`docs/engineering/morgan-status-sync-fix.md\`

---

**No Manual Configuration Required!** 🎉"

echo ""
echo "✅ Pull request created!"


