# Alert A8: Workflow Step Timeout

## Detected Condition

A workflow step has exceeded its expected runtime.

- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Agent**: {{agent}}
- **Task ID**: {{task_id}}
- **Running Duration**: {{duration}}
- **Expected Timeout**: {{timeout}}

## Container Logs

```
{{logs}}
```

## Your Task

### Step 1: Create GitHub Issue

```bash
ISSUE_URL=$(gh issue create \
  --repo 5dlabs/cto \
  --title "[HEAL-A8] Step Timeout: {{agent}} running {{duration}}" \
  --label "heal,remediation,a8" \
  --body "🔍 Analyzing workflow step timeout... Full analysis to follow.")
if [ -z "$ISSUE_URL" ]; then
  echo "❌ Failed to create GitHub issue"
  exit 1
fi
ISSUE_NUMBER=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')
if [ -z "$ISSUE_NUMBER" ]; then
  echo "❌ Failed to extract issue number from: $ISSUE_URL"
  exit 1
fi
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
# Step Timeout: {{pod_name}}

## Summary
[Stuck, looping, or legitimately long-running?]

## Duration
- **Running**: {{duration}}
- **Expected**: {{timeout}}

## Recent Activity
[What has the agent been doing? Last productive action?]

## Root Cause
[Infinite loop, blocked on external API, complex task, resource constraints?]

## Remediation Steps
1. [Analyze recent logs for patterns]
2. [Determine if agent is making progress]
3. [Kill and retry, let continue, or fix underlying issue]
PROMPT
```

### Step 4: Write acceptance-criteria.md

```bash
cat > "${ISSUE_DIR}/acceptance-criteria.md" << CRITERIA
# Acceptance Criteria - Issue #${ISSUE_NUMBER}

## Definition of Done

- [ ] Root cause of timeout identified
- [ ] Either: Agent completes successfully, OR
- [ ] Agent terminated and restarted with fix, OR
- [ ] Timeout threshold adjusted if legitimate
- [ ] Task {{task_id}} progresses to completion
- [ ] No new A8 alerts for similar timeouts
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
  --alert a8 \
  --task-id {{task_id}} \
  --issue-number ${ISSUE_NUMBER}
```
