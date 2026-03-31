Implement subtask 6004: Implement MinIO availability health check monitor

## Objective
Create a periodic MinIO health check that runs every 60 seconds, logs health status as structured fields, and emits a rollback trigger after 3 consecutive unreachable checks.

## Steps
Step-by-step:
1. Create `src/modules/hermes/logging/minio-health-monitor.ts`.
2. Implement `MinioHealthMonitor` class:
   - Constructor accepts: MinIO client/config (from ConfigMap), HermesLogger instance, bucket name.
   - `start()` method: begins a `setInterval` loop every 60 seconds (configurable via `MINIO_HEALTH_CHECK_INTERVAL_MS`).
   - Each check: perform a HEAD/stat request against the Hermes bucket using the MinIO client.
   - Map result to health status: successful response → `'ok'`, slow response (>5s) → `'degraded'`, connection error/timeout → `'unreachable'`.
   - Log each check result: `hermesLogger.info('minio_health_check', { minio_health: status })`.
3. Track consecutive unreachable count in a local variable.
   - On `'ok'` or `'degraded'`: reset counter to 0.
   - On `'unreachable'`: increment counter.
   - If counter reaches 3: emit rollback trigger via logger with `error_code: 'MINIO_UNREACHABLE'` and `rollback_trigger: true`.
4. Implement `stop()` method to clear the interval (for graceful shutdown).
5. Handle the case where the MinIO client itself throws during initialization (log and treat as unreachable).
6. Export the monitor class.

## Validation
Test healthy: Mock MinIO client to return success — after 3 intervals, verify 3 logs with `minio_health: 'ok'` and 0 rollback triggers. Test unreachable: Mock MinIO client to throw connection error — after 3 intervals, verify `minio_health: 'unreachable'` logs and a rollback trigger on the 3rd check. Test recovery: Mock to fail twice then succeed — verify counter resets and no rollback trigger. Test degraded: Mock to respond after 6 seconds — verify `minio_health: 'degraded'` log. Verify `stop()` clears the interval (no further checks after stop).