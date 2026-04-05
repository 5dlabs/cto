Implement subtask 4010: Implement currency rate sync background task with Valkey caching

## Objective
Build the background tokio task that fetches exchange rates every 6 hours from an external API, stores them in the currency_rates table, and caches them in Valkey with a 6-hour TTL.

## Steps
1. Create `src/currency/sync.rs`:
   - Define the supported currencies: USD, CAD, AUD, NZD, EUR, GBP.
   - Implement `fetch_rates(http_client: &reqwest::Client, base: &str) -> Result<HashMap<String, Decimal>>` that calls the exchange rate API (e.g., frankfurter.app/latest?from=USD&to=CAD,AUD,NZD,EUR,GBP).
   - Parse JSON response, extract rates as rust_decimal::Decimal.
2. Implement `store_rates(pool, rates: &[(base, target, rate, fetched_at)])` — UPSERT into currency_rates using ON CONFLICT (base_currency, target_currency) DO UPDATE.
3. Implement `cache_rates(valkey_conn, rates)` — store each rate as a Valkey key `currency:USD:CAD` with the rate as string value, TTL 21600 seconds (6 hours).
4. Create `src/currency/service.rs`:
   - `get_rate(valkey_conn, pool, base, target) -> Result<Decimal>` — try Valkey first, fall back to DB, return error if not found.
5. Create `src/background/currency_sync.rs`:
   - `spawn_currency_sync_task(pool, valkey, http_client)` — spawns a tokio task that loops: fetch_rates → store_rates → cache_rates → tokio::time::sleep(6 hours).
   - On error, log and retry after 5 minutes.
6. Implement `GET /api/v1/currency/rates` handler:
   - Returns all current rates from DB (or Valkey cache).
7. Call spawn_currency_sync_task from main.rs after server setup.
8. In test/dev, the task should still start but can use mock data.

## Validation
Unit test: parse mock API response JSON into HashMap<String, Decimal> correctly. Integration test: mock HTTP endpoint returning known rates, trigger sync, verify rates are stored in DB with correct values. Integration test: verify rates are cached in Valkey with correct TTL. Integration test: get_rate returns cached value from Valkey; after Valkey flush, falls back to DB. Integration test: GET /api/v1/currency/rates returns all stored rates.