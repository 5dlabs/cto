Implement subtask 10019: Monitoring: create Grafana dashboards (platform overview, per-service, PostgreSQL, Morgan)

## Objective
Create four Grafana dashboard JSON definitions: platform overview, per-service detail, PostgreSQL metrics, and Morgan/Signal-CLI health.

## Steps
Step-by-step:
1. **Platform Overview Dashboard** (`grafana-dashboard-overview.json`):
   - Row 1: Service health status panels (up/down) for all sigma1 services using `up{namespace="sigma1"}` metric
   - Row 2: Total request rate (`sum(rate(http_requests_total{namespace="sigma1"}[5m]))`) and total error rate (`sum(rate(http_requests_total{namespace="sigma1",status=~"5.."}[5m]))`)
   - Row 3: Pod count, CPU usage, memory usage aggregated
2. **Per-Service Dashboard** (`grafana-dashboard-service.json`):
   - Template variable: `service` dropdown
   - Panels: request rate, error rate, latency percentiles (p50, p95, p99), CPU, memory, restart count
3. **PostgreSQL Dashboard** (`grafana-dashboard-postgresql.json`):
   - Active connections, connection pool utilization, query latency histogram, replication lag, WAL size, table sizes
   - Use `cnpg_*` metrics from CloudNativePG operator
4. **Morgan Dashboard** (`grafana-dashboard-morgan.json`):
   - Conversation count (if metric exposed), MCP tool call latency, Signal-CLI pod status/restarts, memory usage trend
5. Create a ConfigMap for each dashboard in the monitoring namespace with the Grafana sidecar label `grafana_dashboard: "1"` so they auto-import.
6. Store dashboard JSON files in `k8s/monitoring/dashboards/`.

## Validation
Apply dashboard ConfigMaps. Open Grafana UI and verify all four dashboards appear in the dashboard list. For each dashboard, verify panels load data (not 'No data' or 'N/A') when services are running. Check that the per-service template variable correctly filters metrics.