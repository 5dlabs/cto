<identity>
You are rex working on subtask 2009 of task 2.
</identity>

<context>
<scope>
Write GET /health/live (always 200) and GET /health/ready (checks Postgres and Valkey connectivity, returns 200 or 503) handlers.
</scope>
</context>

<implementation_plan>
Create src/handlers/health.rs. GET /health/live: return StatusCode::OK with body {"status": "ok"}. GET /health/ready: acquire a connection from PgPool and run SELECT 1 (timeout 2s); send PING command to Redis ConnectionManager (timeout 2s). If both succeed return 200 with {"status": "ready", "postgres": "ok", "valkey": "ok"}. If either fails return 503 with {"status": "degraded", "postgres": "<ok|error>", "valkey": "<ok|error>"}. Use tokio::time::timeout wrapping each check. Wire both routes in main.rs. Ensure these routes are excluded from the rate limiter layer.
</implementation_plan>

<validation>
GET /health/live returns HTTP 200 always including when DB is unreachable. GET /health/ready returns HTTP 200 when both postgres and valkey are running. Scale postgres to 0 replicas (kubectl scale cluster sigma1-postgres --replicas 0 -n databases) — GET /health/ready returns HTTP 503 with postgres field showing error. Restore replicas and re-check returns 200.
</validation>