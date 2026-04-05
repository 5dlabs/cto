Implement subtask 6017: Create Kubernetes deployment manifest for social-engine

## Objective
Write the Kubernetes Deployment, Service, and related manifests for the social-engine in the sigma1 namespace with proper resource configuration, environment injection from sigma1-infra-endpoints ConfigMap, and health probes.

## Steps
1. Create `k8s/social-engine-deployment.yaml`:
   - Namespace: `sigma1`
   - Deployment: 1 replica, image: `social-engine:latest` (or appropriate registry path).
   - Container port: 3000.
   - `envFrom`: reference `sigma1-infra-endpoints` ConfigMap for POSTGRES_URL, R2_ENDPOINT, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET, OPENAI_API_KEY, JWT_SECRET, and platform API credentials.
   - Liveness probe: `GET /health/live` port 3000, initialDelaySeconds 10, periodSeconds 15.
   - Readiness probe: `GET /health/ready` port 3000, initialDelaySeconds 15, periodSeconds 10.
   - Resource requests: 128Mi memory, 100m CPU. Limits: 512Mi memory, 500m CPU.
2. Create `k8s/social-engine-service.yaml`:
   - ClusterIP Service on port 80 targeting container port 3000.
3. Create `k8s/social-engine-serviceMonitor.yaml` (if Prometheus Operator is present):
   - ServiceMonitor targeting the service, scraping `/metrics` on port 80.
4. Ensure all secrets referenced are listed as environment variable sources (they should come from the infra-endpoints ConfigMap or separate Secrets).

## Validation
Apply manifests to a test cluster with `kubectl apply --dry-run=server`. Verify Deployment, Service, and ServiceMonitor are valid. Verify envFrom references sigma1-infra-endpoints. Verify probes point to correct paths and ports.