Implement subtask 6001: Implement structured logging library (hermes-logger wrapper)

## Objective
Create `src/modules/hermes/logging/hermes-logger.ts` — a wrapper around the application's existing logger that enforces required structured fields on all Hermes log entries, with specialized methods for error and migration log entries.

## Steps
Step-by-step:
1. Create `src/modules/hermes/logging/` directory.
2. Define TypeScript types for log entry categories:
   - `HermesBaseLogFields`: `{ module: 'hermes', rollout_phase: 'dev' | 'staging' | 'canary' | 'production', operation: string, duration_ms: number }`
   - `HermesErrorLogFields`: extends base with `{ error_code: string, error_message: string, stack_trace?: string }`
   - `HermesMigrationLogFields`: extends base with `{ migration_step: string, migration_progress: { total: number, completed: number, failed: number, skipped: number } }`
3. Implement `HermesLogger` class that wraps the app's logger (accept logger instance via constructor injection).
4. Methods: `info(operation: string, fields?: Record<string, unknown>)`, `error(operation: string, errorFields: HermesErrorLogFields)`, `migration(step: string, progress: MigrationProgress, fields?: Record<string, unknown>)`.
5. Each method auto-injects `module: 'hermes'` and `rollout_phase` (read once on init from environment/ConfigMap).
6. Define an error code enum/const: `DELIBERATION_FAILURE`, `ARTIFACT_WRITE_FAILURE`, `MIGRATION_FAILURE`, `INTEGRITY_MISMATCH`, `FAILURE_RATE_EXCEEDED`, `LATENCY_EXCEEDED`, `MINIO_UNREACHABLE`.
7. Implement `startTimer()` helper that returns a function to compute `duration_ms`.
8. Export the logger class and all types.

## Validation
Unit test: Create HermesLogger with a mock logger, call `info('test-op')` — verify the mock received a log entry with `module: 'hermes'`, `rollout_phase`, `operation: 'test-op'`, and `duration_ms`. Call `error()` — verify `error_code`, `error_message` fields are present. Call `migration()` — verify `migration_step` and `migration_progress` fields. Verify that omitting required fields causes TypeScript compilation errors (type safety).