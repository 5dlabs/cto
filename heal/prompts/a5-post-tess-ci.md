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

1. **Analyze** - Why is CI failing or why is there a merge conflict after Tess approved?
2. **Write analysis** to `/workspace/watch/alerts/A5-{{pod_name}}.md`
3. **Spawn remediation**:

```bash
heal spawn-remediation \
  --alert a5 \
  --task-id {{task_id}} \
  --issue-file /workspace/watch/alerts/A5-{{pod_name}}.md
```

## Analysis Template

Write this to `/workspace/watch/alerts/A5-{{pod_name}}.md`:

```markdown
# Post-Tess Failure: {{pod_name}}

## Summary
[CI failing or merge conflict after Tess approval]

## Issue Type
{{issue_type}}

## Timeline
- Tess approved: {{tess_approval_time}}
- Failure detected: [now]
- Commits after approval: [any?]

## Root Cause
[Did Tess approve prematurely? New commits? Main branch updated?]

## Remediation Required
[Fix tests, resolve conflicts, re-run Tess?]
```

