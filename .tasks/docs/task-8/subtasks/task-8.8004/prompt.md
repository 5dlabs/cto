Implement subtask 8004: Implement Discord notification verification adapter

## Objective
Create an adapter for verifying Discord notification delivery. Supports verifying that at least 2 notifications (pipeline_start and pipeline_complete) were sent with 2xx responses, with correct timestamp ordering.

## Steps
Step-by-step:
1. Create `tests/e2e/adapters/discord.ts` with interface `DiscordTestAdapter` containing methods: `getNotifications(runId: string): Promise<NotificationRecord[]>`, `verifyNotificationPair(records: NotificationRecord[]): VerificationResult`.
2. Define `NotificationRecord` type: `{ eventType: 'pipeline_start' | 'pipeline_complete' | string, timestamp: string, httpStatus: number }`.
3. Define `VerificationResult`: `{ hasBothEvents: boolean, bothSuccessful: boolean, correctOrder: boolean }`.
4. Implement `LogBasedDiscordAdapter`: queries pipeline state or notification logs from `GET /api/pipeline/:runId/notifications` (or equivalent endpoint) to extract notification records.
5. Implement `InterceptorDiscordAdapter` (alternative): sets up an HTTP interceptor/spy on the Discord bridge URL before pipeline run, captures outbound requests, and returns them as NotificationRecords.
6. Implement `verifyNotificationPair` logic: assert >= 2 records, find 'pipeline_start' and 'pipeline_complete', verify both have httpStatus 2xx, verify start timestamp < complete timestamp.
7. Factory function `createDiscordAdapter()` selects based on `E2E_DISCORD_MODE` env var (default: 'log').

## Validation
verifyNotificationPair correctly identifies valid pairs (both events present, both 2xx, correct order) and rejects invalid cases (missing event, non-2xx status, wrong order). Mock data exercises all code paths.