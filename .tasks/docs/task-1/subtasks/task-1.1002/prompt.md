Implement subtask 1002: Deploy CloudNative-PG PostgreSQL cluster with multi-schema setup

## Objective
Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage, and initialize the required schemas: rms, crm, finance, audit, and public.

## Steps
1. Write a CloudNative-PG Cluster CR YAML: single instance, 50Gi PVC, PostgreSQL 16+, in 'databases' namespace. 2. Configure the CR with an initdb section or a bootstrap SQL ConfigMap that creates schemas: rms, crm, finance, audit, public. 3. Create a dedicated database user per schema or a shared superuser (depending on dp-5 resolution). 4. Set resource requests/limits appropriate for dev (e.g., 512Mi-1Gi RAM, 500m CPU). 5. Apply the CR and wait for the cluster to reach 'Running' phase. 6. Record the resulting connection string (host, port, credentials) for inclusion in the aggregated ConfigMap.

## Validation
CloudNative-PG cluster pod is Running; connect via psql and verify all five schemas exist; run a simple CREATE TABLE / INSERT / SELECT in each schema to confirm write access.