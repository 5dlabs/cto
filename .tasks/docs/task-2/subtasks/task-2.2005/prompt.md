Implement subtask 2005: Implement availability calculation logic and GET /availability endpoint with unit tests

## Objective
Write the core availability computation function and expose it via GET /api/v1/catalog/products/:id/availability?from=&to=, with unit tests covering overlap edge cases.

## Steps
Create src/availability.rs with pub fn compute_availability(stock_quantity: i32, blocks: &[AvailabilityBlock]) -> i32: blocks filtered by date overlap with requested window (overlap condition: block.date_from <= window_to AND block.date_to >= window_from). Sum quantity_reserved + quantity_booked across overlapping blocks. Return max(0, stock_quantity - sum). In handler GET /api/v1/catalog/products/:id/availability: parse from/to as NaiveDate (return 400 if invalid). SELECT stock_quantity FROM catalog.products WHERE id = $1 (return 404 if missing). SELECT * FROM catalog.availability_blocks WHERE product_id = $1 AND date_from <= $3 AND date_to >= $2 (server-side overlap filter for performance). Call compute_availability. Return JSON: { product_id, date_from, date_to, quantity_available, blocks_count }. Add database index usage note (index on product_id, date_from, date_to already created in migration). Unit tests (no DB): test_no_blocks returns stock_quantity, test_partial_overlap counts only overlapping blocks, test_full_overlap, test_zero_available returns 0 not negative.

## Validation
cargo test availability passes all 4+ unit test cases. GET /api/v1/catalog/products/:id/availability?from=2025-01-01&to=2025-01-03 returns HTTP 200 with quantity_available >= 0 in under 500ms (measured with curl -w '%{time_total}'). Insert a blocking availability_block for the same period and verify quantity_available decreases. Invalid date format returns HTTP 400.