Implement subtask 5012: Create Dockerfile and Kubernetes deployment manifest

## Objective
Build the multi-stage Dockerfile for the vetting service and create the Kubernetes Deployment, Service, and related manifests for namespace sigma1 with proper secret and ConfigMap references.

## Steps
1. Create `services/rust/vetting/Dockerfile` with multi-stage build:
   - Stage 1 (builder): rust:1.75-slim, copy workspace, `cargo build --release -p vetting`
   - Stage 2 (runtime): debian:bookworm-slim, copy binary, install ca-certificates and libssl.
   - EXPOSE 8083, ENTRYPOINT ["/usr/local/bin/vetting"]
2. Create `k8s/vetting/deployment.yaml`:
   - namespace: sigma1
   - replicas: 2
   - container port: 8083
   - envFrom: configMapRef sigma1-infra-endpoints
   - env from secret: sigma1-external-apis (OPENCORPORATES_API_KEY, GOOGLE_PLACES_API_KEY)
   - Resource requests/limits: 128Mi/256Mi memory, 100m/500m CPU
   - Readiness probe: GET /health, periodSeconds 10
   - Liveness probe: GET /health, periodSeconds 30
3. Create `k8s/vetting/service.yaml`: ClusterIP service on port 8083.
4. Verify Dockerfile builds successfully and Kubernetes manifests pass `kubectl apply --dry-run=client`.

## Validation
Build Docker image locally and verify it starts and responds to /health on port 8083. Run `kubectl apply --dry-run=client -f k8s/vetting/` to validate manifest syntax. Verify envFrom references correct ConfigMap and secret names. Verify replicas=2, correct port, and probe paths.