# Alert A5: Post-Tess CI/Merge Failure

## Detected Condition

CI checks are failing OR there's a merge conflict AFTER Tess approved the PR.

- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Task ID**: {{task_id}}
- **PR Number**: {{pr_number}}
- **Tess Approval Time**: {{tess_approval_time}}
- **Issue Type**: {{issue_type}} (ci_failure | merge_conflict)
- **Failing Checks**: {{failing_checks}}
- **Mergeable**: {{mergeable}}

## Container Logs

```
{{logs}}
```

## Your Task

### Step 1: Create GitHub Issue

```bash
ISSUE_URL=$(gh issue create \
  --repo 5dlabs/cto \
  --title "[HEAL-A5] Post-Tess Failure: {{issue_type}} on PR #{{pr_number}}" \
  --label "heal,remediation,a5" \
  --body "🔍 Analyzing post-Tess failure... Full analysis to follow.")
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
# Post-Tess Failure: {{pod_name}}

## Summary
[CI failing or merge conflict after Tess approval]

## Issue Type
{{issue_type}}

## Timeline
- **Tess Approved**: {{tess_approval_time}}
- **Failure Detected**: [now]
- **Commits After Approval**: [check if any new commits]

## Failing Checks
{{failing_checks}}

## Mergeable State
{{mergeable}}

## Root Cause
[Did Tess approve prematurely? New commits broke CI? Main branch updated causing conflicts?]

## Remediation Steps
1. [Fix failing tests if CI failure]
2. [Resolve merge conflicts if conflict]
3. [Re-run Tess validation if needed]
PROMPT
```

### Step 4: Write acceptance-criteria.md

```bash
cat > "${ISSUE_DIR}/acceptance-criteria.md" << CRITERIA
# Acceptance Criteria - Issue #${ISSUE_NUMBER}

## Definition of Done

- [ ] Root cause identified
- [ ] If CI failure: Tests fixed and passing
- [ ] If merge conflict: Conflicts resolved
- [ ] PR {{pr_number}} is mergeable
- [ ] All CI checks pass
- [ ] No new A5 alerts for this PR
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
  --alert a5 \
  --task-id {{task_id}} \
  --issue-number ${ISSUE_NUMBER}
```
