Implement subtask 6015: Implement Prometheus metrics and health endpoints

## Objective
Add Prometheus metrics collection (prom-client) and Kubernetes health/readiness endpoints to the Elysia application.

## Steps
1. Install `prom-client`.
2. Create `src/routes/observability.ts`.
3. `GET /metrics`:
   - Initialize prom-client default metrics (process CPU, memory, event loop lag).
   - Add custom metrics:
     - `social_uploads_total` (Counter): total uploads by status (success/failure).
     - `social_drafts_total` (Counter): drafts created, by status transition.
     - `social_publish_total` (Counter): publish attempts by platform and result (success/failure).
     - `social_publish_duration_seconds` (Histogram): publish latency by platform.
     - `social_ai_curation_duration_seconds` (Histogram): AI curation pipeline duration.
   - Return metrics in Prometheus exposition format.
4. `GET /health/live`:
   - Return 200 `{ status: 'ok' }` if process is running.
5. `GET /health/ready`:
   - Check database connectivity (simple SELECT 1 query).
   - Check R2 connectivity (HEAD request on bucket).
   - Return 200 if all checks pass, 503 if any fail with details.
6. Instrument existing route handlers to update counters and histograms at appropriate points.

## Validation
Verify GET /metrics returns valid Prometheus format with default and custom metrics. Verify GET /health/live returns 200. Verify GET /health/ready returns 200 when DB and R2 are reachable, and 503 when DB is unreachable (mock connection failure).