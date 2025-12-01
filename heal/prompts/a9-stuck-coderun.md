# Alert A9: Stuck CodeRun

## Detected Condition

A CodeRun has been in a non-terminal state (Running/Pending) for longer than the threshold without completing.

- **CodeRun Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Phase**: {{phase}}
- **Agent**: {{agent}}
- **Task ID**: {{task_id}}

## Background

This alert fires when a CodeRun CRD has been stuck in Running/Pending state for over 10 minutes without transitioning to Succeeded or Failed. This typically indicates:

1. The agent process inside the pod has stopped making progress (silent hang)
2. The agent exited but the status was never updated
3. Infrastructure issue preventing status updates
4. The underlying pod crashed but the CodeRun controller didn't detect it

## Your Task

1. **Investigate** the CodeRun and its associated pod:

```bash
# Get CodeRun status
kubectl get coderun {{pod_name}} -n {{namespace}} -o yaml

# Find the associated pod (if any)
kubectl get pods -n {{namespace}} -l coderun={{pod_name}}

# Check pod logs if pod exists
kubectl logs -n {{namespace}} -l coderun={{pod_name}} --tail=500
```

2. **Analyze** - What caused the CodeRun to get stuck?

3. **Write analysis** to `/workspace/watch/alerts/A9-{{pod_name}}.md`

4. **Spawn remediation**:

```bash
heal spawn-remediation \
  --alert a9 \
  --task-id {{task_id}} \
  --issue-file /workspace/watch/alerts/A9-{{pod_name}}.md
```

## Analysis Template

Write this to `/workspace/watch/alerts/A9-{{pod_name}}.md`:

```markdown
# Stuck CodeRun: {{pod_name}}

## Summary
[One sentence: CodeRun stuck in phase X for Y minutes, suspected cause]

## Investigation Results
- CodeRun Status: [phase, conditions]
- Associated Pod: [exists/missing, phase, status]
- Pod Logs: [last activity, any errors]

## Root Cause
[Agent hang, silent crash, controller issue, infrastructure?]

## Remediation Required
[Delete CodeRun to retry, fix underlying bug, restart controller?]
```

## Common Causes

1. **Agent Silent Hang**: Agent process running but not making progress
   - Check pod logs for last activity
   - Look for deadlock or infinite loop patterns

2. **Status Update Failure**: Agent completed but status not updated
   - Check controller logs for errors
   - Verify CRD status subresource

3. **Pod Eviction/OOM**: Pod killed but CodeRun not notified
   - Check for OOMKilled or Evicted pods
   - Review node conditions

4. **Network Issues**: Agent cannot communicate with APIs
   - Check for DNS or connectivity errors in logs
