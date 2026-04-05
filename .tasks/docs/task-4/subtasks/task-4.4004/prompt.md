Implement subtask 4004: Implement multi-currency support with scheduled rate sync and Redis caching

## Objective
Build the currency rate synchronization job that fetches exchange rates from an external API on a schedule, stores them in PostgreSQL, and caches current rates in Redis for fast lookups during invoice and payment operations.

## Steps
1. Create `src/services/currency_service.rs`.
2. Implement `fetch_rates(base_currency: &str)`: HTTP client call to the chosen exchange rate API (e.g., Open Exchange Rates). Parse response into Vec<CurrencyRate> structs.
3. Implement `store_rates(rates: Vec<CurrencyRate>)`: batch insert/upsert into currency_rates table with current timestamp.
4. Implement `cache_rates(rates: Vec<CurrencyRate>)`: store each rate in Redis with key pattern `currency_rate:{base}:{target}` and a TTL of 1 hour.
5. Implement `get_rate(base: &str, target: &str) -> Result<Decimal>`: first check Redis cache, fallback to PostgreSQL query for most recent rate, return error if no rate found.
6. Implement `convert_amount(amount: i64, from: &str, to: &str) -> Result<i64>`: use get_rate and perform conversion with proper decimal handling to avoid precision loss.
7. Create `src/jobs/currency_sync.rs`: implement a tokio::spawn background task that runs the rate sync every configured interval (default: every 6 hours). Use tokio::time::interval.
8. Create `src/routes/currency.rs`:
   - GET /api/v1/currency/rates — list current rates
   - GET /api/v1/currency/convert?amount=X&from=USD&to=EUR — convert amount
9. Integrate `convert_amount` into invoice service for multi-currency invoice display.

## Validation
Unit tests: verify rate conversion math with known exchange rates, verify precision is maintained for large amounts. Integration tests: mock external API, run sync job, verify rates stored in DB and cached in Redis. Verify cache fallback: clear Redis, verify DB lookup works. Verify GET /rates returns current rates. Verify GET /convert returns correct converted amount. Verify background job runs on schedule (test with short interval).