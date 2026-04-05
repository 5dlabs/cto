Implement subtask 9001: Scale PostgreSQL to HA multi-replica mode with CloudNative-PG

## Objective
Update the CloudNative-PG Cluster CR to enable multi-replica HA with synchronous replication and automatic failover for the production PostgreSQL instance.

## Steps
1. Edit the CloudNative-PG Cluster CR YAML to set `instances: 3` (1 primary + 2 replicas). 2. Configure `postgresql.synchronous` settings for synchronous replication to at least one replica. 3. Ensure `failover.enabled: true` and set appropriate promotion criteria. 4. Configure anti-affinity rules (`topologyKey: kubernetes.io/hostname`) so replicas spread across nodes. 5. Update PVC storage class and size if needed for production workload. 6. Apply the updated CR and verify all three instances reach streaming replication state. 7. Validate that `kubectl cnpg status <cluster-name>` shows healthy primary and replicas.

## Validation
Verify `kubectl cnpg status` shows 3 healthy instances with streaming replication. Delete the primary pod and confirm automatic failover completes within 30 seconds with a new primary elected. Confirm application connections recover without manual intervention.