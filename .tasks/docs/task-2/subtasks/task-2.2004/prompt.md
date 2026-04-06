Implement subtask 2004: Implement real-time availability endpoint

## Objective
Implement the product availability checking endpoint that queries current availability data and returns date-range availability for a given product.

## Steps
1. Create src/handlers/availability.rs:
   - GET /api/v1/catalog/products/:id/availability: Accept query params: start_date (required, YYYY-MM-DD), end_date (required, YYYY-MM-DD), quantity (optional, default 1).
   - Query the availability table for the product_id within the date range.
   - Return: {product_id, start_date, end_date, dates: [{date, quantity_available, quantity_reserved, is_available: bool}], fully_available: bool}.
   - is_available = (quantity_available - quantity_reserved) >= requested quantity.
   - fully_available = all dates in range have is_available = true.
2. Validate: start_date <= end_date, date range <= 365 days, product_id exists.
3. Use a single SQL query with generate_series to fill in missing dates with default availability.
4. Wire route into Router.
5. Implement the /api/v1/equipment-api/checkout endpoint that accepts a checkout request body (product_id, start_date, end_date, quantity, customer info) and creates a reservation by decrementing quantity_available in a transaction. Return success/failure with reservation_id.

## Validation
GET /api/v1/catalog/products/:id/availability returns correct availability for seeded data. Date range validation rejects invalid ranges (end < start, > 365 days). Missing dates in DB are filled with defaults. Checkout endpoint creates a reservation and decrements availability atomically (verified by re-querying availability). Response time for availability check < 500ms with seeded data for 30-day range.