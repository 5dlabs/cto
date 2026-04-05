Implement subtask 2002: Define data models and create database migrations for catalog schema

## Objective
Define Rust structs for Product, Category, and Availability domain models, and create sqlx migrations to set up the corresponding tables in the public (or rms) PostgreSQL schema.

## Steps
1. Create a models module with structs: Category (id, name, slug, description, parent_id, image_url, created_at, updated_at), Product (id, category_id, name, slug, description, daily_rate, weekly_rate, monthly_rate, image_urls: Vec<String>, specs: JSONB, status: enum Active/Inactive, created_at, updated_at), Availability (id, product_id, date, is_available, reserved_by, reservation_id).
2. Derive sqlx::FromRow, Serialize, Deserialize on all structs.
3. Create sqlx migrations in migrations/ directory:
   - 001_create_categories.sql: CREATE TABLE categories with columns matching the struct.
   - 002_create_products.sql: CREATE TABLE products with FK to categories, GIN index on specs.
   - 003_create_availability.sql: CREATE TABLE availability with FK to products, unique constraint on (product_id, date).
   - Add indexes on slug columns and commonly queried fields.
4. Run migrations with `sqlx migrate run` and verify tables exist.

## Validation
Run `sqlx migrate run` against a test PostgreSQL instance; all migrations succeed. Verify tables categories, products, availability exist with correct columns and constraints. Insert and query test rows to confirm FK relationships work.