# Heal Remediation Flow

When Factory receives an A7 (Pod Failure) alert, it must execute these steps:

## Step 1: Fetch Pod Logs

```bash
heal fetch-logs \
  --pod-name {{pod_name}} \
  --namespace {{namespace}} \
  --output-dir /workspace/watch/logs
```

This downloads:
- `{{pod_name}}-current.log` - Current container logs
- `{{pod_name}}-previous.log` - Previous container logs (if restarted)
- `{{pod_name}}-events.yaml` - Pod events
- `{{pod_name}}-describe.txt` - Full pod description

## Step 2: Analyze the Failure

Review the logs and determine:
- What error/panic/exception caused the failure?
- Is it a code bug, OOM, config issue, or external dependency?
- What specific file/function is responsible?

## Step 3: Write Analysis to PVC

Write your analysis to `/workspace/watch/alerts/A7-{{pod_name}}.md`:

```markdown
# Pod Failure: {{pod_name}}

## Summary
[One sentence: what crashed and why]

## Error
[The specific error/panic/stack trace from logs]

## Root Cause
[Code bug, OOM, config issue, external dependency?]

## Remediation Required
[What needs to be fixed: code change, resource increase, config update?]

## Relevant Logs
- Current: `/workspace/watch/logs/{{pod_name}}-current.log`
- Previous: `/workspace/watch/logs/{{pod_name}}-previous.log`
- Events: `/workspace/watch/logs/{{pod_name}}-events.yaml`
```

## Step 4: Spawn Remediation Agent

```bash
heal spawn-remediation \
  --alert a7 \
  --task-id {{task_id}} \
  --issue-file /workspace/watch/alerts/A7-{{pod_name}}.md
```

This creates a CodeRun CRD that:
- Uses template `heal/claude` (Rex remediation agent)
- Mounts the same `heal-workspace` PVC
- Sets environment variables pointing to the issue file

## Step 5: Exit

Exit with code 0 to indicate successful analysis and remediation dispatch.

---

## CLI Commands

### fetch-logs

Fetches all diagnostic information for a pod:

```bash
heal fetch-logs \
  --pod-name <pod-name> \
  --namespace <namespace> \
  --output-dir /workspace/watch/logs \
  --tail 10000  # 0 for all logs
```

### spawn-remediation

Creates a CodeRun CRD for Rex to fix the issue:

```bash
heal spawn-remediation \
  --alert <alert-type> \
  --task-id <task-id> \
  --issue-file /workspace/watch/alerts/A7-<pod-name>.md
```

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        heal-workspace PVC                        │
│  /workspace/watch/                                               │
│  ├── alerts/                                                     │
│  │   └── A7-{{pod_name}}.md  ◄── Factory writes analysis        │
│  └── logs/                                                       │
│      ├── {{pod_name}}-current.log                               │
│      ├── {{pod_name}}-previous.log                              │
│      ├── {{pod_name}}-events.yaml                               │
│      └── {{pod_name}}-describe.txt                              │
└─────────────────────────────────────────────────────────────────┘
         ▲                                    ▲
         │                                    │
         │ writes                             │ reads
         │                                    │
┌────────┴────────┐                  ┌────────┴────────┐
│   Heal Pod      │                  │   Rex Pod       │
│   + Factory     │   creates        │   (CodeRun)     │
│                 │ ──────────────►  │                 │
│  Analyzes logs  │   CodeRun CRD    │  Fixes code,    │
│  Writes report  │                  │  deploys fix    │
└─────────────────┘                  └─────────────────┘
```

