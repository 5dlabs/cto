Implement task 9: Production Hardening: HA, CDN, TLS, Ingress (Bolt - Kubernetes/Helm)

## Goal
Scale infrastructure for production: enable HA for databases and services, configure Cloudflare CDN, TLS, ingress, and network policies.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: 2, 3, 4, 5, 6, 7, 8

## Implementation Plan
{"steps": ["Scale PostgreSQL and Redis to HA mode (multi-instance)", "Increase replicas for backend services and Morgan agent", "Configure Cloudflare CDN for static assets and SSL termination", "Set up Cloudflare Tunnel ingress for Morgan and web frontend", "Apply Kubernetes network policies to restrict inter-service traffic", "Test failover and recovery for all critical services"]}

## Acceptance Criteria
All services remain available during failover; CDN serves static assets with valid SSL; ingress routes traffic correctly; network policies block unauthorized access; simulate pod/node failure and verify recovery.

## Subtasks
- Scale PostgreSQL to HA mode with CloudNative-PG: Update the CloudNative-PG Cluster CR to run multiple instances (primary + replicas) with streaming replication, configure automatic failover, and validate data consistency across replicas.
- Scale Redis/Valkey to HA mode: Update the Redis/Valkey deployment to run in HA mode with sentinel or replication, configure automatic failover, and validate session/cache continuity during failover.
- Increase replicas for backend services and Morgan agent with HPA: Scale all backend service Deployments to multiple replicas and configure Horizontal Pod Autoscalers (HPA) for CPU/memory-based autoscaling.
- Configure Cloudflare CDN for static assets and SSL termination: Set up Cloudflare CDN to cache and serve static assets from the web frontend, configure SSL/TLS termination at Cloudflare edge, and set appropriate cache rules.
- Set up Cloudflare Tunnel ingress for Morgan and web frontend: Deploy and configure a Cloudflare Tunnel (cloudflared) in the cluster to expose Morgan agent and web frontend services to the internet without opening inbound ports.
- Define and apply Kubernetes NetworkPolicies for inter-service traffic restriction: Map all legitimate inter-service communication flows and create NetworkPolicy resources to enforce least-privilege network access between pods.
- Full failover and recovery simulation testing: Execute comprehensive failover tests for all critical services (PostgreSQL, Redis, backend, cloudflared) to validate HA configurations and document recovery procedures.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.