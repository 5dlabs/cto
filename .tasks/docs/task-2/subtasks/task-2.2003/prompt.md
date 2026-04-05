Implement subtask 2003: Implement category listing endpoint

## Objective
Implement the GET /api/v1/catalog/categories endpoint that returns all equipment categories with optional tree structure support.

## Steps
1. Create a handlers/categories.rs module.
2. Implement `list_categories` handler: query all categories from PostgreSQL, return as JSON array.
3. Support optional query parameter `?tree=true` to return nested parent-child structure.
4. Include image_url fields pointing to S3/R2 URLs.
5. Register the route on the Axum router under /api/v1/catalog/categories.
6. Add appropriate error handling (500 for DB errors, return empty array if no categories).
7. Create a CategoryResponse DTO (separate from the DB model) with serde Serialize.

## Validation
GET /api/v1/catalog/categories returns 200 with a JSON array. With seeded data, verify categories appear with correct fields. With ?tree=true, verify nested structure. With empty DB, returns empty array.