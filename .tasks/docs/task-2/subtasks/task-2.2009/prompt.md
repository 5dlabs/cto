Implement subtask 2009: Implement availability checking endpoint and reservation logic

## Objective
Build the date-range availability checking endpoint and the reservation creation logic with <500ms performance requirement, including the availability calculation (total - reserved - booked = available).

## Steps
1. Define `AvailabilityRow` struct: product_id (Uuid), date (NaiveDate), quantity_total (i32), quantity_reserved (i32), quantity_booked (i32), quantity_available (computed: total - reserved - booked).
2. `GET /api/v1/catalog/products/:id/availability?from=YYYY-MM-DD&to=YYYY-MM-DD` handler:
   - Validate date params (from <= to, range <= 365 days).
   - Query: `SELECT * FROM availability WHERE product_id = $1 AND date BETWEEN $2 AND $3 ORDER BY date`.
   - For dates with no row, assume quantity_available = 0 (or a configurable default).
   - Compute quantity_available = quantity_total - quantity_reserved - quantity_booked for each row.
   - Return array of `{ date, quantity_total, quantity_reserved, quantity_booked, quantity_available }`.
   - This endpoint must respond in <500ms; the GiST index on (product_id, date) supports this.
3. Create an `availability` module with pure functions for computing availability:
   - `fn compute_available(total: i32, reserved: i32, booked: i32) -> i32`
   - `fn is_available_for_quantity(rows: &[AvailabilityRow], required: i32) -> bool` — checks all dates in range have >= required available.
4. Implement reservation helper (used by checkout endpoint): `async fn create_reservation(pool: &PgPool, product_id: Uuid, dates: Vec<NaiveDate>, quantity: i32) -> Result<()>` — UPDATE availability SET quantity_reserved = quantity_reserved + $quantity WHERE product_id = $1 AND date = ANY($2) AND (quantity_total - quantity_reserved - quantity_booked) >= $quantity. Use a transaction, return Conflict error if insufficient availability.
5. Do NOT cache availability queries — real-time requirement.

## Validation
Unit tests (>= 5 cases): (1) compute_available(10, 3, 2) = 5. (2) compute_available(10, 10, 0) = 0. (3) compute_available(5, 0, 5) = 0. (4) is_available_for_quantity with mixed rows and required=2 returns correct boolean. (5) compute_available(0, 0, 0) = 0. Integration tests: (1) Seed availability rows, GET availability endpoint, verify correct quantities. (2) Measure response time is <500ms for a 30-day range query using std::time::Instant. (3) Create reservation, verify quantity_reserved incremented. (4) Attempt reservation exceeding available quantity, verify Conflict error.