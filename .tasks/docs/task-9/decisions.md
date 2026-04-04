## Decision Points

- TLS certificate source: Use cert-manager with Let's Encrypt (ACME) or an organization-internal CA? This affects ClusterIssuer configuration and DNS challenge setup.
- Ingress controller selection: Is nginx-ingress already deployed in the cluster, or should a different controller (e.g., Traefik, Envoy Gateway) be used? Rate-limiting annotations are controller-specific.
- CDN provider selection: If the dashboard is exposed externally, which CDN (Cloudflare, AWS CloudFront, etc.) should front the Ingress? This affects DNS delegation and caching header strategy.
- External access scope: Should the cto-pm API and/or dashboard be publicly accessible via Ingress, or restricted to internal/VPN access only? This fundamentally changes Ingress and NetworkPolicy design.
- Resource limits tuning: The proposed 256Mi/250m requests and 512Mi/500m limits are estimates — should observed metrics from dev be collected first, or should these defaults be applied and adjusted post-deploy?

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm