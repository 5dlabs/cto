# Alert A1: Agent Comment Order Mismatch

## Detected Condition

An agent is running but a preceding agent in the workflow hasn't posted their expected GitHub comment yet.

- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Current Agent**: {{agent}}
- **Task ID**: {{task_id}}
- **Missing Comment From**: {{missing_agent}}
- **PR Number**: {{pr_number}}

## Expected Agent Order

1. **Rex/Blaze** → 2. **Cleo** → 3. **Tess** → 4. **Cipher** → 5. **Atlas**

## Container Logs

```
{{logs}}
```

## Your Task

### Step 1: Create GitHub Issue

```bash
ISSUE_URL=$(gh issue create \
  --repo 5dlabs/cto \
  --title "[HEAL-A1] Comment Order Mismatch: {{missing_agent}} missing" \
  --label "heal,remediation,a1" \
  --body "🔍 Analyzing comment order mismatch... Full analysis to follow.")
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
# Comment Order Mismatch: {{pod_name}}

## Summary
[Which agent ({{missing_agent}}) is missing and why]

## Expected Order
Rex/Blaze → Cleo → Tess → Cipher → Atlas

## Root Cause
[Why the comment is missing - did the agent fail, skip, or not run?]

## Remediation Steps
1. [Check if the missing agent's pod exists]
2. [Verify the agent completed its task]
3. [Trigger missing comment or rerun agent]
```

### Step 4: Write acceptance-criteria.md

```markdown
# Acceptance Criteria - Issue #${ISSUE_NUMBER}

## Definition of Done

- [ ] Identified why {{missing_agent}}'s comment is missing
- [ ] Either: Agent rerun and comment posted, OR
- [ ] Fixed underlying issue preventing comment
- [ ] PR {{pr_number}} has all expected comments in order
- [ ] No A1 alerts for this PR
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
  --alert a1 \
  --task-id {{task_id}} \
  --issue-number ${ISSUE_NUMBER}
```
