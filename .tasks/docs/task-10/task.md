## Production Hardening: HA, Ingress, RBAC, and Security (Bolt - Kubernetes/Helm)

### Objective
Harden the sigma-1-dev deployment for production readiness: configure Cloudflare Tunnel ingress via accesstunnel/clustertunnel CRDs, enforce RBAC on all service accounts, enable audit logging, configure resource limits and health probes, and set up secret rotation policies. This task depends on all implementation tasks being complete.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6, 7, 8, 9

### Implementation Details
1. **Cloudflare Tunnel Ingress (per D8):** Create an `accesstunnel` or `clustertunnel` CR in `sigma-1-dev` namespace to expose the PM server and frontend (if deployed) via Cloudflare Tunnel. Configure TLS termination at the Cloudflare edge. Map routes: `/api/*` → PM server service, `/` → frontend service (if Tasks 6-9 are deployed). Do NOT deploy NGINX or any other ingress controller.
2. **Cloudflare Access (per D7 recommendation):** Configure a Cloudflare Access application on the tunnel to restrict access to authorized users. This provides SSO/MFA without any application-level auth code. If D7 resolves to JWT/RBAC instead, this step would need to be replaced with application-level auth middleware.
3. **RBAC Hardening:** Tighten the `sigma-1-pm-server` ServiceAccount RBAC: read-only access to configmaps and secrets in `sigma-1-dev` only. Create a separate ServiceAccount for the frontend deployment with read-only API access. Ensure no service account has cluster-wide permissions.
4. **Resource Limits and Probes:** Set CPU/memory requests and limits on all deployments (PM server: 256m/512Mi request, 1000m/1Gi limit; frontend: 128m/256Mi request, 500m/512Mi limit). Configure liveness probes (HTTP GET /health, 10s interval) and readiness probes (HTTP GET /ready, 5s interval) on all services.
5. **Network Policies:** Create NetworkPolicy resources to restrict traffic: PM server can reach bridge services, GitHub API, Hermes API, and Linear API. Frontend can only reach PM server. No other egress allowed from sigma-1-dev namespace except DNS.
6. **Audit Logging:** Enable Kubernetes audit logging for the `sigma-1-dev` namespace. Log all create/update/delete operations on secrets, configmaps, and deployments.
7. **Secret Rotation:** Configure external-secrets operator refresh interval to 24h for all secrets in `sigma-1-secrets`. Add annotations for rotation alerts.
8. **HA Consideration:** For dev/validation, single replica is acceptable. Document the Helm values needed to scale to 2+ replicas for production (PM server: 2 replicas with PodDisruptionBudget minAvailable=1).
9. **Note on scope:** If D5 resolves to defer Tasks 6-9, remove frontend-related ingress routes, service accounts, resource limits, and network policies. Update dependencies to [2, 3, 4, 5] only.

### Subtasks
- [ ] Create Cloudflare Tunnel CR with route mapping for PM server and frontend: Create an accesstunnel or clustertunnel Custom Resource in the sigma-1-dev namespace to expose services via Cloudflare Tunnel. Configure route mappings for /api/* to the PM server service and / to the frontend service (if in scope per D5). TLS terminates at Cloudflare edge. No NGINX or other ingress controller.
- [ ] Configure Cloudflare Access application for SSO/MFA on the tunnel: Set up a Cloudflare Access application on the tunnel to restrict access to authorized users, providing SSO/MFA at the edge without application-level auth code. This subtask is contingent on D7 resolving to Cloudflare Access (not JWT/RBAC).
- [ ] Harden RBAC for PM server ServiceAccount: Tighten the sigma-1-pm-server ServiceAccount RBAC to read-only access for configmaps and secrets in the sigma-1-dev namespace only. Ensure no cluster-wide permissions exist.
- [ ] Create frontend ServiceAccount with read-only API access: Create a separate ServiceAccount for the frontend deployment with minimal read-only permissions. Only applicable if D5 includes Tasks 6-9.
- [ ] Configure resource limits and requests on all deployments: Set CPU and memory requests and limits on the PM server and frontend deployments per the specified values: PM server 256m/512Mi request, 1000m/1Gi limit; frontend 128m/256Mi request, 500m/512Mi limit.
- [ ] Configure liveness and readiness probes on all deployments: Add HTTP liveness probes (GET /health, 10s interval) and readiness probes (GET /ready, 5s interval) to all service deployments in sigma-1-dev.
- [ ] Create NetworkPolicy resources restricting egress traffic: Create Kubernetes NetworkPolicy resources to restrict traffic flow: PM server can reach bridge services, GitHub API, Hermes API, and Linear API. Frontend can only reach PM server. Block all other egress from sigma-1-dev except DNS.
- [ ] Enable Kubernetes audit logging for sigma-1-dev namespace: Configure Kubernetes audit logging to capture all create, update, and delete operations on secrets, configmaps, and deployments within the sigma-1-dev namespace.
- [ ] Configure external-secrets operator refresh interval and rotation alerts: Set the external-secrets operator refresh interval to 24 hours for all ExternalSecret resources in sigma-1-secrets and add annotations for rotation alerting.
- [ ] Document HA scaling Helm values and conditional scope adjustment for D5: Document the Helm values needed to scale PM server to 2+ replicas with PodDisruptionBudget for production. Also document the scope adjustments needed if D5 resolves to defer Tasks 6-9 (remove frontend ingress routes, ServiceAccount, resource limits, and NetworkPolicies).
- [ ] Security review of RBAC policies and NetworkPolicy configurations: Review all RBAC Role/RoleBinding manifests and NetworkPolicy manifests for correctness, least-privilege compliance, and absence of over-permissive rules.