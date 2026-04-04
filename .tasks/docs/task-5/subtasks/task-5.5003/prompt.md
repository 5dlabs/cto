Implement subtask 5003: Implement Linear bridge notification integration with session references

## Objective
Add Linear-specific notification methods to NotificationService that POST to LINEAR_BRIDGE_URL. Include Linear session/project references and PR links in the payload.

## Steps
1. In `notification.service.ts` (or a dedicated `linear-notifier.ts` that composes the base service), implement:
   - `notifyLinearPipelineStart(runId: string, prdTitle: string, taskCount: number, linearSessionId?: string)`: payload includes run metadata and optional Linear session/project reference.
   - `notifyLinearPipelineComplete(runId: string, summary: {tasksCreated: number; agentsAssigned: string[]; prUrl?: string; linearSessionId?: string; warnings?: string[]})`: payload includes session reference and PR URL.
   - `notifyLinearPipelineError(runId: string, stage: string, error: string, linearSessionId?: string)`: payload includes error context and session reference.
2. Payload format: construct JSON matching linear-bridge's expected schema. Include fields: `type` (start/complete/error), `runId`, `sessionId`, `prUrl`, `summary`. Add a TODO/comment noting the format may need adjustment after bridge API discovery.
3. Call `postWithRetry(LINEAR_BRIDGE_URL, payload)` for each.
4. Export these methods for use by the lifecycle hook layer.

## Validation
Unit test: notifyLinearPipelineStart constructs a payload with runId, prdTitle, taskCount, and linearSessionId, and calls postWithRetry with LINEAR_BRIDGE_URL. Unit test: notifyLinearPipelineComplete includes prUrl and session reference in the payload. Unit test: notifyLinearPipelineError includes stage and error in the payload. Unit test: when linearSessionId is undefined, it is omitted or null in the payload without error.