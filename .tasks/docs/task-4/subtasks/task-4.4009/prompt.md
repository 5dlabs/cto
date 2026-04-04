Implement subtask 4009: Implement payroll endpoints and currency rate endpoints

## Objective
Build the payroll entry CRUD endpoints under `/api/v1/payroll` and the currency rate query endpoint under `/api/v1/currency/rates`.

## Steps
1. Create `services/rust/finance/src/routes/payroll.rs` and `services/rust/finance/src/db/payroll.rs`.
2. `POST /api/v1/payroll`:
   - Accept: org_id, employee_id, period_start, period_end, amount_cents, currency, type (employee/contractor), notes.
   - Validate period_start < period_end, amount_cents > 0.
   - Insert into `payroll_entries`.
   - Return 201 with created entry.
3. `GET /api/v1/payroll`:
   - Query params: org_id (required), period_start, period_end (for range filtering), employee_id (optional), type (optional), offset, limit.
   - Return paginated list with total count.
4. Create `services/rust/finance/src/routes/currency.rs`.
5. `GET /api/v1/currency/rates`:
   - Query params: base_currency (default USD).
   - First check Valkey cache for key `currency_rates:{base_currency}` with JSON payload.
   - If cache miss, query `currency_rates` table for latest rates by `fetched_at`.
   - Return {base_currency, rates: [{target_currency, rate, fetched_at}]}.
6. Add utoipa annotations.
7. Wire into Axum router.

## Validation
Integration tests: (1) Create payroll entry, verify 201 and correct fields returned. (2) Create 5 entries across 2 periods, GET with period filter returns correct subset. (3) GET with employee_id filter works. (4) Verify validation: period_start >= period_end returns 400. (5) Currency rates: seed rates in DB, GET returns correct rates. (6) Verify Valkey cache is populated after first DB query. (7) Verify cache hit on second request (mock or check Valkey directly).