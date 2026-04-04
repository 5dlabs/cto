Implement subtask 5002: Implement Discord bridge notification with auth and payload formatting

## Objective
Create the Discord-specific notification function that POSTs to the Discord bridge URL with the correct Authorization header and Discord-specific payload shape.

## Steps
1. Create `src/notifications/discord-notifier.ts`.
2. Export `createDiscordNotifier(httpClient: HttpClient, config: { discordBridgeUrl: string, serviceApiKey: string })`.
3. Returns a function `sendDiscordNotification(event: 'pipeline_start' | 'pipeline_complete', payload: PipelineEventPayload): Promise<void>`.
4. Build the outgoing POST body: `{ event, pipelineRunId: payload.pipelineRunId, prdTitle: payload.prdTitle, timestamp: payload.timestamp, status: payload.status, taskCount: payload.taskCount, issueCount: payload.issueCount }`.
5. Set headers: `{ 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + config.serviceApiKey }`.
6. POST to `config.discordBridgeUrl`. If response status >= 400, throw an error including the status code for the retry layer to catch.
7. Add structured log on success: `{ level: 'info', stage: 'notification', bridge: 'discord', event, status: 'sent' }`.

## Validation
Unit test: mock HTTP client returning 200; assert POST was made to the correct URL with correct Authorization header and payload fields. Error test: mock HTTP client returning 502; assert the function throws an error containing the status code. Auth test: assert the Authorization header value matches `Bearer <key>` with the configured API key.