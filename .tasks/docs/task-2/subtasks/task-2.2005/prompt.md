Implement subtask 2005: Implement admin and machine-readable catalog API endpoints

## Objective
Develop API endpoints for administrative product management (create/update) and machine-readable catalog access, including programmatic booking.

## Steps
1. Implement handlers for `POST /api/v1/catalog/products` to create new products.2. Implement handlers for `PATCH /api/v1/catalog/products/:id` to update existing products.3. Implement handlers for `GET /api/v1/equipment-api/catalog` for a machine-readable product list.4. Implement handlers for `POST /api/v1/equipment-api/checkout` for programmatic booking.

## Validation
1. Use `curl` to test `POST` and `PATCH` endpoints, verifying data persistence and updates.2. Verify `GET /api/v1/equipment-api/catalog` returns a structured, machine-readable response.3. Test `POST /api/v1/equipment-api/checkout` with valid data and verify a successful booking response.