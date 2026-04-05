Implement subtask 2005: Implement product availability endpoint

## Objective
Implement GET /api/v1/catalog/products/:id/availability that returns availability data for a specific product within a date range.

## Steps
1. Create a handlers/availability.rs module.
2. Implement `get_availability` handler:
   - Accept path param product_id and query params: start_date, end_date (default to next 30 days if not provided).
   - Query the availability table for the product within the date range.
   - Return an array of date/availability pairs.
   - Return 404 if product_id doesn't exist.
3. Create AvailabilityResponse DTO with fields: product_id, dates (array of {date, is_available, reserved_by?}).
4. Register route: GET /api/v1/catalog/products/:id/availability.

## Validation
GET /api/v1/catalog/products/:id/availability returns 200 with date-availability array. With seeded availability data, verify correct available/unavailable dates. Invalid product ID returns 404. Date range filtering works correctly.