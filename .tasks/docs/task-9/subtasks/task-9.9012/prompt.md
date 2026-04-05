Implement subtask 9012: Create Grafana dashboards for service health, PostgreSQL, Valkey, and latency

## Objective
Configure 4 Grafana dashboards: service health overview for all 6 services, CloudNative-PG metrics, Valkey metrics, and request latency percentiles.

## Steps
1. Service Health Dashboard:
   - Panel: pod count per deployment (current vs desired)
   - Panel: pod restart count per service (last 24h)
   - Panel: HTTP request rate per service (if metrics are exposed)
   - Panel: HTTP error rate (5xx) per service
   - Panel: CPU and memory usage per service
   - Variables: namespace=sigma1, service selector
2. PostgreSQL Dashboard (CNPG metrics):
   - Panel: active connections per instance
   - Panel: transactions per second (commits/rollbacks)
   - Panel: replication lag (bytes and seconds)
   - Panel: WAL generation rate
   - Panel: PgBouncer pool stats (active, waiting, idle)
   - Panel: disk usage per PVC
3. Valkey Dashboard:
   - Panel: connected clients
   - Panel: commands processed per second
   - Panel: memory usage vs max memory
   - Panel: keyspace hits/misses ratio
   - Panel: AOF rewrite status (if persistence enabled)
4. Request Latency Dashboard:
   - Panel: p50 latency per endpoint/service
   - Panel: p95 latency per endpoint/service
   - Panel: p99 latency per endpoint/service
   - Panel: request duration heatmap
   - Variables: service, time range
5. Create dashboards as ConfigMaps with the `grafana_dashboard` label for Grafana sidecar auto-discovery, or use GrafanaDashboard CRs if the Grafana operator is installed.
6. Store dashboard JSON files in the infrastructure repo.

## Validation
Access Grafana UI, verify all 4 dashboards appear in the dashboard list. Open each dashboard and verify: (1) Service health dashboard shows data for all 6 services with no panel errors. (2) PostgreSQL dashboard shows CNPG metrics with replication lag visible. (3) Valkey dashboard shows connected clients and memory usage. (4) Latency dashboard shows p50/p95/p99 panels populated with data.