# Task 8: Configure Kubernetes Deployments (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a DevOps Engineer specializing in Kubernetes implementing Task 8.

## Goal

Create Kubernetes deployment manifests for all services with proper scaling, health checks, secrets management, and network policies. Set up ingress and service mesh for external access.

## Requirements

1. Create deployment manifests for all backend services
2. Set up HPA (Horizontal Pod Autoscaler) for each service
3. Configure service discovery with ClusterIP services
4. Create ingress controller for external traffic routing
5. Set up secrets management for sensitive configuration
6. Implement network policies for service isolation
7. Add persistent volume claims for stateful components
8. Configure health checks (liveness/readiness probes)
9. Set up service accounts and RBAC policies
10. Create monitoring and logging configuration

## Acceptance Criteria

All services deploy successfully to Kubernetes, pods scale up/down with HPA, health checks pass consistently, services can communicate internally, external traffic routes correctly through ingress, secrets are mounted securely, and network policies prevent unauthorized access.

## Constraints

- Match existing codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-8): Configure Kubernetes Deployments (Bolt - Kubernetes)`

## Decision Points

### d15: Should we use a service mesh (Istio/Linkerd) for service-to-service communication?
**Category**: architecture | **Constraint**: escalation | ⚠️ **Requires Approval**

Options:
1. no-service-mesh
2. istio
3. linkerd

### d16: What resource requests and limits should be set for each service?
**Category**: performance | **Constraint**: open

Options:
1. minimal-resources
2. moderate-resources
3. resource-profiling-first


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-1, task-2, task-3, task-4
