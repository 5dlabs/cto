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

1. **Analyze** - What caused the pod to fail?
2. **Write analysis** to `/workspace/watch/alerts/A7-{{pod_name}}.md`
3. **Spawn remediation**:

```bash
play-monitor spawn-remediation \
  --alert a7 \
  --task-id {{task_id}} \
  --issue-file /workspace/watch/alerts/A7-{{pod_name}}.md
```

## Analysis Template

Write this to `/workspace/watch/alerts/A7-{{pod_name}}.md`:

```markdown
# Pod Failure: {{pod_name}}

## Summary
[One sentence: what crashed and why]

## Error
```
[The specific error/panic/stack trace]
```

## Root Cause
[Code bug, OOM, config issue, external dependency?]

## Remediation Required
[Fix code, increase resources, fix config?]
```

