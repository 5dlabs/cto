Implement subtask 2014: Create Kubernetes deployment manifests for catalog service

## Objective
Write Kubernetes Deployment, Service, and related manifests for the catalog service in the sigma1 namespace with proper ConfigMap/Secret references, probes, and resource limits.

## Steps
1. Create `deploy/k8s/catalog/deployment.yaml`:
   - Namespace: `sigma1`
   - Replicas: 2
   - Container image: `catalog:latest` (will be overridden by CI)
   - `envFrom`:
     - `configMapRef: {name: sigma1-infra-endpoints}`
   - `env` from secrets:
     - `sigma1-db-credentials` (POSTGRES_URL)
     - `sigma1-r2-credentials` (R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_ENDPOINT)
     - `sigma1-service-api-keys` (API key values)
   - `livenessProbe`: httpGet `/health/live` port 8080, initialDelaySeconds 5, periodSeconds 10
   - `readinessProbe`: httpGet `/health/ready` port 8080, initialDelaySeconds 10, periodSeconds 5
   - Resources: requests 128Mi/250m, limits 256Mi/500m
   - Security context: runAsNonRoot, readOnlyRootFilesystem
2. Create `deploy/k8s/catalog/service.yaml`:
   - ClusterIP Service targeting port 8080.
   - Named port: `http`.
3. Create `deploy/k8s/catalog/kustomization.yaml` aggregating all manifests.
4. Add ServiceMonitor for Prometheus scraping of `/metrics` endpoint.

## Validation
Validate manifests with `kubectl apply --dry-run=client -f deploy/k8s/catalog/`. Verify kustomize build succeeds: `kubectl kustomize deploy/k8s/catalog/`. Check security context is present. Verify resource limits match spec (256Mi/500m). Verify probes reference correct paths and ports. Deploy to test cluster and verify pods reach Ready state.