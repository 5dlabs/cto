Implement subtask 4007: Implement currency rate sync scheduled job with Redis caching

## Objective
Build a background job that periodically fetches currency exchange rates from an external API, stores them in PostgreSQL, and caches them in Redis.

## Steps
1. Create `src/currency/fetcher.rs`: implement a currency rate fetcher that calls an external API (e.g., exchangerate-api.com, Open Exchange Rates, or ECB). Accept API URL and key from config. 2. Create `src/currency/sync_job.rs`: implement a tokio-based scheduled task using tokio::time::interval (e.g., every 1 hour). On each tick: fetch latest rates, upsert into `finance.currency_rates` table, push to Redis with key pattern `currency:USD:EUR` and TTL of 2 hours. 3. Create `src/handlers/currency.rs`: GET /api/v1/currency/rates — first check Redis cache, fall back to PostgreSQL if cache miss, return rates. Accept query params: base_currency, target_currency (optional, return all if omitted). 4. Create `src/cache/redis.rs`: initialize redis-rs async connection from REDIS_URL env var. Implement get/set/delete helpers with TTL. 5. Start the sync job as a background tokio::spawn in main.rs. 6. Handle external API failures: log error, retain stale rates, retry on next interval. 7. Wire the /api/v1/currency/rates route into main router.

## Validation
Unit tests with mocked external API: sync job correctly parses response and stores rates; Redis cache is populated with correct TTL. Integration tests: sync job runs and populates both PostgreSQL and Redis; GET /api/v1/currency/rates returns cached data; cache miss falls through to PostgreSQL; stale rates are retained when external API is unavailable.