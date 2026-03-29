Implement subtask 2002: Define data models and implement database migrations

## Objective
Define `Product`, `Category`, and `Availability` data models using `sqlx` and create initial database migration scripts for PostgreSQL.

## Steps
1. Add `sqlx` and `chrono` dependencies to `Cargo.toml`.2. Define Rust structs for `Product`, `Category`, and `Availability` with `sqlx::FromRow` and `serde` derives.3. Create `sqlx-cli` migrations for `products`, `categories`, and `availability` tables, including necessary indexes.

## Validation
1. Run `sqlx migrate run` against a local PostgreSQL instance and verify tables are created correctly.2. Write unit tests for data model serialization/deserialization.