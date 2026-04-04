## Decision Points

- Redis HA strategy: Should Redis use the operator's built-in Sentinel/replicated mode, or should a separate StatefulSet with Redis Sentinel be deployed? This depends on operator capabilities and version.
- Ingress controller selection: Use Nginx Ingress Controller or an existing cluster-level ingress (e.g., Traefik, Istio Gateway)? This affects annotation syntax and TLS configuration approach.
- TLS certificate provisioning: Use cert-manager with Let's Encrypt (requires DNS or HTTP challenge configuration) or use a pre-provisioned/self-signed certificate? Affects DNS requirements and automation.
- External secrets backing store: Which secret store backend (AWS Secrets Manager, Vault, GCP Secret Manager, etc.) is used for external-secrets? This determines ExternalSecret spec and rotation capabilities.
- Audit logging approach: Cluster-level Kubernetes audit policy (requires cluster-admin changes) vs. sidecar-based structured logging? Depends on cluster access level and existing audit infrastructure.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm