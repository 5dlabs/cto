Implement subtask 2002: Define data models and create database migrations for Product, Category, and Availability

## Objective
Create sqlx migrations for the rms schema defining categories, products, product_images, and availability tables, along with Rust struct definitions with serde serialization for API responses.

## Steps
1. Create migration file: migrations/001_create_catalog_schema.sql.
2. Define tables in the 'rms' schema:
   - rms.categories: id (UUID PK, default gen_random_uuid()), name (VARCHAR 255 NOT NULL), slug (VARCHAR 255 UNIQUE NOT NULL), description (TEXT), parent_id (UUID FK self-referential, nullable), display_order (INT DEFAULT 0), created_at (TIMESTAMPTZ DEFAULT now()), updated_at (TIMESTAMPTZ DEFAULT now()).
   - rms.products: id (UUID PK), category_id (UUID FK → categories NOT NULL), name (VARCHAR 255 NOT NULL), slug (VARCHAR 255 UNIQUE NOT NULL), description (TEXT), short_description (VARCHAR 500), daily_rate_cents (BIGINT NOT NULL), weekly_rate_cents (BIGINT), specifications (JSONB), weight_kg (NUMERIC(10,2)), is_active (BOOLEAN DEFAULT true), created_at, updated_at.
   - rms.product_images: id (UUID PK), product_id (UUID FK → products NOT NULL), image_key (VARCHAR 512 NOT NULL, the S3 object key), display_order (INT DEFAULT 0), alt_text (VARCHAR 255), created_at.
   - rms.availability: id (UUID PK), product_id (UUID FK → products NOT NULL), date (DATE NOT NULL), total_quantity (INT NOT NULL), reserved_quantity (INT DEFAULT 0), UNIQUE(product_id, date).
3. Add indexes: products(category_id), products(slug), products(is_active), availability(product_id, date), categories(slug).
4. Create Rust models in src/models/: Category, Product, ProductImage, Availability structs with sqlx::FromRow and serde::Serialize derives.
5. Create a ProductResponse struct that includes image URLs (constructed from S3_CDN_BASE_URL + image_key) and nested category info.
6. Add a second migration 002_seed_sample_data.sql with a few sample categories and products for development.

## Validation
sqlx migrate run succeeds against a test database; all tables exist in rms schema with correct columns and constraints; Rust model structs compile and can be deserialized from query results; sample data is queryable.