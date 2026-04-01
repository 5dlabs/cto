## Implement Discord Notification for Pipeline Start/Complete (Nova - Bun/Elysia)

### Objective
Add Discord webhook notification hooks to the PM server pipeline, posting structured embed messages when the pipeline starts and when it completes (successfully or with errors) to the configured Discord channel.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Create a `DiscordNotifier` service class in the PM server.
2. Read `DISCORD_WEBHOOK_URL` from environment (sourced via sigma1-infra-endpoints ConfigMap).
3. Implement `notifyPipelineStart(runId, prdTitle, timestamp)` which sends a POST to the webhook with a Discord embed:
   - Color: blue (#3498db).
   - Title: '🚀 Pipeline Started'.
   - Fields: Run ID, PRD Title, Started At.
4. Implement `notifyPipelineComplete(runId, prdTitle, status, taskCount, issueCount, prUrl, timestamp)` which sends:
   - Color: green (#2ecc71) for success, red (#e74c3c) for failure.
   - Title: '✅ Pipeline Complete' or '❌ Pipeline Failed'.
   - Fields: Run ID, PRD Title, Status, Tasks Generated, Issues Created, PR URL, Completed At.
5. Hook `notifyPipelineStart` at the very beginning of the intake pipeline handler.
6. Hook `notifyPipelineComplete` in the pipeline's finally block so it fires on both success and failure.
7. If `DISCORD_WEBHOOK_URL` is not set, log a warning and skip notifications without throwing.
8. Add retry logic: up to 2 retries with 1-second backoff on 429/5xx responses from Discord.

### Subtasks
- [ ] Implement DiscordNotifier service class with embed builders and retry logic: Create the core DiscordNotifier service class that reads DISCORD_WEBHOOK_URL from the environment, constructs Discord embed payloads for pipeline start and complete events, sends them via fetch POST, and handles retry logic for 429/5xx responses. Gracefully degrades when the webhook URL is not configured.
- [ ] Hook DiscordNotifier into the intake pipeline handler: Wire the DiscordNotifier into the existing intake pipeline handler so that notifyPipelineStart fires at the very beginning of the pipeline and notifyPipelineComplete fires in a finally block upon success or failure.