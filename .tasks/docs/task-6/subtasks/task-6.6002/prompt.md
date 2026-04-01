Implement subtask 6002: Implement LinearDiscordBridge service with 2-second batching and fire-and-forget execution

## Objective
Create the `LinearDiscordBridge` service that listens for `issue.created` events, batches events within a 2-second window, formats a purple Discord embed, and sends via the existing DiscordNotifier in a fire-and-forget manner with error logging. Include the ENABLE_ISSUE_DISCORD_BRIDGE configuration toggle.

## Steps
1. Create `src/services/linear-discord-bridge.ts`.
2. Read `ENABLE_ISSUE_DISCORD_BRIDGE` from env (default `'true'`). If disabled, register the listener as a no-op or skip registration entirely.
3. Subscribe to `issueEvents.on('issue.created', handler)`.
4. Implement a batching buffer: on the first event in an idle state, start a 2-second timer. Accumulate all incoming events into an array. When the timer fires, flush the batch.
5. On flush, build a Discord embed object:
   - Color: `0x9b59b6` (purple).
   - Title: `'📋 New Issue Created'` (or `'📋 New Issues Created'` if batch size > 1).
   - For each issue in the batch, add embed fields: `Issue Title` (value is a markdown link `[title](issueUrl)`), `Assigned To` (`assigneeName || 'Unassigned'`), `Agent Hint`.
6. Call `DiscordNotifier.sendEmbed(embed)` (from Task 5). Wrap the call in a try/catch — on error, log the error with issue IDs but do NOT re-throw or propagate.
7. Ensure the entire listener is fire-and-forget: the `issue.created` handler should not return a promise that the emitter awaits. Use `void` or `queueMicrotask`/`setTimeout` to detach.
8. Export an `initLinearDiscordBridge()` function called at server startup to wire the listener.
9. Handle edge case: if the batch grows very large (e.g., 25+ issues), split into multiple embeds to stay within Discord's 25-field limit per embed.

## Validation
1. Unit test: emit a single `issue.created` event, advance timers by 2 seconds, verify `DiscordNotifier.sendEmbed` is called once with a purple embed containing the correct issue title as a markdown link and correct assignee. 2. Unit test: emit 5 events within 1 second, advance timers, verify only 1 call to `sendEmbed` with all 5 issues in the fields. 3. Unit test: set `ENABLE_ISSUE_DISCORD_BRIDGE=false`, emit events, verify zero calls to `sendEmbed`. 4. Unit test: mock `sendEmbed` to throw, emit event, advance timers, verify the error is logged and no unhandled promise rejection occurs.