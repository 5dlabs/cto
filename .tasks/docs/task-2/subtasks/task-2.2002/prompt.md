Implement subtask 2002: Define database schema and domain models for Product, Category, and Availability

## Objective
Create SQL migration files for the catalog schema (products, categories, availability tables) and corresponding Rust struct models with sqlx FromRow derives and serde serialization.

## Steps
1. Create a migrations directory and use sqlx-cli for migration management. 2. Write migration 001: CREATE SCHEMA IF NOT EXISTS rms; CREATE TABLE rms.categories (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), name VARCHAR(255) NOT NULL, slug VARCHAR(255) UNIQUE NOT NULL, description TEXT, parent_id UUID REFERENCES rms.categories(id), created_at TIMESTAMPTZ DEFAULT now(), updated_at TIMESTAMPTZ DEFAULT now()); 3. Write migration 002: CREATE TABLE rms.products (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), name VARCHAR(255) NOT NULL, slug VARCHAR(255) UNIQUE NOT NULL, description TEXT, category_id UUID REFERENCES rms.categories(id), daily_rate DECIMAL(10,2), weekly_rate DECIMAL(10,2), monthly_rate DECIMAL(10,2), image_key VARCHAR(512), specifications JSONB, created_at TIMESTAMPTZ, updated_at TIMESTAMPTZ); 4. Write migration 003: CREATE TABLE rms.availability (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), product_id UUID REFERENCES rms.products(id), total_quantity INT NOT NULL, reserved_quantity INT DEFAULT 0, available_from DATE, available_until DATE, location VARCHAR(255)); 5. Create Rust models in src/models/: Category, Product, Availability structs with sqlx::FromRow and serde::Serialize derives. 6. Run migrations against the dev database.

## Validation
Migrations run successfully via 'sqlx migrate run'; all tables exist in the rms schema; Rust models compile and can be used in sqlx::query_as! macros without errors.