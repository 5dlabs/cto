# Task 20: Create Docker and Kubernetes deployment configuration

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 20.

## Goal

Build production-ready deployment configuration with multi-stage Docker builds and Kubernetes manifests

## Requirements

1. Create multi-stage Dockerfile for Rust API (build + runtime stages)
2. Create Dockerfile for React dashboard with nginx serving
3. Write Kubernetes manifests: Deployment, Service, ConfigMap, Secret
4. Configure Horizontal Pod Autoscaler (HPA) for API scaling
5. Add ingress configuration for external access
6. Create docker-compose.yml for local development with PostgreSQL and Redis

## Acceptance Criteria

Test Docker builds locally, verify Kubernetes deployment, validate HPA scaling behavior and health checks

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-20): Create Docker and Kubernetes deployment configuration`
