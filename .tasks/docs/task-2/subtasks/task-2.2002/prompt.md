Implement subtask 2002: Define database models and create sqlx migrations for rms schema

## Objective
Define the Product, Category, and Availability domain models in Rust and create sqlx migrations for the corresponding tables in the PostgreSQL rms schema.

## Steps
1. Create migrations directory: migrations/
2. Create migration 001_create_rms_tables.sql:
   - SET search_path TO rms;
   - CREATE TABLE categories (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), name VARCHAR(255) NOT NULL, slug VARCHAR(255) UNIQUE NOT NULL, description TEXT, parent_id UUID REFERENCES categories(id), display_order INT DEFAULT 0, created_at TIMESTAMPTZ DEFAULT NOW(), updated_at TIMESTAMPTZ DEFAULT NOW());
   - CREATE TABLE products (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), tenant_id UUID NOT NULL, category_id UUID NOT NULL REFERENCES categories(id), name VARCHAR(500) NOT NULL, slug VARCHAR(500) NOT NULL, description TEXT, daily_rate_cents BIGINT NOT NULL, weekly_rate_cents BIGINT, monthly_rate_cents BIGINT, image_key VARCHAR(1024), specifications JSONB DEFAULT '{}', is_active BOOLEAN DEFAULT true, created_at TIMESTAMPTZ DEFAULT NOW(), updated_at TIMESTAMPTZ DEFAULT NOW(), UNIQUE(tenant_id, slug));
   - CREATE TABLE availability (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), product_id UUID NOT NULL REFERENCES products(id), date DATE NOT NULL, quantity_available INT NOT NULL DEFAULT 0, quantity_reserved INT NOT NULL DEFAULT 0, updated_at TIMESTAMPTZ DEFAULT NOW(), UNIQUE(product_id, date));
   - Add indexes on products(category_id), products(tenant_id), availability(product_id, date).
3. Define Rust structs in src/models/:
   - src/models/category.rs: Category struct with sqlx::FromRow
   - src/models/product.rs: Product struct with sqlx::FromRow, include image_url computed field
   - src/models/availability.rs: Availability struct with sqlx::FromRow
4. Run migrations in main.rs startup: sqlx::migrate!().run(&pool).await
5. Add compile-time query checking with sqlx prepare if feasible.

## Validation
Migrations run successfully against a test PostgreSQL instance. All tables exist in the rms schema with correct columns and constraints. Rust model structs can be deserialized from query results (integration test inserting and selecting a row for each table).