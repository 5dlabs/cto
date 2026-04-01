Implement subtask 6003: Write comprehensive unit and integration tests for the Linear-Discord bridge

## Objective
Create a full test suite covering single event notification, batched notifications, disabled toggle, Discord failure resilience, and end-to-end integration with the pipeline.

## Steps
1. Create `src/services/__tests__/linear-discord-bridge.test.ts` using Bun's test runner.
2. Mock `DiscordNotifier.sendEmbed` and the `issueEvents` emitter.
3. Test cases:
   a. **Single event**: emit one `issue.created`, advance fake timers by 2s, assert `sendEmbed` called once with embed color `0x9b59b6`, title `'📋 New Issue Created'`, and fields containing the issue title as a link, assignee name, and agent hint.
   b. **Batched events**: emit 5 events within 500ms, advance timers by 2s, assert `sendEmbed` called once with 5 sets of fields.
   c. **Batch splitting**: emit 30 events, verify multiple `sendEmbed` calls each with ≤25 fields.
   d. **Toggle disabled**: set env `ENABLE_ISSUE_DISCORD_BRIDGE=false`, reinitialize bridge, emit events, assert zero `sendEmbed` calls.
   e. **Unassigned handling**: emit event with `assigneeName: null`, verify embed field shows 'Unassigned'.
   f. **Discord failure**: mock `sendEmbed` to reject, emit event, advance timers, verify error is logged (spy on logger) and no exception propagates.
   g. **Non-blocking**: verify that the issue creation function returns before `sendEmbed` is called (using timing assertions or call order tracking).
4. Integration test (can be marked as e2e): trigger a pipeline run that creates 5+ issues via the PM server, capture Discord webhook requests (via a mock server or intercepted fetch), and verify batched embed(s) arrive with correct content.

## Validation
All tests pass with `bun test`. Coverage report shows ≥90% line coverage for `linear-discord-bridge.ts`. Each test case listed above has a corresponding passing test. Integration test verifies end-to-end flow with mock Discord endpoint.