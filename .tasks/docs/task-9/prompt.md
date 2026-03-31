Implement task 9: Production Hardening: HA, CDN, TLS, Ingress (Bolt - Kubernetes/Helm)

## Goal
Harden the Hermes pipeline for production deployment — scale all data services to HA configurations, configure TLS termination, set up ingress routing, apply network policies, and establish resource limits and autoscaling for the application services.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: 1, 2, 3, 4, 5, 6, 7, 8

## Implementation Plan
Step-by-step implementation:

1. **Production namespace:** Create `hermes-production` namespace with appropriate labels and annotations. Apply all ConfigMap and secret patterns from Task 1 with production-specific values.

2. **CloudNative-PG HA scaling:** Update PostgreSQL `Cluster` CR for production:
   - 3 replicas (1 primary + 2 read replicas)
   - Synchronous replication with `minSyncReplicas: 1`
   - Automated failover enabled
   - Backup configuration: scheduled base backups to MinIO (or separate backup bucket), continuous WAL archiving
   - Resource requests/limits: allocate based on expected load (start with 1 CPU / 2Gi memory per replica)
   - PodDisruptionBudget: `maxUnavailable: 1`

3. **Redis HA:** Configure Redis operator CR for production:
   - Sentinel mode with 3 replicas
   - Resource requests/limits
   - PodDisruptionBudget

4. **NATS HA:** Configure NATS operator CR for production:
   - 3-node cluster with JetStream enabled
   - Resource requests/limits

5. **MinIO production hardening:**
   - If using dedicated MinIO tenant: scale to 4+ nodes with erasure coding
   - If using shared MinIO: verify production bucket with appropriate replication policy
   - Lifecycle policy: 365-day retention for production artifacts
   - Enable bucket versioning for accidental deletion protection

6. **TLS termination:** Configure TLS for all public endpoints:
   - Use cert-manager (if available in cluster) for automatic Let's Encrypt certificates
   - Or configure TLS secrets for the Hermes domain
   - Enforce HTTPS redirect on all HTTP endpoints

7. **Ingress configuration:** Create Ingress resources for production:
   - `hermes.{domain}` → Next.js frontend service
   - `hermes-api.{domain}` → Bun/Elysia backend service (or path-based routing: `hermes.{domain}/api/*`)
   - Rate limiting annotations (if using nginx-ingress: `nginx.ingress.kubernetes.io/rate-limit-connections`)
   - CORS headers for frontend-to-API communication

8. **Network policies:** Apply Kubernetes NetworkPolicy resources:
   - Frontend pods can only talk to backend service
   - Backend pods can only talk to PostgreSQL, Redis, NATS, MinIO, and Loki
   - No direct external egress from backend (except for screenshot capture — need egress policy for target URLs)
   - Deny all other inter-namespace traffic

9. **Application autoscaling:**
   - HorizontalPodAutoscaler for Bun/Elysia service: min 2, max 10 replicas, target 70% CPU
   - HorizontalPodAutoscaler for Next.js service: min 2, max 5 replicas, target 70% CPU
   - Resource requests/limits for both services (start with 500m CPU / 512Mi memory)

10. **PodDisruptionBudgets:** For all application services: `minAvailable: 1`

11. **Helm chart updates:** Extend `charts/hermes-infra` with `values-production.yaml` overlay. All production-specific configurations (replica counts, resource limits, TLS, ingress) are driven by values.

## Acceptance Criteria
1. HA PostgreSQL: `kubectl get pods -n hermes-production -l cnpg.io/cluster=hermes-pg` returns 3 Running pods. Killing the primary pod results in automatic failover to a replica within 30 seconds (verified by continuous query during failover).
2. TLS: `curl -v https://hermes.{domain}/api/hermes/deliberations` shows TLS 1.2+ handshake and valid certificate. `curl http://hermes.{domain}` returns 301 redirect to HTTPS.
3. Network policy: A test pod in `hermes-production` namespace can reach PostgreSQL on port 5432 but cannot reach the Kubernetes API server or pods in other namespaces (verified by `kubectl exec` with `curl` and `nc`).
4. Autoscaling: Under load (50 concurrent deliberation requests), HPA scales the backend service beyond 2 replicas within 3 minutes. After load subsides, replicas scale back to 2 within 10 minutes.
5. PodDisruptionBudget: `kubectl get pdb -n hermes-production` shows PDBs for all services with `ALLOWED DISRUPTIONS >= 1`.
6. Ingress routing: `curl https://hermes.{domain}/` returns the Next.js application HTML. `curl https://hermes.{domain}/api/hermes/deliberations` (or `hermes-api.{domain}`) returns JSON from the Elysia service.
7. MinIO versioning: Deleting an object from the production Hermes bucket and then listing versions shows the deleted object as a delete marker with the previous version recoverable.

## Subtasks
- Create hermes-production namespace with ConfigMap and Secret patterns: Create the hermes-production namespace with appropriate labels and annotations, and replicate all ConfigMap and Secret patterns from the dev namespace (Task 1) with production-specific values including the hermes-infra-endpoints ConfigMap.
- Configure CloudNative-PG HA cluster with 3 replicas and synchronous replication: Update the PostgreSQL Cluster CR for production with 3 replicas (1 primary + 2 read replicas), synchronous replication, automated failover, scheduled backups with WAL archiving, resource limits, and a PodDisruptionBudget.
- Configure Redis HA with Sentinel mode and 3 replicas: Configure the Redis operator CR for production with Sentinel mode, 3 replicas, resource requests/limits, and a PodDisruptionBudget.
- Configure NATS HA with 3-node JetStream cluster: Configure the NATS operator CR for production with a 3-node cluster, JetStream enabled, and resource requests/limits.
- Harden MinIO for production with versioning and lifecycle policies: Configure MinIO for production use — either scale a dedicated tenant to 4+ nodes with erasure coding or configure the shared MinIO instance with appropriate replication. Enable bucket versioning and set a 365-day lifecycle retention policy.
- Configure TLS termination with cert-manager and HTTPS enforcement: Set up TLS for all public Hermes endpoints using cert-manager with Let's Encrypt (or pre-provisioned TLS secrets), and enforce HTTPS redirect on all HTTP endpoints.
- Create Ingress resources with routing rules, rate limiting, and CORS: Create Kubernetes Ingress resources for production routing: frontend and API endpoints with rate limiting annotations and CORS headers for frontend-to-API communication.
- Apply Kubernetes NetworkPolicy resources for pod-to-pod isolation: Create NetworkPolicy resources restricting pod-to-pod communication: frontend can only reach backend, backend can only reach data services and Loki, with a special egress exception for headless browser screenshot capture.
- Configure HorizontalPodAutoscalers for backend and frontend services: Create HPA resources for the Bun/Elysia backend (min 2, max 10, target 70% CPU) and Next.js frontend (min 2, max 5, target 70% CPU), with appropriate resource requests and limits on the Deployment specs.
- Create PodDisruptionBudgets for all application services: Create PodDisruptionBudget resources for both the backend and frontend application services with minAvailable: 1 to ensure availability during voluntary disruptions.
- Create Helm values-production.yaml overlay for all production configurations: Extend the hermes-infra Helm chart with a values-production.yaml overlay that drives all production-specific configurations: replica counts, resource limits, TLS, ingress, network policies, HPA, and PDBs.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.