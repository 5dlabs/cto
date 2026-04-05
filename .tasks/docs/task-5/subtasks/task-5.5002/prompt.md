Implement subtask 5002: Define data models and create PostgreSQL migrations for VettingResult and LeadScore

## Objective
Design and implement the VettingResult and LeadScore domain models as Rust structs with serde and sqlx derives. Create sqlx migrations for the vetting-related tables in the appropriate schema (e.g., crm schema). Include fields for business verification status, reputation scores, credit signals, composite risk score, and timestamps.

## Steps
1. Create src/models/mod.rs, src/models/vetting_result.rs, src/models/lead_score.rs.
2. VettingResult struct: id (UUID), org_id (UUID), business_name, registration_status, incorporation_date, linkedin_presence_score (f64), google_reviews_score (f64), google_reviews_count (i32), credit_score (Option<f64>), credit_risk_level (enum: Low/Medium/High/Unknown), composite_risk_score (f64), vetting_status (enum: Pending/Complete/Failed), created_at, updated_at.
3. LeadScore struct: id (UUID), org_id (UUID), vetting_result_id (UUID FK), qualification_score (f64), qualification_tier (enum: Hot/Warm/Cold/Disqualified), scoring_factors (JSONB), created_at.
4. Create migration: migrations/YYYYMMDD_create_vetting_tables.sql with CREATE TABLE statements in crm schema.
5. Add appropriate indexes on org_id and vetting_status.
6. Implement FromRow derives for sqlx queries.

## Validation
Migrations run successfully against a test PostgreSQL instance; sqlx compile-time checks pass; models can be round-tripped (insert and select) in an integration test.