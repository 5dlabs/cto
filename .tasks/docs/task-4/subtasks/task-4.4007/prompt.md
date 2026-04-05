Implement subtask 4007: Implement tax calculation module with configurable jurisdiction rates

## Objective
Build the tax calculation module supporting configurable per-jurisdiction tax rates (GST/HST for Canada, sales tax for US) stored in the tax_configurations table, and integrate it into invoice creation.

## Steps
1. Create `src/tax/mod.rs` and `src/tax/calculator.rs`.
2. Define `TaxConfiguration` model matching the tax_configurations table: jurisdiction_code, jurisdiction_name, tax_type (GST, HST, PST, SALES_TAX), rate (Decimal), active flag, config JSONB for additional rules.
3. Implement `src/db/tax.rs`:
   - `get_tax_config(pool, jurisdiction_code)` → returns TaxConfiguration or None.
   - `list_tax_configs(pool)` → all active configurations.
   - `upsert_tax_config(pool, config)` → create or update a tax configuration.
4. Implement `TaxCalculator` in `src/tax/calculator.rs`:
   - `calculate_tax(subtotal_cents: i64, jurisdiction_code: &str, pool) -> Result<TaxResult>` where TaxResult = { tax_cents: i64, tax_rate: Decimal, jurisdiction: String }.
   - Uses rust_decimal for precise calculation: tax_cents = (subtotal_cents as Decimal * rate).round_dp(0) as i64.
   - If jurisdiction not found or inactive, return tax_cents = 0 (zero-tax jurisdiction).
5. Integrate into invoice creation (subtask 4003's create_invoice):
   - Add optional `jurisdiction_code` field to CreateInvoiceRequest.
   - After computing subtotal_cents, call calculate_tax to get tax_cents.
   - total_cents = subtotal_cents + tax_cents.
   - Store tax_cents on the invoice.
6. Seed initial tax configurations via a migration or seed script:
   - CA-GST: 5% (federal GST)
   - CA-ON: 13% (Ontario HST)
   - CA-BC: 12% (BC GST+PST)
   - CA-AB: 5% (Alberta, GST only)
   - US-ZERO: 0% (placeholder, US sales tax varies by state)
7. No API endpoints for tax config management in v1 — manage via DB seeds. Add a GET /api/v1/tax/configurations endpoint for read-only listing.

## Validation
Unit test: calculate_tax with CA-GST (5%) on subtotal of 10000 cents returns tax_cents = 500. Unit test: calculate_tax with CA-ON (13%) on subtotal of 10000 returns tax_cents = 1300. Unit test: calculate_tax with unknown jurisdiction returns tax_cents = 0. Unit test: rust_decimal rounding is correct for edge cases (e.g., subtotal 333 cents at 13% = 43 cents, not 43.29). Integration test: create invoice with jurisdiction_code='CA-ON', verify tax_cents and total_cents are computed correctly.