Implement subtask 4004: Implement multi-currency support with scheduled exchange rate sync

## Objective
Build the currency rate sync scheduled job, Redis caching layer for rates, and multi-currency conversion utilities used across invoice and payment operations.

## Steps
1. Add redis crate (or deadpool-redis) for Redis connectivity. Initialize Redis client from ConfigMap URL. 2. Implement /src/services/currency.rs: define CurrencyRate struct (from_currency, to_currency, rate, fetched_at). 3. Implement rate fetching function that calls an external exchange rate API (e.g., exchangerate.host or configured provider). Parse the response and store rates in both PostgreSQL (currency_rates table for history) and Redis (with TTL for fast lookups). 4. Implement a scheduled task using tokio::spawn with tokio::time::interval that runs the rate sync (e.g., every 4 hours). 5. Implement currency conversion helper: convert_amount(amount, from_currency, to_currency) that looks up the rate from Redis first, falls back to PostgreSQL. 6. Implement GET /api/v1/currency/rates endpoint that returns current rates. 7. Ensure all invoice and payment operations use these conversion utilities when dealing with non-base currencies.

## Validation
Rate sync job fetches and stores rates correctly; Redis cache is populated with current rates; convert_amount produces accurate conversions (verified against known rate pairs); /api/v1/currency/rates returns current data; fallback to PostgreSQL works when Redis is unavailable; scheduled job runs at configured interval.