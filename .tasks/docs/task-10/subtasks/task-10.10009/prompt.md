Implement subtask 10009: Configure external-secrets operator refresh interval and rotation alerts

## Objective
Set the external-secrets operator refresh interval to 24 hours for all ExternalSecret resources in sigma-1-secrets and add annotations for rotation alerting.

## Steps
1. Edit all `ExternalSecret` CRs in the `sigma-1-secrets` namespace (or the Helm values that generate them).
2. Set `spec.refreshInterval: 24h` on each ExternalSecret resource.
3. Add annotations for monitoring/alerting:
   - `sigma.io/secret-rotation-alert: "true"`
   - `sigma.io/secret-last-rotated: "<timestamp>"` (to be updated by the rotation process).
4. If using a monitoring stack (Prometheus/Grafana), create an alert rule that fires if `externalsecret_sync_status` is not 'SecretSynced' for more than 25 hours (indicating a failed refresh).
5. Apply updated manifests: `kubectl apply -f manifests/production/external-secrets/`.
6. Verify refresh interval with `kubectl get externalsecret -n sigma-1-secrets -o jsonpath='{.items[*].spec.refreshInterval}'`.

## Validation
`kubectl get externalsecret -n sigma-1-secrets -o jsonpath='{.items[*].spec.refreshInterval}'` returns '24h' for all items. `kubectl get externalsecret -n sigma-1-secrets -o jsonpath='{.items[*].metadata.annotations}'` includes the rotation alert annotation on each resource.