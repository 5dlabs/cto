Implement task 6: Implement Rollout and Migration Risk Logging (Nova - Bun/Elysia)

## Goal
Implement structured logging, rollout phase tracking, and alerting for the Hermes pipeline — including Grafana dashboards for migration health, deliberation error rates, and rollback trigger conditions using the existing Loki stack.

## Task Context
- Agent owner: nova
- Stack: Bun/Elysia
- Priority: medium
- Dependencies: 1, 2, 3

## Implementation Plan
Step-by-step implementation:

1. **Structured logging library:** Create `src/modules/hermes/logging/` subdirectory:
   - `hermes-logger.ts` — wrapper around the application's logger that enforces structured fields
   - Required fields on ALL Hermes log entries: `module: 'hermes'`, `rollout_phase` (enum: `dev` | `staging` | `canary` | `production`), `operation`, `duration_ms`
   - Error entries additionally require: `error_code`, `error_message`, `stack_trace`
   - Migration entries additionally require: `migration_step`, `migration_progress`

2. **Rollout phase tracking:** Read `ENVIRONMENT` from `hermes-infra-endpoints` ConfigMap. Map to rollout phase:
   - `dev` → `dev`
   - `staging` → `staging`
   - Production with <10% traffic → `canary`
   - Production at full traffic → `production`
   Track phase transitions as explicit log events.

3. **Rollback trigger conditions:** Define and implement alerting thresholds:
   - Deliberation failure rate > 20% over 5-minute window → emit `rollback_trigger` log
   - Artifact write failure rate > 10% over 5-minute window → emit `rollback_trigger` log
   - Migration failure count > 5 consecutive → emit `rollback_trigger` log
   - P99 deliberation latency > 30s → emit `rollback_trigger` log
   Implement as a lightweight in-process monitor using sliding window counters (Redis-backed for persistence across restarts).

4. **MinIO availability monitoring (per D2 caveat):** Implement periodic MinIO health check:
   - Every 60 seconds, perform a HEAD request against the Hermes bucket
   - Log `minio_health: 'ok' | 'degraded' | 'unreachable'` as structured field
   - If unreachable for 3 consecutive checks, emit `rollback_trigger` log

5. **Grafana dashboard provisioning:** Create dashboard JSON files in `dashboards/hermes/`:
   - **Rollout Health Dashboard:** panels for rollout phase, deployment count, error rate by phase
   - **Deliberation Pipeline Dashboard:** panels for deliberation throughput, latency P50/P95/P99, failure rate, active deliberations
   - **Migration Progress Dashboard:** panels for migration progress (total/completed/failed), artifact copy throughput, MinIO health
   - **Rollback Triggers Dashboard:** panel showing `rollback_trigger` events over time
   All dashboards query Loki via LogQL. Provision via Grafana's sidecar ConfigMap pattern or dashboard API.

6. **Alert rules (optional but recommended):** Create Grafana alert rules for rollback trigger conditions, routing to a Slack/webhook channel if configured.

7. **Retrofit existing code:** Ensure Tasks 2, 3, and 5 code paths use the `hermes-logger` wrapper. Add logging middleware to all Hermes routes that captures request/response metadata.

## Acceptance Criteria
1. Structured log validation: All Hermes API requests generate logs queryable in Loki via `{app="hermes"} | json | module="hermes"` — each entry contains `rollout_phase`, `operation`, and `duration_ms` fields.
2. Rollback trigger: Simulate 25 consecutive deliberation failures; verify that a log entry with `rollback_trigger=true` and `error_code=FAILURE_RATE_EXCEEDED` appears in Loki within 1 minute.
3. MinIO health monitoring: Stop the MinIO service for 4 minutes; verify that logs with `minio_health="unreachable"` appear and a `rollback_trigger` log is emitted after 3 consecutive failures.
4. Grafana dashboards: The Rollout Health Dashboard loads in Grafana without errors and displays at least one data point when the Hermes service has been running for >5 minutes.
5. LogQL queryability: The query `{app="hermes"} | json | migration_step="complete"` returns results after a migration run (cross-validates with Task 5).
6. Error code taxonomy: Deliberation errors, artifact write errors, and migration errors each use distinct `error_code` values — verified by querying `{app="hermes"} | json | error_code!="" | line_format "{{.error_code}}"` and confirming at least 3 distinct codes in test runs.

## Subtasks
- Implement structured logging library (hermes-logger wrapper): Create `src/modules/hermes/logging/hermes-logger.ts` — a wrapper around the application's existing logger that enforces required structured fields on all Hermes log entries, with specialized methods for error and migration log entries.
- Implement rollout phase tracking with environment mapping: Add rollout phase detection logic that reads `ENVIRONMENT` from the `hermes-infra-endpoints` ConfigMap, maps it to the rollout phase enum, and logs phase transitions as explicit events.
- Implement rollback trigger monitoring with sliding window counters: Build a lightweight in-process monitor that tracks failure rates and latencies using sliding window counters (Redis-backed), evaluates rollback trigger conditions, and emits `rollback_trigger` log entries when thresholds are exceeded.
- Implement MinIO availability health check monitor: Create a periodic MinIO health check that runs every 60 seconds, logs health status as structured fields, and emits a rollback trigger after 3 consecutive unreachable checks.
- Provision Grafana dashboard JSON files for Hermes observability: Create four Grafana dashboard JSON files in `dashboards/hermes/` with LogQL queries against Loki for rollout health, deliberation pipeline metrics, migration progress, and rollback triggers. Configure for sidecar ConfigMap provisioning.
- Add Hermes logging middleware to Elysia routes and retrofit existing code paths: Create Elysia middleware that wraps all Hermes route handlers with structured logging (request/response metadata, timing), and retrofit existing Task 2, 3, and 5 code paths to use the hermes-logger wrapper instead of direct console/logger calls.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.