Implement subtask 10003: Configure ExternalSecret resources with refreshInterval for automated secret rotation

## Objective
Update all ExternalSecret CRs in the sigma-1 namespace to include a `refreshInterval` (e.g., 1h) so that secrets are periodically re-synced from the external secret store, enabling automated rotation.

## Steps
Step-by-step:
1. Identify all ExternalSecret resources in the sigma-1 namespace (Linear token, NOUS_API_KEY, GitHub token, Discord webhook URL, etc.).
2. For each ExternalSecret CR, set `spec.refreshInterval: 1h` (or whatever the organization standard is).
3. Verify that the external-secrets operator is running and healthy in the cluster.
4. Apply the updated ExternalSecret manifests.
5. After applying, check that each ExternalSecret's status shows `lastSyncedTime` updating at the configured interval.
6. Ensure the corresponding Kubernetes Secret objects are updated when the source values change.

## Validation
Run `kubectl get externalsecret -n sigma-1 -o jsonpath='{range .items[*]}{.metadata.name}: {.spec.refreshInterval}{"\n"}{end}'` — all show `1h` (or configured interval). Check `kubectl get externalsecret -n sigma-1 -o jsonpath='{range .items[*]}{.metadata.name}: {.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}'` — all show `True`. Verify `lastSyncedTime` is within the last refresh interval.