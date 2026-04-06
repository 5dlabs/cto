Implement subtask 4005: Implement currency rate sync job with Redis caching

## Objective
Build a scheduled background job that fetches current exchange rates from an external API, stores them in PostgreSQL, and caches them in Redis for fast multi-currency conversions.

## Steps
1. Create src/jobs/currency_sync.rs with a CurrencyRateSyncer.
2. Implement fetch_rates() that calls the configured exchange rate API (abstracted behind a trait for swappability):
   - Parse JSON response into rate entries (base_currency, target_currency, rate)
   - Insert/upsert rates into currency_rates table with fetched_at timestamp
   - Cache rates in Redis with key pattern 'currency:USD:CAD' and TTL of 24 hours
3. Create a tokio::spawn background task that runs fetch_rates() on a configurable interval (default: every 6 hours).
4. Implement src/services/currency.rs with:
   - get_rate(from, to) → check Redis first, fallback to DB, return rate
   - convert(amount, from, to) → get_rate and multiply
5. Add endpoint GET /v1/currency/rates?base=USD to return current cached rates.
6. Add endpoint GET /v1/currency/convert?amount=100&from=USD&to=CAD to perform conversion.
7. Ensure graceful handling if rate is unavailable (return error, don't silently use stale data beyond threshold).

## Validation
Sync job fetches rates and stores them in DB and Redis; GET /v1/currency/rates returns cached rates; GET /v1/currency/convert returns correct conversion with proper decimal precision; Redis cache hit returns rate without DB query; stale rate beyond threshold returns appropriate error; mock external API to test sync job logic.