## Production Hardening: HA, CDN, TLS, Ingress (Bolt - Kubernetes/Helm)

### Objective
Scale infrastructure for production: enable HA for databases and services, configure Cloudflare CDN, TLS, ingress, and network policies.

### Ownership
- Agent: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6, 7, 8

### Implementation Details
{"steps": ["Scale PostgreSQL and Redis to HA (multi-replica) mode.", "Increase backend service replicas to meet concurrency targets.", "Configure Cloudflare CDN and TLS for all public endpoints.", "Set up Cloudflare Tunnel ingress for Morgan and web.", "Apply Kubernetes network policies to restrict inter-service access.", "Test failover and rollback procedures."]}

### Subtasks
- [ ] Scale PostgreSQL to HA multi-replica mode with CloudNative-PG: Update the CloudNative-PG Cluster CR to enable multi-replica HA with synchronous replication and automatic failover for the production PostgreSQL instance.
- [ ] Scale Redis/Valkey to HA multi-replica mode: Update the Redis/Valkey operator CR to enable HA with sentinel or cluster mode, providing automatic failover for the production cache layer.
- [ ] Increase backend service replicas for production concurrency: Scale all backend service Deployments to multiple replicas with appropriate resource requests, HPA configuration, and pod disruption budgets.
- [ ] Configure Cloudflare CDN and TLS for all public endpoints: Set up Cloudflare DNS, CDN caching rules, and TLS certificates for all public-facing domains and subdomains.
- [ ] Set up Cloudflare Tunnel ingress for Morgan and website: Deploy Cloudflare Tunnel (cloudflared) as a Kubernetes Deployment to expose Morgan (Discord bot dashboard) and the public website without opening inbound ports.
- [ ] Apply Kubernetes NetworkPolicies for inter-service access restriction: Define and apply NetworkPolicy resources to enforce least-privilege network access between all services, databases, and external egress.
- [ ] Validate failover and rollback procedures with chaos testing: Perform structured chaos testing to validate HA failover for PostgreSQL, Redis, backend services, and Cloudflare Tunnel, and document rollback procedures.