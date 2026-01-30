# Task 8: Service Mesh and Ingress Configuration (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a DevOps Engineer specializing in Kubernetes implementing Task 8.

## Goal

Configure Kubernetes ingress, service mesh networking, and cross-service communication with proper security policies and load balancing.

## Requirements

Deploy ingress controller (NGINX or Traefik), configure service mesh for internal communication, set up network policies for service isolation, implement load balancing for high availability, and configure TLS termination for external traffic.

## Acceptance Criteria

External endpoints are accessible via ingress, internal service communication works correctly, network policies block unauthorized traffic, load balancing distributes traffic evenly, and TLS certificates are valid and auto-renewing

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-8): Service Mesh and Ingress Configuration (Bolt - Kubernetes)`

## Decision Points

### d15: Service mesh technology choice
**Category**: architecture | **Constraint**: open

Options:
1. Istio for full-featured service mesh
2. Linkerd for lightweight service mesh
3. no service mesh, use standard Kubernetes networking

### d16: TLS certificate management strategy
**Category**: security | **Constraint**: soft

Options:
1. cert-manager with Let's Encrypt
2. manual certificate management
3. cloud provider managed certificates


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-1, task-2, task-3, task-4
