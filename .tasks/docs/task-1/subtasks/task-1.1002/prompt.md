Implement subtask 1002: Deploy CloudNative-PG PostgreSQL cluster with schemas

## Objective
Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage, and create the required schemas: rms, crm, finance, audit, public.

## Steps
1. Create a CloudNative-PG Cluster CR manifest:
   - name: sigma1-pg
   - namespace: databases
   - instances: 1 (single replica for dev)
   - storage: 50Gi
   - PostgreSQL version: 16
2. Apply the Cluster CR and wait for the pod to reach Running status.
3. Create an init SQL ConfigMap or use the bootstrap.initdb.postInitSQL field to run: CREATE SCHEMA IF NOT EXISTS rms; CREATE SCHEMA IF NOT EXISTS crm; CREATE SCHEMA IF NOT EXISTS finance; CREATE SCHEMA IF NOT EXISTS audit; (public schema exists by default).
4. Verify the superuser secret is created by CloudNative-PG (sigma1-pg-superuser).
5. Record the internal service DNS: sigma1-pg-rw.databases.svc.cluster.local:5432 for the ConfigMap.

## Validation
Verify pod sigma1-pg-1 is Running in databases namespace. Connect to PostgreSQL using the superuser secret and run `\dn` to confirm rms, crm, finance, audit, public schemas exist. Verify the service sigma1-pg-rw is resolvable within the cluster.