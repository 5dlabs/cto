Implement subtask 2010: Implement GDPR deletion endpoint

## Objective
Build the DELETE /api/v1/gdpr/customer/:id endpoint that removes all customer-related booking data and returns a confirmation response.

## Steps
1. Implement `DELETE /api/v1/gdpr/customer/:id` (protected by API key auth):
   - In a database transaction:
     a. Find all bookings for the given customer_id.
     b. For each booking with status 'reserved', restore availability (decrement reserved count on availability rows).
     c. Delete all bookings for the customer.
     d. Commit transaction.
   - Return JSON: `{"customer_id": "...", "deleted_bookings": N, "status": "completed", "timestamp": "..."}`.
2. Handle edge cases: customer with no bookings returns 200 with deleted_bookings: 0 (not 404 — GDPR deletion should be idempotent).
3. Log the deletion event (customer_id, count, timestamp) for audit trail.
4. Ensure no customer PII is retained in any logs.

## Validation
Integration test: 1) Create bookings for a customer via checkout. 2) Call DELETE endpoint. 3) Verify bookings table has no rows for that customer_id. 4) Verify availability rows have been restored (reserved count decremented). 5) Verify response JSON contains correct deleted count. 6) Call DELETE again for same customer — verify idempotent 200 with deleted_bookings: 0. 7) Call without API key — verify 401.