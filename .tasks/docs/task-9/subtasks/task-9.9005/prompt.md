Implement subtask 9005: Register all services with Prometheus and configure Grafana dashboards

## Objective
Ensure every service exposes metrics and is scraped by Prometheus. Create Grafana dashboards covering service health, request rates, error rates, and resource utilization for all services.

## Steps
1) Verify each service exposes a `/metrics` endpoint (or equivalent) with Prometheus-format metrics. If using ServiceMonitor CRDs (from prometheus-operator), create a ServiceMonitor for each service specifying the metrics port and path. 2) If not using ServiceMonitors, add Prometheus scrape annotations to each Service: `prometheus.io/scrape: 'true'`, `prometheus.io/port`, `prometheus.io/path`. 3) Verify Prometheus targets page shows all services as 'UP'. 4) Create a Grafana dashboard for overall cluster health: CPU/memory usage per service, pod restart counts, replica counts vs desired. 5) Create a Grafana dashboard for application metrics: HTTP request rate, latency percentiles (p50/p95/p99), error rate (5xx), per service. 6) Create alerting rules in Prometheus for critical conditions: service down > 1 min, error rate > 5%, pod restarts > 3 in 5 min, high latency. 7) Export dashboards as JSON and store them as ConfigMaps or in the repo for reproducibility.

## Validation
Open Prometheus targets page and confirm every service shows status 'UP' with recent scrape timestamps. Open Grafana dashboards and verify data is populated for all services (no 'No Data' panels). Trigger a known condition (e.g., kill a pod) and verify the dashboard reflects the change within the scrape interval. Verify alerting rules fire by simulating a service outage and checking AlertManager receives the alert.