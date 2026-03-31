Implement task 1: Provision Staging/Production Infra and Artifact Storage (Bolt - Kubernetes/Helm)

## Goal
Bootstrap the Hermes E2E pipeline infrastructure by creating isolated staging and production namespaces with all backing services, MinIO artifact storage buckets, ConfigMaps for endpoint aggregation, Secrets for credentials, ResourceQuotas, LimitRanges, RBAC RoleBindings, and CiliumNetworkPolicies. This is the foundational task that every downstream service and frontend task will depend on.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
## Step-by-step Implementation

### 1. Helm Chart Structure
Create a Helm chart (e.g., `charts/hermes-infra`) with per-environment values files:
- `values-staging.yaml` (namespace: `hermes-staging`, single-replica operators, 30-day artifact retention)
- `values-production.yaml` (namespace: `hermes-production`, single-replica operators initially, 90-day artifact retention)
- Parameterize all environment-specific values: namespace name, replica counts, retention days, bucket names, resource quotas.

### 2. Namespace Provisioning
- Create namespaces `hermes-staging` and `hermes-production`.
- Label all resources with: `app.kubernetes.io/part-of: hermes`, `hermes.io/environment: staging|production`, `hermes.io/component: <component-name>`.
- Annotate namespaces with project metadata and owner.

### 3. RBAC
- Create namespace-scoped `Role` and `RoleBinding` resources in each namespace.
- Do NOT use `ClusterRoleBinding`. Roles should grant least-privilege access for pipeline service accounts.
- Create a `ServiceAccount` per namespace (e.g., `hermes-pipeline-sa`) for workloads.

### 4. Resource Governance
- Deploy a `ResourceQuota` per namespace. Suggested starting values:
  - Staging: 8 CPU, 16Gi memory, 20 pods max
  - Production: 16 CPU, 32Gi memory, 40 pods max
- Deploy a `LimitRange` per namespace enforcing default container limits (e.g., default 500m CPU / 512Mi, max 2 CPU / 2Gi).

### 5. CiliumNetworkPolicies
- Deploy a default-deny ingress `CiliumNetworkPolicy` in each namespace.
- Add allow rules only for: intra-namespace traffic, egress to `gitlab` namespace for MinIO (`gitlab-minio-svc` on port 9000), and DNS (kube-dns).
- Explicitly block cross-namespace traffic between `hermes-staging` and `hermes-production`.

### 6. MinIO Artifact Storage
- **Capacity verification (GATE):** Before bucket creation, exec into the MinIO pod or use `mc admin info` to check backing PV capacity and current usage. Document total capacity, used capacity, and estimated IOPS. If free capacity is < 50Gi or utilization > 70%, provision a **dedicated MinIO instance** for Hermes (Helm chart: `minio/minio`, deployed in a `hermes-minio` namespace) instead of using GitLab's instance.
- Create dedicated buckets using MinIO Client (`mc`) or a Kubernetes Job:
  - `hermes-staging-artifacts`
  - `hermes-prod-artifacts`
- Configure lifecycle policies per bucket: auto-expire objects older than 30 days (staging) / 90 days (production). Make retention configurable via Helm values.
- Set bucket quotas: cap each bucket at a configurable size (default: 20Gi staging, 50Gi production).
- Create per-environment MinIO access keys (not shared with GitLab). Store access key ID and secret key in Kubernetes Secrets (step 8).
- Verify bucket naming does not conflict with existing buckets in the `gitlab` namespace.

### 7. Backing Services (per namespace)
- **CloudNative-PG (Postgres):** Deploy a `Cluster` CR with 1 replica (dev/staging). Use latest stable CNPG version in cluster. Database name: `hermes`. Configure automated backups to MinIO bucket if CNPG supports it, otherwise defer.
- **Redis:** Deploy a single-replica Redis instance via Helm (e.g., `bitnami/redis`) or a Redis Operator CR if present. Disable persistence for staging; enable for production.
- **NATS:** Deploy a single-replica NATS server via Helm (`nats/nats`). Configure JetStream for durable message delivery.
- All backing service Helm releases should be namespaced and labeled consistently.

### 8. Secrets
- Create Kubernetes `Secret` resources per namespace containing:
  - `MINIO_ACCESS_KEY_ID`, `MINIO_SECRET_ACCESS_KEY` (per-environment MinIO credentials)
  - `POSTGRES_URL` (connection string from CNPG cluster status)
  - `REDIS_URL` (connection string)
  - `NATS_URL` (connection string)
- Secrets must be created via Helm `templates/secrets.yaml` with values sourced from `values-staging.yaml` / `values-production.yaml`.
- Do NOT store any secrets in ConfigMaps.

### 9. ConfigMap — Endpoint Aggregation
- Create a ConfigMap named `hermes-infra-endpoints` in each namespace containing:
  - `CNPG_HERMES_URL` — Postgres connection endpoint (host:port/dbname)
  - `REDIS_HERMES_URL` — Redis endpoint (host:port)
  - `NATS_HERMES_URL` — NATS endpoint (nats://host:port)
  - `MINIO_ENDPOINT` — MinIO service endpoint (http://gitlab-minio-svc.gitlab.svc:9000 or dedicated instance endpoint)
  - `MINIO_BUCKET` — environment-specific bucket name
  - `MINIO_PRESIGN_EXPIRY` — default presigned URL expiry (e.g., 3600s)
  - `ENVIRONMENT` — `staging` or `production`
- Downstream workloads consume this via `envFrom: [{configMapRef: {name: hermes-infra-endpoints}}]`.
- Document the full key inventory in a README or NOTES.txt in the Helm chart.

### 10. PodDisruptionBudgets
- Create PDBs for production namespace backing services (Postgres, Redis, NATS) with `minAvailable: 1`.
- Skip for staging.

### 11. Documentation & Validation
- Helm NOTES.txt should print all provisioned endpoints, bucket names, and ConfigMap/Secret names.
- Include a `templates/tests/` directory with Helm test pods that:
  - Verify Postgres connectivity
  - Verify Redis PING
  - Verify NATS connection
  - Verify MinIO bucket exists and is writable (put/get/delete a test object)
  - Verify ConfigMap and Secret resources exist with expected keys

### Rollout Risks & Migration Concerns
- **Blast radius to GitLab MinIO:** Mitigated by capacity gate check, bucket quotas, and the documented fallback to a dedicated MinIO instance.
- **Operator version skew:** If CNPG, Redis, or NATS operators are already installed cluster-wide, ensure the CR API versions match. Do not install a conflicting operator version.
- **Namespace creation ordering:** Namespaces must exist before any namespaced resources. Helm handles this with proper resource ordering, but verify with `helm template --debug`.
- **Secret rotation:** Native Secrets do not auto-rotate. Document this as a known limitation and reference the deferred ESO migration (Open Question #4).

## Acceptance Criteria
1. **Namespace existence:** `kubectl get namespace hermes-staging hermes-production` returns both namespaces with correct labels (`hermes.io/environment=staging|production`) and annotations.
2. **RBAC verification:** `kubectl auth can-i --as=system:serviceaccount:hermes-staging:hermes-pipeline-sa --namespace=hermes-staging list pods` returns `yes`; same SA cannot access `hermes-production` (`kubectl auth can-i --as=system:serviceaccount:hermes-staging:hermes-pipeline-sa --namespace=hermes-production list pods` returns `no`).
3. **ResourceQuota enforcement:** `kubectl describe resourcequota -n hermes-staging` shows configured CPU/memory/pod limits; `kubectl describe resourcequota -n hermes-production` shows production-tier limits.
4. **LimitRange presence:** `kubectl get limitrange -n hermes-staging -o yaml` confirms default container limits are set.
5. **CiliumNetworkPolicy isolation:** Deploy a test pod in `hermes-staging` and attempt to curl a service in `hermes-production` — connection must be refused/timed out. Intra-namespace traffic must succeed. Egress to `gitlab-minio-svc.gitlab.svc:9000` must succeed.
6. **MinIO capacity documented:** A capacity report (total, used, free, estimated IOPS) is written to the Helm chart's output notes or a dedicated ConfigMap annotation. If capacity was insufficient, a dedicated MinIO instance is deployed and its endpoint is reflected in the ConfigMap.
7. **MinIO buckets exist and are functional:** `mc ls <alias>/hermes-staging-artifacts` and `mc ls <alias>/hermes-prod-artifacts` succeed. A test object can be PUT, GET (via presigned URL returning HTTP 200), and DELETE from each bucket.
8. **Bucket lifecycle policies active:** `mc ilm ls <alias>/hermes-staging-artifacts` shows expiry rule of 30 days; production shows 90 days.
9. **Bucket quotas set:** `mc admin bucket quota <alias>/hermes-staging-artifacts` confirms quota is configured.
10. **Backing services healthy:** Helm test pods in each namespace pass: Postgres accepts connections and responds to `SELECT 1`; Redis responds to `PING` with `PONG`; NATS accepts a connection and echoes a test publish/subscribe cycle.
11. **ConfigMap completeness:** `kubectl get configmap hermes-infra-endpoints -n hermes-staging -o json | jq '.data | keys'` returns all expected keys (`CNPG_HERMES_URL`, `REDIS_HERMES_URL`, `NATS_HERMES_URL`, `MINIO_ENDPOINT`, `MINIO_BUCKET`, `MINIO_PRESIGN_EXPIRY`, `ENVIRONMENT`). Same for production namespace.
12. **Secrets completeness:** `kubectl get secret hermes-infra-secrets -n hermes-staging -o json | jq '.data | keys'` contains `MINIO_ACCESS_KEY_ID`, `MINIO_SECRET_ACCESS_KEY`, `POSTGRES_URL`, `REDIS_URL`, `NATS_URL`. Values are non-empty (base64-decoded length > 0). Same for production.
13. **No secrets in ConfigMaps:** `kubectl get configmap hermes-infra-endpoints -n hermes-staging -o json | jq '.data'` contains no values matching password/key/secret patterns.
14. **PDB presence (production only):** `kubectl get pdb -n hermes-production` lists PDBs for Postgres, Redis, and NATS with `minAvailable: 1`.
15. **Helm idempotency:** Running `helm upgrade --install` twice in succession produces no errors and no resource drift.

## Subtasks
- Scaffold Helm Chart Structure, Namespace Templates, and NOTES.txt Skeleton: Create the charts/hermes-infra Helm chart directory structure with Chart.yaml, per-environment values files, namespace templates with labels/annotations, and a NOTES.txt skeleton that will be populated as services are added. This is the foundational scaffold all other subtasks build upon.
- Create RBAC, ResourceQuota, and LimitRange Templates: Create namespace-scoped ServiceAccount, Role, RoleBinding, ResourceQuota, and LimitRange templates. These are all small governance YAML resources with identical lifecycle and dependencies, grouped into a single subtask.
- Create CiliumNetworkPolicies for Namespace Isolation: Deploy CiliumNetworkPolicy resources implementing default-deny ingress, intra-namespace allow, MinIO egress allow (port 9000 to GitLab or dedicated instance), DNS egress allow, and explicit cross-namespace isolation between hermes-staging and hermes-production.
- MinIO Capacity Gate Check Script: Create a standalone script (and optional Kubernetes Job manifest) that checks the existing GitLab MinIO instance capacity, checks for bucket naming conflicts, and outputs a capacity report. This produces the data needed for the pre-execution MinIO decision point — it does NOT make the decision or provision anything.
- MinIO Bucket Creation, Lifecycle Policies, Quotas, and Access Keys: Create a Helm post-install hook Job that creates MinIO buckets (hermes-staging-artifacts, hermes-prod-artifacts), configures lifecycle expiry policies, sets bucket quotas, and creates per-environment access keys with scoped bucket policies. Assumes the MinIO decision point has been resolved and minio.endpoint/minio.dedicated are set in values.
- Deploy CloudNative-PG Postgres Cluster CR per Namespace: Deploy a CloudNative-PG Cluster custom resource in each namespace with 1 replica, database name 'hermes', and document the operator-generated Secret naming convention for downstream Secret wiring.
- Deploy Redis via Helm Subchart per Namespace: Deploy a single-replica Redis instance using bitnami/redis as a Helm subchart dependency in each namespace, with persistence disabled for staging and enabled for production. Assumes the Redis deployment method decision point has been resolved.
- Deploy NATS with JetStream via Helm Subchart per Namespace: Deploy a single-replica NATS server using nats/nats as a Helm subchart dependency in each namespace with JetStream enabled for durable message delivery.
- Create hermes-infra-secrets Secret Template Referencing Operator-Managed and Helm-Managed Credentials: Create the Helm template for Kubernetes Secret named hermes-infra-secrets in each namespace. For Postgres, reference the CNPG operator-generated Secret (hermes-pg-app) via an init container that extracts the connection URI rather than duplicating credentials in Helm values. For MinIO, Redis, and NATS, source from Helm values.
- Create hermes-infra-endpoints ConfigMap Template: Create the Helm template for ConfigMap named hermes-infra-endpoints in each namespace containing all non-sensitive endpoint keys. All values are deterministic from Helm values (not runtime-derived), so this subtask depends only on the chart scaffold.
- Create PodDisruptionBudgets for Production Backing Services: Create PodDisruptionBudget resources for Postgres, Redis, and NATS in the production namespace only (gated by environment=production) with minAvailable: 1.
- Create Helm Test Pods for Full Infrastructure Validation: Create Helm test pod templates under templates/tests/ that verify Postgres connectivity, Redis PING, NATS JetStream, MinIO bucket read/write, ConfigMap key completeness, and Secret key completeness.
- Write README Documentation and Verify Helm Idempotency: Complete the NOTES.txt with final endpoint values, write comprehensive README.md documenting chart usage, key inventory, capacity gate process, decision records, known limitations, and verify that helm upgrade --install is idempotent.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.