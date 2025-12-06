# Task 28: Create Docker multi-stage build and Kubernetes manifests

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 28.

## Goal

Build optimized Docker image with multi-stage build and create Kubernetes deployment with HPA, ConfigMaps, and Secrets.

## Requirements

1. Create Dockerfile:
   ```dockerfile
   FROM rust:1.75 as builder
   WORKDIR /app
   COPY Cargo.* ./
   RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
   COPY src ./src
   RUN touch src/main.rs && cargo build --release
   
   FROM debian:bookworm-slim
   RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
   COPY --from=builder /app/target/release/teamsync-api /usr/local/bin/
   EXPOSE 8080
   CMD ["teamsync-api"]
   ```
2. Create infra/k8s/namespace.yaml
3. Create infra/k8s/configmap.yaml (non-sensitive config)
4. Create infra/k8s/secret.yaml (JWT_SECRET, DB credentials, etc.)
5. Create infra/k8s/deployment.yaml:
   - 3 replicas, resource limits (500m CPU, 512Mi memory)
   - livenessProbe: /health/live, readinessProbe: /health/ready
   - Environment variables from ConfigMap and Secret
6. Create infra/k8s/service.yaml (ClusterIP, port 8080)
7. Create infra/k8s/hpa.yaml:
   - Min 3, max 10 replicas
   - Target CPU 70%, memory 80%
8. Create infra/k8s/ingress.yaml (HTTPS with cert-manager)

## Acceptance Criteria

Build Docker image, verify size < 100MB. Run container locally, test health endpoints. Deploy to local k8s (minikube/kind), verify pods start. Test HPA by generating load. Verify rolling updates work. Test ConfigMap/Secret changes trigger restart.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-28): Create Docker multi-stage build and Kubernetes manifests`
