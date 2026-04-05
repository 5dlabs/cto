Implement subtask 2006: Implement machine-readable API endpoints for Morgan agent

## Objective
Implement GET /api/v1/equipment-api/catalog and POST /api/v1/equipment-api/checkout endpoints designed for programmatic consumption by the Morgan AI agent.

## Steps
1. Create a handlers/equipment_api.rs module.
2. Implement `agent_catalog` handler (GET /api/v1/equipment-api/catalog):
   - Return a simplified, machine-friendly catalog listing with product IDs, names, rates, availability summary, and image URLs.
   - Include metadata useful for AI agents: structured category hierarchy, rate breakdowns, availability windows.
   - Response format should be flat and easy to parse (avoid deeply nested structures).
3. Implement `agent_checkout` handler (POST /api/v1/equipment-api/checkout):
   - Accept a JSON body with: product_id, customer_info, rental_dates (start, end), delivery_details.
   - Validate availability for the requested dates.
   - Create a reservation record (mark dates as unavailable).
   - Return a reservation confirmation with ID, total cost calculation, and next steps.
   - Handle conflicts (dates already reserved) with a 409 Conflict response.
4. Register routes under /api/v1/equipment-api/.

## Validation
GET /api/v1/equipment-api/catalog returns a flat, parseable JSON with all products and availability summaries. POST /api/v1/equipment-api/checkout with valid data creates a reservation and returns confirmation. Attempting to double-book the same dates returns 409. Invalid product IDs return 404.