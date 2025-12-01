# Alert A9: Stuck CodeRun

## Detected Condition

A CodeRun has been in a non-terminal state (Running/Pending) for longer than the threshold without completing.

- **CodeRun Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Phase**: {{phase}}
- **Agent**: {{agent}}
- **Task ID**: {{task_id}}

## Background

This alert fires when a CodeRun CRD has been stuck in Running/Pending state for over 10 minutes without transitioning to Succeeded or Failed. This typically indicates:

1. The agent process inside the pod has stopped making progress (silent hang)
2. The agent exited but the status was never updated
3. Infrastructure issue preventing status updates
4. The underlying pod crashed but the CodeRun controller didn't detect it

## Your Task

### Step 1: Create GitHub Issue

```bash
ISSUE_URL=$(gh issue create \
  --repo 5dlabs/cto \
  --title "[HEAL-A9] Stuck CodeRun: {{pod_name}} in {{phase}}" \
  --label "heal,remediation,a9" \
  --body "🔍 Analyzing stuck CodeRun... Full analysis to follow.")
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

### Step 3: Investigate

```bash
# Get CodeRun status
kubectl get coderun {{pod_name}} -n {{namespace}} -o yaml

# Find the associated pod (if any)
kubectl get pods -n {{namespace}} -l coderun={{pod_name}}

# Check pod logs if pod exists
kubectl logs -n {{namespace}} -l coderun={{pod_name}} --tail=500
```

### Step 4: Write prompt.md

```bash
cat > "${ISSUE_DIR}/prompt.md" << PROMPT
# Stuck CodeRun: {{pod_name}}

## Summary
[One sentence: CodeRun stuck in phase {{phase}}, suspected cause]

## Investigation Results
- **CodeRun Status**: [phase, conditions from kubectl output]
- **Associated Pod**: [exists/missing, phase, status]
- **Pod Logs**: [last activity, any errors]

## Root Cause
[Agent hang, silent crash, controller issue, infrastructure?]

## Remediation Steps
1. [Delete CodeRun to retry, OR]
2. [Fix underlying bug, OR]
3. [Restart controller if status update issue]
PROMPT
```

### Step 5: Write acceptance-criteria.md

```bash
cat > "${ISSUE_DIR}/acceptance-criteria.md" << CRITERIA
# Acceptance Criteria - Issue #${ISSUE_NUMBER}

## Definition of Done

- [ ] Root cause of stuck CodeRun identified
- [ ] CodeRun either completes or is cleaned up
- [ ] If code bug: fix deployed
- [ ] If infra issue: controller/cluster state verified
- [ ] Task {{task_id}} progresses to completion
- [ ] No new A9 alerts for this CodeRun
CRITERIA
```

### Step 6: Update GitHub Issue

```bash
gh issue edit ${ISSUE_NUMBER} --repo 5dlabs/cto --body "$(cat ${ISSUE_DIR}/prompt.md)

---

$(cat ${ISSUE_DIR}/acceptance-criteria.md)"
```

### Step 7: Spawn Remediation Agent

```bash
heal spawn-remediation \
  --alert a9 \
  --task-id {{task_id}} \
  --issue-number ${ISSUE_NUMBER}
```

## Common Causes

1. **Agent Silent Hang**: Agent process running but not making progress
   - Check pod logs for last activity
   - Look for deadlock or infinite loop patterns

2. **Status Update Failure**: Agent completed but status not updated
   - Check controller logs for errors
   - Verify CRD status subresource

3. **Pod Eviction/OOM**: Pod killed but CodeRun not notified
   - Check for OOMKilled or Evicted pods
   - Review node conditions

4. **Network Issues**: Agent cannot communicate with APIs
   - Check for DNS or connectivity errors in logs
