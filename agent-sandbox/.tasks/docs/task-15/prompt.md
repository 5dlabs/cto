# Task 15: Build Docker multi-stage image and Kubernetes deployment manifests

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 15.

## Goal

Create optimized Docker build with multi-stage compilation, Kubernetes deployment with HPA, and production-ready configuration

## Requirements

1. Create Dockerfile:
   - Stage 1: rust:1.75 as builder, copy source, cargo build --release
   - Stage 2: debian:bookworm-slim, copy binary and templates, install ca-certificates
   - EXPOSE 3000, CMD ["./teamsync-api"]
2. Create infra/k8s/:
   - deployment.yaml: 3 replicas, resource limits (500m CPU, 512Mi memory), liveness/readiness probes
   - service.yaml: ClusterIP service on port 80 -> 3000
   - hpa.yaml: HorizontalPodAutoscaler targeting 70% CPU, min 3, max 10 replicas
   - configmap.yaml: non-sensitive config (LOG_LEVEL, CORS_ORIGINS)
   - secret.yaml: template for DATABASE_URL, JWT_SECRET, etc.
   - ingress.yaml: TLS termination, path routing
3. Add .dockerignore: target/, node_modules/, .git/
4. Create docker-compose.yml for local dev: API, PostgreSQL, Redis
5. Build script: docker build --target production -t teamsync-api:latest .

## Acceptance Criteria

Build Docker image, verify size < 100MB. Run container, test health endpoints. Deploy to local k8s (minikube), verify pods start. Test HPA scales under load (use hey or k6). Verify logs appear in JSON format

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-15): Build Docker multi-stage image and Kubernetes deployment manifests`
