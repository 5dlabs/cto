Implement subtask 1003: Create per-service PostgreSQL roles and schemas via init SQL

## Objective
Run init SQL against the sigma1-postgres cluster to create 6 schemas (catalog, rms, finance, vetting, social, audit) and 5 per-service roles (catalog_svc, rms_svc, finance_svc, vetting_svc, social_svc) with schema-level GRANTs.

## Steps
1. Create a Kubernetes Job or use CloudNative-PG `spec.bootstrap.initdb.postInitSQL` / `postInitApplicationSQL` to run SQL.
2. SQL script must:
   - CREATE SCHEMA catalog, rms, finance, vetting, social, audit;
   - CREATE ROLE catalog_svc LOGIN PASSWORD '<generated>';
   - CREATE ROLE rms_svc LOGIN PASSWORD '<generated>';
   - CREATE ROLE finance_svc LOGIN PASSWORD '<generated>';
   - CREATE ROLE vetting_svc LOGIN PASSWORD '<generated>';
   - CREATE ROLE social_svc LOGIN PASSWORD '<generated>';
   - GRANT USAGE, CREATE ON SCHEMA catalog TO catalog_svc;
   - GRANT USAGE, CREATE ON SCHEMA rms TO rms_svc;
   - GRANT USAGE, CREATE ON SCHEMA finance TO finance_svc;
   - GRANT USAGE, CREATE ON SCHEMA vetting TO vetting_svc;
   - GRANT USAGE, CREATE ON SCHEMA social TO social_svc;
   - GRANT USAGE ON SCHEMA audit TO catalog_svc, rms_svc, finance_svc, vetting_svc, social_svc;
   - ALTER DEFAULT PRIVILEGES for each role in their respective schemas.
3. If using a Job, store the SQL in a ConfigMap and mount it.
4. Store generated passwords in a temporary secret or output them for use in step 1005.

## Validation
Connect to sigma1 database and run `\dn` to list all 6 schemas. Run `\du` to list all 5 service roles. Test each role can connect and create a table in its own schema but NOT in other schemas.