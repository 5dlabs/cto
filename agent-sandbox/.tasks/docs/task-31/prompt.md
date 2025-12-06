# Task 31: Create Docker multi-stage build and Kubernetes manifests

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 31.

## Goal

Build production-ready Docker image with multi-stage build and create Kubernetes deployment with HPA and health checks

## Requirements

1. Create Dockerfile:
   - Stage 1 (builder): FROM rust:1.75-alpine, cargo build --release
   - Stage 2 (runtime): FROM alpine:3.19, copy binary, add ca-certificates
   - EXPOSE 8080
2. Create .dockerignore: target/, node_modules/, .git/
3. Create infra/k8s/deployment.yaml:
   - Deployment with 2 replicas
   - Resources: requests (cpu: 100m, memory: 128Mi), limits (cpu: 500m, memory: 512Mi)
   - livenessProbe: GET /health/live, initialDelaySeconds: 10
   - readinessProbe: GET /health/ready, initialDelaySeconds: 5
   - Env vars from ConfigMap and Secret
4. Create infra/k8s/service.yaml: ClusterIP service on port 80 -> 8080
5. Create infra/k8s/hpa.yaml:
   - HorizontalPodAutoscaler: min 2, max 10 replicas
   - Target: 70% CPU utilization
6. Create infra/k8s/configmap.yaml: non-sensitive config
7. Create infra/k8s/secret.yaml: DATABASE_URL, REDIS_URL, JWT_SECRET
8. Implement health endpoints in api/health.rs:
   - GET /health/live -> 200 OK
   - GET /health/ready -> check DB and Redis connections, return 200 or 503

## Acceptance Criteria

Build Docker image and verify size <50MB, test container startup, deploy to local k8s (minikube), verify HPA scales on load, test health endpoints, verify rolling updates work

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-31): Create Docker multi-stage build and Kubernetes manifests`
