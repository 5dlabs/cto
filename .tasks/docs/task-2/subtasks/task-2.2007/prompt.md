Implement subtask 2007: Implement machine-readable equipment-api endpoints for Morgan agent

## Objective
Implement the /api/v1/equipment-api/catalog and /api/v1/equipment-api/checkout endpoints designed for consumption by the Morgan AI agent and other automated systems.

## Steps
1. Create src/handlers/equipment_api.rs module. 2. Implement GET /api/v1/equipment-api/catalog: return a simplified, machine-optimized JSON format with product IDs, names, categories, rates, and current availability in a flat structure suitable for LLM consumption. Include metadata like total_products, last_updated. 3. Implement POST /api/v1/equipment-api/checkout: accept a JSON body with { product_id, quantity, rental_start, rental_end, customer_info: { name, email, phone } }. Validate availability, create a reservation (INSERT into a rms.reservations table — add migration if needed), decrement available quantity, and return a confirmation with reservation_id. 4. Implement idempotency for checkout using an idempotency_key header. 5. Both endpoints should include structured error responses with error codes that agents can parse programmatically. 6. Register routes in the main router.

## Validation
GET /equipment-api/catalog returns valid JSON with all products and availability; POST /equipment-api/checkout with valid data creates a reservation and returns a reservation_id; duplicate checkout with same idempotency_key returns the same result; checkout with insufficient availability returns a structured error.