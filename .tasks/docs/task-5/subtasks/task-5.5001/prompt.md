Implement subtask 5001: Initialize Rust/Axum project and database schema for vetting models

## Objective
Scaffold the Rust 1.75+ project with Axum 0.7, configure database connectivity using POSTGRES_URL from ConfigMap, and create SQLx migrations for VettingResult and LeadScore tables in the vetting schema.

## Steps
1. Create a new Rust project with `cargo init` and add dependencies: axum 0.7, sqlx (with postgres and runtime-tokio features), serde, serde_json, tokio, tower-http.
2. Set up configuration loading from environment variables (POSTGRES_URL, API keys for OpenCorporates, LinkedIn, Google, credit provider) using `envFrom` referencing the infra ConfigMap and secrets.
3. Define Rust structs for VettingResult (id, org_id, business_verification_data, online_presence_data, reputation_data, credit_data, overall_score enum GREEN/YELLOW/RED, created_at, updated_at) and LeadScore (id, org_id, vetting_result_id, dimension_scores JSONB, composite_score f64, classification enum).
4. Create SQLx migration files: create `vetting` schema, create `vetting_results` table, create `lead_scores` table with appropriate indexes on org_id and created_at.
5. Implement a database connection pool setup using sqlx::PgPool.
6. Set up the basic Axum app skeleton with a router and graceful shutdown.
7. Run migrations on startup or via a separate migration command.

## Validation
Verify the project compiles with `cargo build`. Run `sqlx migrate run` against a test PostgreSQL instance and confirm all tables are created. Verify the app starts and connects to the database without errors.