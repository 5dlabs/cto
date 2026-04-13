<identity>
You are rex working on subtask 4011 of task 4.
</identity>

<context>
<scope>
Expose /metrics, /health/live, and /health/ready endpoints. Register custom Prometheus counters for invoice creation and payment recording.
</scope>
</context>

<implementation_plan>
Create src/handlers/health.rs. GET /health/live: return 200 JSON {status: live}. GET /health/ready: attempt sqlx pool acquire with 1s timeout and redis PING; return 200 if both succeed, 503 with JSON {postgres: false/true, redis: false/true} on any failure. Create a Prometheus Registry in src/metrics.rs using the prometheus crate. Register counters: finance_invoices_created_total (labels: currency), finance_payments_recorded_total (labels: method). Expose GET /metrics returning prometheus text format via prometheus::TextEncoder. Increment counters in the respective handlers. Register /health/live, /health/ready, /metrics on the Axum router without JWT middleware.
</implementation_plan>

<validation>
GET /health/live returns 200. GET /health/ready returns 200 with running DB and Redis; returns 503 when postgres pool is exhausted. GET /metrics returns text/plain with finance_invoices_created_total and finance_payments_recorded_total lines. After creating one invoice, GET /metrics shows finance_invoices_created_total counter >= 1.
</validation>