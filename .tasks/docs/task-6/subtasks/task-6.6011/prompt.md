Implement subtask 6011: Implement health/metrics endpoints, OpenAPI spec, Dockerfile, and Kubernetes deployment manifest

## Objective
Add health check endpoints, Prometheus metrics, OpenAPI documentation via Elysia Swagger plugin, Bun-based Dockerfile, and Kubernetes deployment manifest for the sigma1 namespace.

## Steps
1. Health endpoints in `src/routes/health.ts`:
   - `GET /health/live` — returns 200 `{ status: 'ok' }` (liveness, no dependency checks).
   - `GET /health/ready` — checks Postgres connection and NATS connection. Returns 200 if both healthy, 503 if either down. Response: `{ status: 'ready'|'not_ready', checks: { postgres: 'up'|'down', nats: 'up'|'down' } }`.
2. Metrics endpoint in `src/routes/metrics.ts`:
   - Install `prom-client`.
   - Register default metrics (process CPU, memory, event loop lag).
   - Custom metrics: `social_uploads_total` (counter), `social_drafts_created_total` (counter, label: platform), `social_publish_success_total` (counter, label: platform), `social_publish_failure_total` (counter, label: platform), `social_ai_scoring_duration_seconds` (histogram).
   - `GET /metrics` returns Prometheus text format.
3. OpenAPI/Swagger:
   - Use `@elysiajs/swagger` plugin.
   - Configure with title 'Social Media Engine API', version '1.0.0'.
   - All routes should already have Effect Schema types which Elysia auto-documents.
   - Serve at `/swagger`.
4. Dockerfile:
   - Base: `oven/bun:1` (Bun official image).
   - WORKDIR /app, COPY package.json bun.lockb, RUN bun install --frozen-lockfile.
   - COPY src/, EXPOSE 3000, CMD ["bun", "run", "src/index.ts"].
   - Multi-stage: build stage for type-checking, prod stage minimal.
5. Kubernetes manifest `k8s/deployment.yaml`:
   - Namespace: sigma1.
   - Deployment: 1 replica, resource requests (cpu: 250m, memory: 512Mi), limits (cpu: 500m, memory: 1Gi).
   - envFrom: ConfigMap `sigma1-infra-endpoints` and Secret `social-engine-secrets`.
   - Liveness probe: /health/live, readiness probe: /health/ready.
   - Service: ClusterIP port 3000.
6. Create `src/index.ts` — main entry point that composes Elysia app with all route groups, swagger, health, metrics, and starts NATS subscribers.

## Validation
Test: (1) GET /health/live returns 200. (2) GET /health/ready with healthy DB returns 200 with postgres: 'up'. (3) GET /health/ready with DB down returns 503. (4) GET /metrics returns Prometheus format with custom counters. (5) GET /swagger returns OpenAPI JSON with all documented routes. (6) Docker build succeeds without errors. (7) Kubernetes manifest validates with `kubectl apply --dry-run=client`.