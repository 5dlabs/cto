Implement subtask 4007: Implement scheduled currency rate sync job

## Objective
Create a background task that periodically fetches current exchange rates from an external API and stores them in the currency_rates table.

## Steps
1. Create src/services/currency_sync.rs. 2. Use reqwest to call an exchange rate API (e.g., ExchangeRate-API free tier at https://api.exchangerate-api.com/v4/latest/USD). Configure the API URL and base currency via environment variables. 3. Implement fetch_rates: make HTTP GET request, parse JSON response, extract rates for target currencies relevant to the business (USD, EUR, GBP, CAD, AUD at minimum). 4. Implement sync_rates: call fetch_rates, then upsert each rate into currency_rates table with fetched_at timestamp. 5. In main.rs, spawn a tokio task that runs sync_rates on startup and then every 6 hours (configurable via CURRENCY_SYNC_INTERVAL_SECS env var). 6. Add error handling: if the API call fails, log the error and retry after a backoff period; do not crash the service. 7. Add GET /api/v1/currency/rates endpoint that returns the latest rates from the database.

## Validation
Unit test with mocked HTTP responses verifies rates are correctly parsed and upserted; integration test verifies rates appear in the database after sync; GET /api/v1/currency/rates returns the latest stored rates; service continues running if the external API is unreachable (error is logged, not propagated).