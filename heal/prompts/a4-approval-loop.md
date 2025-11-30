# Alert A4: Repeated Approval Loop

## Detected Condition

An agent has posted multiple approval comments without the workflow advancing.

- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Agent**: {{agent}}
- **Task ID**: {{task_id}}
- **PR Number**: {{pr_number}}
- **Approval Count**: {{approval_count}}

## Container Logs

```
{{logs}}
```

## Your Task

1. **Analyze** - Why is the agent looping on approval?
2. **Write analysis** to `/workspace/watch/alerts/A4-{{pod_name}}.md`
3. **Spawn remediation**:

```bash
play-monitor spawn-remediation \
  --alert a4 \
  --task-id {{task_id}} \
  --issue-file /workspace/watch/alerts/A4-{{pod_name}}.md
```

## Analysis Template

Write this to `/workspace/watch/alerts/A4-{{pod_name}}.md`:

```markdown
# Approval Loop: {{pod_name}}

## Summary
[Agent is saying "approved" but workflow isn't advancing]

## Loop Pattern
[Are the approvals identical? Time between them?]

## Root Cause
[Why isn't the next workflow step triggering?]

## Remediation Required
[Kill pod, fix workflow trigger, manual advancement?]
```

