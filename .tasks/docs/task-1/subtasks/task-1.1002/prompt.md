Implement subtask 1002: Deploy CloudNative-PG Postgres cluster CR and validate readiness

## Objective
Deploy a single-replica CloudNative-PG Cluster custom resource named `sigma-1-pg` in the `sigma-1-dev` namespace with 1Gi storage, and wait until the cluster reaches a healthy state.

## Steps
1. Create a CNPG Cluster CR manifest: name=`sigma-1-pg`, namespace=`sigma-1-dev`, instances=1, storage.size=1Gi, storage.storageClass=(use cluster default or specify).
2. Configure the bootstrap section for a fresh `initdb` with a database name (e.g., `sigma1`) and owner.
3. Apply the manifest: `kubectl apply -f sigma-1-pg-cluster.yaml`.
4. Wait for readiness: poll `kubectl get clusters.postgresql.cnpg.io sigma-1-pg -n sigma-1-dev -o jsonpath='{.status.phase}'` until it returns `Cluster in healthy state` (timeout 5 minutes).
5. Confirm the CNPG operator has created the app-credential secret (e.g., `sigma-1-pg-app`) containing `username`, `password`, `host`, `port`, `dbname`, `uri` keys.
6. Record the secret name for downstream ConfigMap creation.

## Validation
`kubectl get clusters.postgresql.cnpg.io sigma-1-pg -n sigma-1-dev -o jsonpath='{.status.phase}'` returns `Cluster in healthy state`. The pod `sigma-1-pg-1` is Running. The secret `sigma-1-pg-app` exists and contains non-empty `uri` key.