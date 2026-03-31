Implement subtask 1006: Deploy CloudNative-PG Postgres Cluster CR per Namespace

## Objective
Deploy a CloudNative-PG Cluster custom resource in each namespace with 1 replica, database name 'hermes', and document the operator-generated Secret naming convention for downstream Secret wiring.

## Steps
Step-by-step:
1. Research the CNPG operator version installed in the cluster: `kubectl get deployment -n cnpg-system` and `kubectl get crd clusters.postgresql.cnpg.io -o jsonpath='{.spec.versions[*].name}'`. Use the matching API version.
2. Create `templates/cnpg-cluster.yaml` with a `Cluster` CR:
   - `apiVersion`: match installed CNPG version (e.g., `postgresql.cnpg.io/v1`)
   - `metadata.name`: `hermes-pg`
   - `metadata.namespace`: `{{ .Values.namespace }}`
   - `spec.instances`: 1
   - `spec.storage.size`: `{{ .Values.cnpg.storageSize | default "5Gi" }}`
   - `spec.bootstrap.initdb.database`: `hermes`
   - `spec.bootstrap.initdb.owner`: `hermes`
   - Apply standard labels.
3. Optionally gate backup config: `{{ if .Values.cnpg.backupEnabled }}` — configure `spec.backup` to write to MinIO. Default to disabled per decision point.
4. Document in README: CNPG operator auto-creates a Secret named `hermes-pg-app` containing keys `uri`, `username`, `password`, `host`, `port`, `dbname`. The downstream Secret subtask (1009) will reference this operator-managed Secret rather than duplicating credentials in Helm values.
5. Add a comment in the template noting the expected operator-generated Secret name.
6. Verify: `helm template --debug` renders valid Cluster CR for both environments.

## Validation
`kubectl get cluster hermes-pg -n hermes-staging -o jsonpath='{.status.phase}'` returns healthy status. `kubectl get secret hermes-pg-app -n hermes-staging` exists and contains keys: uri, username, password, host, port, dbname. A test pod can connect using the uri from the operator-generated Secret and execute `SELECT 1`. Same for production namespace.