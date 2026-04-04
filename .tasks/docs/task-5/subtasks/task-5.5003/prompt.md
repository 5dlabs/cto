Implement subtask 5003: Implement Linear bridge notification with auth and payload formatting

## Objective
Create the Linear-specific notification function that POSTs to the Linear bridge URL with the correct Authorization header and Linear-specific payload shape including session ID and delegate assignments.

## Steps
1. Create `src/notifications/linear-notifier.ts`.
2. Export `createLinearNotifier(httpClient: HttpClient, config: { linearBridgeUrl: string, serviceApiKey: string })`.
3. Returns a function `sendLinearNotification(event: 'pipeline_start' | 'pipeline_complete', payload: PipelineEventPayload): Promise<void>`.
4. Build the outgoing POST body: `{ event, pipelineRunId: payload.pipelineRunId, prdTitle: payload.prdTitle, timestamp: payload.timestamp, status: payload.status, linearSessionId: payload.linearSessionId, issueCount: payload.issueCount, delegateAssignmentSummary: payload.delegateAssignmentSummary }`.
5. Set headers: `{ 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + config.serviceApiKey }`.
6. POST to `config.linearBridgeUrl`. If response status >= 400, throw an error including the status code.
7. Add structured log on success: `{ level: 'info', stage: 'notification', bridge: 'linear', event, status: 'sent' }`.

## Validation
Unit test: mock HTTP client returning 200; assert POST was made to the correct Linear URL with correct Authorization header and payload containing linearSessionId and delegateAssignmentSummary. Error test: mock HTTP client returning 500; assert the function throws. Auth test: assert Authorization header is present and non-empty.