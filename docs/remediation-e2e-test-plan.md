# Remediation Buttons E2E Test Plan

**Date:** 2026-01-30
**Status:** Testing in progress

## Flow Overview

```
1. PR has failed CI check
2. Morgan posts check run with remediation buttons
3. User clicks "Fix with Rex" button
4. GitHub sends check_run webhook (action: requested_action)
5. Argo Events sensor catches webhook
6. Sensor forwards to PM Server /webhooks/github/remediation
7. PM Server creates CodeRun CR
8. Controller spawns agent pod
9. Agent fixes issue, pushes commit
```

## Component Status

| Component | Status | Notes |
|-----------|--------|-------|
| Morgan button rendering | ✅ Ready | `templates/pm/morgan-pm.sh.hbs` lines 1577-1738 |
| Argo Events sensor | ✅ Deployed | `remediation-button-sensor` running 13h |
| PM Server endpoint | ✅ Ready | `/webhooks/github/remediation` registered |
| PM Server pod | ✅ Running | Scaled up from 0 to 1 |
| Handler code | ✅ Ready | `handle_remediation_webhook()` implemented |
| CodeRun creation | ✅ Ready | `create_remediation_coderun()` implemented |
| Controller | ✅ Ready | Spawns pods from CodeRun CRs |

## Test Cases

### Test 1: Button Rendering on Failed Check

**Steps:**
1. Find a PR with a failed check (or create one)
2. Verify the check run has remediation buttons in the "Actions" section
3. Buttons should show based on detected language

**Verification:**
```bash
# Check a PR's check runs
gh pr checks <PR_NUMBER> --repo 5dlabs/cto

# Or via API to see actions
gh api /repos/5dlabs/cto/commits/<SHA>/check-runs --jq '.check_runs[].actions'
```

### Test 2: Button Click → Webhook → CodeRun

**Steps:**
1. Click a remediation button on a failed check
2. Verify sensor receives the event
3. Verify PM Server receives the webhook
4. Verify CodeRun is created

**Verification:**
```bash
# Check sensor logs
kubectl logs -n automation -l sensor-name=remediation-button --tail=50

# Check PM Server logs
kubectl logs -n cto deployment/cto-pm --tail=50

# Check for new CodeRun
kubectl get coderuns -n cto -l trigger=remediation-button
```

### Test 3: Agent Fixes Issue

**Steps:**
1. Verify agent pod is spawned
2. Verify agent clones repo and checks out PR branch
3. Verify agent makes fixes
4. Verify agent pushes commit

**Verification:**
```bash
# Watch agent pod
kubectl get pods -n cto -w

# Check agent logs
kubectl logs -n cto <pod-name> -c agent -f

# Verify commit on PR
gh pr view <PR_NUMBER> --json commits
```

## Manual Test Simulation

If no failed checks available, simulate the button click:

```bash
# Simulate check_run webhook with requested_action
curl -X POST http://cto-pm.cto.svc.cluster.local:8081/webhooks/github/remediation \
  -H "Content-Type: application/json" \
  -d '{
    "payload": {
      "action": "requested_action",
      "check_run": {
        "id": 12345,
        "name": "test-check",
        "head_sha": "abc123",
        "status": "completed",
        "conclusion": "failure",
        "pull_requests": [{
          "number": 4165,
          "head": {"ref": "test-branch", "sha": "abc123"},
          "base": {"ref": "main", "sha": "def456"}
        }]
      },
      "requested_action": {
        "identifier": "fix-rex-pr4165-12345"
      },
      "repository": {
        "id": 123,
        "name": "cto",
        "full_name": "5dlabs/cto",
        "clone_url": "https://github.com/5dlabs/cto.git",
        "html_url": "https://github.com/5dlabs/cto"
      }
    }
  }'
```

## Gaps Identified

1. **PM Server was scaled to 0** - Fixed by scaling to 1
2. **Need real failed check with buttons** - To test button rendering
3. **Need to verify full webhook path** - sensor → PM → CodeRun

## Next Steps

1. Find or create a PR with failed CI
2. Verify buttons appear on the check run
3. Click button and trace the full flow
4. Document any issues found
