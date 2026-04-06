Implement subtask 1002: Deploy CloudNative-PG PostgreSQL cluster with schema initialization

## Objective
Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage and initialize schemas: rms, crm, finance, audit, public.

## Steps
1. Write a CloudNative-PG Cluster CR manifest: single instance, 50Gi PVC, PostgreSQL 16, deployed to the 'databases' namespace.
2. Configure the CR with a bootstrap initdb section that creates the required schemas (rms, crm, finance, audit, public) via a SQL ConfigMap or inline SQL.
3. Set resource requests/limits appropriate for dev (e.g., 512Mi-1Gi RAM, 250m-500m CPU).
4. Create a Kubernetes Secret for the superuser and app-user credentials.
5. Apply the CR and wait for the cluster to reach 'Cluster in healthy state'.
6. Verify schemas exist by connecting with `psql` and running `\dn`.
7. Record the connection string (host, port, dbname, user) for inclusion in the ConfigMap.

## Validation
Confirm the CloudNative-PG cluster pod is Running and Ready; connect via psql from a test pod and verify all 5 schemas (rms, crm, finance, audit, public) exist; confirm the superuser and app-user secrets are populated.