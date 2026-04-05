Implement subtask 4010: Implement currency rate sync background task

## Objective
Build the background tokio task that fetches currency exchange rates hourly from an external API and stores them in the database and Valkey cache.

## Steps
1. Create `services/rust/finance/src/background/currency_sync.rs`.
2. Define `CurrencyRateSyncer` struct with DB pool, Valkey connection, and reqwest client.
3. Implement `sync_rates(&self) -> Result<()>`:
   - Fetch rates from the configured exchange rate API (e.g., `https://api.exchangerate.host/latest?base=USD&symbols=CAD,AUD,NZD`).
   - Parse JSON response into rate structs.
   - Upsert into `currency_rates` table (ON CONFLICT (base_currency, target_currency) DO UPDATE SET rate = EXCLUDED.rate, fetched_at = now()).
   - Cache in Valkey with key `currency_rates:USD` as JSON, TTL 3600 seconds.
   - Log success/failure with rate values.
4. Implement `run_sync_loop(&self)` that calls `sync_rates()` every hour using `tokio::time::interval`.
5. Spawn the sync loop in `main.rs` as a background tokio task alongside the Axum server.
6. Handle errors gracefully: if API is down, log error and retry next interval; don't crash the service.
7. Support configurable interval via environment variable `CURRENCY_SYNC_INTERVAL_SECS` (default 3600).
8. Fetch supported currency pairs: USD↔CAD, USD↔AUD, USD↔NZD, CAD↔USD.

## Validation
Unit test: mock exchange rate API response, verify rates are parsed correctly and upserted into DB. Verify Valkey cache is set with correct key, value, and TTL. Test API error handling: mock 500 response, verify error is logged but task continues. Test with invalid JSON response, verify graceful error handling. Integration test: run sync once against test DB, verify currency_rates table populated with expected currency pairs.