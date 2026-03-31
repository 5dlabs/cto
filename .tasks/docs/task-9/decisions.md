## Decision Points

- Cloudflare CDN plan tier selection (Free vs Pro vs Business) — determines available features like WAF rules, cache analytics, and page rules
- Cloudflare Tunnel authentication strategy — whether to use Cloudflare Access (Zero Trust) for internal endpoints or rely solely on application-level auth
- PostgreSQL HA topology — streaming replication with Patroni vs CloudNativePG operator built-in HA; impacts failover behavior and management complexity
- Observability stack deployment method — self-hosted Prometheus/Grafana/Loki via Helm charts vs kube-prometheus-stack operator bundle; affects maintainability and resource usage

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm