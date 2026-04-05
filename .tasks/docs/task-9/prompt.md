Implement task 9: Production Hardening: HA, CDN, TLS, Ingress, Network Policies (Bolt - Kubernetes/Helm)

## Goal
Scale all infrastructure for production: CloudNative-PG to 3-instance HA with synchronous replication, Valkey sentinel, Cloudflare Tunnel ingress for all services, CDN configuration for R2 assets, network policy enforcement, resource limit tuning, and HPA configuration.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: 2, 3, 4, 5, 6, 7, 8

## Implementation Plan
1. Scale CloudNative-PG cluster to production:
   - Update `sigma1-postgres` Cluster CR: instances 1 → 3
   - Enable synchronous replication: `minSyncReplicas: 1, maxSyncReplicas: 2`
   - Configure automated backups to R2: `barmanObjectStore` with R2 endpoint, schedule `0 */6 * * *` (every 6 hours)
   - Set resource requests/limits: 1Gi memory, 500m CPU per instance
   - Enable PodDisruptionBudget: maxUnavailable 1
   - Configure connection pooling via PgBouncer sidecar (built into CNPG): max_connections 100 per instance
2. Scale Valkey for production:
   - Option A: Valkey with sentinel (3 nodes) via Opstree operator
   - Option B: Single Valkey with persistence (AOF) if sentinel not supported by operator
   - Enable persistence: appendonly yes, appendfsync everysec
   - Resource limits: 512Mi memory, 250m CPU
   - PodDisruptionBudget: maxUnavailable 1
3. Cloudflare Tunnel ingress configuration:
   - Create ClusterTunnel CR mapping:
     - `sigma-1.com` → Website frontend service (port 3000)
     - `api.sigma-1.com/catalog/*` → equipment-catalog service (port 8080)
     - `api.sigma-1.com/rms/*` → rms service (port 8080)
     - `api.sigma-1.com/finance/*` → finance service (port 8080)
     - `api.sigma-1.com/vetting/*` → customer-vetting service (port 8080)
     - `api.sigma-1.com/social/*` → social-engine service (port 8080)
     - `api.sigma-1.com/ws/*` → morgan service (WebSocket, port 8080)
     - `morgan.sigma-1.com` → Morgan direct access (if needed)
   - Configure Cloudflare Access policies for admin endpoints
   - TLS is automatic via Cloudflare (no cert-manager needed)
4. Cloudflare R2 CDN configuration:
   - Configure custom domain `assets.sigma-1.com` for R2 bucket
   - Set cache rules: images immutable (1 year cache), thumbnails 30 days
   - Enable Cloudflare Polish for image optimization
5. Network policies (Cilium):
   - Default deny all ingress to sigma1 namespace
   - Allow: Cloudflare Tunnel → frontend, backend services
   - Allow: backend services → sigma1-postgres, sigma1-valkey
   - Allow: Morgan → all backend services
   - Allow: frontend (SSR) → backend APIs
   - Deny: backend services → other backend services (except Morgan→all, RMS↔Equipment Catalog if needed)
   - Allow: sigma1 → external APIs (Stripe, OpenCorporates, etc.) via egress policy
6. Horizontal Pod Autoscaler configuration:
   - Equipment Catalog: min 2, max 5, target CPU 70%
   - RMS: min 2, max 5, target CPU 70%
   - Finance: min 2, max 3, target CPU 70%
   - Social Engine: min 1, max 3, target CPU 70%
   - Morgan: min 1, max 2 (stateful, scale carefully)
   - Customer Vetting: min 1, max 2
7. Resource limit tuning based on observed metrics:
   - Review Prometheus metrics from dev deployment
   - Set production requests/limits per service
8. Update sigma1-infra-endpoints ConfigMap with production hostnames.
9. Configure Grafana dashboards for Sigma-1:
   - Service health dashboard (all 6 services)
   - PostgreSQL dashboard (CNPG metrics)
   - Valkey dashboard
   - Request latency dashboard (p50, p95, p99 per endpoint)
10. ArgoCD Application CR for sigma1 namespace with automated sync and self-heal.

## Acceptance Criteria
1. HA test: kill one CNPG replica pod, verify remaining replicas continue serving reads AND writes within 30 seconds (automatic failover). 2. Backup test: trigger manual backup, verify barman object appears in R2 bucket, restore to a test cluster, verify data integrity. 3. Tunnel test: curl https://sigma-1.com returns 200 with website content; curl https://api.sigma-1.com/catalog/health/ready returns 200. 4. CDN test: request image from assets.sigma-1.com, verify CF-Cache-Status header is HIT on second request. 5. Network policy test: exec into finance pod, attempt curl to social-engine pod — verify connection refused/timeout. Exec into Morgan pod, attempt curl to finance pod — verify success. 6. HPA test: generate load on Equipment Catalog (e.g., 50 concurrent requests/sec), verify pod count scales from 2 to 3+ within 2 minutes. 7. PDB test: attempt to drain node hosting 1 of 2 equipment-catalog pods, verify at least 1 pod remains running. 8. Grafana dashboard test: all 4 dashboards load without errors and display data from sigma1 services.

## Subtasks
- Scale CloudNative-PG to 3-instance HA with synchronous replication: Update the sigma1-postgres Cluster CR to run 3 instances with synchronous replication enabled, configure PgBouncer connection pooling, set production resource requests/limits, and add a PodDisruptionBudget.
- Configure CloudNative-PG automated backups to R2: Set up automated barman backups from CloudNative-PG to the Cloudflare R2 bucket on a 6-hour schedule, and verify backup/restore functionality.
- Scale Valkey for production with persistence and PDB: Configure Valkey for production use: either sentinel mode (3 nodes) or single-instance with AOF persistence, resource limits, and a PodDisruptionBudget.
- Configure Cloudflare Tunnel ingress with ClusterTunnel CR: Create the ClusterTunnel CR to route external traffic from sigma-1.com and api.sigma-1.com to all backend services, including WebSocket support for Morgan.
- Configure Cloudflare Access policies for admin endpoints: Set up Cloudflare Access policies to protect admin endpoints and sensitive routes behind authentication.
- Configure Cloudflare R2 CDN with custom domain and cache rules: Set up the custom domain assets.sigma-1.com for the R2 bucket, configure cache rules for images and thumbnails, and enable Cloudflare Polish for image optimization.
- Implement Cilium default-deny and ingress network policies: Create CiliumNetworkPolicy resources to enforce default-deny ingress in the sigma1 namespace, then add allow rules for Cloudflare Tunnel to frontend and backend services, backend to database/cache, Morgan to all backends, and frontend SSR to backend APIs.
- Implement Cilium egress network policies for external API access: Create CiliumNetworkPolicy egress rules to allow sigma1 services to reach external APIs (Stripe, OpenCorporates, etc.) while denying other outbound traffic.
- Configure Horizontal Pod Autoscalers for all services: Create HPA resources for all 6 backend services with appropriate min/max replicas and CPU target thresholds.
- Tune resource requests and limits for all services: Review Prometheus metrics from the dev deployment and set appropriate production resource requests and limits for all 6 backend services and the frontend.
- Update sigma1-infra-endpoints ConfigMap with production hostnames: Update the sigma1-infra-endpoints ConfigMap to reflect production hostnames, CDN URLs, and any changed connection strings for production topology.
- Create Grafana dashboards for service health, PostgreSQL, Valkey, and latency: Configure 4 Grafana dashboards: service health overview for all 6 services, CloudNative-PG metrics, Valkey metrics, and request latency percentiles.
- Create ArgoCD Application CR for sigma1 namespace: Define an ArgoCD Application CR for the sigma1 namespace with automated sync, self-heal, and prune enabled, pointing to the infrastructure Git repository.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.