Implement subtask 5002: Hook DiscordNotifier into the intake pipeline handler

## Objective
Wire the DiscordNotifier into the existing intake pipeline handler so that notifyPipelineStart fires at the very beginning of the pipeline and notifyPipelineComplete fires in a finally block upon success or failure.

## Steps
1. In the intake pipeline handler file (e.g. `src/routes/pipeline.ts` or equivalent Elysia route), import the DiscordNotifier service.
2. At the very beginning of the pipeline handler (after extracting runId, prdTitle, and generating a timestamp), call `await discordNotifier.notifyPipelineStart(runId, prdTitle, new Date().toISOString())`.
3. Wrap the main pipeline logic in a try/finally block.
4. In the finally block, determine the status ('success' or 'failure') based on whether an error was caught.
5. Call `await discordNotifier.notifyPipelineComplete(runId, prdTitle, status, taskCount, issueCount, prUrl, new Date().toISOString())`.
6. Ensure the notifyPipelineComplete call has access to all required data: taskCount and issueCount should default to 0 if the pipeline failed before generating them; prUrl should default to empty string.
7. Ensure that any error from the notifier itself does not mask the original pipeline error (the notifier already swallows its own errors, but verify this at the call site).

## Validation
Integration test: trigger a full pipeline run with a mock/test PRD. Verify that DiscordNotifier.notifyPipelineStart was called once before any task generation begins and DiscordNotifier.notifyPipelineComplete was called once after the pipeline finishes. Test failure path: inject a deliberate error mid-pipeline and verify notifyPipelineComplete is still called with status='failure' and the pipeline error propagates correctly to the caller. End-to-end: with a real DISCORD_WEBHOOK_URL pointed at a test channel, trigger a pipeline run and confirm two embeds appear (start and complete) with the correct run ID.