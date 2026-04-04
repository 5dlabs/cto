Implement subtask 2006: Implement POST /api/pipeline/delegate-status observability endpoint

## Objective
Add a new Elysia route POST /api/pipeline/delegate-status that returns the current agent-to-Linear-user-ID delegate mapping as JSON for observability purposes.

## Steps
1. In the Elysia router file (e.g., `src/routes/pipeline.ts`), add a new route: `app.post('/api/pipeline/delegate-status', handler)`.
2. The handler calls `getDelegateMap()` from the delegate resolution module.
3. Return a 200 response with JSON body: `{ delegates: Record<string, string>, count: number, timestamp: string }`.
4. Add Elysia schema validation on the response body for type safety.
5. Emit a structured log on each call: `{ level: 'info', stage: 'delegate_status', action: 'queried' }`.

## Validation
Integration test: POST /api/pipeline/delegate-status returns 200 with a JSON body containing a 'delegates' object with known agent hints as keys and Linear user IDs as values, a numeric 'count', and an ISO timestamp string.