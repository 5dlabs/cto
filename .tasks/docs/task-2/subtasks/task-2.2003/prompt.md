Implement subtask 2003: Implement public catalog CRUD endpoints with filtering and pagination

## Objective
Implement the public-facing catalog API endpoints: GET /api/v1/catalog/categories, GET /api/v1/catalog/products (with filtering and pagination), GET /api/v1/catalog/products/:id, and GET /api/v1/catalog/products/:id/availability.

## Steps
1. Create a `routes/catalog.rs` module with an Axum Router.
2. Implement `GET /api/v1/catalog/categories`: query all categories from rms.categories, return JSON array. Support optional `parent_id` query param for hierarchical filtering.
3. Implement `GET /api/v1/catalog/products`: query rms.products with support for query params: category_id, search (name ILIKE), min_price, max_price, page (default 1), per_page (default 20, max 100). Return paginated response with { data: [...], total, page, per_page, total_pages }.
4. Implement `GET /api/v1/catalog/products/:id`: fetch single product by ID, return 404 if not found. Include full image URLs constructed from S3 endpoint + bucket + stored keys.
5. Implement `GET /api/v1/catalog/products/:id/availability`: query rms.availability for given product_id, support date_from and date_to query params, return array of availability slots.
6. Add proper error handling with consistent JSON error responses (400, 404, 500).
7. Wire all routes into the main Axum router under the /api/v1/catalog prefix.
8. Instrument all handlers with Prometheus request count and latency metrics.

## Validation
Call each endpoint with seed data and verify correct JSON structures; test filtering by category_id and search term returns filtered results; test pagination returns correct page/total metadata; verify 404 for non-existent product ID; confirm S3 image URLs are properly constructed in product responses.