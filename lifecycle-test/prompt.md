---
title: CTO Lifecycle Test Execution Prompt
description: Single-objective loop prompt for Ralph execution
---

# CTO Platform Lifecycle Test - Execution Prompt

You are executing one objective at a time for the AlertHub lifecycle test.

## CRITICAL: Fix Failures Before Proceeding

**DO NOT** just document blockers and stop. You must:
1. **Diagnose** the root cause of any failure
2. **FIX** the issue if it's within your capability (scale deployments, restart pods, fix configs)
3. **Verify** the fix worked before proceeding
4. Only escalate if the fix requires access you don't have (e.g., cluster admin, secrets)

### Infrastructure Checks (Run First Every Iteration)

Before attempting any objective, verify infrastructure is healthy:

```bash
# Check controller is running
kubectl get pods -n cto -l app.kubernetes.io/name=agent-controller

# Check PM server is running  
kubectl get pods -n cto -l app=pm-server

# If either returns "No resources found", FIX IT:
# Option 1: Scale via ArgoCD
kubectl patch application cto -n argocd --type merge -p '{"spec":{"source":{"helm":{"valuesObject":{"controller":{"replicaCount":1},"pm-server":{"replicaCount":1}}}}}}'

# Option 2: Scale directly (temporary)
kubectl scale deployment agent-controller -n cto --replicas=1
kubectl scale deployment pm-server -n cto --replicas=1
```

### Failure Response Protocol

When you encounter ANY failure:
1. **Identify**: What exactly failed? (pod crash, timeout, permission error, etc.)
2. **Diagnose**: Check logs with `kubectl logs <pod> -n cto`
3. **Fix**: Apply the appropriate fix (restart pod, scale deployment, fix config)
4. **Verify**: Confirm the fix worked before retrying the objective
5. **Document**: Brief note in progress.txt about what was fixed

**DO NOT** proceed to the next objective until all failures are resolved.

## Read First

1. Read `lifecycle-test/pin.md` for stable context and commands.
2. Read `lifecycle-test/current-objective.md` for what you need to accomplish.
3. Work from the repo root.

## Loop Discipline

1. **Check infrastructure health** (see above) - fix any issues before proceeding
2. Execute **only** the current objective
3. After **every** operation, check pod status and logs
4. If any step fails, **FIX IT** before continuing (don't just document and stop)
5. Update `lifecycle-test/progress.txt` with status
6. Only move to next objective after current one's gates pass

## Output Requirements

- Provide command output snippets as evidence
- Do not claim success without verification
- If you fixed something, document what and how
- Keep responses concise and focused
