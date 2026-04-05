Implement subtask 2008: Implement product CRUD endpoints (list, detail, create, update)

## Objective
Build the product listing with pagination/filtering, product detail, admin-only create, and admin-only update endpoints, including image URL resolution to CDN paths.

## Steps
1. Define `Product` model struct matching the products table, with `rust_decimal::Decimal` for any price computations but BIGINT cents for storage/API.
2. Define `ProductListParams` query params: category_id (Option<Uuid>), search (Option<String>), min_price (Option<i64>), max_price (Option<i64>), page (Option<u32>, default 1), per_page (Option<u32>, default 20, max 100).
3. `GET /api/v1/catalog/products` handler: build dynamic SQL query with sqlx. Filter by category_id if present, ILIKE search on name/description if search present, day_rate BETWEEN for price range. Count total for pagination headers. Return paginated response with `{ data: [...], pagination: { page, per_page, total, total_pages } }`.
4. Image URL resolution: before returning any product, map `image_urls` array entries from R2 keys to full CDN URLs using `format!("{}/{}", state.cdn_base_url, key)`. Implement as a method on Product or a helper function.
5. `GET /api/v1/catalog/products/:id` handler: fetch product by UUID, return 404 if not found, include resolved image URLs.
6. `POST /api/v1/catalog/products` handler: protected with `require_role("admin")`. Accept `CreateProductRequest` body (name, category_id, description, sku, barcode, day_rate, weight_kg, dimensions, image_urls, specs). Validate required fields. INSERT and return created product with 201 status.
7. `PATCH /api/v1/catalog/products/:id` handler: protected with `require_role("admin")`. Accept `UpdateProductRequest` with all optional fields. Build dynamic UPDATE SET query for only provided fields. Return updated product.
8. Use `rust_decimal` for any price arithmetic if aggregation is needed in responses.

## Validation
Integration tests: (1) POST a product as admin, verify 201 and returned fields. (2) GET the product by ID, verify all fields match including CDN-resolved image URLs. (3) PATCH the product name, verify only name changed, updated_at changed. (4) GET /products with category_id filter returns only matching products. (5) GET /products with search term finds product by name substring. (6) GET /products with pagination returns correct page/total_pages. (7) POST without admin role returns 403. (8) GET non-existent product ID returns 404.