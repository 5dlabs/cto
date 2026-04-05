Implement subtask 1002: Deploy PostgreSQL 16 via CloudNative-PG with multi-schema setup

## Objective
Deploy a single-replica PostgreSQL 16 cluster using the CloudNative-PG operator in the databases namespace, and create schemas for rms, crm, finance, audit, and public.

## Steps
1. Ensure the CloudNative-PG operator is installed in the cluster (check CRDs: `kubectl get crds | grep cnpg`).
2. Create a `Cluster` CR YAML in the `databases` namespace specifying PostgreSQL 16, single replica, storage class, resource limits.
3. Configure the bootstrap section to run an initdb SQL script that creates schemas: rms, crm, finance, audit, public.
4. Set connection pooling parameters (e.g., PgBouncer sidecar or built-in pooler) appropriate for dev.
5. Apply the CR: `kubectl apply -f postgres-cluster.yaml`.
6. Wait for the cluster to become Ready: `kubectl -n databases get cluster`.
7. Verify schemas exist by exec-ing into the pod and running `\dn` in psql.
8. Record the resulting POSTGRES_URL (host, port, credentials) for ConfigMap creation.

## Validation
Confirm the CloudNative-PG Cluster CR shows status Ready with 1/1 replicas. Connect via psql and verify all 5 schemas (rms, crm, finance, audit, public) exist. Confirm the superuser and app credentials are stored in the expected Kubernetes secret.