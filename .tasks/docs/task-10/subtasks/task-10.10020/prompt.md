Implement subtask 10020: Monitoring: configure Prometheus AlertManager rules

## Objective
Create PrometheusRule CRs for all alerting conditions: service down, error rate, PostgreSQL replication lag, Signal-CLI restarts, Valkey memory, certificate expiry.

## Steps
Step-by-step:
1. Create `k8s/monitoring/alerting-rules.yaml` as a PrometheusRule CR:
   ```yaml
   apiVersion: monitoring.coreos.com/v1
   kind: PrometheusRule
   metadata:
     name: sigma1-alerts
     namespace: monitoring
     labels:
       release: prometheus  # or whatever label the Prometheus Operator selects
   spec:
     groups:
       - name: sigma1.rules
         rules:
           - alert: ServiceDown
             expr: up{namespace="sigma1"} == 0
             for: 2m
             labels: { severity: critical }
             annotations: { summary: "{{ $labels.job }} is down", description: "Service {{ $labels.job }} has been down for more than 2 minutes." }
           - alert: HighErrorRate
             expr: sum(rate(http_requests_total{namespace="sigma1",status=~"5.."}[5m])) by (job) / sum(rate(http_requests_total{namespace="sigma1"}[5m])) by (job) > 0.05
             for: 5m
             labels: { severity: warning }
           - alert: PostgreSQLReplicationLag
             expr: cnpg_pg_replication_lag > 30
             for: 1m
             labels: { severity: critical }
           - alert: SignalCLIFrequentRestarts
             expr: increase(kube_pod_container_status_restarts_total{namespace="openclaw",container="signal-cli"}[10m]) > 3
             for: 0m
             labels: { severity: warning }
           - alert: ValkeyHighMemory
             expr: valkey_memory_used_bytes / valkey_memory_max_bytes > 0.8
             for: 5m
             labels: { severity: warning }
           - alert: CertificateExpirySoon
             expr: certmanager_certificate_expiration_timestamp_seconds - time() < 14 * 24 * 3600
             for: 0m
             labels: { severity: warning }
   ```
2. Configure AlertManager receivers (create a separate AlertManagerConfig or update the existing config):
   - Route critical alerts to the chosen notification channel
   - Route warning alerts to a less urgent channel
3. Apply the PrometheusRule CR.

## Validation
Apply the PrometheusRule. In Prometheus UI, navigate to Alerts and verify all 6 rules appear and are in 'inactive' state (green). Scale equipment-catalog to 0 replicas, wait 2 minutes, verify ServiceDown alert fires (status changes to 'firing' in Prometheus UI). Scale back to 2 and verify alert resolves. Check AlertManager UI to confirm the alert was received and routed.