Implement task 1: Provision Core Infrastructure (Bolt - Kubernetes/Helm)

## Goal
Bootstrap the sigma1 namespace and all shared infrastructure: CloudNative-PG PostgreSQL cluster (instances: 2, PgBouncer sidecar), Valkey 7.2 via existing Redis operator, Cloudflare R2 bucket credentials, Signal-CLI sidecar resource definitions, Cloudflare Tunnel ingress, and the sigma1-infra-endpoints ConfigMap aggregating all connection strings. Also configure observability targets (Prometheus ServiceMonitors, Loki log shipping) for the existing Grafana stack.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
1. Create namespace `sigma1` and namespace `sigma1-db` (or reuse `databases` if cluster convention).
2. Deploy CloudNative-PG Cluster CR `sigma1-postgres`:
   - `instances: 2` (HA failover per D3 resolution)
   - `storage.size: 50Gi`
   - Bootstrap initdb: database `sigma1`, owner `sigma1_user`
   - Enable PgBouncer pooler CR (`Pooler` kind) with `pgbouncer.pool_mode: transaction`, `default_pool_size: 20`
   - Create per-service PostgreSQL roles: `catalog_svc`, `rms_svc`, `finance_svc`, `vetting_svc`, `social_svc` each with schema-level GRANTs
   - Run init SQL to create schemas: `catalog`, `rms`, `finance`, `vetting`, `social`, `audit`
3. Deploy Valkey CR `sigma1-valkey` using existing `redis.redis.opstreelabs.in/v1beta2` operator:
   - Image: `valkey/valkey:7.2-alpine`
   - Resource limits: 256Mi memory, 250m CPU
4. Create Kubernetes Secrets:
   - `sigma1-db-credentials` (PostgreSQL connection per role)
   - `sigma1-r2-credentials` (Cloudflare R2 access key, secret key, bucket name, endpoint URL)
   - `sigma1-stripe-credentials` (placeholder for Stripe API keys)
   - `sigma1-external-apis` (OpenCorporates API key, ElevenLabs API key, Twilio SID/token, Google Calendar API credentials)
   - `sigma1-service-api-keys` (pre-shared API keys for inter-service auth per D7 resolution — one key per service pair)
5. Create ConfigMap `sigma1-infra-endpoints`:
   - `POSTGRES_URL=postgresql://sigma1_user:$(password)@sigma1-postgres-pooler.sigma1-db.svc.cluster.local:5432/sigma1`
   - `VALKEY_URL=redis://sigma1-valkey.sigma1-db.svc.cluster.local:6379`
   - `R2_ENDPOINT=https://<account_id>.r2.cloudflarestorage.com`
   - `R2_BUCKET=sigma1-media`
   - `NATS_URL=nats://openclaw-nats.openclaw.svc.cluster.local:4222` (for social engine only per D4)
6. Deploy Cloudflare Tunnel Deployment + Service for Morgan external access.
7. Define Signal-CLI sidecar container spec as a shared template (ConfigMap or Kustomize component):
   - Image: `bbernhard/signal-cli-rest-api:latest`
   - Resource limits: 512Mi memory (Java process), 500m CPU
   - Liveness probe on Signal-CLI REST health endpoint
   - Restart policy: Always (mitigates memory leak per open question #2)
8. Create Prometheus ServiceMonitor CRs for all sigma1 services (label selector `app.kubernetes.io/part-of: sigma1`).
9. Verify PgBouncer pooler is reachable from sigma1 namespace via NetworkPolicy allowing ingress from `sigma1` to `sigma1-db`.

## Acceptance Criteria
1. `kubectl get cluster sigma1-postgres -n sigma1-db` shows READY with 2/2 instances healthy. 2. `kubectl exec` into a sigma1 pod and verify `psql` connection via PgBouncer pooler URL succeeds and `\dn` lists all 6 schemas (catalog, rms, finance, vetting, social, audit). 3. `redis-cli -u $VALKEY_URL PING` returns PONG. 4. ConfigMap `sigma1-infra-endpoints` exists with all 5 expected keys. 5. All Kubernetes Secrets exist with non-empty data keys. 6. Cloudflare Tunnel pod is Running and tunnel status shows CONNECTED. 7. ServiceMonitor CRs are picked up by Prometheus (check Prometheus targets page). 8. PgBouncer stats show active connection pools when queried via `SHOW POOLS`.

## Subtasks
- Create sigma1 and sigma1-db namespaces with labels: Create the `sigma1` application namespace and the `sigma1-db` database namespace (or reuse `databases` per cluster convention). Apply standard labels including `app.kubernetes.io/part-of: sigma1` for observability selector matching.
- Deploy CloudNative-PG Cluster CR with initdb bootstrap: Deploy the CloudNative-PG `Cluster` CR named `sigma1-postgres` in the `sigma1-db` namespace with 2 instances, 50Gi storage, and initdb bootstrap creating the `sigma1` database owned by `sigma1_user`.
- Create per-service PostgreSQL roles and schemas via init SQL: Run init SQL against the sigma1-postgres cluster to create 6 schemas (catalog, rms, finance, vetting, social, audit) and 5 per-service roles (catalog_svc, rms_svc, finance_svc, vetting_svc, social_svc) with schema-level GRANTs.
- Deploy PgBouncer Pooler CR for connection pooling: Deploy the CloudNative-PG `Pooler` CR to front the sigma1-postgres cluster with PgBouncer in transaction pooling mode with a default pool size of 20.
- Deploy Valkey CR via Redis operator: Deploy the Valkey 7.2 instance using the existing `redis.redis.opstreelabs.in/v1beta2` operator in the `sigma1-db` namespace.
- Create Kubernetes Secrets for database credentials: Create the `sigma1-db-credentials` Secret in the `sigma1` namespace containing PostgreSQL connection strings for each per-service role (catalog_svc, rms_svc, finance_svc, vetting_svc, social_svc) and the sigma1_user superuser.
- Create Kubernetes Secrets for R2, Stripe, external APIs, and inter-service keys: Create the remaining four Kubernetes Secrets: sigma1-r2-credentials, sigma1-stripe-credentials, sigma1-external-apis, and sigma1-service-api-keys in the sigma1 namespace.
- Create sigma1-infra-endpoints ConfigMap: Create the `sigma1-infra-endpoints` ConfigMap in the `sigma1` namespace aggregating all non-secret connection strings and endpoint URLs for consumption via `envFrom` by downstream services.
- Deploy Cloudflare Tunnel Deployment and Service: Deploy a Cloudflare Tunnel (cloudflared) Deployment and associated Service in the sigma1 namespace for Morgan external access routing.
- Define Signal-CLI sidecar container template: Create a reusable Signal-CLI sidecar container specification as a Kustomize component or ConfigMap that downstream service Deployments can include.
- Create Prometheus ServiceMonitor CRs for sigma1 services: Create Prometheus ServiceMonitor custom resources that auto-discover and scrape metrics from all sigma1 services using the common label selector.
- Create NetworkPolicy allowing sigma1 to sigma1-db ingress: Create a NetworkPolicy in the sigma1-db namespace that allows ingress traffic from pods in the sigma1 namespace to the PgBouncer pooler and Valkey services, while denying other ingress by default.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.