## Production Hardening: HA, CDN, TLS, Ingress (Bolt - Kubernetes/Helm)

### Objective
Scale all services to high-availability, configure Cloudflare CDN, TLS, ingress, and network policies for production. Ensure all endpoints are secure and observable.

### Ownership
- Agent: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 2, 3, 4, 5, 6, 7, 8

### Implementation Details
{"steps": ["Update deployments for all backend and frontend services to minimum 2 replicas.", "Configure Cloudflare CDN and TLS for all public endpoints.", "Set up Cloudflare Tunnel ingress for Morgan agent.", "Apply Kubernetes network policies to restrict inter-service traffic.", "Ensure all services are registered with Prometheus/Grafana for observability.", "Test failover and rollback procedures."]}

### Subtasks
- [ ] Scale all backend and frontend deployments to minimum 2 replicas with PodDisruptionBudgets: Update every service Deployment to run at least 2 replicas with appropriate resource requests/limits and create PodDisruptionBudgets to guarantee availability during rolling updates and node maintenance.
- [ ] Configure Cloudflare CDN and TLS for all public-facing endpoints: Set up Cloudflare DNS records, enable CDN caching for static assets, and configure TLS (Full Strict mode) for all public-facing domains and subdomains.
- [ ] Set up Cloudflare Tunnel ingress for Morgan agent and internal services: Deploy and configure a Cloudflare Tunnel (cloudflared) within the cluster to securely expose the Morgan agent endpoint and any other internal services that need external reachability without a public IP or LoadBalancer.
- [ ] Define and apply Kubernetes NetworkPolicies to restrict inter-service traffic: Design an allowed communication matrix for all services and implement Kubernetes NetworkPolicies that default-deny all traffic and explicitly allow only required service-to-service, ingress, and egress paths.
- [ ] Register all services with Prometheus and configure Grafana dashboards: Ensure every service exposes metrics and is scraped by Prometheus. Create Grafana dashboards covering service health, request rates, error rates, and resource utilization for all services.
- [ ] Test failover scenarios and document rollback procedures: Execute structured failover tests for all HA services and document step-by-step rollback procedures for deployments, database migrations, and infrastructure changes.