Implement subtask 2008: Implement availability checking and atomic checkout endpoints

## Objective
Build the real-time availability endpoint (GET /api/v1/catalog/products/:id/availability) and the programmatic checkout endpoint (POST /api/v1/equipment-api/checkout) with atomic inventory decrement.

## Steps
1. Implement `GET /api/v1/catalog/products/:id/availability?from=YYYY-MM-DD&to=YYYY-MM-DD`:
   - Query the `availability` table for the product within the date range.
   - For each day, return `quantity_total - reserved - booked` as available quantity.
   - Return JSON: `{"product_id": "...", "availability": [{"date": "2024-01-01", "available": 3, "total": 5}, ...]}`.
   - If no availability rows exist for requested dates, return 0 available.
   - Optimize query with index on `(product_id, date)` — target p99 < 500ms.
2. Implement `POST /api/v1/equipment-api/checkout`:
   - Request body: `{"product_id": UUID, "customer_id": UUID, "date_from": Date, "date_to": Date, "quantity": u32}`.
   - In a database transaction:
     a. `SELECT ... FOR UPDATE` on availability rows for the product and date range.
     b. Verify all days have sufficient available quantity (total - reserved - booked >= requested quantity).
     c. If insufficient, rollback and return 409 Conflict with details on which dates are unavailable.
     d. UPDATE availability SET reserved = reserved + quantity for each day.
     e. INSERT into bookings table with status 'reserved'.
     f. Commit and return 201 with booking ID.
3. Implement the availability domain logic in a separate `catalog/src/domain/availability.rs` module for testability.
4. Add appropriate error handling for invalid date ranges (from > to), non-existent products, etc.

## Validation
Integration tests: 1) Seed availability rows, query availability and verify correct available counts. 2) Checkout reduces available count atomically. 3) Concurrent checkout test: two simultaneous checkouts for the last available unit — one succeeds, one gets 409. 4) Checkout with insufficient availability returns 409. 5) Checkout for non-existent product returns 404. 6) Invalid date range (from > to) returns 400. Unit tests on domain logic: overlapping date ranges, partial bookings, full capacity scenarios. Benchmark: seed 1000 products with 10,000 bookings, run `wrk` against availability endpoint and verify p99 < 500ms.