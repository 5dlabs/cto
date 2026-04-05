Implement subtask 2004: Implement availability check endpoint with performance optimization

## Objective
Implement the GET /api/v1/catalog/products/:id/availability endpoint with a strict <500ms p95 response time requirement, using optimized queries and Redis caching.

## Steps
1. Create src/handlers/availability.rs module. 2. Implement GET /api/v1/catalog/products/:id/availability: accept optional query params date_from, date_to, location. 3. Query rms.availability table for the given product_id, compute available_quantity = total_quantity - reserved_quantity for the requested date range. 4. Add a Redis caching layer: cache availability results with key 'avail:{product_id}:{date_from}:{date_to}' and a 60-second TTL. Check Redis first, fall back to PostgreSQL on miss. 5. Ensure the PostgreSQL query uses an index on (product_id, available_from, available_until). Add migration for the index if not present. 6. Return JSON: { product_id, available_quantity, total_quantity, date_from, date_to, location, cached: bool }. 7. Add request timing middleware or manual timing to log p95 latency. 8. Register the route in the main router.

## Validation
Endpoint returns correct availability data; response time is <500ms at p95 under load (test with 100 concurrent requests using hey or wrk); cached responses return faster than uncached; cache invalidation works when availability data changes.