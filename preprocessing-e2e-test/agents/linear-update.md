# Linear Update Agent

You are the Linear Update Agent responsible for testing bidirectional sync between Linear and GitHub.

## Issue Logging Protocol

Before executing your tasks, check your issues log:
1. Read `issues/issues-linear-update.md`
2. Address any OPEN issues in your domain first
3. Log new issues as you encounter them

### Issue Format
```
## ISSUE-{N}: {Brief title}
- **Status**: OPEN | IN_PROGRESS | RESOLVED
- **Severity**: BLOCKING | HIGH | MEDIUM | LOW
- **Discovered**: {timestamp}
- **Description**: {what went wrong}
- **Root Cause**: {why it happened}
- **Resolution**: {how it was fixed}
```

## Context

We need to test that changes made in Linear trigger webhooks that update files in GitHub.

## Tasks

### 1. Make a Test Update in Linear

Use the Linear API to update the PRD issue:

```bash
# Add a test comment to the PRD issue
curl -X POST \
     -H "Authorization: Bearer $LINEAR_APP_MORGAN_ACCESS_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{
       "query": "mutation { commentCreate(input: { issueId: \"$ISSUE_ID\", body: \"Test update from Linear Update Agent - $(date)\" }) { success } }"
     }' \
     https://api.linear.app/graphql
```

### 2. Monitor Webhook Reception

```bash
# Watch PM server logs for incoming webhook
tail -f /tmp/cto-launchd/pm-server.log | grep -i "webhook\|linear\|update"
```

### 3. Verify Webhook Processing

Check that the PM server:
- Receives the webhook
- Parses the update correctly
- Triggers appropriate action (file update or CodeRun)

### 4. Test Document Update

Update a Linear document (not just a comment):

```bash
# Update the architecture document content
curl -X POST \
     -H "Authorization: Bearer $LINEAR_APP_MORGAN_ACCESS_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{
       "query": "mutation { documentUpdate(id: \"$DOC_ID\", input: { content: \"Updated content...\" }) { success } }"
     }' \
     https://api.linear.app/graphql
```

### 5. Verify File Changes

After document update:

```bash
# Check if file was updated in repo
git status
git diff

# Or check if CodeRun was created to handle update
kubectl get coderun -n cto --sort-by=.metadata.creationTimestamp | tail -5
```

### 6. Test Conflict Resolution

If updates create conflicts:
- Verify PM server handles gracefully
- Check that appropriate CodeRun is launched
- Verify final state is consistent

## Success Criteria

Update `ralph-coordination.json` milestone `updates_tested` to `true` when:
- Linear update triggers webhook
- PM server receives and processes webhook
- File changes are reflected in repo (or CodeRun handles it)
- No data loss or conflicts

## Report Format

```
Linear Update Agent Report
==========================
Test Update Made: YES | NO
Webhook Received: YES | NO
Webhook Processing: SUCCESS | FAILED
File Updated: YES | NO | N/A (CodeRun handled)
CodeRun Created: {id or NONE}
Conflicts: {count or NONE}
Final State: CONSISTENT | INCONSISTENT
```
