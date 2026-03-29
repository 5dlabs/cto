## Decision Points

- PostgreSQL backup target: object store (S3/GCS/MinIO) vs PVC-based WAL archiving — depends on available cloud infrastructure and backup retention requirements
- Redis HA mode: Sentinel vs Redis Cluster — Sentinel is simpler for single-master with failover, Cluster provides sharding but adds complexity; depends on expected cache workload
- Secret management: external-secrets-operator vs sealed-secrets — depends on existing secret management infrastructure (e.g., Vault, AWS Secrets Manager) vs GitOps-only approach
- Ingress controller type: nginx-ingress vs other (Traefik, Envoy) — rate limiting annotations are nginx-specific; confirm cluster's ingress controller
- Container registry: which registry to push Docker images to (GHCR, ECR, GCR, Docker Hub) — affects CI/CD pipeline configuration
- Production domain: the actual domain for `notifycore.{domain}` needs to be decided for the Ingress and TLS certificate

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm