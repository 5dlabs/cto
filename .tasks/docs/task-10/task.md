## Production Hardening, Security & CI/CD (Bolt - Kubernetes/Helm)

### Objective
Harden the entire Sigma-1 platform for production: HA scaling for all services, CDN/TLS/ingress configuration, network policies, RBAC, secret rotation, GDPR deletion orchestrator, CI/CD pipelines with automated QA agents (Stitch, Cleo, Tess, Cipher, Atlas), ArgoCD GitOps, and comprehensive monitoring/alerting.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6, 7, 8, 9

### Implementation Details
1. **HA Scaling**:
   - Equipment Catalog: 2 replicas with pod anti-affinity (already in Task 2 manifest, verify)
   - RMS: 2 replicas with pod anti-affinity
   - Finance: 2 replicas with pod anti-affinity
   - Customer Vetting: 2 replicas with pod anti-affinity
   - Social Engine: 1→2 replicas (upgrade from medium priority)
   - Morgan: 1 replica (stateful with workspace PVC; HA requires session affinity design — document limitation)
   - PostgreSQL: already instances: 2 from Task 1. Add resource requests/limits tuning based on observed usage.
   - Valkey: Configure Sentinel mode if operator supports, or document single-instance limitation.
   - HorizontalPodAutoscaler (HPA) for Equipment Catalog (CPU > 70% → scale to 4).
2. **Network Policies**:
   - Default deny all ingress/egress in `sigma1` namespace.
   - Allow: sigma1 services → sigma1-db namespace (PostgreSQL, Valkey) on specific ports.
   - Allow: sigma1 services → openclaw namespace (NATS) on port 4222 (social engine only).
   - Allow: openclaw namespace (Morgan) → sigma1 services on service ports.
   - Allow: sigma1 services → external APIs (OpenCorporates, Stripe, Google, etc.) via egress.
   - Allow: ingress controller → sigma1 services for public endpoints.
   - Deny: direct cross-service communication except documented paths.
3. **Ingress & TLS**:
   - Cloudflare Tunnel configuration for all public endpoints.
   - TLS termination at Cloudflare edge.
   - Internal mTLS between services using cert-manager if available, or document as Phase 2.
   - CDN caching rules: equipment images (1 year cache), API responses (no cache or short TTL).
4. **RBAC & Security**:
   - Per-service Kubernetes ServiceAccounts with minimal RBAC roles.
   - PodSecurityPolicies/PodSecurityStandards: restricted profile (no root, read-only rootfs, drop all capabilities).
   - Image scanning: configure policy to block images with critical CVEs.
   - Secret rotation: document process for rotating DB passwords, API keys, Stripe keys. If external-secrets-operator is available, configure automatic rotation.
5. **GDPR Deletion Orchestrator** (per D12 resolution):
   - Build a Rust CLI binary (in Rex Cargo workspace, new member `gdpr-orchestrator`) that:
     a. Accepts `--customer-id <UUID>` argument
     b. Calls GDPR deletion endpoint on each service in order: Vetting → Social → Finance → RMS → Catalog
     c. Collects HTTP response status and confirmation from each service
     d. Writes structured JSON audit log to `audit` schema table: `gdpr_deletions` (request_id, customer_id, service, status, response, completed_at)
     e. Exits with code 0 only if all services return success
   - Package as Docker image.
   - Create Kubernetes Job template that Morgan can trigger via MCP tool `sigma1_gdpr_delete`.
   - Alternatively, CronJob that processes pending deletion requests from a queue table.
6. **CI/CD Pipeline** (GitHub Actions):
   - **On PR**:
     a. Lint: Clippy (Rust), golangci-lint (Go), Biome (Node.js/TypeScript), ESLint (Next.js)
     b. Build: Cargo build (all workspace members), go build, bun build, next build
     c. Test: cargo test, go test, bun test, vitest — enforce 80% minimum coverage
     d. Security scan: Semgrep + CodeQL + Dependabot alerts
     e. Container image build (no push)
   - **On merge to main**:
     a. Build + push container images to registry (ghcr.io or Cloudflare Container Registry)
     b. Update ArgoCD Application manifests with new image tags
     c. ArgoCD syncs automatically
   - **Deployment**:
     a. ArgoCD Applications for each service in `sigma1` namespace
     b. Sync policy: automated with self-heal and prune
     c. Rollback: ArgoCD automatic rollback on health check failure
7. **Monitoring & Alerting**:
   - Grafana dashboards:
     a. Platform overview: all service health, request rates, error rates
     b. Per-service dashboard: latency percentiles, error rates, resource usage
     c. PostgreSQL dashboard: connections, query latency, replication lag
     d. Morgan dashboard: conversation count, tool call latency, Signal-CLI health
   - Prometheus AlertManager rules:
     a. Service down > 2 minutes → PagerDuty/Signal alert
     b. Error rate > 5% for 5 minutes → warning
     c. PostgreSQL replication lag > 30 seconds → critical
     d. Signal-CLI pod restart > 3 in 10 minutes → warning (memory leak indicator)
     e. Valkey memory > 80% → warning
     f. Certificate expiry < 14 days → warning
8. **Audit Logging**:
   - All services log to stdout in structured JSON (already configured in individual tasks).
   - Loki collects logs from all sigma1 and openclaw namespaces.
   - Audit-specific events (login, data access, deletion) tagged with `audit=true` label for filtered queries.
   - Retention: 90 days for standard logs, 1 year for audit logs.

### Subtasks
- [ ] HA scaling: update replica counts and pod anti-affinity for all application services: Update Kubernetes deployment manifests for Equipment Catalog, RMS, Finance, Customer Vetting, and Social Engine to 2 replicas each with pod anti-affinity rules. Document Morgan single-replica limitation with session affinity notes.
- [ ] HA scaling: configure HPA for Equipment Catalog: Create a HorizontalPodAutoscaler resource for the Equipment Catalog service that scales from 2 to 4 replicas when CPU utilization exceeds 70%.
- [ ] HA scaling: PostgreSQL and Valkey resource tuning and Valkey Sentinel evaluation: Tune PostgreSQL resource requests/limits based on observed usage from Task 1 (already 2 instances). Evaluate Valkey operator for Sentinel mode support and either configure it or document the single-instance limitation.
- [ ] Network policies: default deny all ingress/egress in sigma1 namespace: Create a default-deny NetworkPolicy in the sigma1 namespace that blocks all ingress and egress traffic by default.
- [ ] Network policies: allow sigma1 services to sigma1-db namespace (PostgreSQL and Valkey): Create NetworkPolicy allowing sigma1 service pods to reach PostgreSQL on port 5432 and Valkey on port 6379 in the sigma1-db namespace.
- [ ] Network policies: allow sigma1 social-engine to NATS in openclaw namespace: Create NetworkPolicy allowing only the social-engine pod in sigma1 to reach NATS on port 4222 in the openclaw namespace.
- [ ] Network policies: allow Morgan (openclaw) to sigma1 services and ingress controller to sigma1: Create NetworkPolicy allowing Morgan pods in openclaw namespace to reach sigma1 service ports, and allowing the ingress controller to reach sigma1 public endpoints.
- [ ] Network policies: allow sigma1 egress to external APIs: Create egress NetworkPolicy allowing sigma1 services to reach external APIs (OpenCorporates, Stripe, Google, etc.) and DNS resolution.
- [ ] Ingress and TLS: configure Cloudflare Tunnel for all public endpoints: Configure a Cloudflare Tunnel (cloudflared) deployment to expose all sigma1 public endpoints with TLS termination at the Cloudflare edge.
- [ ] Ingress and CDN: configure CDN caching rules for equipment images vs API responses: Set up Cloudflare CDN caching rules: 1-year cache for equipment images, no-cache or short TTL for API responses.
- [ ] RBAC: create per-service ServiceAccounts with minimal roles: Create dedicated Kubernetes ServiceAccounts for each sigma1 service with minimal RBAC Roles and RoleBindings scoped to only what each service needs.
- [ ] Pod security: apply PodSecurity Standards restricted profile and image scanning policy: Enforce PodSecurity Standards restricted profile on the sigma1 namespace (no root, read-only rootfs, drop all capabilities) and configure image scanning to block critical CVEs.
- [ ] Secret rotation: document process and configure automation if external-secrets-operator is available: Document the process for rotating all secrets (DB passwords, API keys, Stripe keys) and configure external-secrets-operator for automatic rotation if available in the cluster.
- [ ] GDPR deletion orchestrator: implement Rust CLI binary: Build a Rust CLI binary (new Cargo workspace member `gdpr-orchestrator`) that accepts a customer ID, calls GDPR deletion endpoints on each service in order, and writes a structured audit log.
- [ ] GDPR deletion orchestrator: create Dockerfile and Kubernetes Job template: Package the GDPR orchestrator CLI as a Docker image and create a Kubernetes Job manifest template that Morgan can trigger via MCP tool.
- [ ] CI/CD: GitHub Actions PR workflow (lint, build, test, security scan): Create a GitHub Actions workflow that runs on every PR: linting for all languages, building all projects, running tests with 80% coverage enforcement, and security scanning.
- [ ] CI/CD: GitHub Actions merge-to-main workflow (image build, push, manifest update): Create a GitHub Actions workflow triggered on merge to main that builds and pushes container images to the registry and updates ArgoCD application manifests with new image tags.
- [ ] ArgoCD GitOps: create Application CRs for all sigma1 services with automated sync: Create ArgoCD Application custom resources for each sigma1 service with automated sync, self-heal, prune, and rollback on health check failure.
- [ ] Monitoring: create Grafana dashboards (platform overview, per-service, PostgreSQL, Morgan): Create four Grafana dashboard JSON definitions: platform overview, per-service detail, PostgreSQL metrics, and Morgan/Signal-CLI health.
- [ ] Monitoring: configure Prometheus AlertManager rules: Create PrometheusRule CRs for all alerting conditions: service down, error rate, PostgreSQL replication lag, Signal-CLI restarts, Valkey memory, certificate expiry.
- [ ] Audit logging: configure Loki log collection with retention policies: Configure Loki to collect logs from sigma1 and openclaw namespaces, with audit-tagged event filtering and differentiated retention (90 days standard, 1 year audit).