# Alert A4: Repeated Approval Loop

## Detected Condition

An agent has posted multiple approval comments without the workflow advancing.

- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Agent**: {{agent}}
- **Task ID**: {{task_id}}
- **PR Number**: {{pr_number}}
- **Approval Count**: {{approval_count}}

## Container Logs

```
{{logs}}
```

## Your Task

### Step 1: Create GitHub Issue

```bash
ISSUE_URL=$(gh issue create \
  --repo 5dlabs/cto \
  --title "[HEAL-A4] Approval Loop: {{agent}} posted {{approval_count}} approvals" \
  --label "heal,remediation,a4" \
  --body "🔍 Analyzing approval loop... Full analysis to follow.")
ISSUE_NUMBER=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')
echo "✅ Created issue #${ISSUE_NUMBER}"
```

### Step 2: Create Issue Folder

```bash
ISSUE_DIR="/workspace/watch/issues/${ISSUE_NUMBER}"
mkdir -p "${ISSUE_DIR}"
```

### Step 3: Write prompt.md

```markdown
# Approval Loop: {{pod_name}}

## Summary
[Agent is saying "approved" but workflow isn't advancing]

## Loop Pattern
- **Approval Count**: {{approval_count}}
- **Time Between Approvals**: [analyze]
- **Are approvals identical?**: [yes/no]

## Root Cause
[Why isn't the next workflow step triggering? Controller issue? PR state?]

## Remediation Steps
1. [Check PR state and labels]
2. [Verify workflow trigger conditions]
3. [Kill pod, fix workflow trigger, or manual advancement]
```

### Step 4: Write acceptance-criteria.md

```markdown
# Acceptance Criteria - Issue #${ISSUE_NUMBER}

## Definition of Done

- [ ] Root cause of approval loop identified
- [ ] Workflow advances past stuck step
- [ ] PR {{pr_number}} progresses to next phase
- [ ] No repeated approval comments
- [ ] No new A4 alerts for this PR
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
  --alert a4 \
  --task-id {{task_id}} \
  --issue-number ${ISSUE_NUMBER}
```
