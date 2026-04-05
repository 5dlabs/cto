Implement subtask 4014: Create Dockerfile and Kubernetes deployment manifests

## Objective
Build the multi-stage Dockerfile for the finance service and create Kubernetes Deployment, Service, and related manifests for namespace sigma1.

## Steps
1. Create `services/rust/finance/Dockerfile`:
   - Stage 1 (builder): `FROM rust:1.75-bookworm`, copy workspace Cargo.toml/Cargo.lock and all crate directories, `cargo build --release -p finance`.
   - Stage 2 (runtime): `FROM debian:bookworm-slim`, install `ca-certificates` and `libssl3`, copy binary from builder, set `ENTRYPOINT ["./finance"]`.
   - Use `.dockerignore` to exclude `target/`, `.git/`, etc.
2. Create `k8s/finance/deployment.yaml`:
   - Namespace: sigma1.
   - Replicas: 2.
   - Container: image from Dockerfile, port 8082.
   - `envFrom`: reference `sigma1-infra-endpoints` ConfigMap.
   - Environment variables from secrets: `sigma1-postgres-credentials`, `sigma1-stripe-credentials`, API key secret.
   - Resource requests/limits: 256Mi memory, 250m CPU request; 512Mi memory, 500m CPU limit.
   - Readiness probe: GET /health, port 8082, initialDelaySeconds 5.
   - Liveness probe: GET /health, port 8082, initialDelaySeconds 10.
3. Create `k8s/finance/service.yaml`:
   - ClusterIP service, port 80 → targetPort 8082.
   - Selector matching deployment labels.
4. Create `k8s/finance/hpa.yaml` (optional, simple):
   - Min 2, max 4 replicas, target CPU 70%.
5. Verify Dockerfile builds successfully with `docker build`.

## Validation
Verify Dockerfile builds without errors using `docker build -t finance:test .` from workspace root. Verify resulting image runs and responds to GET /health. Verify Kubernetes manifests are valid YAML (use `kubectl apply --dry-run=client`). Verify deployment references correct ConfigMap and secrets. Verify service selector matches deployment labels. Verify resource requests/limits are set.