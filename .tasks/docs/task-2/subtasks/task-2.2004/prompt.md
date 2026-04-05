Implement subtask 2004: Implement product listing and product detail endpoints

## Objective
Implement GET /api/v1/catalog/products (with filtering and pagination) and GET /api/v1/catalog/products/:id for individual product details.

## Steps
1. Create a handlers/products.rs module.
2. Implement `list_products` handler:
   - Query products from PostgreSQL with pagination (limit/offset or cursor-based).
   - Support query parameters: category_id, status, search (ILIKE on name/description), sort_by, order.
   - Return paginated response with items, total_count, page, page_size.
   - Map image_urls to full S3/R2 URLs using the S3_ENDPOINT and S3_PRODUCT_IMAGES_BUCKET from config.
3. Implement `get_product` handler:
   - Query a single product by ID (UUID or integer).
   - Return full product details including category info (JOIN or nested query).
   - Return 404 if product not found.
4. Create ProductListResponse and ProductDetailResponse DTOs.
5. Register routes: GET /api/v1/catalog/products and GET /api/v1/catalog/products/:id.

## Validation
GET /api/v1/catalog/products returns 200 with paginated JSON. Filtering by category_id returns only matching products. GET /api/v1/catalog/products/:id returns correct product with full details. Non-existent ID returns 404. Image URLs contain the correct S3 endpoint prefix.