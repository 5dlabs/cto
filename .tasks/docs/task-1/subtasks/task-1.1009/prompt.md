Implement subtask 1009: Create hermes-infra-secrets Secret Template Referencing Operator-Managed and Helm-Managed Credentials

## Objective
Create the Helm template for Kubernetes Secret named hermes-infra-secrets in each namespace. For Postgres, reference the CNPG operator-generated Secret (hermes-pg-app) via an init container that extracts the connection URI rather than duplicating credentials in Helm values. For MinIO, Redis, and NATS, source from Helm values.

## Steps
Step-by-step:
1. Create `templates/secrets.yaml` defining a Secret resource.
2. `metadata.name`: `hermes-infra-secrets`, `metadata.namespace`: `{{ .Values.namespace }}`, `type: Opaque`.
3. For POSTGRES_URL: Use CNPG's operator-generated Secret. Two approaches (choose based on CNPG version):
   a. **Preferred**: Reference the `uri` key from the `hermes-pg-app` Secret directly. Create a helper template or document that downstream workloads should mount both `hermes-infra-secrets` and the CNPG Secret, OR:
   b. **Alternative**: Use a Helm lookup function `{{ (lookup "v1" "Secret" .Values.namespace "hermes-pg-app").data.uri | b64dec }}` to copy the URI at install time. Note: lookup fails on `helm template` (dry-run) — gate with `{{ if .Release.IsInstall }}`.
   c. Document the chosen approach clearly in the template comments.
4. For other keys, use `stringData`:
   - `MINIO_ACCESS_KEY_ID`: `{{ .Values.minio.accessKeyId }}`
   - `MINIO_SECRET_ACCESS_KEY`: `{{ .Values.minio.secretAccessKey }}`
   - `REDIS_URL`: `redis://:{{ .Values.redis.auth.password }}@hermes-redis-master.{{ .Values.namespace }}.svc:6379`
   - `NATS_URL`: `nats://hermes-nats.{{ .Values.namespace }}.svc:4222`
5. Apply standard labels.
6. Add template comment: "NOTE: Native Secrets do not auto-rotate. ESO migration is deferred — see decision_points."
7. Ensure NO credential values appear in the ConfigMap template.
8. If approach (a) is chosen for Postgres, add `POSTGRES_SECRET_REF: hermes-pg-app` as a non-sensitive key so downstream consumers know which Secret to mount.

## Validation
`kubectl get secret hermes-infra-secrets -n hermes-staging -o json | jq '.data | keys'` contains MINIO_ACCESS_KEY_ID, MINIO_SECRET_ACCESS_KEY, REDIS_URL, NATS_URL (and POSTGRES_URL or POSTGRES_SECRET_REF depending on approach). Each key's base64-decoded value is non-empty. No secret values appear in hermes-infra-endpoints ConfigMap. Same for production.