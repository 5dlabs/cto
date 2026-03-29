Implement subtask 4001: Initialize Rust project, Axum, and define finance data models

## Objective
Set up a new Rust project for the Finance Service, configure Axum, and define `Invoice`, `Payment`, `InvoiceStatus` data models with `sqlx` migrations.

## Steps
1. Run `cargo new --bin finance-service` targeting Rust 1.77.2.2. Add `axum`, `tokio`, `sqlx`, `chrono`, `serde` dependencies.3. Define Rust structs for `Invoice`, `Payment`, `InvoiceStatus`.4. Create `sqlx-cli` migrations for `invoices` and `payments` tables.

## Validation
1. Run `cargo run` to verify basic server starts.2. Run `sqlx migrate run` against a local PostgreSQL instance and verify table creation.3. Write unit tests for data model serialization/deserialization.