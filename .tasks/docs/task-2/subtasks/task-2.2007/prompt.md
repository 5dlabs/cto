Implement subtask 2007: Implement category listing endpoint with product counts

## Objective
Build the GET /api/v1/catalog/categories endpoint returning all categories with their hierarchical structure and product counts per category.

## Steps
1. Define `Category` model struct with serde Serialize: id (Uuid), name (String), parent_id (Option<Uuid>), icon (Option<String>), sort_order (i32), product_count (i64), created_at (DateTime<Utc>).
2. Implement `GET /api/v1/catalog/categories` handler: query categories LEFT JOIN products to get counts, ordered by sort_order. SQL: `SELECT c.*, COUNT(p.id) as product_count FROM categories c LEFT JOIN products p ON p.category_id = c.id GROUP BY c.id ORDER BY c.sort_order`.
3. Return JSON array of categories.
4. This endpoint is public (no auth required).
5. Add the route to the catalog router nest.

## Validation
Integration test: seed categories and some products, GET /api/v1/catalog/categories, verify response contains all 24 categories with correct product_count values. Verify categories with no products have product_count = 0. Verify ordering matches sort_order.