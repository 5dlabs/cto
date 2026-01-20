# CTO Lifecycle Test Prompt

You are running the CTO lifecycle test, validating that the full Play workflow
executes correctly: **Intake → Play → Quality → Security → Testing → Integration → Done**.

---

## CRITICAL REQUIREMENTS (MUST READ FIRST)

### 1. Failure Protocol - ALWAYS BACKTRACK

**When ANY step fails:**
1. **STOP immediately** - Do not continue
2. **Document** the failure in progress.txt
3. **Cleanup** all resources (see commands below)
4. **Investigate** root cause
5. **Fix** the issue
6. **Restart from the BEGINNING** of the current phase

**NEVER forge forward with failures. Always backtrack and start clean.**

### 2. Linear Integration (Required)

Each task in the Play workflow MUST have:
- **Linear issue created** with subtasks defined
- **Agent assigned** (delegated) to the issue
- **CLI output streaming** to the agent dialog via Linear sync sidecar
- Same behavior as Morgan's intake workflow

**CRITICAL**: Linear task issues are created via callback AFTER intake PR merge:
1. Intake generates tasks.json and creates PR
2. PR merge triggers webhook to PM server
3. PM server calls `/callbacks/tasks-json` to create Linear issues
4. Play workflow starts AFTER task issues exist

If Linear issues are NOT being created - check:
```bash
# Check if callback was called
curl -sf http://localhost:8081/api/linear/project/prd-alerthub-e2e-test/issues | jq '.issues | length'

# If 0 issues, callback was never triggered - this is a bug
```

If sidecar is NOT running - check pod containers:
```bash
kubectl get pods -n cto -o json | jq '[.items[] | {name: .metadata.name, containers: [.spec.containers[].name]}]'
```

### 3. Pod Lifecycle Issue (Watch For This)

**Known Bug:** Primary container terminates while sidecar remains up.
- Main container exits but Linear sidecar keeps running
- Prevents pod cleanup and workflow progression

Detection command:
```bash
kubectl get pods -n cto -o json | jq '.items[] | select(.status.containerStatuses | any(.name != "linear-sync" and .state.terminated)) | select(.status.containerStatuses | any(.name == "linear-sync" and .state.running)) | .metadata.name'
```

If you find such pods - this is a bug. Stop and fix the sidecar termination logic.

---

## Infrastructure Health Checks

Before any work, verify infrastructure:

```bash
curl -sf http://localhost:8080/health   # Controller
curl -sf http://localhost:8081/health   # PM Server
curl -sf http://localhost:8082/health   # Healer
kubectl cluster-info >/dev/null         # K8s access
```

If any fail, fix before proceeding.

---

## Instructions

1. Read `lifecycle-test/pin.md` for stable context and commands.
2. Read `lifecycle-test/current-objective.md` for what you need to accomplish.
3. Work from the repo root.

## Loop Discipline

1. **Check infrastructure health** - fix any issues before proceeding
2. Execute **only** the current objective
3. After **every** operation, check pod status and logs
4. If any step fails: **STOP, CLEANUP, INVESTIGATE, FIX, RESTART**
5. Update `lifecycle-test/progress.txt` with status
6. Only move to next objective after current one's gates pass

## Cleanup Commands (Run on ANY Failure)

```bash
kubectl delete coderuns -n cto -l service=prd-prd-alerthub-e2e-test --wait=false 2>/dev/null || true
kubectl delete pods -n cto --field-selector=status.phase!=Running --wait=false 2>/dev/null || true
kubectl delete pvc -n cto -l service=prd-prd-alerthub-e2e-test --wait=false 2>/dev/null || true
sleep 10
```

## Output Requirements

- Record evidence in `lifecycle-test/report.json`
- Update `lifecycle-test/progress.txt` with status and any issues found
- If you make code changes, list files changed
- If you encounter bugs, document them AND fix them before proceeding

## Known Issues and Workarounds

### 1. CodeRun Status Lag
- **Issue**: CodeRun `.status.phase` may remain "Running" even after pod succeeds
- **Workaround**: Check pod status directly instead of CodeRun status
```bash
kubectl get pods -n cto -o json | jq '[.items[] | select(.metadata.name | contains("intake")) | {name: .metadata.name, phase: .status.phase}]'
```

### 2. ConfigMap Naming Convention
- **Issue**: ConfigMaps use project-id, not service name
- **Pattern**: `cto-config-project-{uuid}` instead of `cto-config-{service}`
```bash
kubectl get configmap -n cto | grep cto-config-project
```

### 3. Linear Task Issues Not Created Automatically
- **Issue**: `/callbacks/tasks-json` must be called after PR merge to create Linear task issues
- **Check**: If no task issues exist after PR merge, this callback was not triggered
- **Required**: Implement webhook handler for intake PR merge → callback trigger

---

## Key Verifications for Play Workflow

### Linear Task Issues Created
```bash
# Count task issues (should match tasks.json count after PR merge)
TASK_COUNT=$(gh api repos/5dlabs/prd-prd-alerthub-e2e-test/contents/prd-prd-alerthub-e2e-test/.tasks/tasks/tasks.json --jq '.content' | base64 -d | jq '.tasks | length')
ISSUE_COUNT=$(curl -sf http://localhost:8081/api/linear/project/prd-alerthub-e2e-test/issues | jq '.issues | length')
echo "Tasks: $TASK_COUNT, Issues: $ISSUE_COUNT"
```

### Linear Sidecar Running
```bash
# Check for linear-sidecar container in pods
kubectl get pods -n cto -o json | jq '[.items[] | select(.spec.containers | map(.name) | any(. == "linear-sidecar"))]'
```

### Agent Assignment in Linear
```bash
# Verify agent is assigned (delegated) to task issue
curl -sf http://localhost:8081/api/linear/task/current | jq '{assignee: .assignee, delegate: .delegate}'
```

### Agent Dialogue Activities
```bash
# Check activities posted to Linear (should be > 0 for active agents)
curl -sf http://localhost:8081/api/linear/task/current/activities | jq '.activities | length'
```

### PR Review Loop
The workflow supports change requests:
- Quality agents can request changes
- Implementation agent makes fixes
- Re-review until acceptance
