## Production Hardening: HA, CDN, TLS, Ingress (Bolt - Kubernetes/Helm)

### Objective
Scale infrastructure for production: enable HA for databases and services, configure CDN, TLS, ingress, and network policies for secure, reliable operation.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6, 7, 8

### Implementation Details
{"steps":["Scale PostgreSQL and Redis to HA (multi-replica) mode.","Increase backend service replicas for Equipment Catalog, RMS, Finance, Vetting, Social, and Morgan.","Configure Cloudflare CDN for static assets and SSL termination.","Set up Cloudflare Tunnel ingress for Morgan and web frontend.","Apply Kubernetes network policies to restrict inter-service traffic.","Verify all endpoints are accessible via secure ingress URLs."]}

### Subtasks
- [ ] Scale PostgreSQL to HA multi-replica mode: Update the PostgreSQL operator CR (e.g., CloudNativePG Cluster or Zalando postgresql) to run multiple replicas with streaming replication, ensuring automatic failover and read replicas are configured for production workloads.
- [ ] Scale Redis to HA multi-replica mode: Update the Redis operator CR or Helm values to enable Redis Sentinel or Redis Cluster mode with multiple replicas for automatic failover and high availability.
- [ ] Scale backend service replicas for all application services: Increase Deployment replica counts for Equipment Catalog, RMS, Finance, Vetting, Social, and Morgan services to at least 2 replicas each, with appropriate PodDisruptionBudgets and anti-affinity rules.
- [ ] Configure Cloudflare CDN for static assets and SSL termination: Set up Cloudflare CDN to cache and serve static assets (images, JS, CSS) with SSL termination, appropriate cache rules, and security headers for the web frontend.
- [ ] Set up Cloudflare Tunnel ingress for Morgan and web frontend: Configure Cloudflare Tunnel (cloudflared) as the ingress mechanism for Morgan backend and the web frontend, replacing or supplementing any existing ingress controller, with secure routing and DNS configuration.
- [ ] Apply Kubernetes network policies to restrict inter-service traffic: Define and apply NetworkPolicy resources for all namespaces to enforce least-privilege network access between services, allowing only explicitly required communication paths.
- [ ] End-to-end verification of HA, CDN, TLS, and ingress: Perform comprehensive end-to-end testing to verify all production hardening changes work together: HA failover, CDN caching, TLS termination, ingress routing, and network policy enforcement.