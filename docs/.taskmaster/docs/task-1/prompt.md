Implement a small runner (bash script or minimal WorkflowTemplate) that:


- Lists `docs/.taskmaster/docs/task-*` and extracts numeric IDs ascending
- For each ID: calls `argo submit --from workflowtemplate/play-workflow-template -p task-id=<id>` in namespace agent-platform


- Waits for completion of the submitted workflow (Succeeded) before proceeding


- Emits progress logs; provide `--start-from <id>` option
Prefer a bash script placed at `scripts/play-runner.sh` with clear usage.
