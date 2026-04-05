Implement subtask 2003: Implement core catalog CRUD endpoints (categories, products, availability)

## Objective
Implement the public REST API endpoints for browsing the equipment catalog: list categories, list/filter products, get product by ID, and check product availability.

## Steps
1. Create src/handlers/catalog.rs with the following Axum handler functions:
2. GET /api/v1/catalog/categories → list_categories: query rms.categories ordered by display_order, support optional ?parent_id filter, return JSON array.
3. GET /api/v1/catalog/products → list_products: query rms.products WHERE is_active=true, support query params: ?category_id, ?search (ILIKE on name/description), ?page (default 1), ?per_page (default 20, max 100). Join product_images to include image URLs. Return paginated response with { data: [], pagination: { page, per_page, total, total_pages } }.
4. GET /api/v1/catalog/products/:id → get_product: query single product by UUID, include all images, category info, and specifications. Return 404 with structured error if not found.
5. GET /api/v1/catalog/products/:id/availability → get_availability: accept query params ?start_date, ?end_date (required), query rms.availability for the product in the date range, compute available_quantity = total_quantity - reserved_quantity for each date. Return array of { date, available_quantity, total_quantity }.
6. Create structured error types in src/errors.rs (AppError enum implementing IntoResponse) with consistent JSON error format: { error: { code, message, details } }.
7. Register all routes on the Axum Router with /api/v1/catalog prefix.
8. Use tower_http::trace for request logging on all routes.

## Validation
Unit tests for each handler using sqlx::test with a test database; list_categories returns seeded data; list_products supports pagination and filtering; get_product returns 404 for non-existent IDs; get_availability computes correct available quantities; all responses match the expected JSON schema.