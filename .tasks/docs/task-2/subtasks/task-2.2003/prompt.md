Implement subtask 2003: Implement core CRUD API endpoints for categories and products

## Objective
Implement the category listing, product listing with filtering/pagination, and single product detail endpoints with input validation and JSON serialization.

## Steps
1. Create src/handlers/categories.rs:
   - GET /api/v1/catalog/categories: List all categories with optional tree structure (parent_id). Return Vec<Category> as JSON. Support ?parent_id= filter.
2. Create src/handlers/products.rs:
   - GET /api/v1/catalog/products: List products with pagination (limit, offset query params), filtering by category_id, tenant_id (from auth context or header), search by name. Return paginated response {items: Vec<Product>, total: i64, limit: i32, offset: i32}.
   - GET /api/v1/catalog/products/:id: Return single product by UUID. Return 404 if not found.
3. Create request/response DTOs in src/dto/:
   - ProductListQuery: category_id, limit (default 20, max 100), offset (default 0), search
   - ProductResponse: Product fields + computed image_url (S3 URL prefix + image_key)
   - CategoryResponse: Category fields
4. Add input validation using axum-extra or custom extractors: validate UUID format, limit/offset bounds.
5. Wire routes into the Axum Router in main.rs.
6. Implement the /api/v1/equipment-api/catalog endpoint as an alias or aggregation of categories + featured products for the equipment API consumer.
7. Add structured error responses with consistent JSON error format: {error: string, code: string, details: optional}.

## Validation
Integration tests: GET /api/v1/catalog/categories returns 200 with valid JSON array. GET /api/v1/catalog/products returns 200 with paginated response. GET /api/v1/catalog/products/:id returns 200 for existing product, 404 for non-existent UUID, 400 for invalid UUID format. Pagination params are respected (limit, offset). Category filter works correctly.