Implement subtask 4007: Implement scheduled currency rate sync job with Redis caching

## Objective
Build a background job that periodically fetches exchange rates from an external API, stores them in PostgreSQL, and caches them in Redis for fast lookups.

## Steps
1. Create src/services/currency_service.rs.
2. Implement external rate fetching: use reqwest to call a free currency rate API (e.g., exchangerate-api.com or open.er-api.com). Parse response into (base, target, rate) tuples for a configured list of currencies (USD, EUR, GBP, CAD, AUD minimum).
3. Implement rate storage: upsert into currency_rates table with current timestamp.
4. Implement Redis caching: after DB write, set key `currency:USD:EUR` → rate string with 1-hour TTL. Implement get_rate(base, target) that checks Redis first, falls back to DB.
5. Create src/routes/currency.rs:
   - GET /api/v1/currency/rates → list all current rates
   - GET /api/v1/currency/rates/:base/:target → get specific rate
   - POST /api/v1/currency/convert → body: {amount, from, to} → returns converted amount using latest rate
6. Implement the scheduled job: use tokio::spawn with tokio::time::interval to run rate sync every hour (configurable via env var CURRENCY_SYNC_INTERVAL_SECS, default 3600).
7. Handle external API failures gracefully: log error, keep using last known rates, don't crash the service.
8. On service startup, trigger an immediate sync before entering the interval loop.

## Validation
Unit tests with mocked HTTP responses verify rate parsing and DB upsert; Redis cache is populated after sync and get_rate returns cached value; convert endpoint returns mathematically correct result using Decimal; rate sync continues working after external API failure (uses stale rates); startup triggers immediate sync.