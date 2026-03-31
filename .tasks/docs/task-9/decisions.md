## Decision Points

- Ingress routing strategy: subdomain-based (`hermes-api.{domain}`) vs path-based (`hermes.{domain}/api/*`) — affects CORS complexity and ingress controller configuration
- TLS certificate provisioning: cert-manager with Let's Encrypt vs pre-provisioned TLS secrets — depends on cluster capabilities and domain DNS control
- MinIO deployment model: dedicated MinIO tenant with erasure coding (4+ nodes) vs shared existing MinIO instance with replication policy — cost and isolation trade-off
- Headless browser egress policy: allow all external egress from backend pods (simpler but less secure) vs maintain an explicit allow-list of target domains for screenshot capture (more secure but operationally complex)

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm