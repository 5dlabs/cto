Implement subtask 1002: Deploy PostgreSQL 16 via CloudNative-PG operator with multi-schema setup

## Objective
Deploy a single-replica PostgreSQL 16 cluster in the databases namespace using the CloudNative-PG operator, then create the required schemas (rms, crm, finance, audit, public) and initial roles for each downstream service.

## Steps
1. Ensure the CloudNative-PG operator is installed (reference existing operator or add to Helm dependencies).
2. Author a CloudNative-PG Cluster CR in infra/postgres/cluster.yaml: single replica, PostgreSQL 16, storage class per cluster default, resource requests (1 CPU / 2Gi RAM), in namespace 'databases'.
3. Create a post-init SQL ConfigMap containing: CREATE SCHEMA IF NOT EXISTS rms; CREATE SCHEMA IF NOT EXISTS crm; CREATE SCHEMA IF NOT EXISTS finance; CREATE SCHEMA IF NOT EXISTS audit; and grants for dedicated roles (rms_user, crm_user, finance_user, audit_user).
4. Configure the Cluster CR's bootstrap.initdb.postInitSQL to reference this ConfigMap.
5. Create Kubernetes Secrets for each role's credentials (auto-generated passwords) so downstream services can consume them.
6. Verify the cluster reaches 'Cluster in healthy state' status.

## Validation
kubectl get cluster -n databases shows status 'Cluster in healthy state'; connect via psql from a test pod and run '\dn' to confirm schemas rms, crm, finance, audit exist; verify each role can connect and access only its schema.