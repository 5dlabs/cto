## Production Hardening: HA, CDN, TLS, Ingress (Bolt - Kubernetes/Helm)

### Objective
Scale infrastructure for production: enable HA for databases and services, configure CDN, TLS, ingress, and network policies. Ensures reliability, security, and performance for all user-facing endpoints.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6, 7, 8

### Implementation Details
{"steps": ["Scale PostgreSQL and Redis to HA (multi-replica, failover enabled).", "Configure Cloudflare CDN for static assets and SSL termination.", "Set up Cloudflare Tunnel ingress for Morgan and web endpoints.", "Apply Kubernetes network policies to restrict inter-service traffic.", "Enable Prometheus/Grafana/Loki for observability.", "Test failover and rollback scenarios."]}

### Subtasks
- [ ] Scale PostgreSQL to HA with multi-replica failover: Configure the PostgreSQL operator CR to run multiple replicas with streaming replication and automatic failover enabled. Update the existing PostgreSQL CR to increase replica count, configure replication slots, and set failover policies.
- [ ] Scale Redis to HA with Sentinel or cluster mode: Configure the Redis operator CR to enable high availability with automatic failover using Redis Sentinel or cluster mode. Ensure existing services can connect through the HA-aware endpoint.
- [ ] Configure Cloudflare CDN for static assets and SSL/TLS termination: Set up Cloudflare CDN to cache and serve static assets with SSL/TLS termination at the edge. Configure caching rules, SSL mode, and security headers.
- [ ] Set up Cloudflare Tunnel ingress for application endpoints: Deploy and configure a Cloudflare Tunnel (cloudflared) in the Kubernetes cluster to expose Morgan and web endpoints securely without opening inbound ports. Configure TLS enforcement on all routes.
- [ ] Define and apply Kubernetes network policies for inter-service traffic restriction: Create NetworkPolicy resources to enforce least-privilege network access between services. Only explicitly allowed traffic should be permitted; all other inter-pod communication should be denied by default.
- [ ] Deploy Prometheus for metrics collection: Deploy Prometheus in the cluster to scrape metrics from all application services, databases, and infrastructure components. Configure service discovery, scrape configs, and persistent storage for metrics retention.
- [ ] Deploy Grafana with dashboards for observability: Deploy Grafana and configure it with Prometheus and Loki as data sources. Create or import dashboards for application services, database health, and infrastructure metrics.
- [ ] Deploy Loki for centralized log aggregation: Deploy Grafana Loki and Promtail (or similar log shipper) to aggregate logs from all pods in the cluster. Configure log retention and label-based querying.
- [ ] Validate failover and rollback scenarios: Run structured failover and rollback tests to verify HA configurations work correctly under failure conditions. Document runbooks for operational incident response.