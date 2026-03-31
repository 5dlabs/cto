Implement subtask 9007: Deploy Grafana with dashboards for observability

## Objective
Deploy Grafana and configure it with Prometheus and Loki as data sources. Create or import dashboards for application services, database health, and infrastructure metrics.

## Steps
1. Deploy Grafana using the Helm chart (`grafana/grafana`).
2. Configure Prometheus as a data source via provisioning (ConfigMap or values.yaml).
3. Configure Loki as a data source via provisioning.
4. Import or create dashboards:
   - Kubernetes cluster overview (import dashboard ID 6417 or similar)
   - PostgreSQL health: connections, replication lag, query latency
   - Redis health: memory usage, connected clients, hit ratio
   - Application services: request rate, error rate, latency (p50/p95/p99)
   - Cloudflare Tunnel status
5. Store dashboard JSON as ConfigMaps with the `grafana_dashboard` label for auto-provisioning.
6. Configure persistent storage for Grafana (to retain user-created dashboards).
7. Expose Grafana via internal service; optionally add to Cloudflare Tunnel for authenticated access.

## Validation
Verify Grafana pod is running and accessible. Confirm Prometheus and Loki data sources are green (healthy) in Grafana settings. Verify all provisioned dashboards load and display data. Confirm dashboards persist across Grafana pod restarts.