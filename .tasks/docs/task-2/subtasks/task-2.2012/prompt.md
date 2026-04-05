Implement subtask 2012: Implement machine-readable equipment API endpoints for AI agents

## Objective
Build the GET /api/v1/equipment-api/catalog and POST /api/v1/equipment-api/checkout endpoints providing a simplified, machine-optimized interface for AI agent integration.

## Steps
1. `GET /api/v1/equipment-api/catalog` handler:
   - Query all products with category name joined.
   - Return simplified JSON array: `[{ "id": "...", "name": "...", "category": "...", "day_rate_cents": 1500, "sku": "...", "available": true }]`.
   - `available` field: check if product has any availability rows with quantity_available > 0 for today.
   - No pagination — return all products (for AI consumption).
   - Include `Content-Type: application/json` and `X-API-Version: v1` headers.
2. `POST /api/v1/equipment-api/checkout` handler:
   - Requires authentication (any valid JWT).
   - Accept body: `{ "product_id": "uuid", "dates": ["2024-01-15", "2024-01-16"], "quantity": 1 }`.
   - Call the reservation logic from subtask 2009 (create_reservation).
   - Return 201 with reservation confirmation: `{ "reservation_id": "uuid", "product_id": "...", "dates": [...], "quantity": 1, "status": "reserved" }`.
   - Return 409 Conflict if insufficient availability.
3. Generate a reservation_id as a new UUID for tracking.
4. Add these routes under `/api/v1/equipment-api` nest in the router.

## Validation
Integration tests: (1) GET /equipment-api/catalog returns JSON array with all products, each having id, name, category, day_rate_cents, sku, available fields. (2) POST /equipment-api/checkout with valid product and dates returns 201 with reservation_id. (3) POST checkout for dates with no availability returns 409. (4) POST checkout without auth returns 401. (5) Verify X-API-Version header is present on catalog response.