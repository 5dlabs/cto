Implement subtask 5004: Wire notification methods into pipeline lifecycle hooks

## Objective
Integrate the Discord and Linear notification methods into the pipeline execution flow so that notifications fire at pipeline start, completion, and error events. Ensure notification failures never block or abort the pipeline.

## Steps
1. Identify the pipeline orchestration entry points in cto-pm (e.g., the main pipeline runner function/class).
2. At pipeline initialization (after runId is assigned and tasks are parsed):
   - Call `notifyDiscordPipelineStart(runId, prdTitle, taskCount)` and `notifyLinearPipelineStart(runId, prdTitle, taskCount, linearSessionId)` concurrently using `Promise.allSettled`.
3. At pipeline completion (after all stages — issue creation, PR creation — are done):
   - Gather summary: tasksCreated, agentsAssigned, prUrl (from Task 4 output), any accumulated warnings.
   - Call `notifyDiscordPipelineComplete(runId, summary)` and `notifyLinearPipelineComplete(runId, summary)` concurrently via `Promise.allSettled`.
4. At pipeline error (on any stage failure or non-fatal warning):
   - Call `notifyDiscordPipelineError(runId, stage, error)` and `notifyLinearPipelineError(runId, stage, error, linearSessionId)` concurrently via `Promise.allSettled`.
   - Ensure the pipeline continues with graceful degradation after the error notification.
5. Wrap all notification dispatches in `Promise.allSettled` (not `Promise.all`) so a rejected notification promise never propagates.
6. Add structured log entries at each hook point: 'notifications_dispatched', event type, number of bridges notified, any settled rejections.

## Validation
Unit test: mock NotificationService methods; trigger pipeline start and verify both notifyDiscordPipelineStart and notifyLinearPipelineStart are called with correct args. Unit test: trigger pipeline complete and verify both complete notifications fire with summary including prUrl. Unit test: trigger pipeline error and verify both error notifications fire; confirm the pipeline function does not throw. Integration test: run a full pipeline flow (mocked stages) and verify at least 2 Discord and 2 Linear notification calls occur (start + complete). Integration test: make one bridge mock return a rejection — verify the other bridge still receives its notification and the pipeline completes successfully.