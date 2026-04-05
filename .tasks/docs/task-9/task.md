## Production Hardening: HA, CDN, TLS, Ingress (Bolt - Kubernetes/Helm)

### Objective
Scale infrastructure for production: enable HA for databases and caches, configure CDN/TLS/ingress, and enforce network policies for all services.

### Ownership
- Agent: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6, 7, 8

### Implementation Details
{"steps": ["Scale PostgreSQL and Redis/Valkey to HA (multi-replica, failover enabled).", "Configure Cloudflare CDN for static assets and SSL termination.", "Set up ingress rules and Cloudflare Tunnel for Morgan and web frontend.", "Enforce Kubernetes network policies to restrict inter-service traffic.", "Test failover and recovery for all stateful services.", "Update ConfigMap endpoints as needed for HA.", "Document all production ingress and scaling configurations."]}

### Subtasks
- [ ] Scale PostgreSQL to HA with CloudNative-PG multi-replica and failover: Update the CloudNative-PG cluster CR to enable multi-replica (at least 2 replicas + 1 primary), configure streaming replication, automatic failover, and pod anti-affinity rules for production resilience.
- [ ] Scale Redis/Valkey to HA with replica and sentinel/cluster configuration: Update the Redis/Valkey operator CR to enable HA mode with multiple replicas and sentinel-based or cluster-based failover for production cache resilience.
- [ ] Configure Cloudflare CDN for static assets and SSL termination: Set up Cloudflare CDN to serve static assets (product images, frontend bundles) with proper cache rules and configure SSL/TLS termination at the Cloudflare edge.
- [ ] Configure ingress rules and Cloudflare Tunnel for Morgan and web frontend routing: Set up Kubernetes ingress resources and Cloudflare Tunnel to route external traffic to the Morgan chatbot service and web frontend, with proper path-based and host-based routing.
- [ ] Enforce Kubernetes network policies to restrict inter-service traffic: Define and apply Kubernetes NetworkPolicy resources for all namespaces to enforce least-privilege network access between services, blocking unauthorized pod-to-pod communication.
- [ ] Failover testing and ConfigMap endpoint updates for HA services: Perform comprehensive failover testing for all HA stateful services (PostgreSQL, Redis/Valkey) and update ConfigMap endpoints to reflect HA-aware service addresses. Document all production configurations.