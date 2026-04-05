Implement subtask 1011: Create Prometheus ServiceMonitor CRs for sigma1 services

## Objective
Create Prometheus ServiceMonitor custom resources that auto-discover and scrape metrics from all sigma1 services using the common label selector.

## Steps
1. Create a ServiceMonitor CR `sigma1-services-monitor` in `sigma1` namespace:
   - `spec.selector.matchLabels: { app.kubernetes.io/part-of: sigma1 }`
   - `spec.namespaceSelector.matchNames: [sigma1]`
   - `spec.endpoints[0].port: metrics` (assuming services expose a port named 'metrics')
   - `spec.endpoints[0].interval: 30s`
   - `spec.endpoints[0].path: /metrics`
2. Create a separate ServiceMonitor for the database namespace if needed: `sigma1-db-monitor` in `sigma1-db` namespace for CloudNative-PG and Valkey metrics.
3. Ensure ServiceMonitor labels match the Prometheus operator's `serviceMonitorSelector` (check existing Prometheus CR for required labels).
4. Apply ServiceMonitor YAMLs.

## Validation
`kubectl get servicemonitor -n sigma1` lists the sigma1-services-monitor CR. Prometheus targets page (via port-forward to Prometheus UI) shows sigma1 targets being discovered (may show 0 active targets until services are deployed, but the ServiceMonitor itself must be listed). `kubectl get servicemonitor -n sigma1-db` lists db monitor if created.