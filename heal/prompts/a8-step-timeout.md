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

1. **Analyze** - Is the agent stuck or legitimately busy?
2. **Write analysis** to `/workspace/watch/alerts/A8-{{pod_name}}.md`
3. **Spawn remediation**:

```bash
heal spawn-remediation \
  --alert a8 \
  --task-id {{task_id}} \
  --issue-file /workspace/watch/alerts/A8-{{pod_name}}.md
```

## Analysis Template

Write this to `/workspace/watch/alerts/A8-{{pod_name}}.md`:

```markdown
# Step Timeout: {{pod_name}}

## Summary
[Stuck, looping, or legitimately long-running?]

## Duration
- Running: {{duration}}
- Expected: {{timeout}}

## Recent Activity
[What has the agent been doing?]

## Root Cause
[Infinite loop, blocked on external, complex task?]

## Remediation Required
[Kill and retry, let continue, increase timeout?]
```

