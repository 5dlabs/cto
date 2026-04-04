Implement subtask 1004: Deploy PgBouncer Pooler CR for connection pooling

## Objective
Deploy the CloudNative-PG `Pooler` CR to front the sigma1-postgres cluster with PgBouncer in transaction pooling mode with a default pool size of 20.

## Steps
1. Create `Pooler` CR YAML named `sigma1-postgres-pooler` in `sigma1-db` namespace:
   - `spec.cluster.name: sigma1-postgres`
   - `spec.type: rw` (read-write pooler pointing to primary)
   - `spec.pgbouncer.poolMode: transaction`
   - `spec.pgbouncer.parameters.default_pool_size: '20'`
   - `spec.instances: 1` (single replica for dev)
2. Apply the Pooler CR.
3. Verify the pooler service is created: `sigma1-postgres-pooler.sigma1-db.svc.cluster.local:5432`.
4. Test connectivity through the pooler from a temporary pod in sigma1 namespace.

## Validation
PgBouncer pod is Running. `psql -h sigma1-postgres-pooler.sigma1-db.svc.cluster.local -U sigma1_user -d sigma1 -c 'SELECT 1'` succeeds from a pod in sigma1 namespace. `SHOW POOLS` via PgBouncer admin interface shows pool_mode=transaction.