Implement subtask 1002: Deploy CloudNative-PG PostgreSQL cluster with multi-schema setup

## Objective
Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage. Configure the database with five schemas: rms, crm, finance, audit, and public. Create appropriate roles and grant schema-level permissions.

## Steps
1. Ensure the CloudNative-PG operator CRD is installed in the cluster (check with `kubectl get crd clusters.postgresql.cnpg.io`).
2. Create a `Cluster` CR YAML in the `databases` namespace: name `sigma1-pg`, instances: 1, storage size: 50Gi, PostgreSQL version 16.
3. Define a bootstrap `initdb` section with a post-init SQL script that creates the five schemas: `CREATE SCHEMA IF NOT EXISTS rms; CREATE SCHEMA IF NOT EXISTS crm; CREATE SCHEMA IF NOT EXISTS finance; CREATE SCHEMA IF NOT EXISTS audit;` (public already exists).
4. Create application-level roles for each schema (e.g., `rms_app`, `crm_app`, `finance_app`, `audit_app`) with appropriate GRANT statements.
5. Apply the Cluster CR with `kubectl apply -f sigma1-pg-cluster.yaml`.
6. Wait for the cluster to reach `Cluster in healthy state` via `kubectl get cluster sigma1-pg -n databases`.
7. Extract the auto-generated connection secret name for use in the ConfigMap subtask.

## Validation
Verify the CNPG Cluster resource is in 'Cluster in healthy state'. Port-forward to the primary pod and connect with psql. Run `\dn` to confirm all five schemas (rms, crm, finance, audit, public) exist. Run `\du` to confirm application roles are created. Insert and select a test row in each schema to validate write access.