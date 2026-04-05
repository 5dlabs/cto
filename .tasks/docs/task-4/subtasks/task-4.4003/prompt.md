Implement subtask 4003: Implement tax calculation engine with seed data

## Objective
Build the tax calculation module that determines applicable tax (GST/HST/sales tax) based on customer jurisdiction, and seed the `tax_rules` table with Canadian GST (5%), provincial HST rates, and US placeholder sales tax.

## Steps
1. Create `services/rust/finance/src/services/tax.rs`.
2. Define a `TaxCalculator` struct that takes a reference to the DB pool.
3. Implement `calculate_tax(&self, jurisdiction: &str, subtotal_cents: i64) -> Result<TaxResult>` where `TaxResult` includes `tax_type`, `rate_percent`, `tax_cents`.
4. Query `tax_rules` for the given jurisdiction where `effective_from <= now()`, ordered by `effective_from DESC`, limit 1.
5. Compute `tax_cents = (subtotal_cents * rate_percent) / 100`, rounding using banker's rounding (round half to even) to avoid cent discrepancies.
6. Handle case where no tax rule exists for jurisdiction (return 0 tax with a warning log).
7. Create seed migration (008) to insert tax rules:
   - Canada federal GST: jurisdiction='CA', rate=5.0
   - Ontario HST: jurisdiction='CA-ON', rate=13.0
   - Nova Scotia HST: jurisdiction='CA-NS', rate=15.0
   - New Brunswick HST: jurisdiction='CA-NB', rate=15.0
   - Newfoundland HST: jurisdiction='CA-NL', rate=15.0
   - PEI HST: jurisdiction='CA-PE', rate=15.0
   - US placeholder: jurisdiction='US', rate=0.0 (placeholder)
8. Write parameterized unit tests: given subtotal 10000 cents in CA-ON → tax_cents = 1300; in CA-NS → 1500; in CA (GST only) → 500; in US → 0.
9. Test edge cases: zero subtotal, unknown jurisdiction, negative amounts (reject).

## Validation
Parameterized unit tests verifying exact tax_cents output for each Canadian province (CA-ON: 13%, CA-NS/NB/NL/PE: 15%, CA: 5%, US: 0%). Test with subtotals of 10000, 9999, 1, and 0 cents. Verify unknown jurisdiction returns 0 tax. Verify negative subtotal is rejected with error. All tests use sqlx::test with seeded tax_rules.