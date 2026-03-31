Implement task 1: Provision Dev and Staging Infrastructure (Bolt - Kubernetes/Helm)

## Goal
Bootstrap dev and staging namespaces with all operator-managed data services, dedicated MinIO bucket, secrets, ConfigMap, and ArgoCD Application CRs for the Hermes pipeline. This is the foundational task — every subsequent task depends on it.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
Step-by-step implementation:

1. **Namespace creation:** Create `hermes-dev` and `hermes-staging` namespaces with appropriate labels (`app.kubernetes.io/part-of: hermes`, `environment: dev|staging`).

2. **CloudNative-PG PostgreSQL:** Deploy single-replica `Cluster` CRs in each namespace. Configure `initdb` with a `hermes` database. Store connection credentials in namespace-scoped secrets (`hermes-pg-credentials`).

3. **Redis operator:** Deploy single-replica Redis CRs in each namespace. Store connection string in namespace-scoped secret (`hermes-redis-credentials`).

4. **NATS operator:** Deploy single-replica NATS CRs in each namespace. Not actively wired but must be available for future decoupling per D1. Store connection string in secret (`hermes-nats-credentials`).

5. **MinIO — dedicated bucket (CRITICAL per D2):** Do NOT reuse `gitlab/gitlab-minio-svc`. Either:
   a. Deploy a dedicated MinIO tenant via the MinIO Operator in each namespace, OR
   b. If using the existing MinIO cluster, create a dedicated bucket (`hermes-artifacts-dev`, `hermes-artifacts-staging`) with independent IAM credentials and lifecycle policy.
   Store S3 endpoint, access key, secret key, and bucket name in secret (`hermes-minio-credentials`).
   Configure lifecycle policy: 90-day retention for dev, 365-day for staging (configurable via values).

6. **ConfigMap aggregation:** Create `hermes-infra-endpoints` ConfigMap in each namespace aggregating all connection strings:
   - `CNPG_HERMES_URL` — PostgreSQL connection string
   - `REDIS_HERMES_URL` — Redis connection string
   - `NATS_HERMES_URL` — NATS connection string
   - `MINIO_HERMES_ENDPOINT` — MinIO S3 endpoint
   - `MINIO_HERMES_BUCKET` — Bucket name
   - `ENVIRONMENT` — `dev` or `staging`
   All later tasks reference this ConfigMap via `envFrom`.

7. **Secrets management:** Each namespace gets independent secrets for all services. No cross-namespace secret references. Use `SealedSecrets` or existing secret management pattern in the cluster.

8. **ArgoCD Application CRs (gap resolution):** Create ArgoCD `Application` CRs for:
   - `hermes-backend-dev` and `hermes-backend-staging` (Bun/Elysia service)
   - `hermes-frontend-dev` and `hermes-frontend-staging` (Next.js app)
   Configure automated sync for dev, manual sync with auto-prune for staging. Staging promotion should be gated by successful E2E test runs (annotation hook for Task 7 integration).

9. **Loki verification:** Confirm `openclaw/loki-*` services are accessible from the new namespaces. No provisioning needed, but verify log shipping works by deploying a test pod that emits structured JSON.

10. **Helm chart structure:** Package all of the above into a Helm chart (`charts/hermes-infra`) with `values-dev.yaml` and `values-staging.yaml` overlays. Single `helm upgrade --install` per environment.

## Acceptance Criteria
1. `kubectl get namespace hermes-dev hermes-staging` returns both namespaces in Active state.
2. `kubectl get clusters.postgresql.cnpg.io -n hermes-dev` shows a Ready cluster with 1 replica.
3. `kubectl get configmap hermes-infra-endpoints -n hermes-dev -o jsonpath='{.data.CNPG_HERMES_URL}'` returns a valid PostgreSQL connection string.
4. `kubectl exec` a test pod in `hermes-dev` that connects to PostgreSQL, Redis, NATS, and MinIO using only env vars from `hermes-infra-endpoints` ConfigMap and mounted secrets — all four connections succeed.
5. MinIO bucket `hermes-artifacts-dev` exists and is writable with dedicated credentials, AND those credentials do NOT have access to any GitLab-owned buckets.
6. ArgoCD UI shows `hermes-backend-dev` and `hermes-frontend-staging` Application CRs in Synced/Healthy state (initially empty).
7. A structured JSON log emitted from a test pod in `hermes-dev` is queryable in Loki via LogQL within 30 seconds.

## Subtasks
- Create hermes-dev and hermes-staging namespaces with labels and Loki verification: Create both Kubernetes namespaces with standard labels and verify Loki log shipping is accessible from the new namespaces.
- Deploy CloudNative-PG PostgreSQL Cluster CRs and credential secrets: Deploy single-replica CNPG Cluster custom resources in both hermes-dev and hermes-staging namespaces with initdb configuration and credential secrets.
- Deploy Redis operator CRs and credential secrets: Deploy single-replica Redis custom resources in both namespaces with connection string stored in namespace-scoped secrets.
- Deploy NATS operator CRs and credential secrets: Deploy single-replica NATS custom resources in both namespaces for future decoupling, with connection strings stored in secrets.
- Provision dedicated MinIO bucket with IAM credentials and lifecycle policies: Create a dedicated MinIO bucket for Hermes artifacts in each namespace with independent IAM credentials and configurable retention lifecycle policies. Must NOT reuse GitLab-owned buckets.
- Create hermes-infra-endpoints ConfigMap aggregating all service connection strings: Create the hermes-infra-endpoints ConfigMap in each namespace that aggregates all infrastructure connection strings for downstream service consumption via envFrom.
- Create ArgoCD Application CRs for backend and frontend in both environments: Create ArgoCD Application custom resources for hermes-backend and hermes-frontend in both dev and staging namespaces with appropriate sync policies.
- Package Helm chart with values overlays for dev and staging: Structure and finalize the charts/hermes-infra Helm chart with values-dev.yaml and values-staging.yaml overlays, ensuring a single helm upgrade --install per environment deploys everything.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.