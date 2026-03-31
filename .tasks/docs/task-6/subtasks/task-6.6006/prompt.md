Implement subtask 6006: Add Hermes logging middleware to Elysia routes and retrofit existing code paths

## Objective
Create Elysia middleware that wraps all Hermes route handlers with structured logging (request/response metadata, timing), and retrofit existing Task 2, 3, and 5 code paths to use the hermes-logger wrapper instead of direct console/logger calls.

## Steps
Step-by-step:
1. Create `src/modules/hermes/logging/hermes-logging-middleware.ts`.
2. Implement an Elysia `onBeforeHandle` + `onAfterHandle` plugin:
   - `onBeforeHandle`: record start time, attach to request context.
   - `onAfterHandle`: compute `duration_ms`, log via HermesLogger with fields: `operation: '{method} {path}'`, `status_code`, `duration_ms`, `request_id` (from header or generated UUID).
   - `onError`: log via `hermesLogger.error()` with `error_code` derived from the error type, `error_message`, and `stack_trace`.
3. Register the middleware on all Hermes route groups (deliberation routes, artifact routes, migration admin routes).
4. Retrofit existing code in Task 2 (deliberation pipeline), Task 3 (artifact storage), and Task 5 (migration) modules:
   - Search for direct `console.log`, `logger.info`, etc. calls in `src/modules/hermes/` directories.
   - Replace with `hermesLogger.info()`, `hermesLogger.error()`, or `hermesLogger.migration()` as appropriate.
   - Ensure all error catches pass structured error codes rather than raw error messages.
5. Add `RollbackMonitor.recordDeliberationResult()` calls in deliberation handler, `recordArtifactWriteResult()` in artifact write handlers.
6. Verify that after retrofitting, all log entries from Hermes routes include the required structured fields.

## Validation
Integration test: Make an API request to a Hermes deliberation endpoint — verify the response includes a request ID header and that Loki query `{app="hermes"} | json | module="hermes"` returns an entry with `operation`, `duration_ms`, `rollout_phase`, and `status_code` fields. Error test: Trigger a deliberation error — verify the error log contains `error_code`, `error_message`, and `stack_trace`. Retrofit validation: Grep `src/modules/hermes/` for any remaining `console.log` or non-HermesLogger log calls — expect zero results. Verify RollbackMonitor integration: trigger 25 deliberation failures via API — verify rollback trigger log appears.