Implement subtask 1002: Deploy CloudNative-PG Cluster CR with initdb bootstrap

## Objective
Deploy the CloudNative-PG `Cluster` CR named `sigma1-postgres` in the `sigma1-db` namespace with 2 instances, 50Gi storage, and initdb bootstrap creating the `sigma1` database owned by `sigma1_user`.

## Steps
1. Create `Cluster` CR YAML for `sigma1-postgres` in `sigma1-db` namespace:
   - `spec.instances: 2`
   - `spec.storage.size: 50Gi`
   - `spec.bootstrap.initdb.database: sigma1`
   - `spec.bootstrap.initdb.owner: sigma1_user`
   - `spec.bootstrap.initdb.secret.name: sigma1-postgres-superuser` (auto-created by operator)
2. Apply the Cluster CR.
3. Wait for cluster to reach READY state with 2/2 instances healthy.
4. Verify the `sigma1` database exists and `sigma1_user` is the owner.

## Validation
`kubectl get cluster sigma1-postgres -n sigma1-db` shows READY with 2/2 instances. `kubectl exec` into primary pod and run `psql -U sigma1_user -d sigma1 -c '\l'` to confirm database exists.