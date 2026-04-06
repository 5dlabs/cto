Implement subtask 9001: Scale PostgreSQL to HA mode with CloudNative-PG

## Objective
Update the CloudNative-PG Cluster CR to run multiple instances (primary + replicas) with streaming replication, configure automatic failover, and validate data consistency across replicas.

## Steps
1. Edit the CloudNative-PG Cluster CR to set `instances: 3` (1 primary, 2 replicas). 2. Configure `postgresql.synchronous` settings for synchronous replication if data durability requires it, otherwise async. 3. Set resource requests/limits appropriate for production workloads. 4. Configure PodDisruptionBudgets (minAvailable: 2) to ensure quorum during rolling updates. 5. Set anti-affinity rules to spread instances across nodes. 6. Apply the updated CR and verify all replicas reach streaming state. 7. Test a manual switchover using `kubectl cnpg promote` to validate failover readiness.

## Validation
Verify 3 PostgreSQL pods are running and healthy. Confirm replication lag is near zero. Perform a manual switchover: promote a replica, verify the old primary becomes a replica, and confirm the application reconnects without errors. Run a data consistency check across instances.