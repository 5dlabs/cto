Implement subtask 2003: Implement category and product listing/detail CRUD endpoints

## Objective
Implement the core REST endpoints for categories listing, products listing with filtering/pagination, and product detail retrieval.

## Steps
1. Create src/handlers/catalog.rs module. 2. Implement GET /api/v1/catalog/categories: query rms.categories, return JSON array with optional tree structure (parent_id nesting). 3. Implement GET /api/v1/catalog/products: query rms.products with optional query params: category_id (UUID filter), search (ILIKE on name/description), page (default 1), per_page (default 20). Return paginated JSON with items, total_count, page, per_page. 4. Implement GET /api/v1/catalog/products/:id: query rms.products by UUID, join with category name, return full product detail including specifications JSONB and image URL (constructed from S3 endpoint + image_key). 5. Create an AppState struct holding PgPool, RedisPool, and S3 config, pass as Axum State. 6. Register all routes in the main router with /api/v1/catalog prefix. 7. Implement proper error handling: 404 for not found, 400 for bad parameters, structured JSON error responses.

## Validation
GET /api/v1/catalog/categories returns a valid JSON array; GET /api/v1/catalog/products returns paginated results; GET /api/v1/catalog/products/:id returns the correct product; 404 returned for non-existent product ID; filtering by category_id works correctly.