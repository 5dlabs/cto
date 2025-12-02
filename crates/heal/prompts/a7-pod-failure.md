# Alert A7: Pod Failure

## Detected Condition

A pod has entered Failed/Error/CrashLoopBackOff state.

- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Phase**: {{phase}}
- **Agent**: {{agent}}
- **Task ID**: {{task_id}}

## Container Logs

```
{{logs}}
```

## Your Task

### Step 1: Create GitHub Issue

Create a GitHub issue first to get the issue number:

```bash
ISSUE_URL=$(gh issue create \
  --repo 5dlabs/cto \
  --title "[HEAL-A7] Pod Failure: {{pod_name}}" \
  --label "heal,remediation,a7" \
  --body "🔍 Analyzing pod failure for {{pod_name}}... Full analysis to follow.")
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
echo "📁 Created ${ISSUE_DIR}"
```

### Step 3: Write prompt.md

Analyze the logs and write your analysis to `${ISSUE_DIR}/prompt.md`:

```markdown
# Pod Failure: {{pod_name}}

## Summary
[One sentence: what crashed and why]

## Error
```
[The specific error/panic/stack trace from logs above]
```

## Root Cause
[Code bug, OOM, config issue, external dependency?]

## Affected Files
[List the files that likely need to be modified]

## Remediation Steps
1. [Specific step to fix the issue]
2. [Validation step]
3. [Deployment verification]
```

### Step 4: Write acceptance-criteria.md

Write acceptance criteria to `${ISSUE_DIR}/acceptance-criteria.md`:

```markdown
# Acceptance Criteria - Issue #${ISSUE_NUMBER}

## Definition of Done

### Code Fix
- [ ] Root cause identified and documented
- [ ] Fix implemented with minimal changes
- [ ] Code passes `cargo fmt --all --check`
- [ ] Code passes `cargo clippy --all-targets -- -D warnings`
- [ ] All tests pass: `cargo test --workspace`

### Deployment
- [ ] PR created and linked to issue #${ISSUE_NUMBER}
- [ ] CI checks pass
- [ ] PR merged to main
- [ ] ArgoCD sync successful
- [ ] Pod {{pod_name}} running without errors

### Verification
- [ ] Pod has 0 restarts for 5+ minutes
- [ ] No error logs in pod output
- [ ] Heal monitoring shows no new A7 alerts for {{pod_name}}
```

### Step 5: Update GitHub Issue

Update the GitHub issue with the full analysis:

```bash
gh issue edit ${ISSUE_NUMBER} --repo 5dlabs/cto --body "$(cat << EOF
# Pod Failure Analysis: {{pod_name}}

## Alert Details
- **Alert Type**: A7 (Pod Failure)
- **Pod**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Phase**: {{phase}}
- **Agent**: {{agent}}

---

$(cat ${ISSUE_DIR}/prompt.md)

---

$(cat ${ISSUE_DIR}/acceptance-criteria.md)
EOF
)"
echo "📝 Updated issue #${ISSUE_NUMBER} with analysis"
```

### Step 6: Spawn Remediation Agent

```bash
heal spawn-remediation \
  --alert a7 \
  --task-id {{task_id}} \
  --target-pod "{{pod_name}}" \
  --issue-number ${ISSUE_NUMBER}
```

## Important Notes

- The issue number becomes the unique identifier for this remediation
- Both `prompt.md` and `acceptance-criteria.md` are synced to GitHub
- The remediation agent will link its PR to this issue with `Fixes #${ISSUE_NUMBER}`
