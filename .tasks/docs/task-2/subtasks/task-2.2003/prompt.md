Implement subtask 2003: Implement machine-readable equipment-api endpoints

## Objective
Build the machine-readable API endpoints (/equipment-api/catalog and /equipment-api/checkout) designed for AI agent consumption with structured, deterministic response formats.

## Steps
1. Implement `GET /api/v1/equipment-api/catalog`:
   - Return a structured JSON response optimized for machine parsing.
   - Include all products with their availability status, categories, pricing tiers (daily/weekly/monthly), and specs.
   - Use a flat, denormalized format: each item includes inline category name and availability summary.
   - Support query params: category, available_after (date), min_quantity.
2. Implement `POST /api/v1/equipment-api/checkout`:
   - Accept a JSON body with: items (array of {product_id, quantity, start_date, end_date}), customer_info (name, email, phone).
   - Validate all product IDs exist and quantities are available for the requested dates.
   - Return a quote summary: line items with calculated pricing, total, and a quote_id for reference.
   - Do NOT actually create an order; this is a quote/validation endpoint.
3. Use strongly-typed request/response structs with serde.
4. Return machine-friendly error codes (not just HTTP status) in error responses for agent consumption.

## Validation
Test the catalog endpoint returns all products in the expected flat format with inline category and availability. Test the checkout endpoint with valid items returns a correct quote with pricing math. Test with invalid product IDs, insufficient availability, and malformed dates to verify structured error responses.