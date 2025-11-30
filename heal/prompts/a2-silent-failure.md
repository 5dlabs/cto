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

1. **Analyze** - What crashed? The workflow thinks it's healthy but the agent is dead.
2. **Write analysis** to `/workspace/watch/alerts/A2-{{pod_name}}.md`
3. **Spawn remediation**:

```bash
play-monitor spawn-remediation \
  --alert a2 \
  --task-id {{task_id}} \
  --issue-file /workspace/watch/alerts/A2-{{pod_name}}.md
```

## Analysis Template

Write this to `/workspace/watch/alerts/A2-{{pod_name}}.md`:

```markdown
# Silent Failure: {{pod_name}}

## Summary
[Container crashed but pod looks healthy due to sidecars]

## Crash Point
```
[Last log lines before crash]
```

## Root Cause
[Panic, OOM, signal, etc.]

## Remediation Required
[Kill pod to surface failure, fix underlying issue]
```

