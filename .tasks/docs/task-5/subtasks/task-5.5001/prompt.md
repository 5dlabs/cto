Implement subtask 5001: Initialize Rust project, Axum, and define vetting data models

## Objective
Set up a new Rust project for the Customer Vetting Service, configure Axum, and define `VettingResult` and `LeadScore` data models with `sqlx` migrations.

## Steps
1. Run `cargo new --bin customer-vetting` targeting Rust 1.77.2.2. Add `axum`, `tokio`, `sqlx`, `serde` dependencies.3. Define Rust structs for `VettingResult` and `LeadScore`.4. Create `sqlx-cli` migrations for `vetting_results` table.

## Validation
1. Run `cargo run` to verify basic server starts.2. Run `sqlx migrate run` against a local PostgreSQL instance and verify table creation.3. Write unit tests for data model serialization/deserialization.