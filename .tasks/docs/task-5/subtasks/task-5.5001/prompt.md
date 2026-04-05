Implement subtask 5001: Initialize Rust/Axum project with data models and PostgreSQL schema

## Objective
Set up the Rust 1.75+ Axum 0.7 project structure for the Customer Vetting Service, define VettingResult and LeadScore domain models, and create the PostgreSQL migration for storing vetting results.

## Steps
1. Create a new Rust project with `cargo init` using Rust 1.75+. Add dependencies: axum 0.7, tokio, serde, sqlx (with postgres feature), utoipa (for OpenAPI). 2. Define the `VettingResult` struct containing fields: org_id, business_verification (status, incorporation_date, registered_address, etc.), online_presence (linkedin_url, linkedin_followers, website_url), reputation (avg_google_rating, review_count, sentiment_summary), credit_score (provider, score, risk_level), overall_score (GREEN/YELLOW/RED enum), created_at, updated_at. 3. Define the `LeadScore` struct with composite scoring fields. 4. Create SQLx migrations for the `vetting_results` and `lead_scores` tables in PostgreSQL. 5. Set up the database connection pool using SQLx with config from environment variables (referencing the infra-endpoints ConfigMap). 6. Implement basic CRUD repository functions for vetting results. 7. Set up the Axum router skeleton with placeholder handlers for /api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id.

## Validation
Project compiles successfully; migrations run against a test PostgreSQL instance; CRUD operations on vetting_results table work (insert, select by org_id); Axum server starts and placeholder endpoints return 501 Not Implemented.