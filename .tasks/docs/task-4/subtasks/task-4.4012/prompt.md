Implement subtask 4012: Implement Kubernetes deployment manifest for finance service

## Objective
Create the Kubernetes Deployment, Service, and related manifests for the finance service in the sigma1 namespace with 2 replicas and envFrom sigma1-infra-endpoints ConfigMap.

## Steps
1. Create `k8s/finance/` directory with:
2. `deployment.yaml`: Deployment in namespace `sigma1`, 2 replicas, container image `sigma1/finance:latest`, port 8080, envFrom referencing `sigma1-infra-endpoints` ConfigMap, resource requests (128Mi memory, 100m CPU) and limits (512Mi memory, 500m CPU), readinessProbe on `/readyz`, livenessProbe on `/healthz`.
3. `service.yaml`: ClusterIP Service exposing port 8080, targeting the finance pods.
4. Add env vars: DATABASE_URL (from ConfigMap/Secret), STRIPE_SECRET_KEY (from Secret), STRIPE_WEBHOOK_SECRET (from Secret), VALKEY_URL (from ConfigMap), RUST_LOG=info.
5. Create a `secret.yaml` template (with placeholder values) for Stripe keys.
6. Ensure labels are consistent: app=finance, service=sigma1-finance.
7. Add a ServiceAccount if needed for RBAC.

## Validation
Verify manifests pass `kubectl apply --dry-run=client -f k8s/finance/`. Verify deployment spec has 2 replicas, correct envFrom reference, correct probes. Verify service selector matches deployment pod labels.