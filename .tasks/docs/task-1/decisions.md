## Decision Points

- Specific S3/R2 provider and region to use (e.g., Cloudflare R2, AWS S3).
- Granularity of initial network policies (e.g., namespace-wide vs. pod-specific).
- Choice of PostgreSQL and Redis/Valkey operator versions and their upgrade strategy.
- Initial resource requests/limits for infrastructure components in development environment.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm