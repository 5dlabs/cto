Implement subtask 5001: Scaffold Rust/Axum project with sqlx, models, and database migrations

## Objective
Initialize the Customer Vetting Rust project with Axum 0.7 and sqlx. Define VettingResult and LeadScore domain models as per PRD. Create PostgreSQL migrations for vetting_results and lead_scores tables. Connect to infra via envFrom: sigma1-infra-endpoints ConfigMap.

## Steps
1. `cargo init customer-vetting` with workspace layout.
2. Add dependencies: axum 0.7, sqlx (with postgres and runtime-tokio features), serde, serde_json, tokio, tower-http.
3. Define `VettingResult` struct with fields: id (UUID), org_id, opencorporates_data (JSONB), linkedin_data (JSONB), google_reviews_data (JSONB), credit_data (JSONB), overall_score (enum GREEN/YELLOW/RED), created_at, updated_at.
4. Define `LeadScore` struct with fields: id, org_id, dimension scores (financial_score, reputation_score, legal_score), composite_score, rating (GREEN/YELLOW/RED).
5. Write sqlx migrations in `migrations/` directory for both tables with appropriate indexes on org_id.
6. Configure database connection pool using DATABASE_URL from sigma1-infra-endpoints ConfigMap.
7. Set up Axum router skeleton with placeholder routes.
8. Add health endpoint GET /healthz that checks DB connectivity.

## Validation
Project compiles without errors; `sqlx migrate run` succeeds against a test PostgreSQL instance; health endpoint returns 200 with DB connection confirmed; models serialize/deserialize correctly via unit tests.