Implement subtask 2004: Implement core read-only catalog API endpoints

## Objective
Develop the read-only API endpoints for categories, products, product details, and product availability, interacting with the PostgreSQL database.

## Steps
1. Implement handlers for `GET /api/v1/catalog/categories` to list all categories.2. Implement handlers for `GET /api/v1/catalog/products` with filtering capabilities.3. Implement handlers for `GET /api/v1/catalog/products/:id` to retrieve a single product.4. Implement handlers for `GET /api/v1/catalog/products/:id/availability?from=&to=` to check product availability within a date range.

## Validation
1. Seed test data into the PostgreSQL database.2. Use `curl` to call each endpoint and verify correct JSON responses and data retrieval.