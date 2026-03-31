Implement subtask 4009: Implement currency rate sync scheduled job with Redis caching

## Objective
Build the background job that periodically fetches currency exchange rates from an external API and caches them in Redis, plus the /api/v1/currency/rates endpoint.

## Steps
1. Create `src/services/currency_service.rs`. 2. Implement `fetch_currency_rates(base_currency)` — HTTP client call to the configured CURRENCY_API_URL (e.g., exchangeratesapi.io or similar). Parse response to extract rates for supported currencies. 3. Implement `sync_currency_rates()` — fetch rates for USD (primary base), store each rate pair in the `finance.currency_rates` table (upsert on unique constraint). Also cache in Redis with key pattern `currency:rate:{base}:{target}` and TTL of 1 hour. 4. Implement `get_cached_rate(base, target)` — check Redis first; if miss, query database; if miss, trigger fetch. Return the rate as NUMERIC(18,8). 5. Implement `convert_amount(amount, from_currency, to_currency)` — use cached rates for conversion. 6. Create background task with tokio::time::interval running every 30 minutes to call sync_currency_rates(). Spawn in main.rs alongside the reminder job. 7. GET /api/v1/currency/rates — return all cached rates. Optional query param for base currency. 8. GET /api/v1/currency/rates/:base/:target — return specific rate pair. 9. Register routes under `/api/v1/currency`.

## Validation
Mock the external currency API and verify rates are fetched and stored in both PostgreSQL and Redis. Verify cache hit returns rate without DB query. Verify the /rates endpoint returns all rates. Verify conversion function produces correct results for known rate pairs. Verify scheduled job runs at configured interval.