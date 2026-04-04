Implement subtask 5001: Create notification dispatcher module with uniform interface and injectable HTTP client

## Objective
Build the core notification dispatcher module that defines the PipelineEventPayload type, the uniform `notify(event, payload)` interface, and accepts an injectable HTTP client for testability. This module orchestrates calls to individual bridge notifiers and applies graceful degradation (try/catch per bridge, warn-level logging on final failure, never throws).

## Steps
1. Create `src/notifications/dispatcher.ts`.
2. Define `PipelineEventPayload` type with fields: `pipelineRunId: string`, `prdTitle: string`, `timestamp: string`, `status: string`, `taskCount?: number`, `issueCount?: number`, `linearSessionId?: string`, `delegateAssignmentSummary?: Record<string, unknown>`.
3. Define an `HttpClient` interface: `{ post(url: string, body: unknown, headers: Record<string, string>): Promise<{ status: number }> }`. Default implementation wraps `fetch`.
4. Export a factory function `createNotificationDispatcher(httpClient: HttpClient, config: { discordBridgeUrl: string, linearBridgeUrl: string, serviceApiKey: string })` that returns `{ notify(event: 'pipeline_start' | 'pipeline_complete', payload: PipelineEventPayload): Promise<void> }`.
5. Inside `notify`, call Discord and Linear notifiers concurrently via `Promise.allSettled`. For each settled rejection, log at warn level `{ level: 'warn', stage: 'notification', bridge, event, error }` and continue. Never throw from `notify`.
6. Read `DISCORD_BRIDGE_URL`, `LINEAR_BRIDGE_URL`, and `SERVICE_API_KEY` from environment/ConfigMap in the config initialization code.

## Validation
Unit test: create dispatcher with a mock HTTP client and both URLs. Call `notify('pipeline_start', payload)`. Assert that the mock HTTP client's `post` was called twice (once per bridge). Graceful degradation test: have both mock calls reject; assert `notify` resolves without throwing and warn logs are emitted.