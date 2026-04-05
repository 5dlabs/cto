Implement subtask 1010: Deploy ServiceMonitor CRs for Prometheus scraping

## Objective
Create ServiceMonitor custom resources so Prometheus automatically discovers and scrapes metrics from all services deployed in the sigma1 namespace.

## Steps
1. Create a namespace-wide `ServiceMonitor` CR `sigma1-services-monitor.yaml`:
   ```yaml
   apiVersion: monitoring.coreos.com/v1
   kind: ServiceMonitor
   metadata:
     name: sigma1-services
     namespace: sigma1
     labels:
       release: prometheus  # must match Prometheus operator's serviceMonitorSelector
   spec:
     selector:
       matchLabels:
         monitoring: sigma1
     namespaceSelector:
       matchNames:
         - sigma1
     endpoints:
       - port: metrics
         interval: 30s
         path: /metrics
   ```
2. Create a dedicated ServiceMonitor for CNPG PostgreSQL metrics (CNPG exposes metrics on its own port):
   ```yaml
   metadata:
     name: sigma1-postgres-monitor
   spec:
     selector:
       matchLabels:
         cnpg.io/cluster: sigma1-postgres
     endpoints:
       - port: metrics
         interval: 30s
   ```
3. Create a ServiceMonitor for Valkey exporter metrics (if redis-exporter sidecar is enabled in the Opstree CR).
4. Apply all ServiceMonitor manifests.
5. Ensure the label selector on ServiceMonitors matches the Prometheus operator's `serviceMonitorSelector` in the cluster.

## Validation
`kubectl get servicemonitors -n sigma1` lists all created ServiceMonitor CRs. Prometheus targets page (or `kubectl port-forward svc/prometheus 9090` and check `/targets`) shows sigma1 namespace targets being scraped. CNPG metrics endpoint returns PostgreSQL metrics.