Implement subtask 1002: Deploy CloudNative-PG operator and PostgreSQL cluster

## Objective
Deploy the CloudNative-PG operator and provision a single-instance PostgreSQL cluster named 'sigma1-postgres' with a 'sigma1' database and 'sigma1_user' owner, 50Gi storage, within the 'databases' namespace.

## Steps
1. Install CloudNative-PG operator using Helm or kubectl apply.2. Create a `Cluster` custom resource for 'sigma1-postgres' in the 'databases' namespace, specifying 50Gi storage, single instance, 'sigma1' database, and 'sigma1_user'.3. Ensure the operator creates the necessary PVCs and Pods.

## Validation
1. Verify CloudNative-PG operator pods are running in the 'databases' namespace.2. Confirm 'sigma1-postgres' Cluster resource is in 'Ready' state.3. Connect to the 'sigma1' database using `psql` from within the cluster and verify 'sigma1_user' exists and can create schemas.