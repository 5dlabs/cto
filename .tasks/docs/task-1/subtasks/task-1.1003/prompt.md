Implement subtask 1003: Author CNPG post-init SQL for schemas and scoped users

## Objective
Write the post-init SQL that creates all 7 schemas, 6+ per-service users, and fine-grained GRANT restrictions preventing cross-schema JOINs. Integrate this SQL into the CNPG Cluster CR's postInitApplicationSQL or as a ConfigMap-mounted script.

## Steps
1. Create a SQL script `post-init.sql` (or embed in the Cluster CR's `spec.bootstrap.initdb.postInitApplicationSQL`) that:
   a. Creates schemas: `CREATE SCHEMA IF NOT EXISTS rms; CREATE SCHEMA IF NOT EXISTS crm; CREATE SCHEMA IF NOT EXISTS finance; CREATE SCHEMA IF NOT EXISTS vetting; CREATE SCHEMA IF NOT EXISTS social; CREATE SCHEMA IF NOT EXISTS audit;` (public already exists).
   b. Creates per-service roles/users:
      - `CREATE USER sigma1_catalog WITH PASSWORD '...'; GRANT USAGE ON SCHEMA public TO sigma1_catalog; GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO sigma1_catalog; ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO sigma1_catalog;`
      - `CREATE USER sigma1_rms WITH PASSWORD '...'; GRANT USAGE ON SCHEMA rms TO sigma1_rms; GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA rms TO sigma1_rms; ALTER DEFAULT PRIVILEGES IN SCHEMA rms GRANT ALL ON TABLES TO sigma1_rms;`
      - Repeat pattern for `sigma1_finance` → `finance`, `sigma1_vetting` → `vetting`, `sigma1_social` → `social`, `sigma1_audit` → `audit`.
   c. For the audit schema, grant INSERT-only to all service users: `GRANT USAGE ON SCHEMA audit TO sigma1_catalog, sigma1_rms, sigma1_finance, sigma1_vetting, sigma1_social; GRANT INSERT ON ALL TABLES IN SCHEMA audit TO sigma1_catalog, sigma1_rms, sigma1_finance, sigma1_vetting, sigma1_social; ALTER DEFAULT PRIVILEGES IN SCHEMA audit GRANT INSERT ON TABLES TO sigma1_catalog, sigma1_rms, sigma1_finance, sigma1_vetting, sigma1_social;`
   d. Revoke default public schema access: `REVOKE ALL ON SCHEMA public FROM PUBLIC;` then selectively grant back to sigma1_catalog.
   e. Explicitly revoke cross-schema access: each user should have NO grants on schemas other than their own (and audit INSERT). Set `search_path` per user to their own schema only.
2. Store user passwords in Kubernetes Secrets (one Secret per user, e.g., `sigma1-postgres-sigma1-catalog`, etc.) so they can be referenced in connection strings.
3. If using `postInitApplicationSQL`, embed the SQL directly in the Cluster CR. If too large, mount a ConfigMap as `postInitApplicationSQLRefs`.
4. Re-apply or update the CNPG Cluster CR to trigger the init SQL execution (note: postInit only runs on initial bootstrap; if cluster already exists, connect manually or recreate).

## Validation
For each of the 6 users: `psql -U sigma1_catalog -d sigma1 -c 'CREATE TABLE public.test_cat(id int); DROP TABLE public.test_cat;'` succeeds. `psql -U sigma1_catalog -d sigma1 -c 'CREATE TABLE rms.test_cat(id int);'` fails with permission denied. `psql -U sigma1_rms -d sigma1 -c 'SELECT * FROM public.test_cat;'` fails. All service users can `INSERT INTO audit.<table>` but cannot `SELECT` or `DELETE` from audit schema. Verify `search_path` for each user shows only their schema.