Implement subtask 1006: Create Kubernetes Secrets for database credentials

## Objective
Create the `sigma1-db-credentials` Secret in the `sigma1` namespace containing PostgreSQL connection strings for each per-service role (catalog_svc, rms_svc, finance_svc, vetting_svc, social_svc) and the sigma1_user superuser.

## Steps
1. Create an Opaque Secret `sigma1-db-credentials` in `sigma1` namespace with data keys:
   - `POSTGRES_URL` (sigma1_user connection via pooler)
   - `CATALOG_SVC_POSTGRES_URL` (catalog_svc role connection via pooler)
   - `RMS_SVC_POSTGRES_URL` (rms_svc role connection via pooler)
   - `FINANCE_SVC_POSTGRES_URL` (finance_svc role connection via pooler)
   - `VETTING_SVC_POSTGRES_URL` (vetting_svc role connection via pooler)
   - `SOCIAL_SVC_POSTGRES_URL` (social_svc role connection via pooler)
2. All URLs should point to the PgBouncer pooler endpoint: `sigma1-postgres-pooler.sigma1-db.svc.cluster.local:5432`.
3. Passwords must match those set in the init SQL step (1003).
4. Apply the Secret YAML.

## Validation
`kubectl get secret sigma1-db-credentials -n sigma1 -o json | jq '.data | keys'` lists all 6 expected keys. Each decoded URL is a valid PostgreSQL connection string. Test at least one role connection from a pod using the secret value.