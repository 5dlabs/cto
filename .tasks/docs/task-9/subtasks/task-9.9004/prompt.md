Implement subtask 9004: Configure NATS HA with 3-node JetStream cluster

## Objective
Configure the NATS operator CR for production with a 3-node cluster, JetStream enabled, and resource requests/limits.

## Steps
1. Create the production NATS CR in `hermes-production` namespace with `spec.size: 3` for a 3-node cluster.
2. Enable JetStream with appropriate storage configuration (file-based storage for durability).
3. Set resource requests/limits: `requests: {cpu: 250m, memory: 256Mi}`, `limits: {cpu: 500m, memory: 512Mi}` per node.
4. Configure JetStream stream replication factor of 3 for production streams.
5. Create a PodDisruptionBudget: `maxUnavailable: 1` for NATS pods.
6. Update `hermes-infra-endpoints` ConfigMap with the NATS production service URL.

## Validation
Verify `kubectl get pods -n hermes-production -l app=hermes-nats` (or equivalent) returns 3 Running NATS pods. Verify JetStream is healthy: `kubectl exec` into a NATS pod and run `nats server list` showing 3 nodes. Verify PDB exists for NATS.