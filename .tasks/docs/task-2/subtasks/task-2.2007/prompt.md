Implement subtask 2007: Standardize structured JSON logging across all pipeline stages

## Objective
Ensure all pipeline stages (delegate resolution, idempotency checks, issue creation, retries, delegate-status endpoint) emit structured JSON logs with a consistent schema including the 'stage' field for filtering.

## Steps
1. Create `src/lib/logger.ts` — export a `pipelineLog(entry: { level: 'debug'|'info'|'warn'|'error', stage: string, [key: string]: unknown }): void` function that serializes to JSON and writes to stdout.
2. Audit all log calls across the modules created in previous subtasks (resolve-agent-delegates.ts, idempotency.ts, linear-retry.ts, pipeline routes, create-issues.ts).
3. Replace any `console.log`/`console.error` calls with `pipelineLog()` to ensure consistent schema.
4. Ensure every log entry includes at minimum: `level`, `stage`, `timestamp` (ISO 8601).
5. Verify that error logs for unmapped delegates include `{ agent, reason }`, retry logs include `{ attempt, maxRetries, nextRetryMs }`, and idempotency skip logs include `{ idempotencyKey, action }`.

## Validation
Capture stdout during a full pipeline integration test run. Parse each line as JSON. Assert every log line has 'level', 'stage', and 'timestamp' fields. Assert at least one log from each stage: 'delegate_resolution', 'issue_creation', 'issue_query'. Verify logs are valid JSON (no raw text lines).