# Implementation Prompt for Task 8

## Context
You are implementing "Service Mesh and Ingress Configuration (Bolt - Kubernetes)" for the AlertHub notification platform.

## PRD Reference
See `../../prd.md` for full requirements.

## Task Requirements
Configure Kubernetes ingress, service mesh networking, and cross-service communication with proper security policies and load balancing.

## Implementation Details
Deploy ingress controller (NGINX or Traefik), configure service mesh for internal communication, set up network policies for service isolation, implement load balancing for high availability, and configure TLS termination for external traffic.

## Dependencies
This task depends on: task-1, task-2, task-3, task-4. Ensure those are complete before starting.

## Testing Requirements
External endpoints are accessible via ingress, internal service communication works correctly, network policies block unauthorized traffic, load balancing distributes traffic evenly, and TLS certificates are valid and auto-renewing

## Decision Points to Address

The following decisions need to be made during implementation:

### d15: Service mesh technology choice
**Category**: architecture | **Constraint**: open

Options:
1. Istio for full-featured service mesh
2. Linkerd for lightweight service mesh
3. no service mesh, use standard Kubernetes networking

Document your choice and rationale in the implementation.

### d16: TLS certificate management strategy
**Category**: security | **Constraint**: soft

Options:
1. cert-manager with Let's Encrypt
2. manual certificate management
3. cloud provider managed certificates

Document your choice and rationale in the implementation.


## Deliverables
1. Source code implementing the requirements
2. Unit tests with >80% coverage
3. Integration tests for external interfaces
4. Documentation updates as needed
5. Decision point resolutions documented

## Notes
- Follow project coding standards
- Use Effect TypeScript patterns where applicable
- Ensure proper error handling and logging
