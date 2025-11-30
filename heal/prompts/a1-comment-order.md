# Alert A1: Agent Comment Order Mismatch

## Detected Condition

An agent is running but a preceding agent in the workflow hasn't posted their expected GitHub comment yet.

- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Current Agent**: {{agent}}
- **Task ID**: {{task_id}}
- **Missing Comment From**: {{missing_agent}}
- **PR Number**: {{pr_number}}

## Expected Agent Order

1. **Rex/Blaze** → 2. **Cleo** → 3. **Tess** → 4. **Cipher** → 5. **Atlas**

## Container Logs

```
{{logs}}
```

## Your Task

1. **Analyze** - Why is `{{missing_agent}}`'s comment missing?
2. **Write analysis** to `/workspace/watch/alerts/A1-{{pod_name}}.md`
3. **Spawn remediation** using the CLI:

```bash
play-monitor spawn-remediation \
  --alert a1 \
  --task-id {{task_id}} \
  --issue-file /workspace/watch/alerts/A1-{{pod_name}}.md
```

## Analysis Template

Write this to `/workspace/watch/alerts/A1-{{pod_name}}.md`:

```markdown
# Comment Order Mismatch: {{pod_name}}

## Summary
[Which agent is missing and why]

## Root Cause
[Why the comment is missing - did the agent fail, skip, or not run?]

## Remediation Required
[What the remediation agent should do to fix this]
```

