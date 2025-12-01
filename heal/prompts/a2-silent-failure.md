# Alert A2: Silent Agent Failure

## Detected Condition

A container terminated with non-zero exit code but pod is still "Running" (sidecar keeping it alive).

- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Pod Phase**: {{phase}}
- **Agent**: {{agent}}
- **Task ID**: {{task_id}}

## Container Logs

```
{{logs}}
```

## Your Task

### Step 1: Create GitHub Issue

```bash
ISSUE_URL=$(gh issue create \
  --repo 5dlabs/cto \
  --title "[HEAL-A2] Silent Failure: {{pod_name}}" \
  --label "heal,remediation,a2" \
  --body "🔍 Analyzing silent agent failure for {{pod_name}}... Full analysis to follow.")
ISSUE_NUMBER=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')
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
# Silent Failure: {{pod_name}}

## Summary
[Container crashed but pod looks healthy due to sidecars]

## Crash Point
```
[Last log lines before crash - copy from logs above]
```

## Root Cause
[Panic, OOM, signal, unhandled error, etc.]

## Affected Files
[List files that likely need modification]

## Remediation Steps
1. [Identify and fix the root cause]
2. [Add error handling if needed]
3. [Test the fix locally]
```

### Step 4: Write acceptance-criteria.md

```bash
cat > "${ISSUE_DIR}/acceptance-criteria.md" << CRITERIA
# Acceptance Criteria - Issue #${ISSUE_NUMBER}

## Definition of Done

### Code Fix
- [ ] Root cause of silent failure identified
- [ ] Fix implemented to prevent crash/unhandled error
- [ ] Error handling improved if applicable
- [ ] Code passes \`cargo fmt --all --check\`
- [ ] Code passes \`cargo clippy --all-targets -- -D warnings\`
- [ ] All tests pass: \`cargo test --workspace\`

### Deployment
- [ ] PR created and linked to issue #${ISSUE_NUMBER}
- [ ] CI checks pass
- [ ] PR merged to main
- [ ] ArgoCD sync successful

### Verification
- [ ] Agent completes successfully with exit code 0
- [ ] No silent failures in subsequent runs
- [ ] Heal monitoring shows no new A2 alerts for similar failures
CRITERIA
```

### Step 5: Update GitHub Issue

```bash
gh issue edit ${ISSUE_NUMBER} --repo 5dlabs/cto --body "$(cat << EOF
# Silent Failure Analysis: {{pod_name}}

## Alert Details
- **Alert Type**: A2 (Silent Agent Failure)
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
  --alert a2 \
  --task-id {{task_id}} \
  --issue-number ${ISSUE_NUMBER}
```
