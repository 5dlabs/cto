<identity>
You are rex working on subtask 2001 of task 2.
</identity>

<context>
<scope>
Bootstrap the services/equipment-catalog Cargo workspace with Cargo.toml, all required crate dependencies pinned to specified versions, and write the three sqlx migration files for the catalog schema tables.
</scope>
</context>

<implementation_plan>
Create services/equipment-catalog/Cargo.toml as a workspace with a single member 'equipment-catalog'. Create services/equipment-catalog/equipment-catalog/Cargo.toml with [dependencies]: axum = { version = '0.7', features = ['macros'] }, tokio = { version = '1', features = ['full'] }, sqlx = { version = '0.7', features = ['runtime-tokio', 'postgres', 'uuid', 'chrono', 'rust_decimal', 'json'] }, redis = { version = '0.24', features = ['tokio-comp', 'connection-manager'] }, aws-sdk-s3 = '1', tower = '0.4', tower-http = { version = '0.5', features = ['cors', 'trace', 'compression-gzip'] }, prometheus = '0.13', uuid = { version = '1', features = ['v4', 'serde'] }, serde = { version = '1', features = ['derive'] }, serde_json = '1', anyhow = '1', tracing = '0.1', tracing-subscriber = { version = '0.3', features = ['env-filter'] }, chrono = { version = '0.4', features = ['serde'] }, rust_decimal = { version = '1', features = ['serde'] }. Create migrations/ directory. Migration 0001_create_catalog_tables.sql: SET search_path TO catalog; CREATE TABLE IF NOT EXISTS categories (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), name TEXT NOT NULL, parent_id UUID REFERENCES categories(id), icon TEXT, sort_order INT NOT NULL DEFAULT 0); CREATE TABLE IF NOT EXISTS products (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), category_id UUID REFERENCES categories(id), name TEXT NOT NULL, description TEXT, day_rate NUMERIC(10,2) NOT NULL, weight_kg FLOAT4, dimensions JSONB, image_urls TEXT[], specs JSONB, barcode TEXT, stock_quantity INT NOT NULL DEFAULT 1, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now()); CREATE TABLE IF NOT EXISTS availability_blocks (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), product_id UUID NOT NULL REFERENCES products(id), date_from DATE NOT NULL, date_to DATE NOT NULL, quantity_reserved INT NOT NULL DEFAULT 0, quantity_booked INT NOT NULL DEFAULT 0); CREATE INDEX IF NOT EXISTS idx_availability_product_dates ON availability_blocks (product_id, date_from, date_to);. Run sqlx migrate run against dev database to verify.
</implementation_plan>

<validation>
cargo check completes with no errors. sqlx migrate run against a test Postgres instance returns 'Applied 1 migration'. All three tables exist under catalog schema: SELECT table_name FROM information_schema.tables WHERE table_schema='catalog' returns categories, products, availability_blocks.
</validation>