Implement subtask 3001: Upgrade CloudNativePG to 3-replica HA with synchronous replication and backups

## Objective
Update the CloudNativePG Cluster CR for production: 3 replicas with synchronous replication (minSyncReplicas: 1), PodDisruptionBudget (maxUnavailable: 1), and automated backup configuration.

## Steps
1. In `infra/notifycore/templates/postgres-cluster.yaml`, add conditional logic based on values:
   - `spec.instances`: parameterize from values (3 for prod).
   - `spec.minSyncReplicas: 1` (ensures at least one synchronous standby).
   - `spec.maxSyncReplicas: 1`.
2. Create `infra/notifycore/templates/postgres-pdb.yaml`:
   - PodDisruptionBudget targeting the CloudNativePG pods (label selector matching `cnpg.io/cluster: notifycore-pg`).
   - `spec.maxUnavailable: 1`.
3. Add backup configuration to the Cluster CR:
   - `spec.backup.barmanObjectStore` section (parameterized: object store endpoint, bucket, credentials secret) OR `spec.backup.volumeSnapshot` for PVC-based.
   - Configure `spec.backup.retentionPolicy: "30d"`.
   - Create a `ScheduledBackup` CR for daily backups at 2 AM.
4. Update `values-prod.yaml` with postgres.instances: 3, postgres.backup settings.
5. Ensure `values-dev.yaml` remains unchanged (instances: 1, no backups).

## Validation
`helm template infra/notifycore -f infra/notifycore/values-prod.yaml` renders a CloudNativePG Cluster with instances=3, minSyncReplicas=1. A PodDisruptionBudget resource is rendered with maxUnavailable=1. Backup configuration section is present in the Cluster CR. `values-dev.yaml` still renders with instances=1 and no backup section.