# Objective: Complete Full Lifecycle Test

## Goal

Run the complete lifecycle from intake through play completion. Focus on:

1. **Intake Phase**: Submit intake for AlertHub with ~30 tasks (reduced from 50 to avoid output truncation)
2. **Play Phase**: After intake succeeds, start play workflow
3. **Monitor**: Watch for pod failures and report them immediately

## Current Status

Previous intake failed with output truncation (65 tasks generated but JSON was cut off at token limit).

## Action Steps

### 1. Submit Intake (Reduced Task Count)
```bash
# Use MCP tool with reduced task count
mcp_cto_intake(project_name="alerthub-e2e-test", num_tasks=30)
```

### 2. Monitor Intake CodeRun
```bash
# Check CodeRun status
kubectl get coderuns -n cto -l type=intake

# Check pod status (source of truth!)
kubectl get pods -n cto -l type=intake -o wide

# If pod shows Error, get logs immediately
kubectl logs <pod-name> -n cto -c claude-claude-opus-4-5-20251101 --tail=100
```

### 3. Verify Intake Success
```bash
# Gate 1: CodeRun succeeded
kubectl get coderuns -n cto -l type=intake -o json | jq -e '.items[0].status.phase == "Succeeded"'

# Gate 2: tasks.json exists
gh api repos/5dlabs/prd-alerthub-e2e-test/contents/prd-alerthub-e2e-test/.tasks/tasks/tasks.json >/dev/null
```

### 4. Start Play
```bash
# Use MCP tool to start play
mcp_cto_play()
```

## Gates

- intake-succeeded: `kubectl get coderuns -n cto -l type=intake -o json | jq -e '.items[0].status.phase == "Succeeded"'`
- tasks-json-exists: `gh api repos/5dlabs/prd-alerthub-e2e-test/contents/prd-alerthub-e2e-test/.tasks/tasks/tasks.json >/dev/null`
- play-started: `kubectl get coderuns -n cto -l type=implementation -o json | jq -e '.items | length > 0'`

## Evidence

- Record all command outputs in lifecycle-test/report.json
- Update lifecycle-test/progress.txt with outcomes
- If pod fails, capture logs immediately

## Critical Monitoring

**ALWAYS check pod status, not just CodeRun status.** The Ralph loop now has automatic pod monitoring every 30 seconds, but manual verification is still important.
