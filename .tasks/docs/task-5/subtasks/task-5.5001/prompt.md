Implement subtask 5001: Initialize Rust/Axum project with PostgreSQL connectivity and configuration

## Objective
Scaffold the Rust Axum 0.7 service, configure connection to PostgreSQL and load external API keys from the 'sigma1-infra-endpoints' ConfigMap via envFrom. Set up project structure with modules for models, handlers, services, and integrations.

## Steps
1. Create new Rust project with `cargo init`, add dependencies: axum 0.7, tokio, sqlx (with postgres feature), serde, serde_json, dotenvy, tracing, tracing-subscriber. 2. Set up main.rs with Axum router skeleton and tracing initialization. 3. Create a `config` module that reads DATABASE_URL, OPENCORPORATES_API_KEY, LINKEDIN_API_KEY, GOOGLE_REVIEWS_API_KEY, CREDIT_API_KEY from environment variables (sourced from sigma1-infra-endpoints ConfigMap). 4. Create a `db` module with sqlx::PgPool initialization and a health check query. 5. Create module stubs: `models/`, `handlers/`, `services/`, `integrations/`. 6. Add a GET /health endpoint returning 200 with pool status. 7. Create a Dockerfile for the service.

## Validation
Service starts and responds 200 on GET /health; PostgreSQL connection pool initializes successfully; all environment variables are read without panic; `cargo build` compiles without errors.