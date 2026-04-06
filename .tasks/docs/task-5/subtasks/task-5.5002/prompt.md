Implement subtask 5002: Define VettingResult and LeadScore data models and database migrations

## Objective
Create the VettingResult and LeadScore domain models as Rust structs with serde and sqlx derives, and write SQL migrations to create the corresponding PostgreSQL tables.

## Steps
1. In `models/vetting_result.rs`, define VettingResult struct with fields: id (UUID), org_id (UUID), business_verification (JSON/struct), online_presence (JSON/struct), reputation_analysis (JSON/struct), credit_signals (JSON/struct), overall_score (enum GREEN/YELLOW/RED), created_at, updated_at. 2. In `models/lead_score.rs`, define LeadScore struct with fields: id (UUID), org_id (UUID), score (f64), classification (GREEN/YELLOW/RED), component_scores (JSON), created_at. 3. Define the ScoreClassification enum (GREEN, YELLOW, RED) with sqlx::Type and serde derives. 4. Create sqlx migrations: `CREATE TABLE vetting_results (...)` and `CREATE TABLE lead_scores (...)` with proper indexes on org_id. 5. Add a migration for any enum types needed in PostgreSQL. 6. Implement From/Into conversions and validation methods on the models.

## Validation
Migrations run successfully against a test database; models serialize/deserialize correctly to/from JSON; sqlx compile-time checks pass; round-trip insert and select of VettingResult and LeadScore works.