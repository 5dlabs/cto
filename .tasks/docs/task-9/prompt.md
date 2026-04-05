Implement task 9: Production Hardening: HA, CDN, TLS, Ingress (Bolt - Kubernetes/Helm)

## Goal
Scale infrastructure for production: enable HA for databases and services, configure Cloudflare CDN, TLS, ingress, and network policies. Ensures reliability, security, and global access.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: 2, 3, 4, 5, 6, 7, 8

## Implementation Plan
{"steps": ["Scale PostgreSQL and Redis to HA (multi-replica) mode using operator CRs.", "Configure Cloudflare CDN for static assets and SSL termination.", "Set up Cloudflare Tunnel ingress for Morgan and web endpoints.", "Apply Kubernetes network policies to restrict inter-service traffic.", "Update service manifests for readiness/liveness probes and resource limits.", "Test failover and recovery scenarios."]}

## Acceptance Criteria
All services remain available during failover; SSL/TLS is enforced; CDN serves static assets globally; ingress routes are correct; network policies block unauthorized access.

## Subtasks
- Scale PostgreSQL to HA multi-replica mode via operator CR: Update the PostgreSQL operator custom resource to enable high-availability with multiple replicas, streaming replication, and automatic failover. Configure synchronous replication settings and pod anti-affinity to spread replicas across nodes.
- Scale Redis to HA multi-replica mode via operator CR: Update the Redis operator custom resource to enable high-availability with Sentinel or Redis Cluster mode, ensuring automatic failover and read replica distribution across nodes.
- Configure Cloudflare CDN for static assets and SSL termination: Set up Cloudflare CDN to cache and serve static assets globally with SSL/TLS termination at the edge. Configure cache rules, page rules, and SSL mode for all public-facing domains.
- Set up Cloudflare Tunnel ingress for Morgan and web endpoints: Deploy and configure Cloudflare Tunnel (cloudflared) as the ingress mechanism for Morgan assistant and web application endpoints, replacing or augmenting any existing in-cluster ingress.
- Apply Kubernetes network policies to restrict inter-service traffic: Define and apply NetworkPolicy resources to enforce least-privilege network access between pods, namespaces, and external endpoints. Only allow traffic paths that are required by the application architecture.
- Update service manifests with readiness/liveness probes and resource limits: Add or update readiness probes, liveness probes, startup probes, resource requests, and resource limits for all application Deployments and StatefulSets to ensure production reliability and proper scheduling.
- Comprehensive failover and recovery testing: Execute end-to-end failover and recovery tests for all HA components (PostgreSQL, Redis, cloudflared, application services) to validate production resilience under failure conditions.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.