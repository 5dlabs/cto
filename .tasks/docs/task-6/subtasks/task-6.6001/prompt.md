Implement subtask 6001: Emit issue.created event from the issue creation flow

## Objective
Extend the PM server's Linear issue creation logic (from Task 2) to emit an internal `issue.created` event after each successful issue creation, carrying the full notification payload.

## Steps
1. In the existing issue creation module (from Task 2), locate the success path after the Linear API call returns.
2. Define a typed event payload interface: `{ issueId: string, issueUrl: string, title: string, agentHint: string, assigneeName: string | null }`.
3. Use a lightweight in-process event emitter (e.g., Node/Bun `EventEmitter` or a typed wrapper). Export a singleton `issueEvents` emitter from a shared module like `src/events/issue-events.ts`.
4. After successful Linear issue creation, call `issueEvents.emit('issue.created', payload)`. Ensure the emit is non-blocking — do not await any listeners.
5. If `assigneeName` is not available from the Linear response, pass `null` so downstream consumers can display 'Unassigned'.
6. Export the payload type and the emitter for consumers to import.

## Validation
Unit test: mock the Linear API response, call the issue creation function, and verify that the `issue.created` event is emitted with the correct payload shape and values. Verify the event is emitted after (not before) the successful API call.