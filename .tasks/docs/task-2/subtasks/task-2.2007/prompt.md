Implement subtask 2007: Implement category and product CRUD endpoints with pagination and search

## Objective
Build the core catalog REST endpoints: categories listing with parent_id filter, products listing with pagination/category filter/trigram search/JSONB spec queries, product detail, and admin create/update endpoints.

## Steps
1. Define Axum router in `catalog/src/routes/mod.rs`.
2. Implement `GET /api/v1/catalog/categories`:
   - Query params: `parent_id` (optional UUID) to filter children of a category.
   - Returns JSON array of categories.
3. Implement `GET /api/v1/catalog/products`:
   - Query params: `page` (default 1), `per_page` (default 20, max 100), `category_id` (optional), `search` (optional, trigram similarity on name with `%` ILIKE or `similarity()` function), `specs` (optional, JSONB containment query e.g., `specs @> '{"weight_kg": 5}'`).
   - Return `{"data": [...], "meta": {"page", "per_page", "total", "total_pages"}}`.
   - Use `sqlx::query_as!` with dynamic query building for filters.
4. Implement `GET /api/v1/catalog/products/:id`:
   - Join with categories to include category name.
   - Return 404 if not found.
5. Implement `POST /api/v1/catalog/products` (protected by API key auth middleware):
   - Validate request body: name required, day_rate > 0, category_id must exist.
   - Return 201 with created product.
6. Implement `PATCH /api/v1/catalog/products/:id` (protected by API key auth middleware):
   - Accept partial updates (only provided fields are updated).
   - Update `updated_at` timestamp.
   - Return 200 with updated product.
7. Define SQLx model structs with `FromRow` for categories and products, and request/response DTOs with `serde::Serialize/Deserialize`.

## Validation
Integration tests with sqlx::test: 1) Create categories, list all, filter by parent_id. 2) Create products, verify pagination meta is correct. 3) Search products by name trigram and verify relevant results appear. 4) Filter by JSONB specs containment. 5) Get product by ID returns category name. 6) Get non-existent ID returns 404. 7) POST without API key returns 401. 8) POST with valid key creates product and returns 201. 9) PATCH updates only provided fields.