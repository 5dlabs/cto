Implement subtask 2005: Create database schema migrations for categories, products, and availability tables

## Objective
Define sqlx migrations for the equipment catalog database schema including categories (with self-referencing hierarchy), products (with SKU, barcode, pricing, JSONB fields), and availability tables with appropriate indexes and seed data for 24 categories.

## Steps
1. Create `services/equipment-catalog/migrations/` directory.
2. Migration 001 — `categories` table: id UUID PK DEFAULT gen_random_uuid(), name VARCHAR(255) NOT NULL, parent_id UUID NULL REFERENCES categories(id) ON DELETE SET NULL, icon VARCHAR(100), sort_order INTEGER NOT NULL DEFAULT 0, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(). Index on parent_id.
3. Migration 002 — `products` table: id UUID PK DEFAULT gen_random_uuid(), name VARCHAR(500) NOT NULL, category_id UUID NOT NULL REFERENCES categories(id), description TEXT, sku VARCHAR(50) UNIQUE NOT NULL, barcode VARCHAR(100) UNIQUE, day_rate BIGINT NOT NULL (cents), weight_kg NUMERIC(8,2), dimensions JSONB, image_urls TEXT[] DEFAULT '{}', specs JSONB DEFAULT '{}', created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(). Indexes: products(category_id), products(sku), products(barcode).
4. Migration 003 — `availability` table: product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE, date DATE NOT NULL, quantity_total INTEGER NOT NULL DEFAULT 0, quantity_reserved INTEGER NOT NULL DEFAULT 0, quantity_booked INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (product_id, date). Enable btree_gist extension. Create GiST index on (product_id, date) for range queries. Add CHECK constraint: quantity_reserved + quantity_booked <= quantity_total.
5. Migration 004 — Seed 24 categories from the PRD (Camera, Lenses, Lighting, Grip, Audio, etc.) with appropriate sort_order and parent hierarchy.
6. Migration 005 — Add updated_at trigger function and trigger on products table to auto-update on row modification.

## Validation
Integration test: run all migrations against a clean PostgreSQL database, verify all tables exist with correct columns using information_schema queries. Verify 24 categories seeded. Insert a test product and availability row, verify CHECK constraint rejects quantity_reserved + quantity_booked > quantity_total. Verify updated_at trigger fires on product update.