Implement subtask 1002: Deploy CloudNative-PG PostgreSQL 16 Cluster CR (sigma1-postgres)

## Objective
Define and apply the CloudNative-PG Cluster custom resource for sigma1-postgres in the databases namespace with PostgreSQL 16, 1 replica (dev), 50Gi storage, and required extensions.

## Steps
Create sigma1/infra/templates/cnpg-cluster.yaml. Spec: apiVersion: postgresql.cnpg.io/v1, kind: Cluster, metadata.name: sigma1-postgres, metadata.namespace: databases. Set spec.instances: 1, spec.postgresql.version: '16', spec.storage.size: 50Gi. Under spec.bootstrap.initdb: set database: sigma1, owner: sigma1_user, postInitSQL list with: CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; CREATE EXTENSION IF NOT EXISTS pgcrypto; CREATE EXTENSION IF NOT EXISTS pg_trgm;. Set spec.affinity or tolerations as appropriate for dev cluster. Ensure the CNPG operator is installed in the cluster (document as a prerequisite). Apply via Helm. Wait for cluster to reach Ready phase: kubectl wait cluster/sigma1-postgres -n databases --for=condition=Ready --timeout=300s.

## Validation
kubectl get cluster sigma1-postgres -n databases -o jsonpath='{.status.phase}' returns Ready. kubectl get pods -n databases shows sigma1-postgres-1 in Running state. psql -U sigma1_user -d sigma1 -c 'SELECT extname FROM pg_extension;' lists uuid-ossp, pgcrypto, pg_trgm.