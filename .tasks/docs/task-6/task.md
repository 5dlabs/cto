## Implement Linear-Discord Bridge for Issue Notifications (Nova - Bun/Elysia)

### Objective
Bridge Linear issue creation events to Discord, so that each new issue created during the pipeline run is announced in real time to the configured Discord channel with its title, assignee, and link.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: medium
- Status: pending
- Dependencies: 1, 2, 5

### Implementation Details
1. Extend the PM server's issue creation flow (from Task 2) to emit an internal event `issue.created` after each successful Linear issue creation, with payload `{ issueId, issueUrl, title, agentHint, assigneeName }`.
2. Create a `LinearDiscordBridge` service that listens for `issue.created` events.
3. On each event, use the existing `DiscordNotifier` (from Task 5) to post an embed:
   - Color: purple (#9b59b6).
   - Title: '📋 New Issue Created'.
   - Fields: Issue Title (linked to issueUrl), Assigned To (assigneeName or 'Unassigned'), Agent Hint.
4. Batch notifications: if multiple issues are created within a 2-second window, batch them into a single embed with multiple field rows to avoid Discord rate limiting.
5. Ensure the bridge does not block the main issue creation flow — use fire-and-forget with error logging.
6. Add a configuration toggle `ENABLE_ISSUE_DISCORD_BRIDGE` (default: true) to allow disabling without code changes.

### Subtasks
- [ ] Emit issue.created event from the issue creation flow: Extend the PM server's Linear issue creation logic (from Task 2) to emit an internal `issue.created` event after each successful issue creation, carrying the full notification payload.
- [ ] Implement LinearDiscordBridge service with 2-second batching and fire-and-forget execution: Create the `LinearDiscordBridge` service that listens for `issue.created` events, batches events within a 2-second window, formats a purple Discord embed, and sends via the existing DiscordNotifier in a fire-and-forget manner with error logging. Include the ENABLE_ISSUE_DISCORD_BRIDGE configuration toggle.
- [ ] Write comprehensive unit and integration tests for the Linear-Discord bridge: Create a full test suite covering single event notification, batched notifications, disabled toggle, Discord failure resilience, and end-to-end integration with the pipeline.