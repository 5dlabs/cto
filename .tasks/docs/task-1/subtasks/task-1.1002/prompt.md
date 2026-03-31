Implement subtask 1002: Deploy CloudNative-PG PostgreSQL Cluster CRs and credential secrets

## Objective
Deploy single-replica CNPG Cluster custom resources in both hermes-dev and hermes-staging namespaces with initdb configuration and credential secrets.

## Steps
1. Create a Helm template for the CNPG `Cluster` CR in `charts/hermes-infra/templates/cnpg-cluster.yaml`.
2. Configure single-replica (1 instance) for both dev and staging.
3. Set `initdb` to create a `hermes` database with appropriate encoding (UTF-8).
4. The CNPG operator auto-generates secrets; ensure the generated secret is named or aliased to `hermes-pg-credentials` in each namespace.
5. Verify the secret contains keys: `host`, `port`, `dbname`, `username`, `password`, or a composite `uri` key.
6. Add values for storage class and size in `values-dev.yaml` (1Gi) and `values-staging.yaml` (5Gi).

## Validation
`kubectl get clusters.postgresql.cnpg.io -n hermes-dev` shows a Ready cluster with 1 replica. `kubectl get secret hermes-pg-credentials -n hermes-dev` exists and contains valid PostgreSQL connection parameters.