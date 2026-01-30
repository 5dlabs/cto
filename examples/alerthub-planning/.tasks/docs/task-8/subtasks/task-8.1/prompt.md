# Subtask 8.1: Create Core Kubernetes Deployment Manifests

**Parent Task:** Configure Kubernetes Deployments (Bolt - Kubernetes)
**Agent:** bolt | **Language:** yaml

## Description

Create deployment manifests for all backend services including proper resource allocation, scaling configuration, and health checks

## Details

Create deployment YAML files for each backend service with proper resource requests/limits, replica counts, container specifications, environment variables, and liveness/readiness probes. Include HPA (Horizontal Pod Autoscaler) configurations for automatic scaling based on CPU/memory metrics. Set up service accounts and RBAC policies for each service.

## Dependencies

None

## Acceptance Criteria

- [ ] Subtask requirements implemented
- [ ] Parent task requirements still satisfied

## Resources

- Parent task: `.tasks/docs/task-8/prompt.md`
- PRD: `.tasks/docs/prd.md`
