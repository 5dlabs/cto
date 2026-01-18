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
- **Agent assigned** to the issue
- **CLI output streaming** to the agent dialog via Linear sync sidecar
- Same behavior as Morgan's intake workflow

If Linear issues are NOT being created or sidecar is NOT running - this is a bug. Stop and fix it.

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

## Key Verifications for Play Workflow

### Linear Issues Created
```bash
# Each task should have a Linear issue
# Check via Linear API or CLI
```

### Linear Sidecar Running
```bash
# CodeRun pods should have linear-sync sidecar
kubectl get pods -n cto -l type=implementation -o json | jq '.items[].spec.containers[].name'
```

### Agent Assignment
```bash
# Working agent should be assigned to Linear issue
```

### PR Review Loop
The workflow supports change requests:
- Quality agents can request changes
- Implementation agent makes fixes
- Re-review until acceptance
