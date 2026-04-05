Implement subtask 2001: Initialize Rust/Axum project with database connection and schema migrations

## Objective
Set up the Rust 1.75+ project with Axum 0.7, configure database connectivity from the sigma1-infra-endpoints ConfigMap, and create SQLx migrations for Product, Category, and Availability models.

## Steps
1. Create a new Rust project with `cargo init equipment-catalog`.
2. Add dependencies in Cargo.toml: axum 0.7, tokio, sqlx (with postgres feature), serde, serde_json, dotenvy (for local dev).
3. Set up the application entrypoint (main.rs) to read POSTGRES_URL, REDIS_URL, S3_URL from environment (injected via sigma1-infra-endpoints ConfigMap in K8s).
4. Configure a SQLx connection pool with appropriate pool size (max 10 for dev).
5. Create SQLx migrations:
   - `001_create_categories.sql`: categories table (id UUID PK, name VARCHAR, slug VARCHAR UNIQUE, description TEXT, parent_id UUID nullable FK, created_at, updated_at).
   - `002_create_products.sql`: products table (id UUID PK, name VARCHAR, sku VARCHAR UNIQUE, description TEXT, category_id UUID FK, daily_rate DECIMAL, weekly_rate DECIMAL, monthly_rate DECIMAL, image_key VARCHAR, specs JSONB, created_at, updated_at).
   - `003_create_availability.sql`: availability table (id UUID PK, product_id UUID FK, date DATE, available_quantity INT, total_quantity INT, UNIQUE(product_id, date)).
6. Run migrations with `sqlx migrate run`.
7. Define Rust structs for Product, Category, Availability with serde Serialize/Deserialize.
8. Create a shared AppState struct holding the PgPool, and wire it into the Axum router.

## Validation
Migrations run successfully against PostgreSQL. The application starts without errors, connects to the database, and the AppState is accessible in route handlers. Run `cargo build` and `cargo test` with a test database to verify schema creation.