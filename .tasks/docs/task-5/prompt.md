Implement task 5: Implement Discord and Linear Bridge Notifications (Nova - Bun/Elysia)

## Goal
Extend the PM server to dispatch pipeline lifecycle notifications (start and complete events) to both the in-cluster Discord bridge and Linear bridge services. Notification failures must not block pipeline completion — implement graceful degradation with retries and logging. Uses the existing bots/discord-bridge-http and bots/linear-bridge services with API key authentication per D2 and D5.

## Task Context
- Agent owner: nova
- Stack: Bun/Elysia
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Create a notification dispatcher module with a uniform interface: `notify(event: 'pipeline_start' | 'pipeline_complete', payload: PipelineEventPayload): Promise<void>`.
2. Implement Discord notification: POST to the `DISCORD_BRIDGE_URL` (from ConfigMap) with the `SERVICE_API_KEY` in an `Authorization: Bearer <key>` header. Payload: `{ event, pipelineRunId, prdTitle, timestamp, status, taskCount?, issueCount? }`.
3. Implement Linear notification: POST to the `LINEAR_BRIDGE_URL` (from ConfigMap) with the same auth header pattern. Payload includes Linear session ID, issue count, and delegate assignment summary.
4. Implement retry policy: 3 attempts with exponential backoff (1s, 2s, 4s) for each bridge call independently.
5. Implement graceful degradation: wrap each notification call in a try/catch. On final failure after retries, log `{ level: 'warn', stage: 'notification', bridge: 'discord' | 'linear', event, error: message }` and return without throwing. The pipeline must continue regardless.
6. Hook the dispatcher into the pipeline orchestrator: call `notify('pipeline_start', ...)` at the beginning of the pipeline, and `notify('pipeline_complete', ...)` at the end (after issue creation, regardless of partial failures in other stages).
7. Add structured logging for each notification attempt and outcome.
8. Ensure the notification module is independently testable (dependency injection for HTTP client).

## Acceptance Criteria
1. Unit test: mock both bridge endpoints returning 200; assert `notify('pipeline_start', payload)` makes POST requests to both URLs with correct Authorization header and payload shape. 2. Retry test: mock Discord bridge returning 500 twice then 200; assert 3 attempts made, final result is success, 2 retry log entries emitted. 3. Graceful degradation test: mock both bridges returning 500 for all attempts; assert `notify` resolves (does not throw), warn-level logs are emitted for both bridges, and the calling pipeline stage continues. 4. Integration test: trigger a full pipeline run; assert at least 2 notification POST requests were made (one start, one complete) by inspecting PM server logs for `stage: 'notification'` entries. 5. Auth test: assert all outbound requests include the `Authorization: Bearer` header with a non-empty value.

## Subtasks
- Create notification dispatcher module with uniform interface and injectable HTTP client: Build the core notification dispatcher module that defines the PipelineEventPayload type, the uniform `notify(event, payload)` interface, and accepts an injectable HTTP client for testability. This module orchestrates calls to individual bridge notifiers and applies graceful degradation (try/catch per bridge, warn-level logging on final failure, never throws).
- Implement Discord bridge notification with auth and payload formatting: Create the Discord-specific notification function that POSTs to the Discord bridge URL with the correct Authorization header and Discord-specific payload shape.
- Implement Linear bridge notification with auth and payload formatting: Create the Linear-specific notification function that POSTs to the Linear bridge URL with the correct Authorization header and Linear-specific payload shape including session ID and delegate assignments.
- Implement retry policy with exponential backoff and structured logging: Create a generic retry wrapper that applies 3 attempts with exponential backoff (1s, 2s, 4s) to any async function, with structured logging for each attempt and outcome. Wire it into the Discord and Linear notifiers within the dispatcher.
- Integrate notification dispatcher into pipeline orchestrator lifecycle hooks: Hook the notification dispatcher into the pipeline orchestrator so that `notify('pipeline_start')` fires at pipeline start and `notify('pipeline_complete')` fires at the end regardless of partial failures in other stages.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.