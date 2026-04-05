Implement subtask 5013: Create Kubernetes deployment manifest for customer-vetting service

## Objective
Write the Kubernetes Deployment, Service, and related manifests for the customer-vetting service in the sigma1 namespace with 1 replica and envFrom sigma1-infra-endpoints ConfigMap.

## Steps
1. Create `k8s/customer-vetting/deployment.yaml`:
   - namespace: sigma1
   - replicas: 1
   - Container image: placeholder for CI to fill
   - envFrom: configMapRef sigma1-infra-endpoints
   - Additional env vars: CREDIT_API_ENABLED (default "false"), OPENCORPORATES_API_KEY (from Secret), GOOGLE_PLACES_API_KEY (from Secret), LINKEDIN_API_KEY (from Secret)
   - Resources: requests 128Mi/100m, limits 512Mi/500m
   - Liveness probe: GET /healthz, initialDelaySeconds 10, periodSeconds 30
   - Readiness probe: GET /healthz, initialDelaySeconds 5, periodSeconds 10
2. Create `k8s/customer-vetting/service.yaml`:
   - ClusterIP service exposing port 8080
3. Create `k8s/customer-vetting/secret.yaml` (ExternalSecret or sealed-secret template) for API keys.
4. Dockerfile in service crate root: multi-stage build (builder with cargo-chef for caching, runtime with distroless/cc).
5. Verify manifests pass `kubectl apply --dry-run=client`.

## Validation
Run `kubectl apply --dry-run=client -f k8s/customer-vetting/` with no errors. Dockerfile builds successfully. Deployment references correct ConfigMap and Secrets. Health probes point to /healthz.