Implement subtask 5001: Initialize Rust/Axum project with PostgreSQL connectivity and database migrations

## Objective
Scaffold the Rust project with Axum 0.7, configure PostgreSQL connection using the infra ConfigMap (envFrom), set up SQLx or Diesel for database access, and create migrations for VettingResult and LeadScore tables.

## Steps
1. Create a new Rust binary crate with Cargo.toml specifying axum 0.7, tokio, sqlx (with postgres feature), serde, serde_json, uuid, chrono. 2. Configure database connection pool using DATABASE_URL from the infra ConfigMap environment variables. 3. Define the VettingResult model with fields: id (UUID), org_id (UUID), business_verification_status, online_presence_score, reputation_score, credit_score, composite_lead_score, raw_data (JSONB), created_at, updated_at. 4. Define the LeadScore model with fields: id (UUID), org_id (UUID), score (f64), breakdown (JSONB), scoring_version, created_at. 5. Write SQL migrations to create the vetting schema and both tables with appropriate indexes on org_id. 6. Add a health check endpoint GET /healthz that verifies database connectivity. 7. Set up the Axum router skeleton with a shared AppState holding the DB pool.

## Validation
Run migrations against a test PostgreSQL instance; verify tables are created with correct columns and indexes; health endpoint returns 200 with valid DB connection and 503 without.