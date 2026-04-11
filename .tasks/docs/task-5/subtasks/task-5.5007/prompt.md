<identity>
You are rex working on subtask 5007 of task 5.
</identity>

<context>
<scope>
Wire up JWT authentication middleware for all non-internal routes, implement /health/live, /health/ready, /metrics, and the GDPR export/delete internal endpoints. Configure Kubernetes Deployment manifest for 2 replicas.
</scope>
</context>

<implementation_plan>
JWT middleware: use axum::middleware::from_fn with a layer that extracts Authorization: Bearer token, validates with jsonwebtoken crate using the JWT_SECRET env var (HS256), returns 401 on failure. Apply to all /api/v1/* routes. Health: GET /health/live returns 200 {status:'ok'} unconditionally. GET /health/ready pings postgres (SELECT 1) and redis (PING); returns 200 if both succeed, 503 with failing deps listed otherwise. Metrics: initialize prometheus::Registry, register http_requests_total (Counter, labels: method, path, status) and http_request_duration_seconds (Histogram). Expose GET /metrics in text format. GDPR: GET /internal/gdpr/export/:org_id — query vetting_results WHERE org_id=$1, return JSON row or 404. DELETE /internal/gdpr/delete/:org_id — DELETE FROM vetting_results WHERE org_id=$1, DELETE FROM vetting_requests WHERE org_id=$1, INSERT INTO audit.deletion_log (entity='vetting', org_id, deleted_at=now()), return 200. Kubernetes: write k8s/deployment.yaml with 2 replicas, resource limits (256Mi/500m), envFrom referencing customer-vetting-config ConfigMap and customer-vetting-secret Secret, liveness/readiness probes on /health/live and /health/ready.
</implementation_plan>

<validation>
Request without JWT to /api/v1/vetting/run returns 401. Request with valid JWT returns 202. GET /health/ready with postgres and redis running returns 200; with redis stopped returns 503 with 'redis' in body. GET /metrics contains http_requests_total in prometheus text format. DELETE /internal/gdpr/delete/:org_id returns 200; subsequent GET /api/v1/vetting/:org_id returns 404; audit.deletion_log contains one row for that org_id. `kubectl apply -f k8s/deployment.yaml` dry-run succeeds.
</validation>