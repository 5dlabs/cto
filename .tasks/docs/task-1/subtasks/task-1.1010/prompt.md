Implement subtask 1010: Create hermes-infra-endpoints ConfigMap Template

## Objective
Create the Helm template for ConfigMap named hermes-infra-endpoints in each namespace containing all non-sensitive endpoint keys. All values are deterministic from Helm values (not runtime-derived), so this subtask depends only on the chart scaffold.

## Steps
Step-by-step:
1. Create `templates/configmap-endpoints.yaml` defining a ConfigMap resource.
2. `metadata.name`: `hermes-infra-endpoints`, `metadata.namespace`: `{{ .Values.namespace }}`.
3. `data` keys (all non-sensitive, derived from Helm values):
   - `CNPG_HERMES_URL`: `hermes-pg-rw.{{ .Values.namespace }}.svc:5432/hermes`
   - `REDIS_HERMES_URL`: `hermes-redis-master.{{ .Values.namespace }}.svc:6379`
   - `NATS_HERMES_URL`: `nats://hermes-nats.{{ .Values.namespace }}.svc:4222`
   - `MINIO_ENDPOINT`: `{{ .Values.minio.endpoint }}`
   - `MINIO_BUCKET`: `{{ .Values.minio.bucketName }}`
   - `MINIO_PRESIGN_EXPIRY`: `{{ .Values.minio.presignExpiry | default "3600" }}`
   - `ENVIRONMENT`: `{{ .Values.environment }}`
4. Apply standard labels.
5. Ensure NO credential/password/secret values are present.
6. Add comment: downstream workloads consume via `envFrom: [{configMapRef: {name: hermes-infra-endpoints}}]`.
7. Update NOTES.txt to list all ConfigMap keys and their descriptions.
8. Verify: `helm template` renders all 7 keys with correct values for both environments.

## Validation
`kubectl get configmap hermes-infra-endpoints -n hermes-staging -o json | jq '.data | keys'` returns exactly 7 keys: CNPG_HERMES_URL, REDIS_HERMES_URL, NATS_HERMES_URL, MINIO_ENDPOINT, MINIO_BUCKET, MINIO_PRESIGN_EXPIRY, ENVIRONMENT. All values non-empty. ENVIRONMENT=staging for staging, ENVIRONMENT=production for production. No values contain password/secret/key patterns. Same for production.