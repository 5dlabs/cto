Implement subtask 2009: Multi-stage Dockerfile and Kubernetes Deployment manifest

## Objective
Create a multi-stage Dockerfile producing an image under 100MB and a Kubernetes Deployment manifest referencing the notifycore-infra-endpoints ConfigMap with health probes and resource limits.

## Steps
1. **Dockerfile** (at project root):
   - Stage 1 (builder): `FROM rust:1.75-slim AS builder`. Install required system deps (libssl-dev, pkg-config if needed). `WORKDIR /app`. Copy `Cargo.toml`, `Cargo.lock`, create dummy `src/main.rs` for dependency caching. `RUN cargo build --release`. Copy actual source. `RUN cargo build --release`.
   - Stage 2 (runtime): `FROM debian:bookworm-slim`. Install minimal runtime deps (`ca-certificates`, `libssl3` if needed). Copy binary from builder: `COPY --from=builder /app/target/release/notifycore /usr/local/bin/`. `EXPOSE 8080`. `ENTRYPOINT ["notifycore"]`.
   - Add `.dockerignore` excluding `target/`, `.git/`, `tests/`.
   - Verify image size < 100MB.
2. **Kubernetes Deployment** (`k8s/deployment.yaml` or within the Helm chart):
   - Deployment with 1 replica (dev), image reference placeholder.
   - `envFrom: [{configMapRef: {name: notifycore-infra-endpoints}}]`.
   - Also mount secrets for DATABASE_URL password and REDIS_URL password if using separate secret approach.
   - Liveness probe: `httpGet: {path: /health, port: 8080}`, initialDelaySeconds: 5, periodSeconds: 10.
   - Readiness probe: same path, initialDelaySeconds: 3, periodSeconds: 5.
   - Resources: requests: {memory: 64Mi, cpu: 100m}, limits: {memory: 256Mi, cpu: 500m}.
   - Container port: 8080.
3. Create a Service manifest exposing port 8080 (ClusterIP).

## Validation
`docker build -t notifycore:test .` completes successfully. `docker image inspect notifycore:test --format '{{.Size}}'` shows size < 100MB (104857600 bytes). `kubectl apply --dry-run=client -f k8s/deployment.yaml` succeeds. Deployment manifest contains envFrom referencing notifycore-infra-endpoints, both liveness and readiness probes on /health, and resource requests/limits matching spec.