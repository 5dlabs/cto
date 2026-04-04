Implement subtask 2006: Create SQLx migrations for catalog schema

## Objective
Write and validate SQLx migrations for the catalog database schema: categories, products (with JSONB specs, TEXT[] image_urls, DECIMAL day_rate), availability, and bookings tables with appropriate indexes.

## Steps
1. Create `catalog/migrations/` directory.
2. Migration 001 — `categories` table:
   - `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`, `name VARCHAR(255) NOT NULL`, `parent_id UUID REFERENCES categories(id)`, `description TEXT`, `created_at TIMESTAMPTZ DEFAULT now()`, `updated_at TIMESTAMPTZ DEFAULT now()`.
   - Index on `parent_id`.
3. Migration 002 — `products` table:
   - `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`, `category_id UUID NOT NULL REFERENCES categories(id)`, `name VARCHAR(500) NOT NULL`, `description TEXT`, `specs JSONB NOT NULL DEFAULT '{}'`, `image_urls TEXT[] NOT NULL DEFAULT '{}'`, `day_rate DECIMAL(10,2) NOT NULL`, `sku VARCHAR(100) UNIQUE`, `barcode VARCHAR(100) UNIQUE`, `active BOOLEAN DEFAULT true`, `created_at TIMESTAMPTZ DEFAULT now()`, `updated_at TIMESTAMPTZ DEFAULT now()`.
   - GIN index on `specs` for JSONB queries.
   - GIN trigram index on `name` for search: `CREATE INDEX idx_products_name_trgm ON products USING gin (name gin_trgm_ops);` (requires `pg_trgm` extension).
   - Index on `category_id`.
4. Migration 003 — Enable `pg_trgm` extension: `CREATE EXTENSION IF NOT EXISTS pg_trgm;` (must come before the trigram index or combine appropriately).
5. Migration 004 — `availability` table:
   - `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`, `product_id UUID NOT NULL REFERENCES products(id)`, `date DATE NOT NULL`, `quantity_total INT NOT NULL`, `reserved INT NOT NULL DEFAULT 0`, `booked INT NOT NULL DEFAULT 0`, `UNIQUE(product_id, date)`.
   - Index on `(product_id, date)`.
   - CHECK constraint: `reserved + booked <= quantity_total`.
6. Migration 005 — `bookings` table:
   - `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`, `customer_id UUID NOT NULL`, `product_id UUID NOT NULL REFERENCES products(id)`, `date_from DATE NOT NULL`, `date_to DATE NOT NULL`, `quantity INT NOT NULL DEFAULT 1`, `status VARCHAR(50) NOT NULL DEFAULT 'reserved'`, `created_at TIMESTAMPTZ DEFAULT now()`, `updated_at TIMESTAMPTZ DEFAULT now()`.
   - Indexes on `customer_id`, `product_id`, `(product_id, date_from, date_to)`.
7. Verify migrations run cleanly with `sqlx migrate run`.

## Validation
Run `sqlx migrate run` against a fresh PostgreSQL database and verify all tables are created with correct columns, types, indexes, and constraints. Verify the CHECK constraint on availability by attempting an INSERT that violates it. Verify the UNIQUE constraint on availability(product_id, date). Run `sqlx migrate run` twice to confirm idempotency.