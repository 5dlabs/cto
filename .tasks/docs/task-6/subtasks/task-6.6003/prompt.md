Implement subtask 6003: Implement rollback trigger monitoring with sliding window counters

## Objective
Build a lightweight in-process monitor that tracks failure rates and latencies using sliding window counters (Redis-backed), evaluates rollback trigger conditions, and emits `rollback_trigger` log entries when thresholds are exceeded.

## Steps
Step-by-step:
1. Create `src/modules/hermes/logging/rollback-monitor.ts`.
2. Implement `SlidingWindowCounter` class:
   - Backed by Redis sorted sets (ZADD with timestamp scores, ZRANGEBYSCORE for window queries, ZREMRANGEBYSCORE for cleanup).
   - Methods: `increment(key: string)`, `getCount(key: string, windowMs: number): Promise<number>`, `getRate(successKey: string, failureKey: string, windowMs: number): Promise<number>`.
   - Use configurable Redis connection from environment/ConfigMap.
3. Implement `RollbackMonitor` class that uses `SlidingWindowCounter`:
   - `recordDeliberationResult(success: boolean, latencyMs: number)` — increments counters.
   - `recordArtifactWriteResult(success: boolean)` — increments counters.
   - `recordMigrationFailure()` — increments consecutive failure counter.
   - `recordMigrationSuccess()` — resets consecutive failure counter.
4. Implement `evaluateThresholds()` method (called after each `record*` call):
   - Deliberation failure rate > 20% over 5-minute window → emit rollback trigger.
   - Artifact write failure rate > 10% over 5-minute window → emit rollback trigger.
   - Migration consecutive failures > 5 → emit rollback trigger.
   - P99 deliberation latency > 30s → emit rollback trigger (maintain a sorted list of recent latencies for percentile calculation).
5. When a threshold is breached, call `hermesLogger.error()` with `rollback_trigger: true`, appropriate `error_code` (e.g., `FAILURE_RATE_EXCEEDED`, `LATENCY_EXCEEDED`), and the current metric values.
6. Add debouncing: don't emit the same rollback trigger more than once per 5-minute window.
7. Export `RollbackMonitor` singleton factory.

## Validation
Unit test with mock Redis: Record 25 deliberation failures and 0 successes — verify `evaluateThresholds()` emits a rollback trigger log with `error_code: 'FAILURE_RATE_EXCEEDED'`. Record 80 successes and 20 failures (20% rate) — verify trigger fires. Record 79 successes and 21 failures — verify no trigger at exactly 20% boundary. Test debounce: trigger twice within 5 minutes — verify only one log emitted. Test P99 latency: record 99 deliberations at 1s and 1 at 35s — P99 is 35s > 30s threshold — verify trigger. Test consecutive migration failures: record 6 failures — verify trigger; record 4 failures then 1 success then 4 failures — verify NO trigger (reset on success).