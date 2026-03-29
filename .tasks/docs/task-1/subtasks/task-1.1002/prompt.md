Implement subtask 1002: Deploy CloudNativePG Cluster CR for PostgreSQL

## Objective
Create the CloudNativePG `Cluster` CR named `notifycore-pg` with a single replica, database `notifycore`, user `notifycore_app`, and credentials stored in Secret `notifycore-pg-app`.

## Steps
1. Create `infra/notifycore/templates/postgres-cluster.yaml` with a CloudNativePG `Cluster` CR:
   - metadata.name: `notifycore-pg`, namespace: `notifycore`
   - spec.instances: 1
   - spec.bootstrap.initdb.database: `notifycore`
   - spec.bootstrap.initdb.owner: `notifycore_app`
   - spec.storage.size: `1Gi` (dev sizing from values-dev.yaml)
2. The CloudNativePG operator will auto-generate a Secret `notifycore-pg-app` containing the credentials.
3. Parameterize replica count and storage size through values-dev.yaml.
4. Verify the CR schema is correct for the target CloudNativePG operator version.

## Validation
`kubectl get cluster notifycore-pg -n notifycore` shows the cluster in healthy state. `kubectl get secret notifycore-pg-app -n notifycore` exists with `username` and `password` keys. Pod `notifycore-pg-1` is Running/Ready within 120s.