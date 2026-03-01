# Task 16: Service Deployment and Configuration (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 16.

## Goal

Create Kubernetes manifests and deploy notification router service

## Requirements

1. Create Dockerfile with multi-stage build\n2. Build Kubernetes Deployment manifest\n3. Configure Service and optional Ingress\n4. Add ConfigMap for environment variables\n5. Set up horizontal pod autoscaling

## Acceptance Criteria

Service deploys successfully, health checks pass, scales based on load, accessible via Kubernetes service

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-16): Service Deployment and Configuration (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 8, 13, 14, 15
