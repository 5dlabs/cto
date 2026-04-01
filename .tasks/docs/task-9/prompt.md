Implement task 9: Production Hardening: HA Scaling, CDN, TLS, Ingress (Bolt - Kubernetes/Helm)

## Goal
Harden the Sigma-1 pipeline production deployment by enabling high-availability scaling for all services, configuring TLS termination, ingress routing, and CDN for the web frontend.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: medium
- Dependencies: 2, 3, 4, 5, 6, 7, 8

## Implementation Plan
1. Update Helm values for production (`values-sigma1-prod.yaml`):
   a. PM server: replicas 3, resource requests (256Mi RAM, 250m CPU), resource limits (512Mi RAM, 500m CPU).
   b. Frontend: replicas 2, resource requests (128Mi RAM, 100m CPU).
2. Configure HorizontalPodAutoscaler for PM server: min 3, max 10, target CPU 70%.
3. Create Ingress resource for the PM server API:
   a. Host: `api.sigma1.5dlabs.io`.
   b. TLS via cert-manager with Let's Encrypt ClusterIssuer.
   c. Annotations for rate limiting (100 req/s per IP).
4. Create Ingress resource for the web frontend:
   a. Host: `sigma1.5dlabs.io`.
   b. TLS via cert-manager.
   c. CDN cache headers: `Cache-Control: public, max-age=3600` for static assets.
5. Configure PodDisruptionBudgets: PM server minAvailable 2, frontend minAvailable 1.
6. Add readiness and liveness probes for all deployments:
   - PM server: HTTP GET `/health` on port 3000, initialDelay 10s, period 15s.
   - Frontend: HTTP GET `/` on port 3000, initialDelay 5s, period 10s.
7. Configure anti-affinity rules to spread PM server pods across availability zones.

## Acceptance Criteria
1. `kubectl get hpa -n sigma1-prod` shows PM server HPA with correct min/max/target. 2. `kubectl get ingress -n sigma1-prod` shows both ingresses with TLS configured. 3. `curl -I https://api.sigma1.5dlabs.io/health` returns 200 with valid TLS certificate. 4. `curl -I https://sigma1.5dlabs.io` returns 200 with Cache-Control header on static assets. 5. PDB validation: `kubectl get pdb -n sigma1-prod` shows correct minAvailable values. 6. Kill one PM server pod; verify traffic continues to be served (zero downtime confirmed by continuous health check). 7. Pod anti-affinity: verify pods are distributed across at least 2 nodes/zones via `kubectl get pods -o wide`.

## Subtasks
- Update Helm production values for PM server replicas and resource limits: Create or update `values-sigma1-prod.yaml` to set PM server deployment to 3 replicas with production-grade resource requests and limits.
- Update Helm production values for frontend replicas and resource limits: Update `values-sigma1-prod.yaml` to set the frontend deployment to 2 replicas with appropriate resource requests.
- Configure HorizontalPodAutoscaler for PM server: Create an HPA manifest for the PM server with min 3, max 10 replicas targeting 70% average CPU utilization.
- Create Ingress resource for PM server API with TLS and rate limiting: Define an Ingress manifest for `api.sigma1.5dlabs.io` with cert-manager TLS and rate-limiting annotations.
- Create Ingress resource for web frontend with TLS and CDN cache headers: Define an Ingress manifest for `sigma1.5dlabs.io` with cert-manager TLS and CDN-friendly cache-control headers for static assets.
- Configure PodDisruptionBudgets for PM server and frontend: Create PDB manifests ensuring PM server has minAvailable=2 and frontend has minAvailable=1 during voluntary disruptions.
- Add readiness and liveness probes to PM server deployment: Configure HTTP health probes for the PM server: readiness and liveness via GET /health on port 3000.
- Add readiness and liveness probes to frontend deployment: Configure HTTP health probes for the frontend: readiness and liveness via GET / on port 3000.
- Configure pod anti-affinity rules for PM server cross-zone distribution: Add pod anti-affinity rules to the PM server Deployment to spread pods across availability zones and nodes.
- Validate full production hardening deployment end-to-end: Deploy all production hardening manifests to the sigma1-prod namespace and run comprehensive validation tests.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.