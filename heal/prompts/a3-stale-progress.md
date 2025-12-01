# Alert A3: Stale Agent Progress

## Detected Condition

An agent pod has been running but no new commits have been pushed to the feature branch for an extended period.

- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Agent**: {{agent}}
- **Task ID**: {{task_id}}
- **Branch**: {{branch}}
- **Last Commit**: {{last_commit_time}}
- **Stale Duration**: {{stale_duration}}

## Container Logs

```
{{logs}}
```

## Your Task

### Step 1: Create GitHub Issue

```bash
ISSUE_URL=$(gh issue create \
  --repo 5dlabs/cto \
  --title "[HEAL-A3] Stale Progress: {{agent}} no commits for {{stale_duration}}" \
  --label "heal,remediation,a3" \
  --body "🔍 Analyzing stale agent progress... Full analysis to follow.")
ISSUE_NUMBER=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')
echo "✅ Created issue #${ISSUE_NUMBER}"
```

### Step 2: Create Issue Folder

```bash
ISSUE_DIR="/workspace/watch/issues/${ISSUE_NUMBER}"
mkdir -p "${ISSUE_DIR}"
```

### Step 3: Write prompt.md

```bash
cat > "${ISSUE_DIR}/prompt.md" << PROMPT
# Stale Progress: {{pod_name}}

## Summary
[Pod running but no commits in {{stale_duration}}]

## Last Productive Action
[What was the agent last doing before going stale?]

## Timing
- **Last Commit**: {{last_commit_time}}
- **Stale Duration**: {{stale_duration}}

## Root Cause
[Why no commits - API limits, auth failure, infinite loop, legitimately complex task?]

## Remediation Steps
1. [Check agent logs for errors or loops]
2. [Determine if intervention needed]
3. [Kill pod, fix config, or let continue]
PROMPT
```

### Step 4: Write acceptance-criteria.md

```bash
cat > "${ISSUE_DIR}/acceptance-criteria.md" << CRITERIA
# Acceptance Criteria - Issue #${ISSUE_NUMBER}

## Definition of Done

- [ ] Root cause of stale progress identified
- [ ] Either: Agent makes progress (new commits), OR
- [ ] Agent terminated and restarted with fix, OR
- [ ] Underlying issue (auth, API limits) resolved
- [ ] Branch {{branch}} has new commits
- [ ] No new A3 alerts for this agent
CRITERIA
```

### Step 5: Update GitHub Issue

```bash
gh issue edit ${ISSUE_NUMBER} --repo 5dlabs/cto --body "$(cat ${ISSUE_DIR}/prompt.md)

---

$(cat ${ISSUE_DIR}/acceptance-criteria.md)"
```

### Step 6: Spawn Remediation Agent

```bash
heal spawn-remediation \
  --alert a3 \
  --task-id {{task_id}} \
  --issue-number ${ISSUE_NUMBER}
```
