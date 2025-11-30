# Alert A3: Stale Progress - No Commits

## Detected Condition

An agent pod has been running but no new commits have been pushed to the feature branch.

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

1. **Analyze** - Is the agent stuck, looping, or blocked?
2. **Write analysis** to `/workspace/watch/alerts/A3-{{pod_name}}.md`
3. **Spawn remediation**:

```bash
play-monitor spawn-remediation \
  --alert a3 \
  --task-id {{task_id}} \
  --issue-file /workspace/watch/alerts/A3-{{pod_name}}.md
```

## Analysis Template

Write this to `/workspace/watch/alerts/A3-{{pod_name}}.md`:

```markdown
# Stale Progress: {{pod_name}}

## Summary
[Is agent stuck, looping, or legitimately working?]

## Last Productive Action
[What was the agent last doing before going stale?]

## Root Cause
[Why no commits - API limits, auth failure, infinite loop?]

## Remediation Required
[Kill pod, fix config, manual intervention?]
```

