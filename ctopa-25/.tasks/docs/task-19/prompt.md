# Task 19: Create Docker and Kubernetes deployment

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 19.

## Goal

Setup containerization with multi-stage builds and K8s manifests with HPA

## Requirements

1. Create Dockerfile with multi-stage build (builder + runtime)
2. Add docker-compose.yml for local development
3. Create k8s/ directory with deployment, service, ingress manifests
4. Setup HorizontalPodAutoscaler based on CPU/memory
5. Add ConfigMap and Secret manifests
6. Configure resource limits and health checks

## Acceptance Criteria

Test Docker build, local compose stack, and Kubernetes deployment with scaling

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-19): Create Docker and Kubernetes deployment`
