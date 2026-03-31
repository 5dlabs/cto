Implement subtask 9006: Deploy Prometheus for metrics collection

## Objective
Deploy Prometheus in the cluster to scrape metrics from all application services, databases, and infrastructure components. Configure service discovery, scrape configs, and persistent storage for metrics retention.

## Steps
1. Deploy Prometheus using the Helm chart (e.g., `prometheus-community/prometheus` or `kube-prometheus-stack` if chosen).
2. Configure persistent storage for Prometheus with a PVC (at least 50Gi, retention 15 days).
3. Set up ServiceMonitor or PodMonitor CRDs for each application service that exposes a `/metrics` endpoint.
4. Configure scrape intervals: 15s for application services, 30s for infrastructure.
5. Add scrape configs for PostgreSQL exporter, Redis exporter, and cloudflared metrics.
6. Configure Alertmanager with basic alert rules: service down, high error rate, disk usage >80%, PostgreSQL replication lag >10s.
7. Set resource requests and limits for Prometheus pods.
8. Expose Prometheus UI internally (ClusterIP service only, not externally).

## Validation
Verify Prometheus pod is running and scraping targets (check `/targets` endpoint shows all expected targets as UP). Confirm metrics are being stored (query `up{}` returns results for all services). Verify Alertmanager fires a test alert when conditions are met. Check PVC is bound and metrics persist across pod restarts.