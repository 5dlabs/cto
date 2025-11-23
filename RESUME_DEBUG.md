# Resume Debug Log (Cleo Stage)

_Last updated: 2025-11-23_

## Observed Behavior
- Rex completes successfully but the workflow never advances into the Cleo quality stage; when the play workflow is re-run it restarts at Task 1 (implementation) instead of resuming at Cleo.
- Sensors meant to resume quality work do trigger, but they find no suspended workflow waiting at the Cleo entry point.

## Expected Resume Flow (from code)
- **Resume gate**: the DAG always begins by calling `determine-resume-point`, which looks up the per-repo `play-progress-{repo}` ConfigMap to decide which stage should run. Missing ConfigMaps or mismatched task IDs force a restart from `implementation`.

```393:446:infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml
- - name: check-resume-point
    template: determine-resume-point
    arguments:
      parameters:
        - name: task-id
          value: "{{`{{workflow.parameters.task-id}}`}}"
        - name: repository
          value: "{{`{{workflow.parameters.repository}}`}}"
...
        case "$STORED_STAGE" in
          "quality-in-progress")
            RESUME_STAGE="quality"
            echo "✅ Resuming at: Code Quality (Cleo)"
            ;;
```

- **Stage skipping logic**: `check-stage-needed` compares the requested resume stage with each stage in order (implementation → quality → …). Anything “behind” the resume point is skipped, so Cleo should not rerun if the ConfigMap said `quality`.

```2274:2328:infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml
STAGES=("implementation" "quality" "security" "testing" "waiting-merge" "completed")
...
if [ $CURRENT_IDX -lt $RESUME_IDX ]; then
  echo "⏭️  Skipping $CURRENT_STAGE (already completed, resuming at $RESUME_STAGE)"
  echo "false" > /tmp/should-run.txt
else
  echo "▶️  Running $CURRENT_STAGE"
  echo "true" > /tmp/should-run.txt
fi
```

- **ConfigMap updates**: Each stage transition should patch the same ConfigMap with the new stage label, preserving state for later resumes.

```1505:1524:infra/charts/controller/templates/workflowtemplates/play-workflow-template.yaml
if kubectl get configmap "$CONFIGMAP_NAME" -n {{ .Release.Namespace }} >/dev/null 2>&1; then
  kubectl patch configmap "$CONFIGMAP_NAME" \
    -n {{ .Release.Namespace }} \
    --type merge \
    -p "{\"data\":{\"stage\":\"{{`{{inputs.parameters.new-stage}}`}}\",\"last-updated\":\"$CURRENT_TIMESTAMP\",\"workflow-name\":\"{{`{{workflow.name}}`}}\",\"current-task-id\":\"$TASK_ID\"}}"
else
  echo "ConfigMap does not exist yet - will be created by controller"
fi
```

- **Initial ConfigMap creation**: The MCP server writes the `play-progress` ConfigMap as soon as the workflow submission succeeds, seeding it with `stage=implementation`.

```2492:2504:mcp/src/main.rs
if let Some(ref wf_name) = workflow_name {
    let progress = PlayProgress {
        repository: repository.clone(),
        branch: "main".to_string(),
        current_task_id: Some(task_id),
        workflow_name: Some(wf_name.clone()),
        status: PlayStatus::InProgress,
        stage: Some("implementation".to_string()),
    };

    if let Err(e) = write_play_progress(&progress) {
        eprintln!("⚠️  Failed to write progress ConfigMap: {e}");
    }
}
```

- **PR-created sensor expectations**: The webhook sensor still polls for a suspend node named `wait-for-pr-created`, even though that suspend point was removed from the workflow. When it fails to find that node, it logs “skipping resume,” leaving the workflow to time out.

```499:543:infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml
echo "Polling for workflow reaching wait-for-pr-created suspend point..."
MAX_WORKFLOW_ATTEMPTS=90
...
SUSPEND_NODE=$(echo "$WORKFLOW_JSON" | \
  jq -r '.items[0].status.nodes | to_entries[] | select(.value.displayName == "wait-for-pr-created") | .value.id' \
  2>/dev/null || echo "")
...
if [ -z "$SUSPEND_NODE" ]; then
  echo "⚠️ Workflow did not reach wait-for-pr-created suspend point in time; skipping resume"
  exit 0
fi
```

## Findings So Far
1. **ConfigMap gating makes Cleo entirely dependent on stage writes.** If the ConfigMap is missing or still says the task is in `implementation`, the `determine-resume-point` step will instruct the DAG to start over. Any failure to patch the ConfigMap (RBAC, race conditions, or the MCP write never running) explains why reruns always restart at Rex.
2. **Stage updates are best-effort.** `update-workflow-stage` only patches the ConfigMap if it already exists; otherwise it logs a warning and moves on. If the MCP-side write did not occur (or was GC’d), Cleo will never see `quality-in-progress`.
3. **Sensors are still waiting on a non-existent suspend node.** The PR-created sensor explicitly looks for the `wait-for-pr-created` suspend node to know when to issue `argo resume`. That node no longer exists in the workflow, so the sensor always aborts, which matches the reported behavior of “it should have resumed, but it didn’t.” Instead of resuming the running workflow, users are re-submitting a brand new workflow that naturally replays Rex.
4. **Manual re-runs can never “skip ahead.”** When we submit a fresh workflow, the MCP bootstrap writes `stage=implementation` into the ConfigMap before Rex even runs. Unless we manually update the ConfigMap, every new workflow necessarily starts at stage 1. The resume story relies on sensors resuming the *existing* workflow, not on submitting a second workflow.
5. **Missing visibility data.** We still need real cluster data (ConfigMap contents, workflow labels, sensor logs) to confirm whether the ConfigMap was ever updated past `implementation`. Right now the evidence is purely from code inspection.

## Evidence We Still Need
Run these in the cluster to validate assumptions:
- `kubectl get configmap play-progress-<owner>-<repo> -n agent-platform -o yaml` → verify `current-task-id`, `stage`, and `last-updated`.
- `kubectl get wf -n agent-platform -l task-id=<id>,workflow-type=play-orchestration -o yaml` → confirm the `current-stage` label after Rex succeeds.
- `kubectl logs -n argo -l sensor-name=play-workflow-pr-created` → look for the “Skipping resume” message tied to `wait-for-pr-created`.
- `kubectl logs -n agent-platform -l workflows.argoproj.io/workflow=<wf-name>` → inspect `update-workflow-stage` output to see if ConfigMap patches failed.

## Suggested Next Steps
1. Capture the actual ConfigMap + workflow labels immediately after Rex completes to see whether stage persisted to `quality-in-progress`.
2. Patch or replace the PR-created sensor so it no longer relies on the removed `wait-for-pr-created` suspend node.
3. Decide whether `update-workflow-stage` should create the ConfigMap when it is missing (right now it silently skips).
4. Once the above is fixed, rerun a single task and confirm Cleo picks up automatically without re-submitting a fresh workflow.

_End of Cleo-focused log._

