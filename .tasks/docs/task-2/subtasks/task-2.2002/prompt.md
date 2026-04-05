Implement subtask 2002: Implement public catalog read endpoints

## Objective
Build the public read-only REST endpoints for browsing the equipment catalog: list categories, list products (with filtering/pagination), get product by ID, and check product availability.

## Steps
1. Implement `GET /api/v1/catalog/categories`:
   - Query all categories from DB, return as JSON array.
   - Support optional `parent_id` query param for sub-category filtering.
2. Implement `GET /api/v1/catalog/products`:
   - Support query params: category_id, search (name/sku), page, per_page (default 20, max 100).
   - Return paginated JSON with items array and total count.
   - Include product image URL (placeholder; actual signed URL logic in a later subtask).
3. Implement `GET /api/v1/catalog/products/:id`:
   - Fetch single product by UUID, return 404 if not found.
   - Include full product details with specs JSONB.
4. Implement `GET /api/v1/catalog/products/:id/availability`:
   - Accept query params: start_date, end_date (required).
   - Query availability table for the date range.
   - Return array of {date, available_quantity, total_quantity}.
5. Use Axum extractors (Query, Path, State) idiomatically.
6. Implement proper error handling with consistent JSON error responses (status, message).
7. Add request validation (date format, UUID format, pagination bounds).

## Validation
Write integration tests seeding 24 categories and 50+ products. Verify: categories endpoint returns all categories; products endpoint supports pagination and filtering; single product endpoint returns correct data or 404; availability endpoint returns correct date-range data. All responses are valid JSON with correct HTTP status codes.