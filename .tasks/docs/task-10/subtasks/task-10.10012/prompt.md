Implement subtask 10012: Create AlertManager rules for service monitoring and alerting

## Objective
Define Prometheus AlertManager rules for: service down (0 ready pods), CNPG replica lag, error rate spikes, certificate/token expiry warnings, and PVC disk usage warnings.

## Steps
1. Create a PrometheusRule CR (or AlertManager config) `sigma1-alerts` with the following alert rules:
2. Critical alerts:
   - `ServiceDown`: `kube_deployment_status_replicas_available == 0` for any deployment in sigma1 namespace. Severity: critical. For: 1m.
3. Warning alerts:
   - `CNPGReplicaLag`: `cnpg_pg_replication_lag > 10` for CNPG clusters. Severity: warning. For: 5m.
   - `HighErrorRate`: `rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.05` per service. Severity: warning. For: 5m.
   - `TokenExpiryWarning`: custom metric or CronJob-based check that fires when JWT tokens or TLS certificates expire within 14 days. Severity: warning.
   - `PVCDiskUsage`: `kubelet_volume_stats_used_bytes / kubelet_volume_stats_capacity_bytes > 0.8` for PVCs in sigma1. Severity: warning. For: 10m.
4. Configure alert routing: critical → PagerDuty/Slack immediate, warning → Slack channel.
5. Add labels for routing: `team: sigma1`, `environment: production`.
6. If a PrometheusRule CRD is available (kube-prometheus-stack), use it. Otherwise, create a ConfigMap with AlertManager rule files.
7. For token/cert expiry, consider a small CronJob that checks expiry dates and pushes a metric to Prometheus Pushgateway.

## Validation
Apply the PrometheusRule CR. Verify it is picked up by Prometheus (check Prometheus UI /rules). Scale a deployment to 0 replicas and verify the `ServiceDown` alert fires within 1 minute. Verify alert labels and annotations are correct. Check AlertManager UI to confirm alert routing configuration.