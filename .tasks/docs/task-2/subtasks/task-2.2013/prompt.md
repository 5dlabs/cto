Implement subtask 2013: Create multi-stage Dockerfile and Kubernetes deployment manifest

## Objective
Build the multi-stage Dockerfile using rust:1.75-slim builder and distroless runtime, plus the Kubernetes Deployment, Service, and related manifests for the equipment-catalog service.

## Steps
1. Dockerfile at `services/equipment-catalog/Dockerfile`:
   - Stage 1 (builder): FROM rust:1.75-slim AS builder. Install required system deps (libssl-dev, pkg-config). Copy full workspace (Cargo.toml, Cargo.lock, crates/*, services/equipment-catalog/). Run `cargo build --release --bin equipment-catalog`. Use cargo-chef pattern for layer caching if practical.
   - Stage 2 (runtime): FROM gcr.io/distroless/cc-debian12. COPY --from=builder the compiled binary. COPY migrations directory. EXPOSE 8080. ENTRYPOINT ["./equipment-catalog"].
2. Kubernetes manifests in `services/equipment-catalog/k8s/`:
   - `deployment.yaml`: namespace sigma1, 2 replicas, container image placeholder, envFrom configMapRef sigma1-infra-endpoints, resource requests (128Mi/125m), limits (256Mi/250m), livenessProbe httpGet /health/live port 8080 (initialDelaySeconds 5, periodSeconds 10), readinessProbe httpGet /health/ready port 8080 (initialDelaySeconds 10, periodSeconds 5).
   - `service.yaml`: ClusterIP service on port 80 targeting container port 8080.
   - Labels: app=equipment-catalog, part-of=sigma1.
3. Add `.dockerignore` to exclude target/, .git/, etc.
4. Verify the Dockerfile builds successfully with `docker build`.

## Validation
Build the Docker image locally and verify it starts (with mock env vars). Verify the image size is reasonable (<100MB for distroless). Validate Kubernetes manifests with `kubectl apply --dry-run=client`. Verify liveness and readiness probe paths match the implemented health endpoints. Verify envFrom references sigma1-infra-endpoints ConfigMap.