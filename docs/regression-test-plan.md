### Regression test plan (scope, flow, and what we’ll test)

- **Scope**
  - Validate currently deployed event correlation and suspend/resume behavior (Task 5) in cluster now.
  - After Task 7 merges, re-run the same tests to additionally confirm atomic post-agent stage label updates.

- **Flow overview (where each test sits)**
  - Rex completes → suspend at `waiting-pr-created` → PR created resumes → Cleo runs → suspend at `waiting-ready-for-qa` → `ready-for-qa` label resumes → Tess runs → suspend at `waiting-pr-approved` → PR approval resumes → completion.
  - Remediation path: Implementation push (Rex/Blaze/Morgan) triggers cleanup of Cleo/Tess runs for that `task-id`.

### Preconditions

- **Sensors** deployed/Ready in `argo`:
  - `multi-agent-workflow-resume` (PR created), `ready-for-qa-label` (QA label), `pr-approval` (approval), `rex-remediation` (implementation push)
- **Workflow** can be submitted that:
  - Sets labels `workflow-type=play-orchestration`, `task-id=<N>`, `current-stage=waiting-pr-created`
  - Suspends at each stage for event-driven resumption

### Test cases (actions and expectations)

- **Test 1 — PR created resume**
  - **Where in flow**: Resume from `waiting-pr-created` → start Cleo
  - **Action**:
    - Submit/suspend workflow for `task-id=<N>`
    - Open a PR from branch `task-<N>-<slug>` (ensure PR has `task-<N>` label)
  - **Expect**:
    - Sensor correlates `task-id` and resumes workflow from `waiting-pr-created`
    - Workflow proceeds into Cleo stage

- **Test 2 — Ready‑for‑QA label resume**
  - **Where in flow**: Resume from `waiting-ready-for-qa` → start Tess
  - **Action**: Add `ready-for-qa` label to the PR (by Cleo or equivalent bot)
  - **Expect**:
    - Sensor resumes workflow from `waiting-ready-for-qa`
    - Workflow proceeds into Tess stage

- **Test 3 — PR approval resume**
  - **Where in flow**: Resume from `waiting-pr-approved` → finalize workflow
  - **Action**: Approve the PR (by Tess or equivalent bot)
  - **Expect**:
    - Sensor resumes workflow from `waiting-pr-approved`
    - Workflow completes and runs completion/cleanup steps

- **Test 4 — Implementation push remediation**
  - **Where in flow**: While Cleo/Tess are active or pending
  - **Action**: Push a commit from an implementation agent branch (e.g., `task-<N>-fix`) by Rex/Blaze/Morgan
  - **Expect**:
    - Remediation sensor cancels Cleo/Tess CodeRuns for `task-id=<N>`
    - Logs indicate remediation and targeted cancellation occurred

### What to verify at each step

- **Workflow labels**
  - Correct `task-id` and `current-stage` before/after resume points
  - Post‑Task 7: label updates occur immediately after agent completion (atomic JSON merge patch)
- **Sensor behavior**
  - Correct event filters, extraction, and single-target correlation (no false positives)
- **Observability**
  - Sensor logs reflect matched event and resume operation
  - Workflow node graph shows expected transitions

### Pass/fail criteria

- **Pass**
  - Each event resumes exactly the intended workflow at the correct stage
  - No cross‑task interference; remediation cancels only the intended runs
- **Fail**
  - Resume at wrong stage or wrong workflow; missing/late label transitions; remediation not constrained to `task-id`

### Operator runbook (quick commands)

```bash
# Sensors present
kubectl get sensors -n argo

# Submit a workflow (example template)
argo submit -n argo --from workflowtemplate/play-workflow-template \
  -p task-id=<N> -p repository=5dlabs/cto

# Inspect workflow labels
kubectl get workflows -n argo \
  -o custom-columns=NAME:.metadata.name,STAGE:.metadata.labels.current-stage,TASK:.metadata.labels.task-id
kubectl get workflow <name> -n argo -o jsonpath='{.metadata.labels}'

# Sensor logs (adjust name)
kubectl logs -n argo deploy/multi-agent-workflow-resume --tail=200 | cat
```

### After Task 7 lands

- Re-run all tests to confirm immediate, atomic label updates after agent completion (not only at suspend points)
- Verify no regressions in resume targeting and remediation behavior


