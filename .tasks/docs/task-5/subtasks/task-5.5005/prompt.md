Implement subtask 5005: Integrate notification dispatcher into pipeline orchestrator lifecycle hooks

## Objective
Hook the notification dispatcher into the pipeline orchestrator so that `notify('pipeline_start')` fires at pipeline start and `notify('pipeline_complete')` fires at the end regardless of partial failures in other stages.

## Steps
1. Locate the pipeline orchestrator entry point (e.g., `src/pipeline/orchestrator.ts` or equivalent).
2. Import and instantiate the notification dispatcher using environment config (`DISCORD_BRIDGE_URL`, `LINEAR_BRIDGE_URL`, `SERVICE_API_KEY`).
3. At the very beginning of the pipeline run function, call `await dispatcher.notify('pipeline_start', { pipelineRunId, prdTitle, timestamp: new Date().toISOString(), status: 'started' })`.
4. At the very end of the pipeline (in a `finally` block or equivalent), call `await dispatcher.notify('pipeline_complete', { pipelineRunId, prdTitle, timestamp: new Date().toISOString(), status, taskCount, issueCount, linearSessionId, delegateAssignmentSummary })`.
5. Ensure the `pipeline_complete` notification fires even if issue creation or other stages partially failed — place it after all other stages in a finally/catch-and-continue pattern.
6. Add structured log: `{ level: 'info', stage: 'pipeline', action: 'notification_hook', event: 'pipeline_start' | 'pipeline_complete' }` at each hook point.

## Validation
Integration test: trigger a full pipeline run with mocked bridge endpoints returning 200; assert PM server logs contain `stage: 'notification'` entries for both `pipeline_start` and `pipeline_complete`. Failure resilience test: trigger a pipeline run where issue creation fails; assert `pipeline_complete` notification was still attempted. Isolation test: have both bridge endpoints return 500 for all retries; assert the pipeline run completes successfully (exits without error).