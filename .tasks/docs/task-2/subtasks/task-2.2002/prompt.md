Implement subtask 2002: Define data models and create database migrations

## Objective
Define Product, Category, and Availability domain models in Rust and create sqlx database migrations for the catalog schema tables in the rms PostgreSQL schema.

## Steps
1. Create a `models/` module with structs: Category (id, name, slug, description, parent_id, created_at, updated_at), Product (id, name, slug, description, category_id, daily_rate, weekly_rate, monthly_rate, image_urls: Vec<String>, specifications: serde_json::Value, created_at, updated_at), Availability (id, product_id, date, is_available, reserved_by, notes).
2. Derive Serialize, Deserialize, sqlx::FromRow on each struct.
3. Create sqlx migration files in migrations/ directory:
   - 001_create_categories.sql: CREATE TABLE rms.categories (...)
   - 002_create_products.sql: CREATE TABLE rms.products (...) with FK to categories
   - 003_create_availability.sql: CREATE TABLE rms.availability (...) with FK to products, index on (product_id, date)
4. Add seed data migration with sample categories (e.g., Excavators, Loaders, Compactors) and sample products.
5. Run migrations with `sqlx migrate run` and verify tables exist.
6. Create response DTOs (ProductResponse, CategoryResponse, AvailabilityResponse) for API serialization, including S3 image URL construction from stored image keys.

## Validation
Run `sqlx migrate run` successfully against the PostgreSQL instance; verify tables rms.categories, rms.products, rms.availability exist with correct columns and constraints; seed data is queryable; Rust structs compile and can be deserialized from DB rows.