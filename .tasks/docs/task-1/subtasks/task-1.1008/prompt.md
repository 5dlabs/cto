Implement subtask 1008: Create sigma1-infra-endpoints ConfigMap

## Objective
Create the `sigma1-infra-endpoints` ConfigMap in the `sigma1` namespace aggregating all non-secret connection strings and endpoint URLs for consumption via `envFrom` by downstream services.

## Steps
1. Create ConfigMap `sigma1-infra-endpoints` in `sigma1` namespace with data:
   - `POSTGRES_URL=postgresql://sigma1_user:$(password_ref)@sigma1-postgres-pooler.sigma1-db.svc.cluster.local:5432/sigma1` (Note: actual password should come from secret; this ConfigMap holds the host/port/db template)
   - `VALKEY_URL=redis://sigma1-valkey.sigma1-db.svc.cluster.local:6379`
   - `R2_ENDPOINT=https://<account_id>.r2.cloudflarestorage.com`
   - `R2_BUCKET=sigma1-media`
   - `NATS_URL=nats://openclaw-nats.openclaw.svc.cluster.local:4222`
2. Decide: POSTGRES_URL in ConfigMap should be the template without password (password injected separately from secret) or the full URL. Recommendation: put host/port/db in ConfigMap, password in secret, let app construct URL.
3. Apply ConfigMap YAML.
4. Verify all 5 keys are present.

## Validation
`kubectl get configmap sigma1-infra-endpoints -n sigma1 -o json | jq '.data | keys'` returns exactly 5 keys: POSTGRES_URL, VALKEY_URL, R2_ENDPOINT, R2_BUCKET, NATS_URL. Each value is non-empty and contains the correct service DNS name.