Implement subtask 5001: Implement DiscordNotifier service class with embed builders and retry logic

## Objective
Create the core DiscordNotifier service class that reads DISCORD_WEBHOOK_URL from the environment, constructs Discord embed payloads for pipeline start and complete events, sends them via fetch POST, and handles retry logic for 429/5xx responses. Gracefully degrades when the webhook URL is not configured.

## Steps
1. Create `src/services/discord-notifier.ts`.
2. In the constructor, read `DISCORD_WEBHOOK_URL` from `process.env` (populated via sigma1-infra-endpoints ConfigMap envFrom). Store it as a private field. If not set, store `null`.
3. Implement a private `sendWebhook(payload: object)` method that:
   a. If `this.webhookUrl` is null, log a warning (`console.warn('DISCORD_WEBHOOK_URL not set, skipping notification')`) and return immediately.
   b. POST to the webhook URL with `Content-Type: application/json` and the serialized payload.
   c. On 429 or 5xx response, retry up to 2 times with a 1-second delay (`await Bun.sleep(1000)`).
   d. On final failure after retries, log the error but do NOT throw (fire-and-forget semantics).
4. Implement `notifyPipelineStart(runId: string, prdTitle: string, timestamp: string)` that calls `sendWebhook` with a Discord embed: color `0x3498db` (blue), title `🚀 Pipeline Started`, fields for Run ID, PRD Title, Started At.
5. Implement `notifyPipelineComplete(runId: string, prdTitle: string, status: 'success' | 'failure', taskCount: number, issueCount: number, prUrl: string, timestamp: string)` that calls `sendWebhook` with: color `0x2ecc71` for success / `0xe74c3c` for failure, title `✅ Pipeline Complete` or `❌ Pipeline Failed`, fields for Run ID, PRD Title, Status, Tasks Generated, Issues Created, PR URL, Completed At.
6. Export the class as a singleton or as a constructable service for DI.

## Validation
Unit test: mock global fetch. Call notifyPipelineStart and assert the POST body contains the correct embed structure with blue color, correct title, and all three fields. Call notifyPipelineComplete with status='success' and verify green color and success title; call with status='failure' and verify red color and failure title with all seven fields present. Test missing DISCORD_WEBHOOK_URL: instantiate with env unset, call both methods, verify zero fetch calls and a warning was logged. Test retry: mock fetch to return 429 on first call then 200 on second; verify two fetch calls were made with ~1s delay.