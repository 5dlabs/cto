Implement task 3: Production Hardening and CI/CD Pipeline (Bolt - Kubernetes/Helm)

## Goal
Harden the NotifyCore deployment for production readiness: scale PostgreSQL to HA (3 replicas), configure Redis sentinel, add TLS ingress, network policies restricting traffic to the notifycore namespace, resource quota enforcement, and a CI/CD pipeline definition that builds, tests, and deploys the service.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: medium
- Dependencies: 1, 2

## Implementation Plan
1. **PostgreSQL HA**: Update CloudNativePG `Cluster` CR to 3 replicas with synchronous replication (minSyncReplicas: 1). Add PodDisruptionBudget (maxUnavailable: 1). Configure automated backups to an object store or PVC-based WAL archiving.
2. **Redis HA**: Switch to Redis Sentinel or a 3-node Redis cluster. Update `notifycore-infra-endpoints` ConfigMap REDIS_URL to sentinel-aware connection string.
3. **Ingress & TLS**: Create an Ingress resource for `notifycore.{domain}` with TLS via cert-manager ClusterIssuer (Let's Encrypt). Add annotations for rate limiting at the ingress level (nginx: `nginx.ingress.kubernetes.io/limit-rps: "100"`).
4. **Network Policies**: (a) Default deny all ingress/egress in `notifycore` namespace. (b) Allow notifycore pods → notifycore-pg on 5432. (c) Allow notifycore pods → notifycore-redis on 6379. (d) Allow ingress controller → notifycore pods on 8080. (e) Allow DNS egress to kube-dns.
5. **HPA**: HorizontalPodAutoscaler for notifycore deployment: min 2, max 10, target CPU 70%.
6. **Secret rotation**: Annotate secrets for external-secrets-operator or sealed-secrets compatibility. Document rotation procedure.
7. **RBAC**: ServiceAccount `notifycore-sa` with minimal permissions. No cluster-wide roles.
8. **Resource Quotas**: Namespace-level ResourceQuota (requests.cpu: 2, requests.memory: 1Gi, limits.cpu: 4, limits.memory: 2Gi).
9. **CI/CD Pipeline** (GitHub Actions or similar): (a) On PR: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`. (b) On merge to main: Docker build + push to registry, Helm upgrade to staging. (c) Manual approval gate for production Helm upgrade. (d) Include sqlx migration check step (`cargo sqlx prepare --check`).
10. Update `infra/notifycore/values-prod.yaml` with all HA settings. Keep `values-dev.yaml` unchanged for local development.

## Acceptance Criteria
1. `helm template` with `values-prod.yaml` renders without errors and produces: 3-replica CloudNativePG cluster, HPA with min=2/max=10, at least 4 NetworkPolicy resources, an Ingress with TLS block, a ResourceQuota, and a ServiceAccount. 2. `kubectl apply --dry-run=server` of all production manifests succeeds against a test cluster API. 3. NetworkPolicy validation: a test pod in the notifycore namespace can reach notifycore-pg on 5432 and notifycore-redis on 6379, but cannot reach external IPs or other namespaces. A pod outside the namespace cannot reach notifycore pods on 8080 (only the ingress controller can). 4. CI pipeline definition file exists and contains stages for lint, test, build, and deploy with the sqlx prepare check step present. 5. PodDisruptionBudget allows at most 1 unavailable pod for both the app and database deployments.

## Subtasks
- Upgrade CloudNativePG to 3-replica HA with synchronous replication and backups: Update the CloudNativePG Cluster CR for production: 3 replicas with synchronous replication (minSyncReplicas: 1), PodDisruptionBudget (maxUnavailable: 1), and automated backup configuration.
- Configure Redis HA with Sentinel mode: Switch the Redis deployment from standalone to Sentinel mode for production, with 3 nodes and automatic failover, and update the REDIS_URL in the ConfigMap to a sentinel-aware connection string.
- Create Ingress resource with TLS via cert-manager and rate limiting: Create an Ingress resource for the NotifyCore service with TLS termination via cert-manager ClusterIssuer (Let's Encrypt) and nginx rate limiting annotations.
- Implement namespace NetworkPolicies for default-deny and allowed traffic: Create five NetworkPolicy resources: default deny all, allow app→PostgreSQL, allow app→Redis, allow ingress controller→app, and allow DNS egress.
- Configure HPA, RBAC ServiceAccount, and ResourceQuota: Create the HorizontalPodAutoscaler (min 2, max 10, 70% CPU), a minimal ServiceAccount with no cluster-wide roles, and a namespace-level ResourceQuota.
- Add secret rotation annotations and document rotation procedure: Annotate existing secrets for compatibility with external-secrets-operator or sealed-secrets, and create documentation for the secret rotation procedure.
- Create values-prod.yaml consolidating all production settings: Create the consolidated values-prod.yaml file integrating all HA, security, and scaling settings while ensuring values-dev.yaml remains unchanged.
- Define CI/CD pipeline with lint, test, build, and deploy stages: Create a GitHub Actions (or equivalent) CI/CD pipeline definition with PR checks (fmt, clippy, test, sqlx prepare check), Docker build+push on merge, Helm deploy to staging, and manual approval for production.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.