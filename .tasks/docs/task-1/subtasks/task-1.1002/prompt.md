Implement subtask 1002: Deploy CloudNative-PG PostgreSQL cluster with schema initialization

## Objective
Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage and initialize all required schemas (rms, crm, finance, audit, public).

## Steps
1. Create a CloudNative-PG Cluster CR in the databases namespace: single instance, 50Gi PVC, PostgreSQL 16.
2. Configure the CR with superuser credentials stored in a Kubernetes Secret.
3. Create an initdb SQL script (via ConfigMap or bootstrap.initdb) that creates schemas: rms, crm, finance, audit, public.
4. Apply the Cluster CR and wait for the pod to reach Running and the cluster to report Ready.
5. Verify schema creation by connecting and running \dn.
6. Record the POSTGRES_URL connection string (host, port, dbname, credentials) for later ConfigMap aggregation.

## Validation
Verify the CloudNative-PG Cluster CR status is 'Cluster in healthy state'. Connect from a test pod using psql and confirm all five schemas exist. Verify the Secret containing credentials is present in the databases namespace.