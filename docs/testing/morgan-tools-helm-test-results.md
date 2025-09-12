# Morgan Tools Helm Verification – Test Results

> Purpose: Capture a clear, reproducible record that Helm values for `agents.morgan.tools` are present in the controller’s mounted config and reflected by a DocsRun. Fill in every section. Keep raw evidence in the Evidence blocks.

## Test Metadata
- Date/Time (UTC): <!-- yyyy-mm-dd hh:mm -->
- Cluster/Context: <!-- e.g., kind-dev / prod-gke -->
- Namespace: <!-- e.g., agent-platform -->
- Helm Release: <!-- e.g., controller -->
- Git Ref (charts + controller image): <!-- commit/tag -->
- Agent Under Test: Morgan (`githubApp: 5DLabs-Morgan`)

## Goals
- Verify `.Values.agents.morgan.tools.remote` in Helm values is rendered into the controller ConfigMap (`config.yaml`).
- Verify the running controller reads `/config/config.yaml` with Morgan’s tools present.
- Verify a DocsRun using `githubApp: 5DLabs-Morgan` generates a client-config.json whose `remoteTools` count matches Helm.

## Inputs (Helm Values Snapshot)
Paste the exact Helm values relevant to Morgan that were used for this test.

```yaml
agents:
  morgan:
    githubApp: "5DLabs-Morgan"
    tools:
      remote:
        - rustdocs_query_rust_docs
        # - brave_web_search  # (include here only if you’re testing two tools via Helm)
```

## Procedure
1) Render Helm and confirm Morgan tools appear under `agents:` in `task-controller-config.yaml`.
2) Inspect the live ConfigMap in-cluster and confirm the same.
3) Confirm the controller pod can read `/config/config.yaml` (mounted file) and includes Morgan.
4) Create a minimal DocsRun using `githubApp: 5DLabs-Morgan`.
5) Inspect the docs pod logs for the client-config summary and verify `remoteTools=<N>` where N equals the number of Helm tools.

## Commands Used
Update placeholders and run these. Capture outputs in Evidence sections below.

```bash
# 1) Render chart
helm template <release> infra/charts/controller \
  -n <namespace> \
  | sed -n '/name: .*task-controller-config/,$p' \
  | sed -n '/config.yaml:/,$p' > _evidence/01-helm-template-task-controller-config.yaml

# 2) Live ConfigMap
kubectl -n <namespace> get cm <release>-task-controller-config -o yaml \
  | sed -n '/config.yaml:/,$p' > _evidence/02-live-cm-task-controller-config.yaml

# 3) Controller mounted file
kubectl -n <namespace> exec deploy/<release> -c controller -- \
  sh -lc 'sed -n "1,200p" /config/config.yaml' > _evidence/03-controller-mounted-config.yaml

# 4) Submit DocsRun (adjust repo/branch)
cat > _evidence/docsrun.yaml <<'YAML'
apiVersion: agents.platform/v1
kind: DocsRun
metadata:
  generateName: morgan-helm-test-
  namespace: <namespace>
spec:
  workingDirectory: "."
  githubApp: "5DLabs-Morgan"
  sourceBranch: "main"
  repositoryUrl: "https://github.com/5dlabs/cto"
  model: "claude-sonnet-4-20250514"
  includeCodebase: false
YAML
kubectl apply -f _evidence/docsrun.yaml

# Wait for the pod, then get logs
DOCSRUN=$(kubectl -n <namespace> get docsruns -o jsonpath='{.items[-1:].0.metadata.name}')
kubectl -n <namespace> get pods -l agents.platform/run-name=$DOCSRUN -o name
POD=$(kubectl -n <namespace> get pods -l agents.platform/run-name=$DOCSRUN -o jsonpath='{.items[0].metadata.name}')

# 5) Capture the summary line and file
kubectl -n <namespace> logs $POD -c main | tee _evidence/04-docs-pod.log | rg "\[client-config\] summary:"
kubectl -n <namespace> exec $POD -c main -- \
  sh -lc 'jq .remoteTools $CLAUDE_WORK_DIR/client-config.json' \
  | tee _evidence/05-client-config-remoteTools.json
```

## Evidence

### 1. Helm Template Output (Rendered)
Paste or link snippet showing `agents.morgan.tools`. Keep to the relevant block.

```yaml
# excerpt from _evidence/01-helm-template-task-controller-config.yaml
# ... paste block here ...
```

### 2. Live ConfigMap (In-Cluster)
Confirm Morgan tools match Helm template.

```yaml
# excerpt from _evidence/02-live-cm-task-controller-config.yaml
# ... paste block here ...
```

### 3. Controller Mounted Config (/config/config.yaml)
Confirm Morgan appears here as well.

```yaml
# excerpt from _evidence/03-controller-mounted-config.yaml
# ... paste block here ...
```

### 4. Docs Pod Log Summary
Include the exact line showing counts.

```
# excerpt from _evidence/04-docs-pod.log
[client-config] summary: remoteTools=<N>, localServers.keys=<...>
```

### 5. client-config.json Remote Tools (Inside Pod)
Should list the Helm tools exactly (order not guaranteed).

```json
// from _evidence/05-client-config-remoteTools.json
[
  "rustdocs_query_rust_docs"
]
```

## Results Summary
- Expected Helm tools count: <!-- e.g., 1 or 2 -->
- Observed `remoteTools` count: <!-- from summary/logs -->
- Match? (Yes/No): <!-- choose -->
- Status: <!-- PASS / FAIL -->

| Stage | Expectation | Observed | Status |
|---|---|---|---|
| Helm render | agents.morgan.tools present | <!-- yes/no + snippet ref --> | <!-- PASS/FAIL --> |
| Live ConfigMap | Same as render | <!-- yes/no --> | <!-- PASS/FAIL --> |
| Controller mount | Same as live CM | <!-- yes/no --> | <!-- PASS/FAIL --> |
| DocsRun logs | remoteTools=N | <!-- value --> | <!-- PASS/FAIL --> |
| In-pod file | remoteTools list equals Helm | <!-- list --> | <!-- PASS/FAIL --> |

## Conclusion
- Overall Result: <!-- PASS/FAIL -->
- Notes / Root Cause (if FAIL): <!-- brief, actionable -->

## Follow-Ups
- Helm change needed? <!-- yes/no + summary -->
- Additional tests? <!-- e.g., add brave_web_search to Helm and re-run -->
- Linked issues/PRs: <!-- IDs/links -->

---

## Agent Notes (Freeform)
Provide any anomalies, transient errors, or environment quirks seen during the run.

