Implement subtask 10001: Scale CloudNative-PG cluster to 3 replicas with automated failover

## Objective
Update the CloudNative-PG Cluster CR `sigma-1-pg` to run 3 instances (1 primary, 2 read replicas) with automated failover enabled. Validate that all replicas reach ready state and streaming replication is active.

## Steps
1. Edit the `sigma-1-pg` Cluster CR YAML: set `spec.instances: 3`.
2. Ensure `spec.postgresql.pg_hba` allows replication connections between pods.
3. Verify `spec.failoverDelay` or equivalent failover settings are configured for fast promotion (target <30s).
4. Apply the updated CR: `kubectl apply -f cnpg-cluster.yaml -n sigma-1-dev`.
5. Wait for all 3 pods to reach Running/Ready: `kubectl get pods -l cnpg.io/cluster=sigma-1-pg -n sigma-1-dev`.
6. Confirm `status.instances == 3` and `status.readyInstances == 3` on the Cluster resource.
7. Verify streaming replication: connect to a replica and run `SELECT pg_is_in_recovery();` — should return `true`.

## Validation
Run `kubectl get cluster sigma-1-pg -n sigma-1-dev -o jsonpath='{.status.readyInstances}'` and assert output is `3`. Delete the primary pod with `kubectl delete pod sigma-1-pg-1 -n sigma-1-dev` and verify a new primary is elected within 30 seconds by checking `status.currentPrimary` changes and all 3 instances return to ready state.