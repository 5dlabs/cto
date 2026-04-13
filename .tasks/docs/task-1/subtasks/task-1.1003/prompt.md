<identity>
You are bolt working on subtask 1003 of task 1.
</identity>

<context>
<scope>
Deploy a Kubernetes Job that connects to sigma1-postgres and creates the six application schemas (catalog, rms, finance, vetting, social, audit) plus per-service PostgreSQL roles with scoped USAGE grants.
</scope>
</context>

<implementation_plan>
Create sigma1/infra/templates/schema-init-job.yaml. Job spec: namespace databases, initContainers: none, container uses image postgres:16-alpine. Command: psql $DATABASE_URL -f /sql/init.sql where init.sql is mounted via a ConfigMap (sigma1-schema-init-sql). SQL content: CREATE SCHEMA IF NOT EXISTS catalog; CREATE SCHEMA IF NOT EXISTS rms; CREATE SCHEMA IF NOT EXISTS finance; CREATE SCHEMA IF NOT EXISTS vetting; CREATE SCHEMA IF NOT EXISTS social; CREATE SCHEMA IF NOT EXISTS audit; CREATE ROLE IF NOT EXISTS sigma1_catalog; GRANT USAGE ON SCHEMA catalog TO sigma1_catalog; CREATE ROLE IF NOT EXISTS sigma1_rms; GRANT USAGE ON SCHEMA rms TO sigma1_rms; CREATE ROLE IF NOT EXISTS sigma1_finance; GRANT USAGE ON SCHEMA finance TO sigma1_finance; CREATE ROLE IF NOT EXISTS sigma1_vetting; GRANT USAGE ON SCHEMA vetting TO sigma1_vetting; CREATE ROLE IF NOT EXISTS sigma1_social; GRANT USAGE ON SCHEMA social TO sigma1_social; Set restartPolicy: OnFailure, backoffLimit: 3. DATABASE_URL sourced from CNPG generated secret sigma1-postgres-app. Add Job annotation helm.sh/hook: post-install,post-upgrade and helm.sh/hook-weight: '10' to ensure it runs after the cluster CR is applied.
</implementation_plan>

<validation>
kubectl get job schema-init -n databases shows Completions: 1/1. psql -U sigma1_user -d sigma1 -c '\dn' returns rows for catalog, rms, finance, vetting, social, audit. psql -U sigma1_user -d sigma1 -c '\du' shows roles sigma1_catalog, sigma1_rms, sigma1_finance, sigma1_vetting, sigma1_social.
</validation>